// ───────────────────────── I-3.5（引用区间的精确性）的出现规则：版本都走完之后逐对判 ─────────────────────────
// walk.rs 的 HoleInTheCandidateSet
const APPEARANCE_NO_HOLE: u32 = 0u;
const APPEARANCE_HOLE_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS: u32 = 1u;
const APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS: u32 = 2u;
// 判一版时它引用的落点摊在这里（C748（最终折叠表漏判引用区间的落点）：按落点比，不折成单元去重）：每个落点 2 个字（③ 里那个落点的字：
// 落点表下标连同这一版打的标签码；它折成的单元在走过的单元表里的下标或 NONE，取诞生代号用）。
// 借 C_PENDING 起到 C_SCRATCH_END 为止那一段（一版中央映射只读读法的显式栈也借它），判的时候主走读那几版已经走完、不再读映射
const APPEARANCE_SPREAD_PLACEMENT_WORDS: u32 = 2u;
const APPEARANCE_SPREAD_PLACEMENTS: u32 = C_PENDING;
const APPEARANCE_MAX_SPREAD_PLACEMENTS: u32 = (C_SCRATCH_END - C_PENDING) / APPEARANCE_SPREAD_PLACEMENT_WORDS;

// 一版至多 C_MAX_IN_VERSION 个落点（满了站点 4 先判不了），摊得下（解析内核时判，不进展开量）
const_assert C_MAX_IN_VERSION <= APPEARANCE_MAX_SPREAD_PLACEMENTS;

fn appearance_version_row(version_index: u32) -> u32 {
    return C_APPEARANCE_VERSIONS + version_index * C_APPEARANCE_VERSION_WORDS;
}

fn appearance_version_txg(version_index: u32) -> vec2<u32> {
    let row = appearance_version_row(version_index);
    return vec2<u32>(sw(row + 1u), sw(row + 2u));
}

// 落点表一行在 I-3.5 比相邻两版时认作哪一个（walk.rs PlacementOfTheAppearanceRule::identity_compared_across_versions）：第一次记下时按中央映射 key 折的
// 认它折成的单元（第 3–5 字），别的认落点自己（第 0–2 字）。认作 (盘, 槽 lo, 槽 hi)
fn appearance_identity_of(index: u32) -> vec3<u32> {
    let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
    let first_word = select(0u, 3u, (sw(row + 7u) & APPEARANCE_PLACEMENT_NOTED_THROUGH_THE_CENTRAL_MAPPING) != 0u);
    return vec3<u32>(sw(row + first_word), sw(row + first_word + 1u), sw(row + first_word + 2u));
}

// ③ 里一个落点的字（version_placement_word）带的这一版的标签是不是这一版（② 那一行 version_row）第 root_index 棵用户可见树：
// 按种类与树 ID 比（walk.rs 按 UserVisibleTreeOfTheWalk 比），同一张树表里种类与树 ID 都相同的两棵根算同一棵
fn appearance_label_is_the_tree_of_root(version_placement_word: u32, version_row: u32, root_index: u32) -> bool {
    let label = version_placement_word >> APPEARANCE_LABEL_SHIFT;
    if (label == 0u) {
        return false;
    }
    let labelled_root_row = version_row + C_APPEARANCE_VERSION_ROOTS_OFFSET + (label - 1u) * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
    let root_row = version_row + C_APPEARANCE_VERSION_ROOTS_OFFSET + root_index * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
    return sw(labelled_root_row) == sw(root_row) && sw(labelled_root_row + 1u) == sw(root_row + 1u) && sw(labelled_root_row + 2u) == sw(root_row + 2u);
}

// 一版引用的落点摊进 APPEARANCE_SPREAD_PLACEMENTS，交回几个。每个落点带它折成的单元（落点表第 3–5 字，全部版本走完之后的那一张）在走过的单元表里的下标：
// walk.rs 取诞生代号按 unit_whose_birth_txg_it_takes 找 birth_txg_of_walked_units 那一句
fn appearance_spread_the_placements_of_a_version(version_index: u32) -> u32 {
    let row = appearance_version_row(version_index);
    let first = sw(row + 4u);
    let count = sw(row + 5u);
    for (var position = 0u; position < count; position = position + 1u) {
        let version_placement_word = sw(C_APPEARANCE_VERSION_PLACEMENTS + first + position);
        let placement_row = C_APPEARANCE_PLACEMENTS + (version_placement_word & APPEARANCE_PLACEMENT_INDEX_MASK) * C_APPEARANCE_PLACEMENT_WORDS;
        sws(APPEARANCE_SPREAD_PLACEMENTS + position * APPEARANCE_SPREAD_PLACEMENT_WORDS, version_placement_word);
        sws(APPEARANCE_SPREAD_PLACEMENTS + position * APPEARANCE_SPREAD_PLACEMENT_WORDS + 1u,
            w_visited_index_in_the_main_walk(sw(placement_row + 3u), vec2<u32>(sw(placement_row + 4u), sw(placement_row + 5u))));
    }
    return count;
}

// 摊开的第 u 个落点折成的单元有没有记下诞生代号（被引用单元那一支标记 1、实例表一片标记 2；头读不出的没有）
fn appearance_unit_birth_is_known(u: u32) -> bool {
    let visited_index = sw(APPEARANCE_SPREAD_PLACEMENTS + u * APPEARANCE_SPREAD_PLACEMENT_WORDS + 1u);
    return visited_index != NONE && w_birth_mark_of_a_visited_row(visited_index) != 0u;
}

fn appearance_unit_birth_txg(u: u32) -> vec2<u32> {
    let visited_index = sw(APPEARANCE_SPREAD_PLACEMENTS + u * APPEARANCE_SPREAD_PLACEMENT_WORDS + 1u);
    return vec2<u32>(sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 1u), sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 2u));
}

// txg 为 previous_txg 的那几版（R⁻ 那一组）里有没有一版引用了与落点表第 placement 行认作同一个的落点（appearance_identity_of）
fn appearance_previous_group_references_the_identity_of(previous_txg: vec2<u32>, placement: u32) -> bool {
    let identity = appearance_identity_of(placement);
    for (var other = 0u; other < appearance_version_count; other = other + 1u) {
        if (!eq64(appearance_version_txg(other), previous_txg)) {
            continue;
        }
        let row = appearance_version_row(other);
        let first = sw(row + 4u);
        let count = sw(row + 5u);
        for (var k = 0u; k < count; k = k + 1u) {
            if (all(appearance_identity_of(sw(C_APPEARANCE_VERSION_PLACEMENTS + first + k) & APPEARANCE_PLACEMENT_INDEX_MASK) == identity)) {
                return true;
            }
        }
    }
    return false;
}

// walk.rs hole_in_the_candidate_set_between：R⁻ 那一组（txg previous_txg 的那几版）与这一版 (version_instance, version_txg) 之间的洞；
// 接得上 ⟺ R⁻ 的某个实例 i：最新根的实例表里有一行 (i, T = R⁻.txg)，这一版的实例 v > i，i 与 v 之间每个实例都有一行、每一行都是 (j, 0, 0)，
// 且这一版的实例表指针是这一版新建的（第四条件，C699 取号跳过改认出生代号：R 所在实例自己更早的版本不算跳过）
fn appearance_hole_between(previous_txg: vec2<u32>, version_txg: vec2<u32>, version_instance: u32, version_instance_table_pointer_is_born_with_it: bool) -> u32 {
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
            // 中间的实例按不同的代号数（同一个实例有两行只算一个）：第 k 行之前有同实例的行 ⟺ 这个实例的第一行不是第 k 行
            if (w_first_instance_row_of(instance) == k) {
                instances_between = instances_between + 1u;
            }
        }
        if (row_of_the_previous_version && instances_between == version_instance - previous_instance - 1u && every_row_between_published_nothing
            && version_instance_table_pointer_is_born_with_it) {
            return APPEARANCE_HOLE_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS;
        }
    }
    return APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS;
}

// 这一版（引用的落点摊开在 APPEARANCE_SPREAD_PLACEMENTS 里，placements_spread 个）树表里第 root_index 棵用户可见树是不是整棵继承下来的：根指针出生 txg 严格小于 R⁻.txg，
// 且这一版里那棵树下的落点折成的单元读得出的诞生代号都不大于它
fn appearance_tree_is_inherited_whole(version_index: u32, root_index: u32, placements_spread: u32, previous_txg: vec2<u32>) -> bool {
    let version_row = appearance_version_row(version_index);
    let root_row = version_row + C_APPEARANCE_VERSION_ROOTS_OFFSET + root_index * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
    let root_pointer_birth_txg = vec2<u32>(sw(root_row + 3u), sw(root_row + 4u));
    if (!lt64(root_pointer_birth_txg, previous_txg)) {
        return false;
    }
    for (var u = 0u; u < placements_spread; u = u + 1u) {
        if (!appearance_label_is_the_tree_of_root(sw(APPEARANCE_SPREAD_PLACEMENTS + u * APPEARANCE_SPREAD_PLACEMENT_WORDS), version_row, root_index) || !appearance_unit_birth_is_known(u)) {
            continue;
        }
        if (!le64(appearance_unit_birth_txg(u), root_pointer_birth_txg)) {
            return false;
        }
    }
    return true;
}

// walk.rs judge_the_appearance_rule_of_adjacent_candidate_versions：回退候选集（走过的每一版）按 txg 升序、同 txg 并成一组，
// 每一版 R 与 txg 比它小的那几版里 txg 最大的那一组 R⁻ 比：R 引用而 R⁻ 不引用（按 appearance_identity_of 认，C748）、折成的单元诞生代号读得出的
// 每个落点 b 满足 (a) birth(b) > R⁻.txg，或 (b) b 打着一棵整棵继承下来的用户可见树的标签、且 R 的谱系记录里有一条 (T_rb, T_old) 满足
// R⁻.txg < T_rb ≤ R.txg、那棵树的根指针出生 txg ≤ T_old（谱系那一半，⑤ 那一行；同一棵树在树表里有几条根时取最后一条的根指针出生 txg，
// 同 walk.rs 按树收的那张表）。有洞而实例表行接不上时打着用户可见树标签的不判（N3）；
// R 或 R⁻ 那一组有一版走读记过失败或树表里有「走到没有条款的形状」的条目（两样都记进 ② 第 3 字）的那一对、诞生代号读不出的落点、(a) 不成立而落在整棵继承下来的树里、R 的谱系读不全的落点不判，
// 记射程里判不了（C701：CPU 报部分没判，几样都不置判到位；谱系读不全只让要靠谱系那一半判的落点不判，同一对里别的落点照判）。
// 一个落点都没判到时 CPU 报不适用，违例位上与成立一样是 0
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
        if (!has_previous) {
            continue;
        }
        if (sw(row + 3u) != 1u) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_5);
            continue;
        }
        var previous_references_are_complete = true;
        for (var other = 0u; other < appearance_version_count; other = other + 1u) {
            if (eq64(appearance_version_txg(other), previous_txg) && sw(appearance_version_row(other) + 3u) != 1u) {
                previous_references_are_complete = false;
            }
        }
        if (!previous_references_are_complete) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_5);
            continue;
        }
        let hole = appearance_hole_between(previous_txg, version_txg, sw(row), sw(row + C_APPEARANCE_VERSION_INSTANCE_TABLE_POINTER_IS_BORN_WITH_THE_VERSION_OFFSET) == 1u);
        let placements_spread = appearance_spread_the_placements_of_a_version(version_index);
        // 这一版里整棵继承下来的用户可见树（第 r 棵是就置位 r）
        var trees_inherited_whole = 0u;
        let roots = sw(row + 6u);
        for (var r = 0u; r < roots; r = r + 1u) {
            if (appearance_tree_is_inherited_whole(version_index, r, placements_spread, previous_txg)) {
                trees_inherited_whole = trees_inherited_whole | (1u << r);
            }
        }
        // 谱系那一半按根各问一次（第 r 棵问过就置 roots_whose_lineage_is_asked 的位 r，谱系担保它是回退带回来的再置另一张的位 r）
        let lineage_is_readable = sw(appearance_lineage_row(version_index)) == 1u;
        var roots_whose_lineage_is_asked = 0u;
        var roots_brought_back_by_a_rollback_on_the_lineage = 0u;
        for (var u = 0u; u < placements_spread; u = u + 1u) {
            let version_placement_word = sw(APPEARANCE_SPREAD_PLACEMENTS + u * APPEARANCE_SPREAD_PLACEMENT_WORDS);
            if (appearance_previous_group_references_the_identity_of(previous_txg, version_placement_word & APPEARANCE_PLACEMENT_INDEX_MASK)) {
                continue;
            }
            let in_a_user_visible_tree = (version_placement_word >> APPEARANCE_LABEL_SHIFT) != 0u;
            if (hole == APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS && in_a_user_visible_tree) {
                continue;
            }
            if (!appearance_unit_birth_is_known(u)) {
                mark_part_of_the_range_not_judged(INVARIANT_I_3_5);
                continue;
            }
            let born_after_the_previous_version = lt64(previous_txg, appearance_unit_birth_txg(u));
            var in_a_tree_inherited_whole = false;
            var root_of_the_labelled_tree = 0u;
            for (var r = 0u; r < roots; r = r + 1u) {
                if (appearance_label_is_the_tree_of_root(version_placement_word, row, r)) {
                    root_of_the_labelled_tree = r;
                    if ((trees_inherited_whole & (1u << r)) != 0u) {
                        in_a_tree_inherited_whole = true;
                    }
                }
            }
            // (b) 的谱系那一半只在 (a) 不成立、前两半成立时用：别的落点不管谱系读不读得全都判得出
            var inherited_tree_brought_back_by_a_rollback_on_the_lineage = false;
            if (!born_after_the_previous_version && in_a_tree_inherited_whole) {
                if (!lineage_is_readable) {
                    mark_part_of_the_range_not_judged(INVARIANT_I_3_5);
                    continue;
                }
                let root_bit = 1u << root_of_the_labelled_tree;
                if ((roots_whose_lineage_is_asked & root_bit) == 0u) {
                    roots_whose_lineage_is_asked = roots_whose_lineage_is_asked | root_bit;
                    let root_row = row + C_APPEARANCE_VERSION_ROOTS_OFFSET + root_of_the_labelled_tree * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
                    if (appearance_lineage_names_a_rollback_between_the_two_versions_to_a_target_not_older_than_the_root(version_index, previous_txg, version_txg,
                        vec2<u32>(sw(root_row + 3u), sw(root_row + 4u)))) {
                        roots_brought_back_by_a_rollback_on_the_lineage = roots_brought_back_by_a_rollback_on_the_lineage | root_bit;
                    }
                }
                inherited_tree_brought_back_by_a_rollback_on_the_lineage = (roots_brought_back_by_a_rollback_on_the_lineage & root_bit) != 0u;
            }
            judge(INVARIANT_I_3_5, born_after_the_previous_version || (in_a_tree_inherited_whole && inherited_tree_brought_back_by_a_rollback_on_the_lineage));
        }
    }
}

// ───────────────────────── 第二批：走读与走读之后（check_pool_image 择根之后那一段）─────────────────────────

// 恢复眼里的 journal 环：每个键 (计数器, 实例代号) 取按盘次序、盘内按环槽次序第一份自证过的（records_as_the_recovery_reads_them：
// 逐盘先到先留、跨盘取第一块盘上的，合起来就是按收的次序先到先留；同一计数器两个实例的记录两条都留）。只收恢复的记录读者读得出的
// （journal_record_reads_as_the_recovery_reader_reads_it；walk.rs 的 records_the_recovery_reader_reads 那一份逐盘表，C897 那一行后半）：
// 同盘同键先到的那一条读不出、后到的读得出时留后到的那条，与 core 扫环选的同一份（core 扫环时读不出的记录根本收不进来）。
// 点名项 flags 非 0 的记录反过来：池级 checker 的记录读者拒、恢复的读者收，这里收（checker_record_valid_past_the_named_entry_flags）
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
            if (!checker_record_valid_past_the_named_entry_flags(s, fsid_low)) {
                continue;
            }
            if (!journal_record_reads_as_the_recovery_reader_reads_it(s.base)) {
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

// 重放前缀读的那一份环（walk.rs 的 records_as_the_recovery_reads_them，恢复视图与重放读同一份）：每块盘上自证过、恢复的记录读者读得出的记录
// （journal_record_reads_as_the_recovery_reader_reads_it：提交标记 0 / 1、记录标志位 0 之外全 0、本次发布内序号不是 0、新根段四条指针头部是
// 第一版的取值；点名项 flags 非 0 的照收，checker_record_valid_past_the_named_entry_flags），按 (实例代号, 计数器) 建键、按盘的次序与环槽次序先到先得，
// 升序排在 C_REPLAY_RECORDS。同一块盘上撞键（冲突布局，I-8.13 判红）时留先到的那一条读得出的。交回条数；多于 MAX_RECORDS 条判不了。
// collect_the_records_the_replay_reads 最近一次收下的条数：I-3.19、I-5.7 在重放那一阶段按实例读那一份环
var<private> c_replay_record_count: u32;

fn collect_the_records_the_replay_reads(fsid_low: vec2<u32>) -> u32 {
    var count = 0u;
    c_replay_record_count = 0u;
    let device_count = hdr(3u);
    let location_count = hdr(6u);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        for (var l = 0u; l < location_count; l = l + 1u) {
            if (location_word(l, 0u) != device || location_word(l, 4u) != LOCATION_JOURNAL_RECORD) {
                continue;
            }
            let s = read_location(l);
            if (!checker_record_valid_past_the_named_entry_flags(s, fsid_low)) {
                continue;
            }
            if (!journal_record_reads_as_the_recovery_reader_reads_it(s.base)) {
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
                c_replay_record_count = count;
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
    c_replay_record_count = count;
    return count;
}

// 重放读的那一份环（前 count 条，按 (实例, 计数器) 严格升序）里第一个键不小于 (instance, counter) 的位置，都小交回 count。
// 二分：每一轮 [low, high) 之前的键都小于它、从 high 起的都不小于它，至多 ⌈log2(MAX_RECORDS + 1)⌉ = 13 轮
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
// ───────────────────────── 批 19 那几条：I-8.14、I-5.7、I-3.19（walk.rs 同名函数）─────────────────────────

// 重放读的那一份环（前 c_replay_record_count 条，按 (实例, 计数器) 升序）里实例 instance 那一段的起点；段到实例代号变了为止
fn replay_records_of_the_instance_from(instance: u32) -> u32 {
    return replay_record_position_not_below(c_replay_record_count, instance, vec2<u32>(0u, 0u));
}

// I-8.14（walk.rs judge_the_first_txg_of_each_instance_above_the_older_instances，C764）：根（R_ROOTS）与恢复眼里的环（重放读的那一份，跨盘取第一份）
// 按实例代号归组，从小到大逐个实例 k：k 那一组最小的 txg 要严格大于实例代号更小的各组里最大的 txg，有更小的实例可比的 k 判一次。
// 根环或 journal 环有一槽读不出：射程里判不了（CPU 报部分没判）。一对都比不到不判（CPU 报不适用）。
// 实例代号逐个取「根与记录里大于上一个的最小者」：记录按实例升序，二分到下一个实例的段首；根至多 MAX_ROOTS 条，逐条比
fn judge_the_first_txg_of_each_instance_above_the_older_instances(g: CheckerSlot, fsid_low: vec2<u32>) {
    let count = collect_the_records_the_replay_reads(fsid_low);
    if (undecidable != 0u) {
        return;
    }
    var some_slot_does_not_read_back = w_some_journal_slot_does_not_read_back;
    for (var position = 0u; position < g.regions * g.slots_per_region; position = position + 1u) {
        if (root_ring_slot_readings[position].x == ROOT_RING_SLOT_DOES_NOT_READ_BACK) {
            some_slot_does_not_read_back = true;
        }
    }
    if (some_slot_does_not_read_back) {
        mark_part_of_the_range_not_judged(INVARIANT_I_8_14);
    }
    var have_older = false;
    var older_highest = vec2<u32>(0u, 0u);
    var have_previous_instance = false;
    var previous_instance = 0u;
    loop {
        var have_next = false;
        var next = 0u;
        for (var i = 0u; i < root_count; i = i + 1u) {
            let instance = checker_root_of(i).instance;
            if ((!have_previous_instance || instance > previous_instance) && (!have_next || instance < next)) {
                next = instance;
                have_next = true;
            }
        }
        var position = 0u;
        if (have_previous_instance) {
            position = count;
            if (previous_instance != 0xFFFFFFFFu) {
                position = replay_record_position_not_below(count, previous_instance + 1u, vec2<u32>(0u, 0u));
            }
        }
        if (position < count) {
            let instance = sw(C_REPLAY_RECORDS + position * C_REPLAY_RECORD_WORDS + 3u);
            if (!have_next || instance < next) {
                next = instance;
                have_next = true;
            }
        }
        if (!have_next) {
            break;
        }
        var lowest = vec2<u32>(0u, 0u);
        var highest = vec2<u32>(0u, 0u);
        var have_carrier = false;
        for (var i = 0u; i < root_count; i = i + 1u) {
            let root = checker_root_of(i);
            if (root.instance != next) {
                continue;
            }
            if (!have_carrier || lt64(root.txg, lowest)) {
                lowest = root.txg;
            }
            if (!have_carrier || lt64(highest, root.txg)) {
                highest = root.txg;
            }
            have_carrier = true;
        }
        for (var k = replay_record_position_not_below(count, next, vec2<u32>(0u, 0u)); k < count; k = k + 1u) {
            if (sw(C_REPLAY_RECORDS + k * C_REPLAY_RECORD_WORDS + 3u) != next) {
                break;
            }
            let txg = replay_record_at(k).txg;
            if (!have_carrier || lt64(txg, lowest)) {
                lowest = txg;
            }
            if (!have_carrier || lt64(highest, txg)) {
                highest = txg;
            }
            have_carrier = true;
        }
        if (have_older) {
            judge(INVARIANT_I_8_14, lt64(older_highest, lowest));
        }
        if (!have_older || lt64(older_highest, highest)) {
            older_highest = highest;
            have_older = true;
        }
        previous_instance = next;
        have_previous_instance = true;
    }
}

// I-5.7（walk.rs judge_allocation_record_tree_roots_born_with_the_versions_applied_only_by_records，C706）在一版上：那次发布的末条（或先到的那一条）
// 在 base，这次发布的记录有一条点名项非空（publish_names_some_unit），新根段里「树表 0 条那一版的分配记录树根指针」不全零时，它的出生 txg 等于这一版的 txg
fn judge_the_allocation_record_tree_root_born_with_the_version(base: u32, publish_names_some_unit: bool) {
    let pointer = parse_node_pointer(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER);
    if (pointer.all_zero || !publish_names_some_unit) {
        return;
    }
    judge(INVARIANT_I_5_7, eq64(pointer.birth_txg, rd64(base, 26u)));
}

// I-3.19（walk.rs judge_the_applied_transaction_high_water_of_the_instance_table_rows，C740）：最新根实例表里每一行 (i, T, W)，W 落在这一行容许的集合里——
// T 为 0、或 T 按落点式该住的根环槽坐着 txg ≥ T 的自证过的根时含 0；环里实例 i、txg 小于 T 的每条根 c 起重放到 T 时含 (c, T] 里实例 i 记录的最大非 0 事务号
// （(c, T] 每个 txg 都要读到一条带末条标志的记录：按计数器次序数，txg 与上一条数过的相同的不重数，数出来的条数等于 T − c 才算读全）。
// 认不出的格（T 那一槽读不出、(c, T] 读不全、根环里有一槽可能藏着被盖掉或读不出的更早的根）在、W 又不在认得出的格里：这一行判不了（CPU 报部分没判）。
// 实例表链没走到末片：判不了。要 c_replay_record_count 与 C_REPLAY_RECORDS 是这一次重放收的那一份
fn judge_the_applied_transaction_high_water_of_the_instance_table_rows(g: CheckerSlot) {
    if (!w_newest_instance_table_chain_reached_its_last_page) {
        mark_part_of_the_range_not_judged(INVARIANT_I_3_19);
    }
    let slot_count = g.regions * g.slots_per_region;
    for (var row_index = 0u; row_index < w_instance_row_count; row_index = row_index + 1u) {
        let row = C_INSTANCE_ROWS + row_index * C_ROW_WORDS;
        let instance = sw(row);
        let published_txg = vec2<u32>(sw(row + 1u), sw(row + 2u));
        let high_water = vec2<u32>(sw(row + 3u), sw(row + 4u));
        var reachable = false;
        var some_starting_point_unreadable = false;
        if (is_zero64(published_txg)) {
            reachable = is_zero64(high_water);
        } else {
            let region = divmod64(published_txg, u64_of(g.regions)).remainder.x;
            let slot = divmod64(divmod64(published_txg, u64_of(g.regions)).quotient, u64_of(g.slots_per_region)).remainder.x;
            let reading = root_ring_slot_readings[region * g.slots_per_region + slot];
            if (reading.x == ROOT_RING_SLOT_SELF_VERIFIED && !lt64(vec2<u32>(reading.y, reading.z), published_txg)) {
                if (is_zero64(high_water)) {
                    reachable = true;
                }
            } else if (reading.x == ROOT_RING_SLOT_DOES_NOT_READ_BACK) {
                some_starting_point_unreadable = true;
            }
        }
        for (var i = 0u; i < root_count; i = i + 1u) {
            let root = checker_root_of(i);
            if (root.instance != instance || !lt64(root.txg, published_txg)) {
                continue;
            }
            var publishes_read_back = vec2<u32>(0u, 0u);
            var have_counted = false;
            var txg_of_the_last_publish_counted = vec2<u32>(0u, 0u);
            var reachable_high_water = vec2<u32>(0u, 0u);
            for (var k = replay_records_of_the_instance_from(instance); k < c_replay_record_count; k = k + 1u) {
                if (sw(C_REPLAY_RECORDS + k * C_REPLAY_RECORD_WORDS + 3u) != instance) {
                    break;
                }
                let record = replay_record_at(k);
                if (!lt64(root.txg, record.txg) || lt64(published_txg, record.txg)) {
                    continue;
                }
                if (lt64(reachable_high_water, record.transaction)) {
                    reachable_high_water = record.transaction;
                }
                if ((record.flags & 1u) != 0u && (!have_counted || !eq64(txg_of_the_last_publish_counted, record.txg))) {
                    publishes_read_back = add64(publishes_read_back, u64_of(1u));
                    txg_of_the_last_publish_counted = record.txg;
                    have_counted = true;
                }
            }
            if (eq64(publishes_read_back, sub64(published_txg, root.txg))) {
                if (eq64(reachable_high_water, high_water)) {
                    reachable = true;
                }
            } else {
                some_starting_point_unreadable = true;
            }
        }
        // 根环里有一槽可能藏着一条 txg 小于 T、被盖掉或读不出的根（walk.rs root_ring_slot_that_may_hide_an_overwritten_root_below）：
        // 区域 r、槽 s 能住的最小正 txg R × s + r（是 0 时取环长）小于 min(那一槽坐着的根的 txg（读不出当无穷大）, T)
        for (var position = 0u; position < slot_count; position = position + 1u) {
            let reading = root_ring_slot_readings[position];
            if (reading.x == ROOT_RING_SLOT_ALL_ZERO) {
                continue;
            }
            let region = position / g.slots_per_region;
            let slot = position % g.slots_per_region;
            var smallest_positive_txg = g.regions * slot + region;
            if (smallest_positive_txg == 0u) {
                smallest_positive_txg = slot_count;
            }
            var overwritten_up_to = vec2<u32>(0xFFFFFFFFu, 0xFFFFFFFFu);
            if (reading.x == ROOT_RING_SLOT_SELF_VERIFIED) {
                overwritten_up_to = vec2<u32>(reading.y, reading.z);
            }
            var bound = published_txg;
            if (lt64(overwritten_up_to, bound)) {
                bound = overwritten_up_to;
            }
            if (lt64(u64_of(smallest_positive_txg), bound)) {
                some_starting_point_unreadable = true;
            }
        }
        if (reachable) {
            judge(INVARIANT_I_3_19, true);
        } else if (!some_starting_point_unreadable) {
            judge(INVARIANT_I_3_19, false);
        } else {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_19);
        }
    }
}

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

// 走读的六个阶段，按次序；每个阶段里要走的每一处都经 walk_and_hand_off 循环里那一个调用点走
// （原来在被抛弃根之后还有一个阶段把候选集之外的根整遍走一次收位置项校验和，判隔离的记录要；池级 checker 只认隔离记号之后那一阶段删了）
const WALK_PHASE_NEWEST_ROOT: u32 = 0u;
const WALK_PHASE_CANDIDATE_ROOTS: u32 = 1u;
const WALK_PHASE_VERSIONS_APPLIED_ONLY_BY_RECORDS: u32 = 2u;
const WALK_PHASE_ABANDONED_ROOTS: u32 = 3u;
const WALK_PHASE_ROOTS_BELOW_THE_FLOOR: u32 = 4u;
const WALK_PHASE_REPLAYED_VERSION: u32 = 5u;
const WALK_PHASE_DONE: u32 = 6u;
// 交回的值是结论里「走到了第几步」：3 走读那一批没判完（中途判不了），4 走读那一批判完、进了收尾
const WALK_REACHED_THE_ROOT_RING: u32 = 3u;
const WALK_REACHED_THE_HAND_OFF: u32 = 4u;

// 池级 checker 判 I-7.4 时违例细节里点名的那一条被抛弃的根（walk.rs 的 judge_blocks_referenced_by_abandoned_roots：细节以
// 「被抛弃的根（实例 X、txg Y）」起头；crash_given_states.rs 的 abandoned_roots_named_in_a_pool_checker_detail 从细节里读出同一条）。
// 入口每个状态清零，走完写进结论的 ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD 起那四个字。
var<private> named_abandoned_root_present: bool;
var<private> named_abandoned_root_instance: u32;
var<private> named_abandoned_root_txg: vec2<u32>;

// 走读那一批连同收尾：择最新根、候选根、由记录施加出来的版本、被抛弃根，判走读之后那几条；
// 收尾是 F 之下、没被抛弃的根各走一遍，
// 它们引用的、候选版本没走到的槽记进交接段（第三批算 defer 那一格要），再把计数与几何写进交接段；
// 最后照下一次挂载的重放施加出那一版、另起一份走读走一遍（I-4.1），它只动上下文 1 的集合；同一份重放第一次施加的发布的末条写进交接段
// （H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED），扫描内核照它读重放读的那一份环里那一行（I-3.10、I-5.4）。
// 写成按阶段推进的一个循环：每一圈先找这一阶段下一处要走的（找不到就做完这一阶段收尾的事、转下一阶段），走，再做走完之后的事。
// ───────────────────────── I-7.18、I-7.19：最新根带的回收门槛 t 与钉槽 p（C773）─────────────────────────
// walk.rs 的 judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root：只判建账的发布（最新根那一遍走树表时记下的记账树根指针
// 出生 txg 等于最新根的 txg），最新根的实例表链没走到末片时两条都不判（CPU 报不适用）。
// I-7.18：t = max(这条根自己带的 F, min(环里按最新根的实例表仍有效的根的最小 txg, p 的钉值))（p 是 255 或不小于环长时第二项只取前者，环里一条都没有时取 F）；
// p 存槽的编码 3·槽 + 区域，钉值是那一槽可能住过的最旧 txg：槽 0 是 mkfs 种在每个区域槽 0 的第 0 代根的 0，别的槽是 p 本身
// （walk.rs 的 pinned_txg_of_the_ring_slot，判决 research/prompts/m3-close-b19-core-r1-main-verification.md 第二节 R1 采乙）；
// I-7.19：p 是 255 时成立；不是 255 又不小于环长 3S 时违例；否则还原出的槽（区域 p mod R、槽 p div R）里不许有实例代号等于最新根的根

// 走最新根那一遍记下的树表里记账树那一条的根指针出生 txg（walk.rs 的 Walk::accounting_tree_root_birth_txg）
var<private> w_newest_accounting_tree_root_birth_known: bool;
var<private> w_newest_accounting_tree_root_birth_txg: vec2<u32>;

fn judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(newest_root: CheckerRoot, g: CheckerSlot) {
    if (!w_newest_accounting_tree_root_birth_known || !eq64(w_newest_accounting_tree_root_birth_txg, newest_root.txg)) {
        return;
    }
    if (!w_newest_instance_table_chain_reached_its_last_page) {
        return;
    }
    let reclaim_threshold = rd64(newest_root.base, ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET);
    let pinned = rd8(newest_root.base, ROOT_RECORD_PINNED_RING_SLOT_OFFSET);
    let ring_length_in_txgs = g.regions * g.slots_per_region;
    let pinned_in_the_ring = pinned != ROOT_RECORD_NO_RING_SLOT_PINNED && pinned < ring_length_in_txgs;
    var have_oldest = false;
    var oldest = vec2<u32>(0u, 0u);
    for (var i = 0u; i < root_count; i = i + 1u) {
        let r = checker_root_of(i);
        if (w_root_abandoned(r.instance, r.txg)) {
            continue;
        }
        if (!have_oldest || lt64(r.txg, oldest)) {
            oldest = r.txg;
            have_oldest = true;
        }
    }
    let pinned_txg = u64_of(select(pinned, 0u, pinned / g.regions == 0u));
    if (pinned_in_the_ring && (!have_oldest || lt64(pinned_txg, oldest))) {
        oldest = pinned_txg;
        have_oldest = true;
    }
    var recomputed_threshold = newest_root.floor;
    if (have_oldest && lt64(recomputed_threshold, oldest)) {
        recomputed_threshold = oldest;
    }
    judge(INVARIANT_I_7_18, eq64(reclaim_threshold, recomputed_threshold));
    if (pinned == ROOT_RECORD_NO_RING_SLOT_PINNED) {
        judge(INVARIANT_I_7_19, true);
        return;
    }
    if (!pinned_in_the_ring) {
        judge(INVARIANT_I_7_19, false);
        return;
    }
    let pinned_region = pinned % g.regions;
    let pinned_slot = pinned / g.regions;
    var holds = true;
    for (var i = 0u; i < root_count; i = i + 1u) {
        if (checker_root_region(i) == pinned_region && checker_root_slot_in_its_region(i) == pinned_slot
            && checker_root_of(i).instance == newest_root.instance) {
            holds = false;
        }
    }
    judge(INVARIANT_I_7_19, holds);
}

// 被抛弃根那一阶段刚走完一条：它引用的槽（上下文 1 的引用集合，按跨度展开）里候选版本没走到的，去重记进 W_ABANDONED_REFERENCED_SLOTS
// （walk.rs 的 slots_referenced_by_abandoned_roots；候选版本走到的那几槽 I-3.1 本来就数了，不必记）。满了判不了（站点 117）
fn w_note_the_slots_referenced_by_the_abandoned_root_just_walked() {
    var count = sw(W_ABANDONED_REFERENCED_SLOT_COUNT);
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
            for (var q = 0u; q < count; q = q + 1u) {
                let noted = W_ABANDONED_REFERENCED_SLOTS + q * 3u;
                if (sw(noted) == device && sw(noted + 1u) == slot.x && sw(noted + 2u) == slot.y) {
                    present = true;
                }
            }
            if (present) {
                continue;
            }
            if (count >= W_MAX_ABANDONED_REFERENCED_SLOTS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (117u << 8u);
                sws(W_ABANDONED_REFERENCED_SLOT_COUNT, count);
                return;
            }
            sws(W_ABANDONED_REFERENCED_SLOTS + count * 3u, device);
            sws(W_ABANDONED_REFERENCED_SLOTS + count * 3u + 1u, slot.x);
            sws(W_ABANDONED_REFERENCED_SLOTS + count * 3u + 2u, slot.y);
            count = count + 1u;
        }
    }
    sws(W_ABANDONED_REFERENCED_SLOT_COUNT, count);
}

fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
    let device_count = hdr(3u);
    c_version_count = 0u;
    c_candidate_count = 0u;
    // 择根：(txg, 实例) 最大，相等取读根环次序里最前的（只在严格更大时才换），与 CPU 的池级 checker 择根那一处
    // （`crates/singlefs-checker/src/walk.rs` 的 fold）同一个择法
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
    w_newest_accounting_tree_root_birth_known = false;
    w_newest_accounting_tree_root_birth_txg = vec2<u32>(0u, 0u);
    sws(W_ABANDONED_REFERENCED_SLOT_COUNT, 0u);
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
                        for (var row_index = w_first_instance_row_of(r.instance); row_index != NONE;
                            row_index = w_next_instance_row_after(row_index)) {
                            let row = C_INSTANCE_ROWS + row_index * C_ROW_WORDS;
                            if (le64(r.txg, vec2<u32>(sw(row + 1u), sw(row + 2u)))) {
                                applied = true;
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
                        if (version_count >= R_DATA_WORDS) {
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
                // I-5.7：由记录施加出来的那几版（walk.rs 照同一份版本清单判）；点名项取同一 (实例, txg) 的记录里有没有一条非空
                for (var v = 0u; v < version_count; v = v + 1u) {
                    let version_record = checker_record_at(sw(R_DATA + v));
                    var publish_names_some_unit = false;
                    for (var k = 0u; k < record_count; k = k + 1u) {
                        let other = checker_record_at(k);
                        if (other.instance == version_record.instance && eq64(other.txg, version_record.txg) && other.named_count != 0u) {
                            publish_names_some_unit = true;
                        }
                    }
                    judge_the_allocation_record_tree_root_born_with_the_version(version_record.base, publish_names_some_unit);
                }
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
                    // inode → 对象出生代那一张一个号至多一行（记的时候同号就地盖掉）
                    let birth_row_index = w_inode_birth_row_of(inode);
                    if (birth_row_index != NONE) {
                        let birth_row = C_INODE_BIRTHS + birth_row_index * 4u;
                        let record_birth = vec2<u32>(sw(birth_row + 2u), sw(birth_row + 3u));
                        judge(INVARIANT_I_9_10, eq64(record_birth, unit_birth));
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
                        for (var row_index = w_first_instance_row_of(write_order_instance); row_index != NONE;
                            row_index = w_next_instance_row_after(row_index)) {
                            let row_txg = vec2<u32>(sw(C_INSTANCE_ROWS + row_index * C_ROW_WORDS + 1u),
                                sw(C_INSTANCE_ROWS + row_index * C_ROW_WORDS + 2u));
                            if (!is_zero64(row_txg) && lt64(row_txg, birth_txg)) {
                                written_after_applied = true;
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
            // 扫描内核 I-7.9 按 journal 重建抬 F 那一刻的根环（C717 丁）读这一份环的条数与根环每一槽的样子
            sws(H_BASE + H_REPLAY_RECORD_COUNT, c_replay_record_count);
            for (var position = 0u; position < g.regions * g.slots_per_region; position = position + 1u) {
                sws(H_ROOT_RING_SLOT_READINGS + position, root_ring_slot_readings[position].x);
            }
            if (undecidable == 0u) {
                // I-3.19 读重放刚收的那一份环与主走读记下的实例表行
                judge_the_applied_transaction_high_water_of_the_instance_table_rows(g);
                // I-5.7：下一次挂载先施加的那一版（walk.rs version_the_next_mount_applies_first）；点名项取同一 (实例, txg) 的记录里有没有一条非空
                let first_publish = replay.last_record_of_the_first_publish_applied;
                if (first_publish != NONE) {
                    let first_record = replay_record_at(first_publish);
                    var publish_names_some_unit = false;
                    for (var k = replay_records_of_the_instance_from(first_record.instance); k < c_replay_record_count; k = k + 1u) {
                        if (sw(C_REPLAY_RECORDS + k * C_REPLAY_RECORD_WORDS + 3u) != first_record.instance) {
                            break;
                        }
                        let other = replay_record_at(k);
                        if (eq64(other.txg, first_record.txg) && other.named_count != 0u) {
                            publish_names_some_unit = true;
                        }
                    }
                    judge_the_allocation_record_tree_root_born_with_the_version(first_record.base, publish_names_some_unit);
                }
            }
            let last_applied = replay.last_record_of_the_last_publish_applied;
            if (last_applied == NONE || undecidable != 0u) {
                phase = WALK_PHASE_DONE;
            } else {
                w_start(1u, false);
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
            appearance_note_the_version_just_walked(walked_instance, w_version_txg, w_failures == failures_before && !appearance_tree_table_of_this_version_has_an_entry_without_a_clause, appearance_instance_table_pointer_is_born_with_the_version);
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
            judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(newest_root, g);
            phase = WALK_PHASE_CANDIDATE_ROOTS;
            cursor = 0u;
        } else if (phase == WALK_PHASE_CANDIDATE_ROOTS || phase == WALK_PHASE_VERSIONS_APPLIED_ONLY_BY_RECORDS) {
            let walked_into = w_mismatches > mismatches_before || w_failures > failures_before;
            w_judge(INVARIANT_I_7_4, !walked_into);
            w_judge(INVARIANT_I_4_8, !walked_into);
        } else if (phase == WALK_PHASE_ABANDONED_ROOTS) {
            let walked_into_a_reallocated_or_erased_unit = w_failures > 0u || w_mismatches > 0u;
            // CPU 池级 checker 一条不变量只记第一处违例的说明：I-7.4 还没红过、这条被抛弃的根是第一处违例时，细节点名的就是它
            let reuse_invariant_was_violated_before = (violated[INVARIANT_I_7_4 / 32u] & (1u << (INVARIANT_I_7_4 % 32u))) != 0u;
            if (walked_into_a_reallocated_or_erased_unit && !reuse_invariant_was_violated_before) {
                // 这一条根是这一阶段刚取的那一条：取的时候游标已经挪到它后面
                let abandoned = checker_root_of(cursor - 1u);
                named_abandoned_root_present = true;
                named_abandoned_root_instance = abandoned.instance;
                named_abandoned_root_txg = abandoned.txg;
            }
            judge(INVARIANT_I_7_4, !walked_into_a_reallocated_or_erased_unit);
            w_note_the_slots_referenced_by_the_abandoned_root_just_walked();
            if (undecidable != 0u) {
                return WALK_REACHED_THE_ROOT_RING;
            }
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

// 一段字节 [start, end) 全 0：先逐字节读到字边界，再按字读，末尾不满一字的再逐字节读（槽尾 3607 字节逐字节读要三千多次 rd8，按字读九百来次）
fn bytes_from_start_to_end_are_zero(base: u32, start: u32, end: u32) -> bool {
    var offset = start;
    while (offset < end && (offset & 3u) != 0u) {
        if (rd8(base, offset) != 0u) {
            return false;
        }
        offset = offset + 1u;
    }
    while (offset + 4u <= end) {
        if (tables[base + (offset >> 2u)] != 0u) {
            return false;
        }
        offset = offset + 4u;
    }
    while (offset < end) {
        if (rd8(base, offset) != 0u) {
            return false;
        }
        offset = offset + 1u;
    }
    return true;
}

// 系统配置字段表的字节数（singlefs_format::SYSTEM_CONFIGURATION_BYTES，D22（单元原子性怎么合成） 已定项 9「合计 489 字节」）：槽尾从这里起
const SYSTEM_CONFIGURATION_FIELD_TABLE_BYTES: u32 = 489u;

// I-2.4 系统配置槽尾那一处：每块盘两槽里每个可择的槽，字段表之后到槽末 [489, 4096) 全 0，每槽判一格
// （image::judge_system_configuration_slot_bytes_past_the_field_table_are_zero，C728「恒 0，由 checker 判」）。只判不断
fn judge_system_configuration_slot_bytes_past_the_field_table_are_zero(device_count: u32) {
    for (var d = 0u; d < device_count; d = d + 1u) {
        for (var k = 0u; k < 2u; k = k + 1u) {
            let slot = slot_readings[d * 2u + k];
            if (slot.reading == SLOT_USABLE) {
                judge(INVARIANT_I_2_4, bytes_from_start_to_end_are_zero(slot.slot_bytes_base, SYSTEM_CONFIGURATION_FIELD_TABLE_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES));
            }
        }
    }
}

// I-7.5：根环 R × S 个槽逐个判，起始偏移（设备内字节）是判定宽度的整数倍（image::judge_root_slots_aligned_to_the_decision_width）。
// 偏移照 image::root_slot_positions 按 64 位算：根环起点 × 16384 + 区域 × P × chunk（区域间距按常量 ROOT_RING_REGION_STRIDE 放，读槽时判过
// 两个字段等于第一版常量）+ 槽 × 槽距。
// 判定宽度 0 不会出现（择到的槽 physical_block_size ≥ 466），出现了这一格判红、不除。
fn judge_root_slots_aligned_to_the_decision_width(g: CheckerSlot, decision_width: u32) {
    let root_ring_base_bytes = slot_to_bytes(g.base_slot);
    for (var region = 0u; region < g.regions; region = region + 1u) {
        let region_bytes = u64_of(region * ROOT_RING_REGION_STRIDE);
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

// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节的起点与字节数（D22（单元原子性怎么合成） 已定项 7 的字段表：回收门槛 t 与钉槽 p 之后，偏移 437、438、450，
// 合计到 466；image::ROOT_RECORD_ALGORITHM_NONCE_AND_MAC 是 437..466）
const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_OFFSET: u32 = 437u;
const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_BYTES: u32 = 29u;

// I-2.4 根记录那一处：根环里每一条自证过的根（checker_collect_roots，不按候选集滤），根槽 437..466 全 0
// （image::judge_root_record_algorithm_nonce_and_mac_are_zero；这一版 checker 只收加密类型 0 的系统配置，加密关着时恒 0）。只判不断
fn judge_root_record_algorithm_nonce_and_mac_are_zero() {
    for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
        judge(INVARIANT_I_2_4, all_zero(checker_root_of(root_index).base, ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_OFFSET, ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_BYTES));
    }
}

// I-2.4 根槽尾那一处：根环里每一条自证过的根（checker_collect_roots，不按候选集滤），根记录之后到槽宽 [466, physical_block_size) 全 0，
// 每条根判一格（image::judge_root_slot_bytes_past_the_record_are_zero；槽宽取择到的那一槽的 physical_block_size，checker_collect_roots
// 按它读根槽，C728「恒 0，由 checker 判」）。只判不断
fn judge_root_slot_bytes_past_the_record_are_zero(root_slot_bytes: u32) {
    for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
        judge(INVARIANT_I_2_4, bytes_from_start_to_end_are_zero(checker_root_of(root_index).base, ROOT_RECORD_BYTES, root_slot_bytes));
    }
}

// check_pool_image 从读系统配置槽到走读那一段，交回结论里「走到了第几步」。整池不作保的两支先判：任一盘任一槽带这一版读者不收的值
// （I-7.13）、任一盘任一可择槽 I-8.1 的三个字段有一格不等（两支都中时走 I-7.13 那一支，image 与 walk 的次序）——中了就只报那一条、
// 别的不判；再 I-8.5（全池没有一个自证过的槽）、择系统配置、系统配置与根环那几条、逐盘 journal，根环里有根才进走读。
// 判到位跟 CPU 的三支走：带越界值那一支只交 I-7.13 那一份判定；I-8.1 三个字段不等那一支 CPU 另起一份判定、只交它（这里清掉之前记的评估过）；
// 一块盘都择不到时 I-8.5 只有判红才算判到（那一支 CPU 判过而成立报不适用）。
// 整池交回的四处（带越界值、环几何三格不等、一块盘都择不到、根环里没有根）CPU 对每条不变量都报那一句不适用、不走到扫描重建，
// I-1.13 的结局码在那几处记进 scan_rebuild_outcome_of_the_whole_pool（入口照它写结论的 SCAN_REBUILD_OUTCOME_WORD，别的状态它是没走到）
var<private> scan_rebuild_outcome_of_the_whole_pool: u32;

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
        scan_rebuild_outcome_of_the_whole_pool = SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_A_SYSTEM_CONFIGURATION_VALUE_THE_READER_DOES_NOT_ACCEPT;
        return 1u;
    }
    for (var d = 0u; d < device_count; d = d + 1u) {
        for (var k = 0u; k < 2u; k = k + 1u) {
            let slot = slot_readings[d * 2u + k];
            if (slot.reading == SLOT_USABLE && !journal_ring_geometry_fields_agree(slot)) {
                clear_judged();
                judge(INVARIANT_I_8_1, false);
                scan_rebuild_outcome_of_the_whole_pool = SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_UNEQUAL_JOURNAL_RING_GEOMETRY_FIELDS;
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
        // I-1.5（walk.rs 择系统配置那一段）：择到的那一槽自举头里写入者身份的实现标识（fsid 之后那 16 字节）不全零。槽的 4096 字节就在判表里
        // （slot_bytes_base），不另带字段
        judge(INVARIANT_I_1_5, !all_zero(chosen.slot_bytes_base, SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER_OFFSET,
            SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER_BYTES));
    }
    if (!chosen_valid) {
        scan_rebuild_outcome_of_the_whole_pool = SCAN_REBUILD_OUTCOME_NO_DEVICE_YIELDS_A_VALID_SYSTEM_CONFIGURATION;
        return 0u;
    }
    judge(INVARIANT_I_8_5, some_slot_verifies_itself);
    judge_system_configuration_slot_generations_follow_the_rotation(device_count);
    judge_system_configuration_slot_bytes_past_the_field_table_are_zero(device_count);
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
    judge_root_slot_bytes_past_the_record_are_zero(g.physical_block_size);
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
    // walk::judge_instance_carriers：① 读不全（some_slot_is_unreadable）时报射程里一部分没判、不判；② 读不全时只在找到较大者时判（红），
    // 读得全时照判。读不全只出在根环槽上（checker_some_root_slot_is_unreadable）：journal 记录槽与单元头都在池里的盘上读、在盘内
    if (every_device_has_a_slot) {
        let some_root_slot_is_unreadable = checker_some_root_slot_is_unreadable(g);
        if (some_root_slot_is_unreadable) {
            mark_part_of_the_range_not_judged(INVARIANT_I_7_7);
        } else {
            judge(INVARIANT_I_7_7, !some_instance_carrier_exceeds(pool_highest, g));
        }
        let witnessed_by_one_device = pool_lowest < pool_highest && some_instance_carrier_exceeds(pool_lowest, g);
        if (witnessed_by_one_device || !some_root_slot_is_unreadable) {
            judge(INVARIANT_I_7_7, !witnessed_by_one_device);
        }
    }
    // journal 那几条与点名项的位置条目次序，逐盘
    w_some_journal_slot_does_not_read_back = false;
    for (var d = 0u; d < device_count; d = d + 1u) {
        checker_scan_journal_of_device(hdr(12u + d), fsid_low, shr64(g.ring_bytes, 12u));
        judge_journal_back_chain_of_device();
        judge_named_entry_location_order_of_device();
        judge_transaction_numbers_of_device();
        judge_transaction_numbers_start_from_one_of_device(shr64(g.ring_bytes, 12u));
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
    // I-8.14（walk.rs 在逐盘 journal 那一段之后、I-7.1 之前判；根环全灭的镜像上照判）
    judge_the_first_txg_of_each_instance_above_the_older_instances(g, fsid_low);
    // I-7.1 / I-7.3
    judge(INVARIANT_I_7_1, root_count != 0u);
    if (root_count == 0u) {
        scan_rebuild_outcome_of_the_whole_pool = SCAN_REBUILD_OUTCOME_ROOT_RING_HOLDS_NO_SELF_VERIFIED_ROOT;
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
    named_abandoned_root_present = false;
    named_abandoned_root_instance = 0u;
    named_abandoned_root_txg = vec2<u32>(0u, 0u);
    let out = local * VERDICT_WORDS;
    sws(H_BASE + H_VALID, 0u);
    let device_count = hdr(3u);
    var reached = 0u;
    scan_rebuild_outcome_of_the_whole_pool = SCAN_REBUILD_OUTCOME_NOT_REACHED;
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
    verdicts[out + SCAN_REBUILD_OUTCOME_WORD] = scan_rebuild_outcome_of_the_whole_pool;
    verdicts[out + SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD] = self_verified_system_configuration_slots_of_each_device(device_count);
    if (named_abandoned_root_present) {
        verdicts[out + ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD] = ABANDONED_ROOT_NAMED;
    } else {
        verdicts[out + ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD] = NO_ABANDONED_ROOT_NAMED;
    }
    verdicts[out + ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD] = named_abandoned_root_instance;
    verdicts[out + ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD] = named_abandoned_root_txg.x;
    verdicts[out + ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD] = named_abandoned_root_txg.y;
}
