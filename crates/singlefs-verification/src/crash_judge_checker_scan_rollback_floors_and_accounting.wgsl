// newest_first_distinct_state_txgs 与 rollback_floor_ceiling_from 合在一起：按 (txg, 实例) 从新到旧计入，交回上限；
// 另记 c_distinct_non_empty_states_newer_than_the_oldest（最旧有效根是 (oldest, oldest_instance)）
fn c_rollback_floor_ceiling(non_empty_count: u32, undeterminable_count: u32, newest_on_every_device: vec2<u32>, oldest: vec2<u32>, oldest_instance: u32) -> vec2<u32> {
    var counted = 0u;
    var counted_states = 0u;
    c_distinct_non_empty_states_newer_than_the_oldest = 0u;
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
            if (!eq64(best_txg, oldest) || best_instance != oldest_instance) {
                c_distinct_non_empty_states_newer_than_the_oldest = c_distinct_non_empty_states_newer_than_the_oldest + 1u;
            }
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

// 抬 F 那一刻一个根环槽里可能住的第几种根（walk.rs 的 possible_roots_in_the_ring_slot_at_the_raise 交回的那一串里的一项）的三种结局
const RING_SLOT_AT_THE_RAISE_HOLDS_A_ROOT: u32 = 0u;

const RING_SLOT_AT_THE_RAISE_EMPTY: u32 = 1u;

const RING_SLOT_AT_THE_RAISE_NOT_RECOGNIZED: u32 = 2u;

// 按 journal 重建抬 F 那一刻的根环时每一槽可能住户的组合至多判这么多种，多于它这条根记射程里判不了（walk.rs 的
// POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST，判决 research/prompts/m3-close-b19-reason-r1-main-verification.md 第二节 Q2「定」那一行要的上界）
const C_POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST: u32 = 4096u;

// 重建那几遍里每一槽可能住户的种数（下标 区域 × S + 槽，至多 MAX_ROOTS 槽）：第一遍认每一槽时记，之后几遍按它拆出每一槽取第几种。
// 放在交接的根环槽样子那一张（H_ROOT_RING_SLOT_READINGS）之后、C_APPEARANCE_PLACEMENTS 之前空着的那一段里（原来的 C_REPLAY_RECORDS 挪走之后空着）
const C_POSSIBLE_ROOT_COUNTS_OF_THE_RING_SLOTS: u32 = H_ROOT_RING_SLOT_READINGS + MAX_ROOTS;

const_assert C_POSSIBLE_ROOT_COUNTS_OF_THE_RING_SLOTS + MAX_ROOTS <= C_APPEARANCE_PLACEMENTS;

// 结局是 RING_SLOT_AT_THE_RAISE_HOLDS_A_ROOT 时那条根：(txg, 实例)、它那一槽的区域所在的盘、读它树表指针的那一份字节的基与指针在里面的偏移
// （基是 NONE 的那一条是已被盖掉的第 0 代根，c_user_visible_pointers 照 mkfs 种的 0 条树表记两棵用户可见树都没有）；possible_root_count 是这一槽可能住户的种数
struct RingSlotRootAtTheRaise {
    outcome: u32,
    txg: vec2<u32>,
    instance: u32,
    device: u32,
    unit_base: u32,
    tree_table_pointer_offset: u32,
    possible_root_count: u32,
};

// C717（回退下界上限的单窗口盲带）丁（「丁：借 journal 判红」），walk.rs 的 possible_roots_in_the_ring_slot_at_the_raise 与
// ring_slots_of_the_image_seen_from_the_raise 合在一起：根环槽（区域 region、槽 slot）在抬 F 那一刻可能住的第 choice 种根（不判有效），连同这一槽一共几种。
// 读交接过去的那一槽的样子（H_ROOT_RING_SLOT_READINGS）：全零 ⇒ 空着（一种）；读不出 ⇒ 认不出；自证过而根表里没有这一槽的根 ⇒ 照读不出办、认不出；
// 坐着 txg 比抬 F 那条根小的根 ⇒ 就是它（一种，树表指针在根记录偏移 36）；坐着 txg 不小于它的根（抬 F 之后被盖掉）⇒ 从落点式在它之下最大的正 txg u
// （walk.rs 的 largest_positive_txg_the_ring_slot_names_below：u = (r − 1) − ((r − 1 − 余数) mod 3S)，余数是 区域 + R × 槽）起往下逐层认
// （判决 research/prompts/m3-close-b19-reason-r1-main-verification.md 第二节 Q2「定」那一行），每一层一种：
// 在重放读的那一份环（C_REPLAY_RECORDS 前 H_REPLAY_RECORD_COUNT 条）里找那一层带提交标记的记录，要恰好一个实例，那个实例的记录取带末条标志的那一条、
// 没有就取先读到的那一条（表按 (实例, 计数器) 升序，同 walk.rs 的 versions_named_by_the_journal_at_the_txgs），找不到或有几个实例 ⇒ 认不出；
// 它与抬 F 那条根同一实例，或它那个实例在抬 F 那条根的实例表（C_R_ROWS，调用方先 r_instance_rows 读好）里有行、每一行都有 u < T，根槽落了盘、到此为止，
// 否则（u ≥ T、没有行）它的根槽可以没落，再往下认上一圈 u − 3S。落点式住进这一槽的正 txg 认完了：槽 0 而抬之前的 F 是 0 时只算第 0 代根一种
// （基记成 NONE，c_user_visible_pointers 照 mkfs 种的 0 条树表记两棵都没有），别的槽空着一种；认到的那一层低于抬之前的 F（窗口外）：记一种空着、到此为止。
// 每一层都要有 journal 里的记录，层数不超过重放读的条数加二
fn c_root_in_the_ring_slot_at_the_raise(region: u32, slot: u32, raising: CheckerRoot, floor_before: vec2<u32>, choice: u32) -> RingSlotRootAtTheRaise {
    let regions = sw(H_BASE + H_REGIONS);
    let slots_per_region = sw(H_BASE + H_SLOTS_PER_REGION);
    var seen: RingSlotRootAtTheRaise;
    seen.outcome = RING_SLOT_AT_THE_RAISE_NOT_RECOGNIZED;
    seen.txg = vec2<u32>(0u, 0u);
    seen.instance = 0u;
    seen.device = handoff_region_device(region);
    seen.unit_base = 0u;
    seen.tree_table_pointer_offset = 0u;
    seen.possible_root_count = 1u;
    let reading = sw(H_ROOT_RING_SLOT_READINGS + region * slots_per_region + slot);
    if (reading == ROOT_RING_SLOT_ALL_ZERO) {
        seen.outcome = RING_SLOT_AT_THE_RAISE_EMPTY;
        return seen;
    }
    if (reading != ROOT_RING_SLOT_SELF_VERIFIED) {
        return seen;
    }
    var root_index = NONE;
    for (var root_table_index = 0u; root_table_index < root_count; root_table_index = root_table_index + 1u) {
        if (checker_root_region(root_table_index) == region && checker_root_slot_in_its_region(root_table_index) == slot) {
            root_index = root_table_index;
            break;
        }
    }
    if (root_index == NONE) {
        return seen;
    }
    let root = checker_root_of(root_index);
    if (lt64(root.txg, raising.txg)) {
        seen.outcome = RING_SLOT_AT_THE_RAISE_HOLDS_A_ROOT;
        seen.txg = root.txg;
        seen.instance = root.instance;
        seen.unit_base = root.base;
        seen.tree_table_pointer_offset = 36u;
        return seen;
    }
    let ring_length = u64_of(regions * slots_per_region);
    let residue = u64_of(region + regions * slot);
    var overwritten_txg = vec2<u32>(0u, 0u);
    if (!is_zero64(raising.txg)) {
        let below = sub64(raising.txg, u64_of(1u));
        if (!lt64(below, residue)) {
            overwritten_txg = sub64(below, divmod64(sub64(below, residue), ring_length).remainder);
        }
    }
    let replay_record_count = sw(H_BASE + H_REPLAY_RECORD_COUNT);
    // level：已经数过的可能住户种数（下一种是第 level 种）
    var level = 0u;
    loop {
        if (is_zero64(overwritten_txg)) {
            // 槽 0 而抬之前的 F 是 0：mkfs 在每个区域的槽 0 都种了第 0 代根，只算它一种、不另算空着（判决
            // research/prompts/m3-close-b19-reason-r2-main-verification.md 第二节 Q3 第 3 问「定」）；别的槽空着一种
            if (slot == 0u && is_zero64(floor_before)) {
                if (choice == level) {
                    seen.outcome = RING_SLOT_AT_THE_RAISE_HOLDS_A_ROOT;
                    seen.txg = vec2<u32>(0u, 0u);
                    seen.instance = 0u;
                    seen.unit_base = NONE;
                }
            } else if (choice == level) {
                seen.outcome = RING_SLOT_AT_THE_RAISE_EMPTY;
            }
            level = level + 1u;
            break;
        }
        if (lt64(overwritten_txg, floor_before)) {
            if (choice == level) {
                seen.outcome = RING_SLOT_AT_THE_RAISE_EMPTY;
            }
            level = level + 1u;
            break;
        }
        var version_instance = NONE;
        var version_base = 0u;
        var version_taken_from_the_last_record = false;
        var more_than_one_instance = false;
        for (var position = 0u; position < replay_record_count; position = position + 1u) {
            let record = replay_record_at(position);
            if (record.commit_byte != 1u || !eq64(record.txg, overwritten_txg)) {
                continue;
            }
            let carries_the_last_record_flag = (record.flags & 1u) != 0u;
            if (version_instance == NONE) {
                version_instance = record.instance;
                version_base = record.base;
                version_taken_from_the_last_record = carries_the_last_record_flag;
            } else if (record.instance != version_instance) {
                more_than_one_instance = true;
            } else if (!version_taken_from_the_last_record && carries_the_last_record_flag) {
                version_base = record.base;
                version_taken_from_the_last_record = true;
            }
        }
        if (version_instance == NONE || more_than_one_instance) {
            seen.outcome = RING_SLOT_AT_THE_RAISE_NOT_RECOGNIZED;
            return seen;
        }
        var its_root_landed = version_instance == raising.instance;
        if (!its_root_landed) {
            var its_instance_has_a_row = false;
            var every_row_of_its_instance_is_after_it = true;
            for (var row_index = 0u; row_index < c_r_row_count; row_index = row_index + 1u) {
                if (sw(C_R_ROWS + row_index * 3u) != version_instance) {
                    continue;
                }
                its_instance_has_a_row = true;
                if (!lt64(overwritten_txg, vec2<u32>(sw(C_R_ROWS + row_index * 3u + 1u), sw(C_R_ROWS + row_index * 3u + 2u)))) {
                    every_row_of_its_instance_is_after_it = false;
                }
            }
            its_root_landed = its_instance_has_a_row && every_row_of_its_instance_is_after_it;
        }
        if (choice == level) {
            seen.outcome = RING_SLOT_AT_THE_RAISE_HOLDS_A_ROOT;
            seen.txg = overwritten_txg;
            seen.instance = version_instance;
            seen.unit_base = version_base;
            seen.tree_table_pointer_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
        }
        level = level + 1u;
        if (its_root_landed) {
            break;
        }
        // 它的根槽可以没落：那一槽仍住着落点式住进它的上一圈那一条（或更早的），往下认
        if (le64(overwritten_txg, ring_length)) {
            overwritten_txg = vec2<u32>(0u, 0u);
        } else {
            overwritten_txg = sub64(overwritten_txg, ring_length);
        }
    }
    seen.possible_root_count = level;
    return seen;
}

// I-7.9 一条有效根那一行（C_VALID_BEFORE_IDENTITIES 里第 valid_root_index 行）的 txg
fn c_valid_root_before_the_raise_txg(valid_root_index: u32) -> vec2<u32> {
    let identity = C_VALID_BEFORE_IDENTITIES + valid_root_index * C_VALID_BEFORE_IDENTITY_WORDS;
    return vec2<u32>(sw(identity), sw(identity + 1u));
}

// I-7.9：只判抬 F 的那一条根，拿它之前的根算上限。两支的「之前的根」都只取 txg 比它小、落在锚在这张镜像根环里最新那条根的 txg 上的
// 一圈窗口里的根（候选过滤那一个；CPU 一侧 walk.rs 同一个口径）。锚在被判那条根自己的 txg 上的那一个窗口不取：最新那条根的 txg 不小于
// 根表里每一条根的（walk_and_hand_off 择 (txg, 实例) 最大的），锚在它上面的窗口本来就落在另一个里面，两个的交就是它，拿掉不改任何判定。
// 不带卸载记号那一支分两遍（walk.rs 的 judge_rollback_floor_raises_against_their_ceilings）：窗口那一遍判成立而窗口定不住抬 F 那一刻的上限
// （walk.rs 的 window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise：根环每个区域所在的盘上窗口里都有有效根、去掉最旧那条之后
// 非空有效根去重仍计入不少于 4 条，两样有一样不成立）时，再按 journal 重建抬 F 那一刻的根环（C717 丁，c_root_in_the_ring_slot_at_the_raise）
// 照同一个上限式子再判：重建出的环判成立、违例照判；有一槽认不出、一条有效根都没有、F 落在上限两沿之间记射程里判不了。
// 被盖那几槽的住户落没落盘判不了时可能的环有几种（walk.rs 的 rollback_floor_against_the_ring_reconstructed_at_the_raise）：重建那一遍拆成几遍，
// 每一遍按混合进制（每一槽一位、那一槽的种数是这一位的进制）取一种可能的环各判一次，种数在第一遍认每一槽时定、多于
// C_POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST 记射程里判不了；每一种都成立才判成立、都越过上限才判红，有一种两沿之间、没有有效根，
// 或有成立有越过的，记射程里判不了。
// 每一遍走同一段上限代码（读树表的 c_user_visible_pointers、算上限的 c_rollback_floor_ceiling、认一槽的 c_root_in_the_ring_slot_at_the_raise
// 各只有这一处调用点，展开预算不随遍数涨）
fn judge_rollback_floor_raises_against_their_ceilings() {
    let regions = sw(H_BASE + H_REGIONS);
    let slots_per_region = sw(H_BASE + H_SLOTS_PER_REGION);
    let root_ring_slot_count = regions * slots_per_region;
    let newest_txg_of_the_image = checker_root_of(c_newest_index).txg;
    for (var i = 0u; i < root_count; i = i + 1u) {
        let raising = checker_root_of(i);
        // 同一实例里 txg 比它小的最新那条根
        var have_previous = false;
        var previous_txg = vec2<u32>(0u, 0u);
        var floor_before = vec2<u32>(0u, 0u);
        // 环里这个实例最旧的那条根（含它自己）：读回全零的槽只认能住的 txg 比它小的（walk.rs 的 oldest_txg_of_the_raising_instance_in_the_ring）
        var oldest_txg_of_the_raising_instance = raising.txg;
        for (var k = 0u; k < root_count; k = k + 1u) {
            let root = checker_root_of(k);
            if (root.instance == raising.instance && lt64(root.txg, raising.txg) && (!have_previous || le64(previous_txg, root.txg))) {
                previous_txg = root.txg;
                floor_before = root.floor;
                have_previous = true;
            }
            if (root.instance == raising.instance && lt64(root.txg, oldest_txg_of_the_raising_instance)) {
                oldest_txg_of_the_raising_instance = root.txg;
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
        // 读回全零的根环槽并进上限（walk.rs rollback_floor_ceiling_held_by_the_ring_slots_reading_back_all_zero，C961（全零根槽没并进 checker 的抬 F 上限），
        // D16（发布语义） 已定项 1「抬 F 的上限」那一行）：每个读回全零的槽取落点式住进它、比抬 F 那条根小的最大正 txg（余数是 区域 + R × 槽），
        // 比环里这个实例最旧的那条根小、又不低于抬之前的 F 的才算；F 越过其中任一个就判红，不再往下看窗口与重建的环
        var floor_passes_a_ring_slot_reading_back_all_zero = false;
        if (!is_zero64(raising.txg)) {
            let below_the_raise = sub64(raising.txg, u64_of(1u));
            for (var ring_slot_index = 0u; ring_slot_index < root_ring_slot_count; ring_slot_index = ring_slot_index + 1u) {
                let region = ring_slot_index / slots_per_region;
                let slot = ring_slot_index % slots_per_region;
                let residue = u64_of(region + regions * slot);
                if (sw(H_ROOT_RING_SLOT_READINGS + region * slots_per_region + slot) != ROOT_RING_SLOT_ALL_ZERO || lt64(below_the_raise, residue)) {
                    continue;
                }
                let largest_txg_the_slot_can_hold = sub64(below_the_raise,
                    divmod64(sub64(below_the_raise, residue), u64_of(root_ring_slot_count)).remainder);
                if (!is_zero64(largest_txg_the_slot_can_hold) && lt64(largest_txg_the_slot_can_hold, oldest_txg_of_the_raising_instance)
                    && !lt64(largest_txg_the_slot_can_hold, floor_before) && lt64(largest_txg_the_slot_can_hold, raising.floor)) {
                    floor_passes_a_ring_slot_reading_back_all_zero = true;
                }
            }
        }
        if (floor_passes_a_ring_slot_reading_back_all_zero) {
            judge(INVARIANT_I_7_9, false);
            continue;
        }
        if (!r_instance_rows(raising.base)) {
            continue;
        }
        // 先取窗口里的根（reconstruction_round 为 0）；窗口那一遍判成立而窗口定不住上限时，再取按 journal 重建的抬 F 那一刻的根环
        // （为 1 起，第 reconstruction_round − 1 种可能的环）。possible_ring_count 在第一种可能的环那一遍认完每一槽时定
        var possible_ring_count = 1u;
        // 已判过的可能的环里有成立的（位 0）、有越过上限的（位 1）
        var possible_ring_verdicts = 0u;
        for (var reconstruction_round = 0u; reconstruction_round <= possible_ring_count; reconstruction_round = reconstruction_round + 1u) {
            let reconstructing_the_ring = reconstruction_round >= 1u;
            // 这一种可能的环按混合进制还没拆的那几位
            var digits_left = 0u;
            if (reconstruction_round > 1u) {
                digits_left = reconstruction_round - 1u;
            }
            // 它之前的有效根：txg ≥ 抬之前的 F、按它自己的实例表不被抛弃；窗口那一遍另要 txg < 它的、在一圈窗口里。按 (txg, 实例) 升序
            var valid_count = 0u;
            var ring_reconstructed = true;
            var candidate_count = root_count;
            if (reconstructing_the_ring) {
                candidate_count = root_ring_slot_count;
            }
            for (var candidate_index = 0u; candidate_index < candidate_count; candidate_index = candidate_index + 1u) {
                var candidate_txg = vec2<u32>(0u, 0u);
                var candidate_instance = 0u;
                var candidate_device = 0u;
                var candidate_unit_base = 0u;
                var candidate_tree_table_pointer_offset = 0u;
                if (reconstructing_the_ring) {
                    var choice = 0u;
                    if (reconstruction_round > 1u) {
                        let possible_root_count = sw(C_POSSIBLE_ROOT_COUNTS_OF_THE_RING_SLOTS + candidate_index);
                        choice = digits_left % possible_root_count;
                        digits_left = digits_left / possible_root_count;
                    }
                    let seen = c_root_in_the_ring_slot_at_the_raise(candidate_index / slots_per_region, candidate_index % slots_per_region, raising, floor_before,
                        choice);
                    if (seen.outcome == RING_SLOT_AT_THE_RAISE_NOT_RECOGNIZED) {
                        ring_reconstructed = false;
                        break;
                    }
                    if (reconstruction_round == 1u) {
                        sws(C_POSSIBLE_ROOT_COUNTS_OF_THE_RING_SLOTS + candidate_index, seen.possible_root_count);
                        possible_ring_count = possible_ring_count * seen.possible_root_count;
                        if (possible_ring_count > C_POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST) {
                            ring_reconstructed = false;
                            break;
                        }
                    }
                    if (seen.outcome == RING_SLOT_AT_THE_RAISE_EMPTY) {
                        continue;
                    }
                    candidate_txg = seen.txg;
                    candidate_instance = seen.instance;
                    candidate_device = seen.device;
                    candidate_unit_base = seen.unit_base;
                    candidate_tree_table_pointer_offset = seen.tree_table_pointer_offset;
                } else {
                    let root = checker_root_of(candidate_index);
                    if (!lt64(root.txg, raising.txg) || !in_the_one_revolution_window(root.txg, newest_txg_of_the_image, root_ring_slot_count)) {
                        continue;
                    }
                    candidate_txg = root.txg;
                    candidate_instance = root.instance;
                    candidate_device = handoff_region_device(checker_root_region(candidate_index));
                    candidate_unit_base = root.base;
                    candidate_tree_table_pointer_offset = 36u;
                }
                if (lt64(candidate_txg, floor_before) || r_rows_abandon(candidate_instance, candidate_txg)) {
                    continue;
                }
                var position = 0u;
                while (position < valid_count) {
                    if (root_key_less(candidate_txg, candidate_instance, c_valid_root_before_the_raise_txg(position),
                                      sw(C_VALID_BEFORE_IDENTITIES + position * C_VALID_BEFORE_IDENTITY_WORDS + 2u))) {
                        break;
                    }
                    position = position + 1u;
                }
                var shifted_row_index = valid_count;
                while (shifted_row_index > position) {
                    for (var word = 0u; word < C_VALID_BEFORE_IDENTITY_WORDS; word = word + 1u) {
                        sws(C_VALID_BEFORE_IDENTITIES + shifted_row_index * C_VALID_BEFORE_IDENTITY_WORDS + word,
                            sw(C_VALID_BEFORE_IDENTITIES + (shifted_row_index - 1u) * C_VALID_BEFORE_IDENTITY_WORDS + word));
                    }
                    shifted_row_index = shifted_row_index - 1u;
                }
                let inserted = C_VALID_BEFORE_IDENTITIES + position * C_VALID_BEFORE_IDENTITY_WORDS;
                sws(inserted, candidate_txg.x);
                sws(inserted + 1u, candidate_txg.y);
                sws(inserted + 2u, candidate_instance);
                sws(inserted + 3u, candidate_device);
                sws(inserted + 4u, candidate_unit_base);
                sws(inserted + 5u, candidate_tree_table_pointer_offset);
                valid_count = valid_count + 1u;
            }
            if (!ring_reconstructed || valid_count == 0u) {
                // 窗口里一条有效根都没有：上限无从算起，这条根不判；重建的环认不出、可能的环多于上界、或这一种一条有效根都没有：射程里判不了
                if (reconstructing_the_ring) {
                    mark_part_of_the_range_not_judged(INVARIANT_I_7_9);
                }
                break;
            }
            let oldest = c_valid_root_before_the_raise_txg(0u);
            let oldest_instance = sw(C_VALID_BEFORE_IDENTITIES + 2u);
            // 每块盘上最新的有效根里最旧的那个（盘取它那一槽的区域所在的盘）
            var have_newest_on_every = false;
            var newest_on_every = vec2<u32>(0u, 0u);
            for (var valid_root_index = 0u; valid_root_index < valid_count; valid_root_index = valid_root_index + 1u) {
                let device = sw(C_VALID_BEFORE_IDENTITIES + valid_root_index * C_VALID_BEFORE_IDENTITY_WORDS + 3u);
                var newest_on_this = c_valid_root_before_the_raise_txg(valid_root_index);
                for (var other_valid_root_index = 0u; other_valid_root_index < valid_count; other_valid_root_index = other_valid_root_index + 1u) {
                    let txg = c_valid_root_before_the_raise_txg(other_valid_root_index);
                    if (sw(C_VALID_BEFORE_IDENTITIES + other_valid_root_index * C_VALID_BEFORE_IDENTITY_WORDS + 3u) == device && lt64(newest_on_this, txg)) {
                        newest_on_this = txg;
                    }
                }
                if (!have_newest_on_every || lt64(newest_on_this, newest_on_every)) {
                    newest_on_every = newest_on_this;
                    have_newest_on_every = true;
                }
            }
            // 根环每个区域所在的盘上都有一条有效根（窗口定得住上限的第一样）
            var every_device_of_the_ring_holds_a_valid_root = true;
            for (var region = 0u; region < regions; region = region + 1u) {
                let device = handoff_region_device(region);
                var this_device_holds_a_valid_root = false;
                for (var valid_root_index = 0u; valid_root_index < valid_count; valid_root_index = valid_root_index + 1u) {
                    if (sw(C_VALID_BEFORE_IDENTITIES + valid_root_index * C_VALID_BEFORE_IDENTITY_WORDS + 3u) == device) {
                        this_device_holds_a_valid_root = true;
                    }
                }
                if (!this_device_holds_a_valid_root) {
                    every_device_of_the_ring_holds_a_valid_root = false;
                }
            }
            // 非空：跟前一条有效根比两棵用户可见树的根指针；最旧的跟「两棵都没有」比
            var non_empty_count = 0u;
            var undeterminable_count = 0u;
            var previous_row = NONE;      // NONE 表示 ABSENT（两棵都没有、判得了）
            var previous_determinable = true;
            for (var valid_root_index = 0u; valid_root_index < valid_count; valid_root_index = valid_root_index + 1u) {
                let row = C_VALID_BEFORE + valid_root_index * C_VALID_BEFORE_WORDS;
                let identity = C_VALID_BEFORE_IDENTITIES + valid_root_index * C_VALID_BEFORE_IDENTITY_WORDS;
                let txg = c_valid_root_before_the_raise_txg(valid_root_index);
                let instance = sw(identity + 2u);
                c_user_visible_pointers(sw(identity + 4u), sw(identity + 5u), row);
                let determinable = sw(row + 1u) == 1u;
                if (determinable && previous_determinable) {
                    var differs = false;
                    if (previous_row == NONE) {
                        // 跟「两棵都没有、一个快照都没有」比（walk.rs 的 UserVisibleTreeRootPointers::ABSENT）
                        differs = sw(row + 2u) != NONE || sw(row + 4u) != NONE || c_next_snapshot_entry(row, 0u) != NONE;
                    } else {
                        differs = !c_pointers_equal(row, previous_row);
                    }
                    if (differs) {
                        sws(C_NON_EMPTY + non_empty_count * 4u, txg.x);
                        sws(C_NON_EMPTY + non_empty_count * 4u + 1u, txg.y);
                        sws(C_NON_EMPTY + non_empty_count * 4u + 2u, instance);
                        sws(C_NON_EMPTY + non_empty_count * 4u + 3u, valid_root_index);
                        non_empty_count = non_empty_count + 1u;
                    }
                } else {
                    sws(C_UNDETERMINABLE + undeterminable_count * 3u, txg.x);
                    sws(C_UNDETERMINABLE + undeterminable_count * 3u + 1u, txg.y);
                    sws(C_UNDETERMINABLE + undeterminable_count * 3u + 2u, instance);
                    undeterminable_count = undeterminable_count + 1u;
                }
                previous_row = row;
                previous_determinable = determinable;
            }
            let lowest = c_rollback_floor_ceiling(non_empty_count, 0u, newest_on_every, oldest, oldest_instance);
            let window_bounds_the_ceiling_at_the_raise = every_device_of_the_ring_holds_a_valid_root
                && c_distinct_non_empty_states_newer_than_the_oldest >= C_NON_EMPTY_VALID_ROOT_RANK_THE_CEILING_TAKES;
            let highest = c_rollback_floor_ceiling(non_empty_count, undeterminable_count, newest_on_every, oldest, oldest_instance);
            if (lt64(lowest, raising.floor) && le64(raising.floor, highest)) {
                // F 落在上限两沿之间：窗口那一遍这条根不判，重建那一遍射程里判不了
                if (reconstructing_the_ring) {
                    mark_part_of_the_range_not_judged(INVARIANT_I_7_9);
                }
                break;
            }
            if (reconstructing_the_ring) {
                possible_ring_verdicts = possible_ring_verdicts | select(1u, 2u, lt64(lowest, raising.floor));
                if (possible_ring_verdicts == 3u) {
                    // 有一种成立、有一种越过上限：判得不一致，射程里判不了
                    mark_part_of_the_range_not_judged(INVARIANT_I_7_9);
                    break;
                }
                if (reconstruction_round == possible_ring_count) {
                    judge(INVARIANT_I_7_9, possible_ring_verdicts == 1u);
                }
                continue;
            }
            if (lt64(lowest, raising.floor) || window_bounds_the_ceiling_at_the_raise) {
                judge(INVARIANT_I_7_9, !lt64(lowest, raising.floor));
                break;
            }
            // 窗口里判成立而窗口定不住抬 F 那一刻的上限：下一遍按 journal 重建那一刻的根环再判
        }
    }
}

// C_GROUPS 一行第四个字（盘位掩码）里这一位：这一组整组豁免了（I-3.16 收占着的槽读它；盘位只占低 MAX_DEVICES 位）
const C_GROUP_EXEMPTED_AS_QUARANTINED: u32 = 0x80000000u;

// 同一个字里这一位：组里有一条记录不带隔离记号（walk.rs quarantined_slots_exempted_per_device：组里每一条都带记号才豁免，
// every_copy_of_the_unit_carries_the_quarantine_mark；归组那一刻就记）
const C_GROUP_SOME_COPY_UNMARKED: u32 = 0x40000000u;

// 最近那一次 c_quarantine_exempted 记进 C_GROUPS 的组数
var<private> c_quarantine_group_count: u32;

// quarantined_slots_exempted_per_device：最新根的分配记录树里未释放、没有根引用起在它那一槽的记录，按单元归组；组里每一条都带隔离记号的整组豁免，
// 只认记号、不读副本（D19（块指针的结构与宽度预算） 已定项 5）。整组豁免的那一行掩码字带上 C_GROUP_EXEMPTED_AS_QUARANTINED：
// walk.rs 交回的 slots 就是这几组每块盘上罩住的槽
// judging_the_quarantine_marks：这一遍顺带判 I-3.17（第二遍、读的就是最新根那几棵；不另走一遍分配记录树，免得 r_allocation_tree 多展开一份）。
// 树表里种类 3 的每一棵读进 C_ALLOC_RECORDS 时接着上一棵往后记、不从 0 数（同 g_allocation_records_of_the_version）：walk.rs
// allocation_records_of_the_newest_root_without_judging 把最新根的全部分配记录树合成一张表，I-3.17 第三句按单元归组归的是这张合起来的表，
// 同一单元的几条记录分落几棵树时也归进一组；读不全的那一棵 r_allocation_tree 复位到它之前的条数，与 walk.rs 跳过那一棵相同
fn c_quarantine_exempted(newest_base: u32, reference_limit: u32, judging_the_quarantine_marks: bool) {
    let device_count = hdr(3u);
    for (var d = 0u; d < 8u; d = d + 1u) {
        c_exempted[d] = 0u;
    }
    c_quarantine_group_count = 0u;
    c_newest_records_read_in_full = false;
    c_newest_tree_table_has_entries_but_no_allocation_record_tree_entry = false;
    c_newest_tree_table_entry_count = NONE;
    c_newest_tree_table_pointer_is_all_zero = false;
    c_newest_tree_held_by_the_root_record = C_TREE_HELD_BY_THE_ROOT_RECORD_NOT_READ;
    c_newest_records_held_by_the_root_record_count = 0u;
    let tree_table_pointer = parse_node_pointer(newest_base, 36u);
    if (tree_table_pointer.all_zero) {
        c_alloc_record_count = 0u;
        c_newest_records_read_in_full = true;
        c_newest_tree_table_entry_count = 0u;
        c_newest_tree_table_pointer_is_all_zero = true;
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (tt.ok) {
        c_newest_tree_table_entry_count = tt.entry_count;
    }
    // I-3.21 第二遍要的根记录直接持有的那一棵（树表 0 条、根记录偏移 342 那条指针非零：树表 0 条、写过行的那一版，C963 候选一）：
    // 与树表里种类 3 的那几棵同一个 r_allocation_tree 调用点，排在最后一步；隔离豁免与 I-3.17 不读它（walk.rs 那两种读法只读树表）
    let reads_the_tree_held_by_the_root_record = judging_the_quarantine_marks && tt.ok && tt.entry_count == 0u
        && !parse_node_pointer(newest_base, G_ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT).all_zero;
    if (!tt.ok || (tt.entry_width < 200u && !reads_the_tree_held_by_the_root_record)) {
        if (judging_the_quarantine_marks) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_17);
        }
        return;
    }
    // 分配记录树的节点按最新根自己的中央映射认位置（walk.rs quarantined_slots_exempted_per_device 的 central_mapping）
    s_use_the_central_mapping_named_at(newest_base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
    var groups = 0u;
    r_noting = false;
    c_alloc_record_count = 0u;
    c_newest_records_read_in_full = true;
    // 种类 3 的条目数：在判根指针全零之前数（walk.rs allocation_records_of_a_root_record_without_judging 的 allocation_record_tree_entries 同一个口径）
    var allocation_record_tree_entries = 0u;
    // 第 step 步是树表第 step 条；第 tt.entry_count 步是根记录直接持有的那一棵，只在 reads_the_tree_held_by_the_root_record 时读
    for (var step = 0u; step <= tt.entry_count; step = step + 1u) {
        let is_the_tree_held_by_the_root_record = step == tt.entry_count;
        var tree_base = newest_base;
        var tree_offset = G_ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT;
        if (is_the_tree_held_by_the_root_record) {
            if (!reads_the_tree_held_by_the_root_record) {
                continue;
            }
        } else {
            let o = entry_offset(tt, step);
            if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION) {
                continue;
            }
            allocation_record_tree_entries = allocation_record_tree_entries + 1u;
            if (parse_node_pointer(tt.base, o + 14u).all_zero) {
                continue;
            }
            tree_base = tt.base;
            tree_offset = o + 14u;
        }
        let records_of_the_earlier_trees = c_alloc_record_count;
        if (!r_allocation_tree(tree_base, tree_offset)) {
            if (is_the_tree_held_by_the_root_record) {
                c_newest_tree_held_by_the_root_record = C_TREE_HELD_BY_THE_ROOT_RECORD_NOT_READ_IN_FULL;
            } else {
                c_newest_records_read_in_full = false;
            }
            continue;
        }
        if (is_the_tree_held_by_the_root_record) {
            // 树表 0 条：它的记录从第 0 行起。隔离豁免、I-3.17 与 I-3.1 照 walk.rs 只读树表的那几种读法看 0 条记录
            c_newest_tree_held_by_the_root_record = C_TREE_HELD_BY_THE_ROOT_RECORD_READ_IN_FULL;
            c_newest_records_held_by_the_root_record_count = c_alloc_record_count;
            c_alloc_record_count = records_of_the_earlier_trees;
            continue;
        }
        for (var i = records_of_the_earlier_trees; i < c_alloc_record_count; i = i + 1u) {
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
            var copy_bits = 1u << device_index;
            if ((sw(a + 7u) & C_ALLOC_RECORD_QUARANTINE_MARK_BIT) == 0u) {
                copy_bits = copy_bits | C_GROUP_SOME_COPY_UNMARKED;
            }
            var found = false;
            for (var k = 0u; k < groups; k = k + 1u) {
                let group = C_GROUPS + k * 4u;
                if (sw(group) == slot.x && sw(group + 1u) == slot.y && sw(group + 2u) == span) {
                    sws(group + 3u, sw(group + 3u) | copy_bits);
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
                sws(C_GROUPS + groups * 4u + 3u, copy_bits);
                groups = groups + 1u;
            }
        }
    }
    c_newest_tree_table_has_entries_but_no_allocation_record_tree_entry = tt.entry_count > 0u && allocation_record_tree_entries == 0u;
    if (judging_the_quarantine_marks) {
        c_judge_the_quarantine_marks_of_the_records_read();
    }
    for (var k = 0u; k < groups; k = k + 1u) {
        let group = C_GROUPS + k * 4u;
        let mask = sw(group + 3u) & ~C_GROUP_SOME_COPY_UNMARKED;
        let span = sw(group + 2u);
        let every_copy_is_marked = (sw(group + 3u) & C_GROUP_SOME_COPY_UNMARKED) == 0u;
        sws(group + 3u, mask);
        if (every_copy_is_marked) {
            for (var d = 0u; d < device_count; d = d + 1u) {
                if ((mask & (1u << d)) != 0u) {
                    c_exempted[d] = c_exempted[d] + span;
                }
            }
            sws(group + 3u, mask | C_GROUP_EXEMPTED_AS_QUARANTINED);
        }
    }
    c_quarantine_group_count = groups;
}

// c_deferred_below_the_floor 记进 C_DEFERRED 的行数（I-3.16 收占着的槽读它）
var<private> c_deferred_count: u32;

// deferred_slots_referenced_only_below_the_floor_per_device
fn c_deferred_below_the_floor(newest_base: u32) {
    c_deferred_count = 0u;
    c_kept_in_defer_by_the_pin_count = 0u;
    let tree_table_pointer = parse_node_pointer(newest_base, 36u);
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok || tt.entry_width < 200u) {
        return;
    }
    // 分配记录树的节点按最新根自己的中央映射认位置（walk.rs deferred_slots_referenced_only_below_the_floor_per_device 的 central_mapping）
    s_use_the_central_mapping_named_at(newest_base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
    var released_above = 0u;
    // 钉带出来的那一截的记录从 C_RELEASED_ABOVE 表尾往前放（与上面那张共用这一段、两头往中间长），只在有记账、t 低于不带钉现算的门槛时收
    let reclaim_threshold_on_the_root = rd64(newest_base, ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET);
    let reclaim_threshold_without_the_pin = c_reclaim_threshold_without_the_pin();
    let collects_the_band_the_pin_kept = w_accounting_seen && lt64(reclaim_threshold_on_the_root, reclaim_threshold_without_the_pin);
    var band_records = 0u;
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
            let generation = vec2<u32>(sw(a + 4u), sw(a + 5u));
            if (collects_the_band_the_pin_kept && sw(a + 6u) != 0u && lt64(reclaim_threshold_on_the_root, generation)
                && le64(generation, reclaim_threshold_without_the_pin)) {
                if (released_above + band_records >= C_MAX_RELEASED_ABOVE) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (115u << 8u);
                    return;
                }
                let band_row = C_RELEASED_ABOVE + (C_MAX_RELEASED_ABOVE - 1u - band_records) * C_RELEASED_ABOVE_WORDS;
                sws(band_row, sw(a));
                sws(band_row + 1u, sw(a + 1u));
                sws(band_row + 2u, sw(a + 2u));
                sws(band_row + 3u, sw(a + 3u));
                band_records = band_records + 1u;
            }
            if (sw(a + 6u) == 0u || !lt64(c_effective_floor, generation)) {
                continue;
            }
            // 两头往中间长，碰上了判不了（与上面收那一截那一处同一个站点）
            if (released_above + band_records >= C_MAX_RELEASED_ABOVE) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (115u << 8u);
                return;
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
    // F 之下、没被抛弃的根引用的、候选版本没走到的槽（走读内核交过来的清单，已去重）里，落在释放代高于 F 的已释放记录罩住的槽上的
    var deferred = 0u;
    let below_floor_slot_count = select(0u, sw(H_BASE + H_BELOW_FLOOR_SLOT_COUNT), released_above > 0u);
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
    c_deferred_count = deferred;
    // I-3.16 占着的槽里钉带出来的那一截（walk.rs released_slots_the_pin_kept_in_defer）：释放代落在 (t, 不带钉现算的门槛] 的记录罩住的槽，去掉候选版本走到的、
    // 第一遍整组豁免的、F 之下仍在 defer 的与被抛弃根引用着的，去重之后接在 C_DEFERRED 那几行后面
    var kept_by_the_pin = 0u;
    for (var k = 0u; k < band_records; k = k + 1u) {
        let record = C_RELEASED_ABOVE + (C_MAX_RELEASED_ABOVE - 1u - k) * C_RELEASED_ABOVE_WORDS;
        let device = sw(record);
        let record_slot = vec2<u32>(sw(record + 1u), sw(record + 2u));
        for (var s = 0u; s < sw(record + 3u); s = s + 1u) {
            let slot = add64(record_slot, u64_of(s));
            if (c_walked_slot(device, slot) || c_slot_exempted_as_quarantined(device, slot)
                || c_slot_referenced_by_an_abandoned_root(device, slot)) {
                continue;
            }
            var already_counted = false;
            for (var q = 0u; q < deferred + kept_by_the_pin; q = q + 1u) {
                if (sw(C_DEFERRED + q * 3u) == device && sw(C_DEFERRED + q * 3u + 1u) == slot.x && sw(C_DEFERRED + q * 3u + 2u) == slot.y) {
                    already_counted = true;
                }
            }
            if (already_counted) {
                continue;
            }
            if (deferred + kept_by_the_pin >= C_MAX_DEFERRED) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (116u << 8u);
                return;
            }
            let row = C_DEFERRED + (deferred + kept_by_the_pin) * 3u;
            sws(row, device);
            sws(row + 1u, slot.x);
            sws(row + 2u, slot.y);
            kept_by_the_pin = kept_by_the_pin + 1u;
        }
    }
    c_kept_in_defer_by_the_pin_count = kept_by_the_pin;
}

// 第一遍整组豁免的那几组（C_GROUPS 掩码字带 C_GROUP_EXEMPTED_AS_QUARANTINED）罩不罩住 (盘, 槽)：walk.rs 交回的隔离豁免 slots
fn c_slot_exempted_as_quarantined(device: u32, slot: vec2<u32>) -> bool {
    let device_count = hdr(3u);
    for (var k = 0u; k < c_quarantine_group_count; k = k + 1u) {
        let group = C_GROUPS + k * 4u;
        let mask = sw(group + 3u);
        if ((mask & C_GROUP_EXEMPTED_AS_QUARANTINED) == 0u) {
            continue;
        }
        let group_slot = vec2<u32>(sw(group), sw(group + 1u));
        if (!le64(group_slot, slot) || !lt64(slot, add64(group_slot, u64_of(sw(group + 2u))))) {
            continue;
        }
        for (var d = 0u; d < device_count; d = d + 1u) {
            if ((mask & (1u << d)) != 0u && hdr(12u + d) == device) {
                return true;
            }
        }
    }
    return false;
}

// 不带钉现算的门槛（walk.rs deferred_slots_reclaim_threshold）：max(F 生效值, 环里按最新根的实例表仍有效的根的最小 txg)
fn c_reclaim_threshold_without_the_pin() -> vec2<u32> {
    var threshold = c_effective_floor;
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
    if (have_oldest && lt64(threshold, oldest)) {
        threshold = oldest;
    }
    return threshold;
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

// ── 一版的树表与分配记录，只读不判（walk.rs 的 tree_table_entries_of_a_version_without_judging、allocation_records_of_a_version_without_judging）：
// I-3.16 读最新那一版的，引用与快照那一截的 I-5.6、I-3.15 读版本清单里每一版的

// 一版的树表：树表指针全零、读不出、有条目比字段表短（树表条目 200 字节）的交回 bad_node（那一版判不了）
fn g_tree_table_of_the_version(base: u32, tree_table_offset: u32) -> Node {
    let pointer = parse_node_pointer(base, tree_table_offset);
    if (pointer.all_zero) {
        return bad_node();
    }
    let tt = r_read_index_node(pointer);
    if (!tt.ok || (tt.entry_count > 0u && tt.entry_width < 200u)) {
        return bad_node();
    }
    return tt;
}

// 一版的分配记录读到了几棵分配记录树（walk.rs 的 AllocationRecordsOfAVersion::allocation_record_trees_read）：g_allocation_records_of_the_version
// 每次从 0 数，读一棵非全零的加一。0 说明这一版的账一棵树都没有，记录表空不等于「没有未释放的记录」（树表缺种类 3 那一条的坏镜像，I-9.16 判它）
var<private> g_allocation_record_trees_read: u32;

// 一版的分配记录记进 C_ALLOC_RECORDS（c_alloc_record_count 从 0 数）：根记录（或记录新根段）直接持有的那一棵（base 里 held_offset 起那条指针，
// 树表 0 条的那一版才有），再是树表 tt 里种类 3 的每一条，非全零的整棵读、记进 g_allocation_record_trees_read；有一棵读不全交回 false。
// 两样同一个 r_allocation_tree 调用点：第 0 步是根记录持有的那一棵，第 k 步是树表第 k − 1 条（GPU 程序把每个调用点展开成一份拷贝）
fn g_allocation_records_of_the_version(base: u32, held_offset: u32, tt: Node) -> bool {
    c_alloc_record_count = 0u;
    g_allocation_record_trees_read = 0u;
    r_noting = false;
    for (var step = 0u; step <= tt.entry_count; step = step + 1u) {
        var tree_base = base;
        var tree_offset = held_offset;
        if (step > 0u) {
            let o = entry_offset(tt, step - 1u);
            if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION) {
                continue;
            }
            tree_base = tt.base;
            tree_offset = o + 14u;
        }
        if (parse_node_pointer(tree_base, tree_offset).all_zero) {
            continue;
        }
        if (!r_allocation_tree(tree_base, tree_offset)) {
            return false;
        }
        g_allocation_record_trees_read = g_allocation_record_trees_read + 1u;
    }
    return true;
}

// ── I-3.16（walk.rs judge_fragmentation_and_empty_segment_statistics，N5 = C413）：最新根记账里每块盘的 runs（第 10 项）与全空聚簇段数（第 11 项）
// 等于全扫重算的值。占着的槽是 I-3.1 那一份并集：全部走过的版本的引用（C_REFERENCES0 前 w_reference_count[0] 条）、第一遍隔离豁免的那几组
// （每块盘上罩住的槽）、F 之下仍在 defer 的槽（C_DEFERRED）。三样记成区间（盘、槽 lo/hi、跨度）排一次序、逐盘按序走一遍：
// 那块盘上占着的槽数（区间并起来的槽数，单元区之外的也算）乘 16 KiB 不等于记账的已分配时这块盘不判（认不出占着哪几槽）；单元区 [起始槽号, 盘末) 里
// 空闲槽的连续段数（walk.rs free_runs_between_the_occupied_slots：区间按起点升序，起点越过「下一个可能空闲的槽」就多一段）比第 10 项；
// 全空段只按记录口径（最新根未释放的分配记录罩住的整段之外的整段数，D3（空间分配） 已定项 10 ①，C372 按记录口径收口）数、比第 11 项，
// 不经占着的槽那一份并集、也不与按占用口径数的比（walk.rs 改成这样之后的写法）；最新根的分配记录读不全、或那一版一棵分配记录树都没有
// （树表里没有种类 3 那一条、根记录也不持有，I-9.16 判它）时第 11 项报部分没判。区间表借引用与快照那一截的 S_KEYED_UNITS 起那一段
// （扫描内核在引用与快照那一截之前跑、那一截从头写它）

const C_OCCUPIED_WORDS: u32 = 4u;            // 盘、槽 lo/hi、跨度

// 区间至多几条：引用集合 C_MAX_REFERENCES、隔离豁免的组 C_MAX_GROUPS 乘每组的盘、F 之下的推迟 C_MAX_DEFERRED，合法镜像上远不到
const C_MAX_OCCUPIED: u32 = 16384u;

const C_OCCUPIED: u32 = S_KEYED_UNITS;

const C_OCCUPIED_ORDER: u32 = C_OCCUPIED + C_OCCUPIED_WORDS * C_MAX_OCCUPIED;

// 区间表与它的次序不越进 I-3.16 同时在用的分配记录次序表（crash_judge_checker_common.wgsl ⑮ 的 C_ALLOCATION_RECORD_ROW_ORDER）
const_assert C_OCCUPIED_ORDER + C_MAX_OCCUPIED <= C_ALLOCATION_RECORD_ROW_ORDER;

// 区间表满了（「条目太多」一类，站点号与别处不撞）
const C_SITE_OCCUPIED_INTERVALS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (100u << 8u);

// 第 10、11 项的统计量标签（walk.rs 的 STATISTIC_FRAGMENTATION_RUNS、STATISTIC_EMPTY_CLUSTER_SEGMENTS）与已分配（STATISTIC_ALLOCATED_BYTES）
const STATISTIC_ALLOCATED_BYTES: u32 = 1u;
const STATISTIC_FRAGMENTATION_RUNS: u32 = 10u;
const STATISTIC_EMPTY_CLUSTER_SEGMENTS: u32 = 11u;

// 一个聚簇段几个槽（singlefs_format::CLUSTER_SEGMENT_SLOTS）与它的位数
const CLUSTER_SEGMENT_SLOTS: u32 = 64u;
const CLUSTER_SEGMENT_SLOT_SHIFT: u32 = 6u;

var<private> c_occupied_count: u32;

fn c_note_an_occupied_interval(device: u32, slot: vec2<u32>, span: u32) {
    if (c_occupied_count >= C_MAX_OCCUPIED) {
        undecidable = C_SITE_OCCUPIED_INTERVALS_PAST_THE_TABLE;
        return;
    }
    let row = C_OCCUPIED + c_occupied_count * C_OCCUPIED_WORDS;
    sws(row, device);
    sws(row + 1u, slot.x);
    sws(row + 2u, slot.y);
    sws(row + 3u, span);
    c_occupied_count = c_occupied_count + 1u;
}

// 第一遍隔离豁免整组豁免的那几组：组里每块盘一条区间
fn c_note_the_quarantined_groups_as_occupied() {
    let device_count = hdr(3u);
    for (var k = 0u; k < c_quarantine_group_count; k = k + 1u) {
        let group = C_GROUPS + k * 4u;
        let mask = sw(group + 3u);
        if ((mask & C_GROUP_EXEMPTED_AS_QUARANTINED) == 0u) {
            continue;
        }
        for (var d = 0u; d < device_count; d = d + 1u) {
            if ((mask & (1u << d)) != 0u) {
                c_note_an_occupied_interval(hdr(12u + d), vec2<u32>(sw(group), sw(group + 1u)), sw(group + 2u));
            }
        }
    }
}

// 一块盘上按起点升序走过的几段 [起点, 终点)（单元区里、非空）罩住的段号（(槽 − 单元区起始槽号) / 64，只数小于 segment_limit 的）：
// counted 是数过的段数，next_uncounted 是数过的最大段号 + 1（一段都没数过是 0）。起点升序时，前面各段与 [这一段的起点所在段, ∞) 的交恰是
// [起点所在段, next_uncounted)：之前每一段起点都不大于它，罩到它之后的那一段就连着罩到自己的终点
struct SegmentTally {
    counted: vec2<u32>,
    next_uncounted: vec2<u32>,
};

fn c_tally_the_segments_of_an_interval(tally: SegmentTally, start: vec2<u32>, end: vec2<u32>, unit_area_start: vec2<u32>,
    segment_limit: vec2<u32>) -> SegmentTally {
    var after = tally;
    let first_segment = shr64(sub64(start, unit_area_start), CLUSTER_SEGMENT_SLOT_SHIFT);
    var last_segment = shr64(sub64(sub64(end, u64_of(1u)), unit_area_start), CLUSTER_SEGMENT_SLOT_SHIFT);
    if (is_zero64(segment_limit)) {
        return after;
    }
    let highest_segment = sub64(segment_limit, u64_of(1u));
    if (lt64(highest_segment, last_segment)) {
        last_segment = highest_segment;
    }
    let first_uncounted_segment = select(first_segment, after.next_uncounted, lt64(first_segment, after.next_uncounted));
    if (le64(first_uncounted_segment, last_segment)) {
        after.counted = add64(after.counted, add64(sub64(last_segment, first_uncounted_segment), u64_of(1u)));
        after.next_uncounted = add64(last_segment, u64_of(1u));
    }
    return after;
}

fn judge_fragmentation_and_empty_segment_statistics() {
    for (var i = 0u; i < w_reference_count[0]; i = i + 1u) {
        let row = C_REFERENCES0 + i * C_REFERENCE_WORDS;
        c_note_an_occupied_interval(sw(row), vec2<u32>(sw(row + 1u), sw(row + 2u)), sw(row + 3u));
    }
    // F 之下仍在 defer 的与钉带出来的那一截（C_DEFERRED 里两段首尾相接）
    for (var q = 0u; q < c_deferred_count + c_kept_in_defer_by_the_pin_count; q = q + 1u) {
        c_note_an_occupied_interval(sw(C_DEFERRED + q * 3u), vec2<u32>(sw(C_DEFERRED + q * 3u + 1u), sw(C_DEFERRED + q * 3u + 2u)), 1u);
    }
    if (undecidable != 0u) {
        return;
    }
    // 最新那一版的分配记录（按记录口径数全空段）：树表或哪一棵读不全，记录口径那一半数不出
    let newest_root = checker_root_of(c_newest_index);
    let newest_tree_table = g_tree_table_of_the_version(newest_root.base, 36u);
    var newest_records_read_in_full = false;
    if (newest_tree_table.ok) {
        // 分配记录树的节点按最新根自己的中央映射认位置（walk.rs allocation_records_of_a_version_without_judging 的 central_mapping）
        s_use_the_central_mapping_named_at(newest_root.base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
        newest_records_read_in_full = g_allocation_records_of_the_version(newest_root.base, 342u, newest_tree_table);
    }
    let newest_allocation_record_trees_read = g_allocation_record_trees_read;
    // 区间与记录各排一次序（同一个调用点：第 0 遍排区间、第 1 遍排记录）
    for (var table = 0u; table < 2u; table = table + 1u) {
        if (table == 1u && !newest_records_read_in_full) {
            break;
        }
        order_rows_by_their_first_four_words(select(C_OCCUPIED, C_ALLOC_RECORDS, table == 1u), select(C_OCCUPIED_WORDS, C_ALLOC_RECORD_WORDS, table == 1u),
            select(c_occupied_count, c_alloc_record_count, table == 1u), select(C_OCCUPIED_ORDER, C_ALLOCATION_RECORD_ROW_ORDER, table == 1u));
    }
    let unit_area_start = handoff_unit_area_start_slot();
    let device_end_slot = shr64(device_bytes(), SLOT_SHIFT);
    let device_count = hdr(3u);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        var allocated = vec2<u32>(0u, 0u);
        if (!c_accounting_row(STATISTIC_ALLOCATED_BYTES, device, &allocated)) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        let unit_area_has_slots = lt64(unit_area_start, device_end_slot);
        let unit_area_slots = select(u64_of(0u), sub64(device_end_slot, unit_area_start), unit_area_has_slots);
        let full_segments = shr64(unit_area_slots, CLUSTER_SEGMENT_SLOT_SHIFT);
        // 占着的槽：整块盘上并起来的槽数；单元区里的空闲连续段数
        var occupied_slots = vec2<u32>(0u, 0u);
        var covered_end = vec2<u32>(0u, 0u);
        var next_free_candidate = unit_area_start;
        var runs = vec2<u32>(0u, 0u);
        for (var position = 0u; position < c_occupied_count; position = position + 1u) {
            let row = C_OCCUPIED + sw(C_OCCUPIED_ORDER + position) * C_OCCUPIED_WORDS;
            if (sw(row) != device) {
                continue;
            }
            let start = vec2<u32>(sw(row + 1u), sw(row + 2u));
            let end = add64(start, u64_of(sw(row + 3u)));
            let counted_from = select(start, covered_end, lt64(start, covered_end));
            if (lt64(counted_from, end)) {
                occupied_slots = add64(occupied_slots, sub64(end, counted_from));
                covered_end = end;
            }
            let in_area_start = select(start, unit_area_start, lt64(start, unit_area_start));
            let in_area_end = select(end, device_end_slot, lt64(device_end_slot, end));
            if (!lt64(in_area_start, in_area_end)) {
                continue;
            }
            if (lt64(next_free_candidate, in_area_start)) {
                runs = add64(runs, u64_of(1u));
            }
            if (lt64(next_free_candidate, in_area_end)) {
                next_free_candidate = in_area_end;
            }
        }
        if (lt64(next_free_candidate, device_end_slot)) {
            runs = add64(runs, u64_of(1u));
        }
        // 槽数乘 16 KiB 溢出 64 位（槽数 ≥ 2⁵⁰）时与记账对不上，同 walk.rs 的 checked_mul
        if (occupied_slots.y >= (1u << (32u - SLOT_SHIFT)) || !eq64(shl64(occupied_slots, SLOT_SHIFT), allocated)) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        if (!unit_area_has_slots) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        // 记账里缺 runs 那一行判红（walk.rs 同一处：没有别的不变量判它）
        var recorded = vec2<u32>(0u, 0u);
        let has_runs_row = c_accounting_row(STATISTIC_FRAGMENTATION_RUNS, device, &recorded);
        judge(INVARIANT_I_3_16, has_runs_row && eq64(recorded, runs));
        if (!newest_records_read_in_full) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        // 最新根那一版一棵分配记录树都没有：记录表空不等于没有未释放的记录，按记录口径数不出（walk.rs 同一句守）
        if (newest_allocation_record_trees_read == 0u) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        // 按记录口径：这块盘上未释放的记录罩住的整段（段号 < 整段数）
        var segments_holding_a_record: SegmentTally;
        segments_holding_a_record.counted = vec2<u32>(0u, 0u);
        segments_holding_a_record.next_uncounted = vec2<u32>(0u, 0u);
        for (var position = 0u; position < c_alloc_record_count; position = position + 1u) {
            let record = C_ALLOC_RECORDS + sw(C_ALLOCATION_RECORD_ROW_ORDER + position) * C_ALLOC_RECORD_WORDS;
            if (sw(record) != device || sw(record + 6u) != 0u) {
                continue;
            }
            let start = vec2<u32>(sw(record + 1u), sw(record + 2u));
            let end = add64_saturating(start, u64_of(sw(record + 3u)));
            let in_area_start = select(start, unit_area_start, lt64(start, unit_area_start));
            let in_area_end = select(end, device_end_slot, lt64(device_end_slot, end));
            if (!lt64(in_area_start, in_area_end)) {
                continue;
            }
            segments_holding_a_record = c_tally_the_segments_of_an_interval(segments_holding_a_record, in_area_start, in_area_end, unit_area_start,
                full_segments);
        }
        let empty_segments_by_the_records = sub64(full_segments, segments_holding_a_record.counted);
        let has_empty_segments_row = c_accounting_row(STATISTIC_EMPTY_CLUSTER_SEGMENTS, device, &recorded);
        judge(INVARIANT_I_3_16, has_empty_segments_row && eq64(recorded, empty_segments_by_the_records));
    }
}

// 最新根引用的槽里有没有 (盘, 槽)（按跨度展开，引用集合前 c_newest_reference_count 行是最新根那一遍记下的；walk.rs slots_covered_by_the_newest_root）
fn c_slot_covered_by_the_newest_root(device: u32, slot: vec2<u32>) -> bool {
    for (var i = 0u; i < c_newest_reference_count; i = i + 1u) {
        let row = C_REFERENCES0 + i * C_REFERENCE_WORDS;
        if (sw(row) != device) {
            continue;
        }
        let start = vec2<u32>(sw(row + 1u), sw(row + 2u));
        if (le64(start, slot) && lt64(slot, add64(start, u64_of(sw(row + 3u))))) {
            return true;
        }
    }
    return false;
}

// walk.rs judge_quarantine_marks_against_the_references（I-3.17）：C_ALLOC_RECORDS 里刚读进来的最新根那几棵分配记录树（树表里种类 3 的每一棵，
// 读得全的合成一张表，c_quarantine_exempted）里，每条带隔离记号的记录判 ①（未释放）与 ②（罩住的槽不被最新根引用），再拿这张合起来的表里
// 它那一单元（起点槽、跨度）的每条记录判 ③（都带记号；同一单元分落几棵树的记录也归进一组，walk.rs records_of_each_unit）。一条带记号的记录都没有时不判
// （CPU 报不适用，这里不置判到位）；树表读不出时调用方记射程里有一部分判不了
fn c_judge_the_quarantine_marks_of_the_records_read() {
    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
        let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
        if ((sw(a + 7u) & C_ALLOC_RECORD_QUARANTINE_MARK_BIT) == 0u) {
            continue;
        }
        let device = sw(a);
        let slot = vec2<u32>(sw(a + 1u), sw(a + 2u));
        let span = sw(a + 3u);
        // ①
        judge(INVARIANT_I_3_17, sw(a + 6u) == 0u);
        // ②
        var covered_slot_referenced = false;
        for (var covered = 0u; covered < span; covered = covered + 1u) {
            if (c_slot_covered_by_the_newest_root(device, add64(slot, u64_of(covered)))) {
                covered_slot_referenced = true;
            }
        }
        judge(INVARIANT_I_3_17, !covered_slot_referenced);
        // ③：同一单元每条记录都带记号
        var every_copy_is_marked = true;
        for (var j = 0u; j < c_alloc_record_count; j = j + 1u) {
            let b = C_ALLOC_RECORDS + j * C_ALLOC_RECORD_WORDS;
            if (sw(b + 1u) == slot.x && sw(b + 2u) == slot.y && sw(b + 3u) == span
                && (sw(b + 7u) & C_ALLOC_RECORD_QUARANTINE_MARK_BIT) == 0u) {
                every_copy_is_marked = false;
            }
        }
        judge(INVARIANT_I_3_17, every_copy_is_marked);
    }
}

// walk.rs allocated_statistic_bounds_of_a_device（I-3.1 RCW）：C_ALLOC_RECORDS 是 c_quarantine_exempted 第二遍读进来的最新根全部分配记录。
// E(θ) = 未释放的与释放代高于 θ 的已释放记录罩住的槽，按槽认、几条记录罩同一槽只数一次（每个槽看前面同一份里有没有记录已经罩过它；
// 不另排一次序：扫描内核里拿分配记录调排序的只许 I-5.4、I-3.16 那两处，crash_judge_gpu_tables_at_their_capacity 的静态用例钉着）。
// U = 已释放、释放代高于 t、遍历没走到（c_walked_slot）的记录罩住的槽。
// every_slot_counted_at_the_root_threshold_is_walked_or_deferred：E(t) 的每一槽都在 Ω ∪ U 里（walk.rs 的
// slots_counted_at_the_root_reclaim_threshold_outside_the_walk_and_the_deferred 是空的），只在按单元那一类数的那一份判，按记录自述那一份恒为 true
// every_unreleased_record_is_disjoint_from_the_other_records：按单元那一类数的那一份里每条未释放记录罩住的槽区间不与这块盘上别的记录相交
// （walk.rs unreleased_records_overlapping_another_record_in_the_unit_class_reading 交回空表；判决 research/prompts/m3-close-b19-reason-r2-main-verification.md
// 第二节 Q1「改法」），也只在按单元那一类数的那一份判，按记录自述那一份恒为 true（自述的跨度相交归 I-5.4）
struct AllocatedStatisticBoundsOfADevice {
    counted_at_the_checker_reclaim_threshold: vec2<u32>,
    counted_at_the_root_reclaim_threshold: vec2<u32>,
    still_deferred_outside_the_walk: vec2<u32>,
    every_walked_slot_is_recorded: bool,
    every_slot_counted_at_the_root_threshold_is_walked_or_deferred: bool,
    every_unreleased_record_is_disjoint_from_the_other_records: bool,
}

// 一条记录（C_ALLOC_RECORDS 里那一行）算不算 E(θ)：未释放，或释放代高于 θ；released_only 时只算已释放的
fn c_record_counted_at(record: u32, reclaim_threshold: vec2<u32>, released_only: bool) -> bool {
    let released = sw(record + 6u) != 0u;
    return (released || !released_only) && (!released || lt64(reclaim_threshold, vec2<u32>(sw(record + 4u), sw(record + 5u))));
}

// 算 I-3.1 那几个界时一条记录按几槽数（research/prompts/m3-close-b19-design-r2-main-verification.md Q9 的条件，
// walk.rs allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading）：按记录自述的跨度段；按单元那一类（I-3.18 判红的那几条改记成
// 起点单元那一类的槽数，c_note_the_unit_class_slots_of_the_records_the_span_invariant_reddens 记在 C_ALLOC_RECORDS 第八个字里，别的照跨度段）
const C_SPAN_READING_AS_RECORDED: u32 = 0u;
const C_SPAN_READING_AS_THE_UNIT_CLASSES: u32 = 1u;

// C_ALLOC_RECORDS 一行第八个字里这几位：c_note_the_unit_class_slots_of_the_records_the_span_invariant_reddens 记的起点单元那一类的槽数
// （0 是没改记）。同一个字的位 31 是隔离记号、低两位是 I-3.15 那一遍的记号（C_ALLOC_RECORD_QUARANTINE_MARK_BIT 上面那段），各占各的位；
// 只在同一块盘刚记过之后按 C_SPAN_READING_AS_THE_UNIT_CLASSES 读
const C_ALLOC_RECORD_SLOTS_OF_THE_UNIT_CLASS_SHIFT: u32 = 8u;
const C_ALLOC_RECORD_SLOTS_OF_THE_UNIT_CLASS_BITS: u32 = 0xff00u;

fn c_span_of_the_record(record: u32, span_reading: u32) -> u32 {
    let span_as_recorded = sw(record + 3u);
    let slots_of_the_unit_class = (sw(record + 7u) & C_ALLOC_RECORD_SLOTS_OF_THE_UNIT_CLASS_BITS) >> C_ALLOC_RECORD_SLOTS_OF_THE_UNIT_CLASS_SHIFT;
    if (span_reading == C_SPAN_READING_AS_RECORDED || slots_of_the_unit_class == 0u) {
        return span_as_recorded;
    }
    return slots_of_the_unit_class;
}

// 起点在 (device, slot) 的单元头用得了（magic、头校验和过、类标签是码 1 / 2 / 3）时那一类占的槽数（码 1、码 3 两槽，码 2 一槽；
// walk.rs class_and_slots_of_the_unit_whose_usable_header_starts_at）；读不出、头用不了交回 0。I-3.18 比跨度段、I-3.1 认 I-3.18 判红的记录用的同一个数
fn c_slots_of_the_class_of_the_unit_whose_usable_header_starts_at(device: u32, slot: vec2<u32>) -> u32 {
    let unit_header = read_at(device, slot_to_bytes(slot), 4096u);
    if (unit_header.version == NONE || rd32(unit_header.base, 0u) != MAGIC_SFSU || (unit_header.c2 & 1u) == 0u) {
        return 0u;
    }
    let unit_class = rd8(unit_header.base, 6u);
    if (unit_class == 1u || unit_class == 3u) {
        return DATA_UNIT_BYTES >> SLOT_SHIFT;
    }
    if (unit_class == 2u) {
        return NODE_BYTES >> SLOT_SHIFT;
    }
    return 0u;
}

// walk.rs records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes：这块盘上每条未释放的记录，起点单元头用得了而那一类的槽数
// 与跨度段不符（I-3.18 判红的那几条）的，把那一类的槽数记进 C_ALLOC_RECORDS 第八个字（C_ALLOC_RECORD_SLOTS_OF_THE_UNIT_CLASS_BITS），别的记 0；
// 交回有没有一条这样的记录
fn c_note_the_unit_class_slots_of_the_records_the_span_invariant_reddens(device: u32) -> bool {
    var some_span_was_noted = false;
    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
        let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
        if (sw(record) != device) {
            continue;
        }
        var slots_of_the_unit_class = 0u;
        if (sw(record + 6u) == 0u) {
            slots_of_the_unit_class = c_slots_of_the_class_of_the_unit_whose_usable_header_starts_at(device, vec2<u32>(sw(record + 1u), sw(record + 2u)));
        }
        if (slots_of_the_unit_class == sw(record + 3u)) {
            slots_of_the_unit_class = 0u;
        }
        some_span_was_noted = some_span_was_noted || slots_of_the_unit_class != 0u;
        sws(record + 7u, (sw(record + 7u) & ~C_ALLOC_RECORD_SLOTS_OF_THE_UNIT_CLASS_BITS)
            | (slots_of_the_unit_class << C_ALLOC_RECORD_SLOTS_OF_THE_UNIT_CLASS_SHIFT));
    }
    return some_span_was_noted;
}

// C_ALLOC_RECORDS 前 before 行里有没有同一份（同一块盘、c_record_counted_at 同样成立）的记录按 span_reading 数的槽罩着 slot
fn c_slot_counted_by_an_earlier_record(device: u32, slot: vec2<u32>, before: u32, reclaim_threshold: vec2<u32>, released_only: bool,
    span_reading: u32) -> bool {
    for (var j = 0u; j < before; j = j + 1u) {
        let record = C_ALLOC_RECORDS + j * C_ALLOC_RECORD_WORDS;
        let record_start = vec2<u32>(sw(record + 1u), sw(record + 2u));
        if (sw(record) == device && c_record_counted_at(record, reclaim_threshold, released_only) && le64(record_start, slot)
            && lt64(slot, add64_saturating(record_start, u64_of(c_span_of_the_record(record, span_reading))))) {
            return true;
        }
    }
    return false;
}

// 第一遍隔离豁免（全部走过的版本）整组豁免的那几组罩不罩住 (盘, 槽)：walk.rs 的 quarantined_exempted_against_every_walked_version.slots。
// 记账头那一趟（accounting_head）第二遍改写 C_GROUPS 之前把那几组逐盘记进 C_OCCUPIED（c_note_the_quarantined_groups_as_occupied），I-3.1 分片那几趟数的时候
// C_OCCUPIED 前 c_occupied_count 行就是它们（F 之下的推迟与引用在 I-3.16 那一遍才接着记）
fn c_slot_exempted_as_quarantined_against_every_walked_version(device: u32, slot: vec2<u32>) -> bool {
    for (var k = 0u; k < c_occupied_count; k = k + 1u) {
        let row = C_OCCUPIED + k * C_OCCUPIED_WORDS;
        let start = vec2<u32>(sw(row + 1u), sw(row + 2u));
        if (sw(row) == device && le64(start, slot) && lt64(slot, add64_saturating(start, u64_of(sw(row + 3u))))) {
            return true;
        }
    }
    return false;
}

// walk.rs unreleased_records_overlapping_another_record_in_the_unit_class_reading（I-3.1 取宽那一支第五判，判决
// research/prompts/m3-close-b19-reason-r2-main-verification.md 第二节 Q1「改法」）里一条记录那一项：C_ALLOC_RECORDS 第 record_index 行（这块盘上的）
// 按单元那一类数（c_span_of_the_record 取 C_SPAN_READING_AS_THE_UNIT_CLASSES，决断那一趟先 c_note_the_unit_class_slots_of_the_records_the_span_invariant_reddens
// 记好）罩住的槽区间，与这块盘上排在它后面的每一条记录比，至少一条未释放时不相交；两条都已释放的不比。整块盘的那一判是每一条的这一项都成立，
// 分片那几趟逐条做、逐条相与（不另排一次序：扫描内核里拿分配记录调排序的只许 I-5.4、I-3.16 那两处）
fn c_unit_class_reading_of_the_record_is_disjoint_from_the_records_after_it(device: u32, record_index: u32) -> bool {
    let record = C_ALLOC_RECORDS + record_index * C_ALLOC_RECORD_WORDS;
    let start = vec2<u32>(sw(record + 1u), sw(record + 2u));
    let end = add64_saturating(start, u64_of(c_span_of_the_record(record, C_SPAN_READING_AS_THE_UNIT_CLASSES)));
    let released = sw(record + 6u) != 0u;
    for (var j = record_index + 1u; j < c_alloc_record_count; j = j + 1u) {
        let other = C_ALLOC_RECORDS + j * C_ALLOC_RECORD_WORDS;
        if (sw(other) != device || (released && sw(other + 6u) != 0u)) {
            continue;
        }
        let other_start = vec2<u32>(sw(other + 1u), sw(other + 2u));
        let other_end = add64_saturating(other_start, u64_of(c_span_of_the_record(other, C_SPAN_READING_AS_THE_UNIT_CLASSES)));
        if (lt64(start, other_end) && lt64(other_start, end)) {
            return false;
        }
    }
    return true;
}

// 一块盘一种读法的那一份界在分片之前的初值：三个计数为 0、三个「每一…」为真（按记录自述那一份后两样恒为真，见 AllocatedStatisticBoundsOfADevice 上面那段）
fn c_initial_allocated_statistic_bounds() -> AllocatedStatisticBoundsOfADevice {
    var bounds: AllocatedStatisticBoundsOfADevice;
    bounds.counted_at_the_checker_reclaim_threshold = vec2<u32>(0u, 0u);
    bounds.counted_at_the_root_reclaim_threshold = vec2<u32>(0u, 0u);
    bounds.still_deferred_outside_the_walk = vec2<u32>(0u, 0u);
    bounds.every_walked_slot_is_recorded = true;
    bounds.every_slot_counted_at_the_root_threshold_is_walked_or_deferred = true;
    bounds.every_unreleased_record_is_disjoint_from_the_other_records = true;
    return bounds;
}

// 交接段里第 device_index 块盘（hdr(12u + device_index) 那一块）按 span_reading 数的那一份界从哪个字起
fn c_accounting_carry_bounds_row(device_index: u32, span_reading: u32) -> u32 {
    return C_ACCOUNTING_CARRY_BOUNDS + (device_index * 2u + span_reading) * C_ACCOUNTING_BOUNDS_WORDS;
}

fn c_accounting_carry_bounds_load(device_index: u32, span_reading: u32) -> AllocatedStatisticBoundsOfADevice {
    let row = c_accounting_carry_bounds_row(device_index, span_reading);
    var bounds: AllocatedStatisticBoundsOfADevice;
    bounds.counted_at_the_checker_reclaim_threshold = vec2<u32>(sw(row), sw(row + 1u));
    bounds.counted_at_the_root_reclaim_threshold = vec2<u32>(sw(row + 2u), sw(row + 3u));
    bounds.still_deferred_outside_the_walk = vec2<u32>(sw(row + 4u), sw(row + 5u));
    bounds.every_walked_slot_is_recorded = sw(row + 6u) != 0u;
    bounds.every_slot_counted_at_the_root_threshold_is_walked_or_deferred = sw(row + 7u) != 0u;
    bounds.every_unreleased_record_is_disjoint_from_the_other_records = sw(row + 8u) != 0u;
    return bounds;
}

fn c_accounting_carry_bounds_store(device_index: u32, span_reading: u32, bounds: AllocatedStatisticBoundsOfADevice) {
    let row = c_accounting_carry_bounds_row(device_index, span_reading);
    sws(row, bounds.counted_at_the_checker_reclaim_threshold.x);
    sws(row + 1u, bounds.counted_at_the_checker_reclaim_threshold.y);
    sws(row + 2u, bounds.counted_at_the_root_reclaim_threshold.x);
    sws(row + 3u, bounds.counted_at_the_root_reclaim_threshold.y);
    sws(row + 4u, bounds.still_deferred_outside_the_walk.x);
    sws(row + 5u, bounds.still_deferred_outside_the_walk.y);
    sws(row + 6u, select(0u, 1u, bounds.every_walked_slot_is_recorded));
    sws(row + 7u, select(0u, 1u, bounds.every_slot_counted_at_the_root_threshold_is_walked_or_deferred));
    sws(row + 8u, select(0u, 1u, bounds.every_unreleased_record_is_disjoint_from_the_other_records));
}

// walk.rs allocated_statistic_bounds_of_a_device（I-3.1 RCW）里一条记录那一项：C_ALLOC_RECORDS 第 record_index 行（这块盘上的）罩住的每一槽
// 按 span_reading（C_SPAN_READING_AS_RECORDED 或 C_SPAN_READING_AS_THE_UNIT_CLASSES）数进三个计数：E(θ) 按它数，几条记录罩同一槽只数一次（每个槽看
// 前面同一份里有没有记录已经罩过它）；按单元那一类数的那一份另判逐槽的上沿（判决 research/prompts/m3-close-b19-reason-r1-main-verification.md
// 第二节 Q3「改法」那一行）：未释放的记录罩住的每一槽都遍历走到（c_walked_slot）、在第一遍整组豁免的那几组里，或有一条已释放、释放代高于 t 的记录
// 罩着（U）——已释放而释放代高于 t 的记录罩住的槽不在 Ω 就在 U 里，不用再查。三个计数相加、上沿相与：分片怎么切都与整块盘一次数完的相同
fn c_count_the_slots_of_a_record_into_the_bounds(bounds: AllocatedStatisticBoundsOfADevice, device: u32, record_index: u32,
    reclaim_threshold_computed_by_the_checker: vec2<u32>, reclaim_threshold_on_the_newest_root: vec2<u32>, span_reading: u32)
    -> AllocatedStatisticBoundsOfADevice {
    var counted = bounds;
    let reads_the_unit_classes = span_reading == C_SPAN_READING_AS_THE_UNIT_CLASSES;
    let record = C_ALLOC_RECORDS + record_index * C_ALLOC_RECORD_WORDS;
    let span = c_span_of_the_record(record, span_reading);
    let start = vec2<u32>(sw(record + 1u), sw(record + 2u));
    let end = add64_saturating(start, u64_of(span));
    let at_the_checker_threshold = c_record_counted_at(record, reclaim_threshold_computed_by_the_checker, false);
    let at_the_root_threshold = c_record_counted_at(record, reclaim_threshold_on_the_newest_root, false);
    let still_deferred = c_record_counted_at(record, reclaim_threshold_on_the_newest_root, true);
    for (var offset = 0u; offset < span; offset = offset + 1u) {
        let slot = add64(start, u64_of(offset));
        if (!lt64(slot, end)) {
            break;
        }
        if (at_the_checker_threshold && !c_slot_counted_by_an_earlier_record(device, slot, record_index, reclaim_threshold_computed_by_the_checker, false,
            span_reading)) {
            counted.counted_at_the_checker_reclaim_threshold = add64(counted.counted_at_the_checker_reclaim_threshold, u64_of(1u));
        }
        if (at_the_root_threshold && !c_slot_counted_by_an_earlier_record(device, slot, record_index, reclaim_threshold_on_the_newest_root, false, span_reading)) {
            counted.counted_at_the_root_reclaim_threshold = add64(counted.counted_at_the_root_reclaim_threshold, u64_of(1u));
        }
        if (still_deferred && !c_walked_slot(device, slot)
            && !c_slot_counted_by_an_earlier_record(device, slot, record_index, reclaim_threshold_on_the_newest_root, true, span_reading)) {
            counted.still_deferred_outside_the_walk = add64(counted.still_deferred_outside_the_walk, u64_of(1u));
        }
        if (reads_the_unit_classes && at_the_root_threshold && !still_deferred
            && counted.every_slot_counted_at_the_root_threshold_is_walked_or_deferred
            && !c_walked_slot(device, slot) && !c_slot_exempted_as_quarantined_against_every_walked_version(device, slot)
            && !c_slot_counted_by_an_earlier_record(device, slot, c_alloc_record_count, reclaim_threshold_on_the_newest_root, true, span_reading)) {
            counted.every_slot_counted_at_the_root_threshold_is_walked_or_deferred = false;
        }
    }
    return counted;
}

// 逐槽的下沿里一条引用那一项：C_REFERENCES0 第 reference_index 行（这块盘上的）罩住的每一槽都有 E(θ_high) 里按 span_reading 数的记录罩着。
// 整块盘的那一判（遍历走到的每个槽都在 E(θ_high) 里）是每一条引用的这一项都成立
fn c_slots_of_a_walked_reference_are_recorded(device: u32, reference_index: u32, reclaim_threshold_computed_by_the_checker: vec2<u32>,
    span_reading: u32) -> bool {
    let row = C_REFERENCES0 + reference_index * C_REFERENCE_WORDS;
    let reference_start = vec2<u32>(sw(row + 1u), sw(row + 2u));
    for (var offset = 0u; offset < sw(row + 3u); offset = offset + 1u) {
        let slot = add64(reference_start, u64_of(offset));
        if (!c_slot_counted_by_an_earlier_record(device, slot, c_alloc_record_count, reclaim_threshold_computed_by_the_checker, false,
            span_reading)) {
            return false;
        }
    }
    return true;
}

// 槽数乘槽宽装得进 64 位（槽数 < 2⁵⁰）
fn c_slot_count_fits_in_bytes(slots: vec2<u32>) -> bool {
    return slots.y < (1u << (32u - SLOT_SHIFT));
}

// walk.rs allocated_statistic_lies_within_its_bounds：(L) 记账不少于 |E(θ_high)|；(P) 遍历走到的每个槽都在 E(θ_high) 里；
// (U) 记账不多于 min(|E(t)|, 遍历的槽 + 隔离豁免 + U)。槽数乘槽宽越过 64 位时：下沿那一份记账一定小于它，上沿那一份记账一定不大于它
fn c_allocated_statistic_lies_within_its_bounds(allocated: vec2<u32>, bounds: AllocatedStatisticBoundsOfADevice, walked_and_exempted: vec2<u32>) -> bool {
    let lower = bounds.counted_at_the_checker_reclaim_threshold;
    let lower_bound_holds = c_slot_count_fits_in_bytes(lower) && le64(shl64(lower, SLOT_SHIFT), allocated);
    let walked_and_still_deferred = add64_saturating(walked_and_exempted, bounds.still_deferred_outside_the_walk);
    let at_the_root = bounds.counted_at_the_root_reclaim_threshold;
    let upper = select(walked_and_still_deferred, at_the_root, lt64(at_the_root, walked_and_still_deferred));
    let upper_bound_holds = !c_slot_count_fits_in_bytes(upper) || le64(allocated, shl64(upper, SLOT_SHIFT));
    return lower_bound_holds && bounds.every_walked_slot_is_recorded && upper_bound_holds
        && bounds.every_unreleased_record_is_disjoint_from_the_other_records;
}

// walk.rs judge_walked_reference_starts_against_the_allocation_records_of_the_newest_root（I-3.21，C944（走到的单元起点没有自己的记录），判决
// research/prompts/m3-close-b19-reason-r2-main-verification.md 第二节 Q2「归属」）开头那一段：遍历全部候选版本走到的每个引用（C_REFERENCES0 前
// w_reference_count[0] 行，与 I-3.1 的遍历同一份）的起点，都是最新根分配记录里一条 c_record_counted_at θ_high 的记录的起点。比的记录是 c_quarantine_exempted
// 第二遍读进 C_ALLOC_RECORDS 的那一份：树表里种类 3 的那几棵（前 c_alloc_record_count 行），树表 0 条时是根记录直接持有的那一棵
// （前 c_newest_records_held_by_the_root_record_count 行，C963 候选一，判决 research/prompts/m3-close-b19-reason-r3-main-verification.md 第五节）。
// 树表 0 条而根记录不持有分配记录树：没有对象（不判、不记）；树表指针全零（Q2 第 1 问）、树表读不出、条目窄、分配记录树读不全
// （根记录持有的那一棵也算）、树表非空而没有种类 3 的条目：记射程里判不了。交回逐条引用比 C_ALLOC_RECORDS 前几行，不判的两种交回 NONE；
// 逐条引用那几格在记账分片按记录自述数的那一轮里判（c_judge_the_start_of_a_walked_reference_against_the_records_of_the_newest_root）
fn c_records_compared_by_the_walked_reference_starts() -> u32 {
    if (c_newest_tree_table_pointer_is_all_zero || c_newest_tree_table_entry_count == NONE) {
        mark_part_of_the_range_not_judged(INVARIANT_I_3_21);
        return NONE;
    }
    var records_compared = c_alloc_record_count;
    if (c_newest_tree_table_entry_count == 0u) {
        if (c_newest_tree_held_by_the_root_record == C_TREE_HELD_BY_THE_ROOT_RECORD_NOT_READ) {
            return NONE;
        }
        if (c_newest_tree_held_by_the_root_record == C_TREE_HELD_BY_THE_ROOT_RECORD_NOT_READ_IN_FULL) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_21);
            return NONE;
        }
        records_compared = c_newest_records_held_by_the_root_record_count;
    } else if (!c_newest_records_read_in_full || c_newest_tree_table_has_entries_but_no_allocation_record_tree_entry) {
        mark_part_of_the_range_not_judged(INVARIANT_I_3_21);
        return NONE;
    }
    return records_compared;
}

// I-3.21 里一条引用那一格：C_REFERENCES0 第 reference_index 行的起点落在池里的盘上时（射程同 I-3.1），判它是不是 C_ALLOC_RECORDS 前 records_compared 行里
// 一条 c_record_counted_at θ_high 的记录的起点；落在池外的盘上不判。每个判的引用各判一格（judge 按位或，次序不改结论）
fn c_judge_the_start_of_a_walked_reference_against_the_records_of_the_newest_root(reference_index: u32, records_compared: u32,
    reclaim_threshold_computed_by_the_checker: vec2<u32>) {
    let device_count = hdr(3u);
    let row = C_REFERENCES0 + reference_index * C_REFERENCE_WORDS;
    let device = sw(row);
    var device_is_in_the_pool = false;
    for (var d = 0u; d < device_count; d = d + 1u) {
        if (hdr(12u + d) == device) {
            device_is_in_the_pool = true;
        }
    }
    if (device_is_in_the_pool) {
        var starts_a_counted_record = false;
        for (var j = 0u; j < records_compared; j = j + 1u) {
            let record = C_ALLOC_RECORDS + j * C_ALLOC_RECORD_WORDS;
            if (sw(record) == device && sw(record + 1u) == sw(row + 1u) && sw(record + 2u) == sw(row + 2u)
                && c_record_counted_at(record, reclaim_threshold_computed_by_the_checker, false)) {
                starts_a_counted_record = true;
                break;
            }
        }
        judge(INVARIANT_I_3_21, starts_a_counted_record);
    }
}

// ── 记账那一判（walk.rs check_pool_image 里隔离豁免、I-3.17、I-3.21、I-3.1、I-5.2、I-3.20、I-3.11、I-3.16 那一段）拆成几趟派活：
// 头（accounting_head_main）、分片（accounting_slice_main，每轮几片）、决断（accounting_decide_main）、收尾（accounting_final_main），各一次提交。
// 一个状态上 I-3.1 按记录数槽那几圈随记录条数平方长（整理夹着回退那一组比重最大那张给定状态表单这一判 5 秒多，驱动掐通道），所以按项切成几片：
// 分片按记录自述那一轮（C_SPAN_READING_AS_RECORDED）做每块盘的记录（数槽）、引用（逐槽的下沿）与 I-3.21 每条引用那一格，按单元那一类那一轮
// （C_SPAN_READING_AS_THE_UNIT_CLASSES）只做决断记了「按单元那一类再数一份」的那几块盘，另加记录两两不相交那一项。跨趟的私有变量存在交接段
// （crash_judge_checker_common.wgsl 的 C_ACCOUNTING_CARRY）

// 决断那一趟逐盘记下的结论（交接段 C_ACCOUNTING_CARRY_DECISION_OF_EACH_DEVICE 每盘一字的位）：按记录自述的界成立；三判不成立而这块盘上有 I-3.18 判红的记录，
// 按单元那一类那一轮要再数一份
const C_ACCOUNTING_DECISION_LIES_WITHIN_THE_BOUNDS_AS_RECORDED: u32 = 1u;
const C_ACCOUNTING_DECISION_COUNTED_AGAIN_AS_THE_UNIT_CLASSES: u32 = 2u;

// 一轮的项按权重摊在 [0, C_ACCOUNTING_SLICE_POSITIONS) 这么多个位置上，一片罩其中 [low, high) 那一段（派活那一侧的 ACCOUNTING_SLICE_POSITIONS 同值）
const C_ACCOUNTING_SLICE_POSITION_BITS: u32 = 16u;
const C_ACCOUNTING_SLICE_POSITIONS: u32 = 1u << C_ACCOUNTING_SLICE_POSITION_BITS;

// 分片那一趟一块盘上的三种项：一条记录罩住的槽数进三个计数、一条引用罩住的槽都在记录里（逐槽的下沿）、一条记录与它后面的记录不相交（只在按单元那一类那一轮）
const C_ACCOUNTING_ITEM_SLOTS_OF_A_RECORD: u32 = 0u;
const C_ACCOUNTING_ITEM_SLOTS_OF_A_WALKED_REFERENCE: u32 = 1u;
const C_ACCOUNTING_ITEM_RECORD_DISJOINT_FROM_THE_RECORDS_AFTER_IT: u32 = 2u;
const C_ACCOUNTING_ITEM_KINDS: u32 = 3u;

// 记账分片没有按次序罩满一轮：某一片的 low 不是这一轮罩到的地方、high 越过 C_ACCOUNTING_SLICE_POSITIONS、轮号认不出，或收尾时有一轮没罩到
// C_ACCOUNTING_SLICE_POSITIONS（派活那一侧一轮最多交 MOST_ACCOUNTING_SLICES_OF_A_ROUND 片）。「条目太多」一类，站点号与别处不撞；这个状态交 CPU 判
const C_SITE_ACCOUNTING_SLICES_DID_NOT_COVER_THEIR_ROUND: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (119u << 8u);

// 分片那一趟的派活参数：哪一轮（span_reading 的取值）、这一片罩的位置 [low, high)。只有入口 accounting_slice_main 读它：管线照入口用到的绑定建布局，
// 第 4 号绑定只进那一条管线；uniform 按 16 字节对齐，第四个字补齐
struct AccountingSliceParameters {
    span_reading: u32,
    low: u32,
    high: u32,
    padding_to_sixteen_bytes: u32,
};

@group(0) @binding(4) var<uniform> accounting_slice_parameters: AccountingSliceParameters;

// 分片那几趟里一项的权重：这一项内层最多转几次（上界，加 1 免得为 0），只用来把一轮的项按工作量摊到各片上，算多算少都不改结论。
// 记录那一项（数槽）：跨度 ×（扫前面那几条记录的次数 × 排在它前面的条数 + 仍推迟时逐槽扫引用 + 按单元那一类数、算进 E(t)、不在推迟里时逐槽的上沿
// 扫引用、隔离组与全部记录）；引用那一项（逐槽的下沿）：槽数 × 记录条数；记录那一项（两两不相交）：排在它后面的条数
fn c_weight_of_an_accounting_item(item_kind: u32, item_index: u32, row: u32, span_reading: u32,
    reclaim_threshold_computed_by_the_checker: vec2<u32>, reclaim_threshold_on_the_newest_root: vec2<u32>) -> vec2<u32> {
    let reference_count = w_reference_count[0];
    if (item_kind == C_ACCOUNTING_ITEM_SLOTS_OF_A_RECORD) {
        let at_the_checker_threshold = c_record_counted_at(row, reclaim_threshold_computed_by_the_checker, false);
        let at_the_root_threshold = c_record_counted_at(row, reclaim_threshold_on_the_newest_root, false);
        let still_deferred = c_record_counted_at(row, reclaim_threshold_on_the_newest_root, true);
        let scans_of_the_earlier_records = select(0u, 1u, at_the_checker_threshold) + select(0u, 1u, at_the_root_threshold)
            + select(0u, 1u, still_deferred);
        var scanned_per_slot = scans_of_the_earlier_records * item_index + select(0u, reference_count, still_deferred);
        if (span_reading == C_SPAN_READING_AS_THE_UNIT_CLASSES && at_the_root_threshold && !still_deferred) {
            scanned_per_slot = scanned_per_slot + reference_count + c_occupied_count + c_alloc_record_count;
        }
        return add64(mul32x32(c_span_of_the_record(row, span_reading), scanned_per_slot), u64_of(1u));
    }
    if (item_kind == C_ACCOUNTING_ITEM_SLOTS_OF_A_WALKED_REFERENCE) {
        return add64(mul32x32(sw(row + 3u), c_alloc_record_count), u64_of(1u));
    }
    return u64_of(c_alloc_record_count - item_index);
}

// 一轮里前面各项权重之和是 start 的那一项落不落在这一片：位置 ⌊start × C_ACCOUNTING_SLICE_POSITIONS / 总权重⌋ 在 [low, high) 里。不做除法：
// 拿 start × C_ACCOUNTING_SLICE_POSITIONS 比 low × 总权重与 high × 总权重（low_scaled、high_scaled）。start 小于总权重，位置落在 [0, C_ACCOUNTING_SLICE_POSITIONS) 里；
// 一轮的各片首尾相接、末片 high 是 C_ACCOUNTING_SLICE_POSITIONS 时每一项恰好落在一片里（总权重不到 2⁴⁴，乘出来不越 64 位）
fn c_accounting_item_falls_in_the_slice(start: vec2<u32>, low_scaled: vec2<u32>, high_scaled: vec2<u32>) -> bool {
    let start_scaled = shl64(start, C_ACCOUNTING_SLICE_POSITION_BITS);
    return le64(low_scaled, start_scaled) && lt64(start_scaled, high_scaled);
}

// 交接段里这一轮那一份接着做的地方（resume_row 起，C_ACCOUNTING_RESUME_WORDS 字的后五字）记成：下一片从第 device_index 块盘的第 item_kind 种项的
// 第 item_index 项接着做，排在它前面各项的权重之和是 weight_before（device_index 是盘数时是 I-3.21 的第 item_index 条引用）
fn c_accounting_note_where_the_next_slice_resumes(resume_row: u32, device_index: u32, item_kind: u32, item_index: u32, weight_before: vec2<u32>) {
    sws(resume_row + 2u, device_index);
    sws(resume_row + 3u, item_kind);
    sws(resume_row + 4u, item_index);
    sws(resume_row + 5u, weight_before.x);
    sws(resume_row + 6u, weight_before.y);
}

// 头那一趟：两遍隔离豁免与 F 之下的推迟（walk.rs check_pool_image 不看有没有记账都算），I-3.17 随第二遍判，I-3.21 开头那一段；
// 分片、决断、收尾那几趟要的计数与标记存进交接段，每块盘两种读法的界清成初值、两轮罩到的地方清成 0
fn accounting_head() {
    let newest_root = checker_root_of(c_newest_index);
    // 两遍同一个调用点：第一遍到走读的引用数为止，存下它的豁免数、算完 F 之下的推迟，第二遍到最新根自己的引用数为止。
    // 两遍隔离豁免与 F 之下的推迟照 walk.rs check_pool_image 不看有没有记账都算；判 I-3.1 那几条才要记账
    var exempted_all: array<u32, 8>;
    for (var exemption_pass = 0u; exemption_pass < 2u; exemption_pass = exemption_pass + 1u) {
        var reference_limit = w_reference_count[0];
        if (exemption_pass == 1u) {
            for (var d = 0u; d < 8u; d = d + 1u) {
                exempted_all[d] = c_exempted[d];
            }
            // I-3.16 占着的槽：第一遍（全部走过的版本）整组豁免的那几组，第二遍改写 C_GROUPS 之前收走
            c_occupied_count = 0u;
            c_note_the_quarantined_groups_as_occupied();
            c_deferred_below_the_floor(newest_root.base);
            reference_limit = c_newest_reference_count;
        }
        // I-3.17（walk.rs 在记账那几条之后、不看有没有记账都判）随第二遍一起判：读的同是最新根那棵分配记录树
        c_quarantine_exempted(newest_root.base, reference_limit, exemption_pass == 1u);
    }
    // I-3.21（walk.rs 在 I-3.1 那一段之前、不看有没有记账判）：读的是第二遍刚读进来的最新根全部分配记录，θ_high 与 I-3.1 同一个
    sws(C_ACCOUNTING_CARRY_RECORDS_COMPARED_BY_THE_WALKED_REFERENCE_STARTS, c_records_compared_by_the_walked_reference_starts());
    // I-3.1（RCW，walk.rs 记账那一段）：读的是第二遍刚读进来的最新根全部分配记录；读不全的、树表非空而一条种类 3 都没有的（I-9.16 ④ 判它），
    // 收尾那一趟每块盘记射程里判不了。最新根带的 F 低于 F 生效值时照判（改成 RCW 之后不再有那一道例外）
    let records_read_in_full = c_newest_records_read_in_full;
    let records_comparable = !c_newest_tree_table_has_entries_but_no_allocation_record_tree_entry;
    sws(C_ACCOUNTING_CARRY_RECORDS_ARE_COUNTED, select(0u, 1u, records_read_in_full && records_comparable));
    sws(C_ACCOUNTING_CARRY_ALLOCATION_RECORD_COUNT, c_alloc_record_count);
    sws(C_ACCOUNTING_CARRY_OCCUPIED_COUNT, c_occupied_count);
    sws(C_ACCOUNTING_CARRY_DEFERRED_COUNT, c_deferred_count);
    sws(C_ACCOUNTING_CARRY_KEPT_IN_DEFER_BY_THE_PIN_COUNT, c_kept_in_defer_by_the_pin_count);
    for (var d = 0u; d < 8u; d = d + 1u) {
        sws(C_ACCOUNTING_CARRY_EXEMPTED_AGAINST_EVERY_WALKED_VERSION + d, exempted_all[d]);
        sws(C_ACCOUNTING_CARRY_EXEMPTED_AGAINST_THE_NEWEST_ROOT + d, c_exempted[d]);
        sws(C_ACCOUNTING_CARRY_DECISION_OF_EACH_DEVICE + d, 0u);
        for (var span_reading = C_SPAN_READING_AS_RECORDED; span_reading <= C_SPAN_READING_AS_THE_UNIT_CLASSES; span_reading = span_reading + 1u) {
            c_accounting_carry_bounds_store(d, span_reading, c_initial_allocated_statistic_bounds());
        }
    }
    sws(C_ACCOUNTING_CARRY_SLICES_COVERED_OF_EACH_ROUND, 0u);
    sws(C_ACCOUNTING_CARRY_SLICES_COVERED_OF_EACH_ROUND + 1u, 0u);
}

// 分片、决断、收尾那几趟开头从交接段读回头那一趟存下的计数与第二遍（最新根）隔离豁免逐盘的豁免槽数（undecidable 由各入口自己读回）
fn accounting_carry_load() {
    c_alloc_record_count = sw(C_ACCOUNTING_CARRY_ALLOCATION_RECORD_COUNT);
    c_occupied_count = sw(C_ACCOUNTING_CARRY_OCCUPIED_COUNT);
    c_deferred_count = sw(C_ACCOUNTING_CARRY_DEFERRED_COUNT);
    c_kept_in_defer_by_the_pin_count = sw(C_ACCOUNTING_CARRY_KEPT_IN_DEFER_BY_THE_PIN_COUNT);
    for (var d = 0u; d < 8u; d = d + 1u) {
        c_exempted[d] = sw(C_ACCOUNTING_CARRY_EXEMPTED_AGAINST_THE_NEWEST_ROOT + d);
    }
}

// 分片那一趟：照派活参数 accounting_slice_parameters 做这一轮里落在 [low, high) 那一段位置上的项，加进交接段里那几块盘这一轮那一份界；按记录自述那一轮另判
// I-3.21 落在这一片里的那几条引用。一轮的项是：要数的每块盘（按记录自述那一轮是最新根的分配记录读全且比得上、有记账时的每一块，按单元那一类那一轮
// 只是决断记了要再数一份的那几块）上的每条记录（数槽）、每条引用（逐槽的下沿）、按单元那一类那一轮的每条记录（两两不相交），再是 I-3.21 的每条引用；
// 一项的位置照它前面各项的权重之和按比例摊（c_accounting_item_falls_in_the_slice）。这一片的 low 要接着这一轮罩到的地方，不然记判不了、不做。
// 一轮的项与权重在各片之间不变（分片那几趟不改 C_ALLOC_RECORDS、引用表与决断的结论），位置随前面各项的权重之和只增不减：
// 第一片（low 是 0）先把这一轮的项整轮加一遍权重得出总权重、记进交接段，从第一项做起；之后每一片读回总权重，从前一片停下的那一项接着做
// （交接段里那一份接着做的地方），走到位置不在 high 之前的那一项就停、记下这一项。每一项仍只落在一片里、落在哪一片与整轮从头数的相同；
// 除第一片的权重那一遍外，每片走过的项只有落在这一片里的与停下的那一项
fn accounting_slice() {
    let span_reading = accounting_slice_parameters.span_reading;
    if (span_reading > C_SPAN_READING_AS_THE_UNIT_CLASSES) {
        undecidable = C_SITE_ACCOUNTING_SLICES_DID_NOT_COVER_THEIR_ROUND;
        return;
    }
    let covered_word = C_ACCOUNTING_CARRY_SLICES_COVERED_OF_EACH_ROUND + span_reading;
    if (accounting_slice_parameters.low != sw(covered_word) || accounting_slice_parameters.high < accounting_slice_parameters.low
        || accounting_slice_parameters.high > C_ACCOUNTING_SLICE_POSITIONS) {
        undecidable = C_SITE_ACCOUNTING_SLICES_DID_NOT_COVER_THEIR_ROUND;
        return;
    }
    sws(covered_word, accounting_slice_parameters.high);
    let records_are_counted = w_accounting_seen && sw(C_ACCOUNTING_CARRY_RECORDS_ARE_COUNTED) != 0u;
    var records_compared = NONE;
    if (span_reading == C_SPAN_READING_AS_RECORDED) {
        records_compared = sw(C_ACCOUNTING_CARRY_RECORDS_COMPARED_BY_THE_WALKED_REFERENCE_STARTS);
    }
    let newest_root = checker_root_of(c_newest_index);
    let reclaim_threshold_computed_by_the_checker = c_reclaim_threshold_without_the_pin();
    let reclaim_threshold_on_the_newest_root = rd64(newest_root.base, ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET);
    let device_count = hdr(3u);
    let resume_row = C_ACCOUNTING_CARRY_RESUME_OF_EACH_ROUND + span_reading * C_ACCOUNTING_RESUME_WORDS;
    let is_the_first_slice_of_the_round = accounting_slice_parameters.low == 0u;
    var weight_total = vec2<u32>(sw(resume_row), sw(resume_row + 1u));
    // 两遍走同一套项、同一个次序：第一遍只加权重得出这一轮的总权重（只在第一片走），第二遍做落在这一片里的那几项（每个调用点只展开一份）
    for (var walk_pass = select(1u, 0u, is_the_first_slice_of_the_round); walk_pass < 2u; walk_pass = walk_pass + 1u) {
        let does_the_items = walk_pass == 1u;
        let low_scaled = mul64_32(weight_total, accounting_slice_parameters.low).value;
        let high_scaled = mul64_32(weight_total, accounting_slice_parameters.high).value;
        // 这一遍从哪一项走起：权重那一遍与第一片从头，别的片从交接段里前一片停下的那一项
        var first_device_index = 0u;
        var first_item_kind = 0u;
        var first_item_index = 0u;
        var weight_before = vec2<u32>(0u, 0u);
        if (does_the_items && !is_the_first_slice_of_the_round) {
            first_device_index = sw(resume_row + 2u);
            first_item_kind = sw(resume_row + 3u);
            first_item_index = sw(resume_row + 4u);
            weight_before = vec2<u32>(sw(resume_row + 5u), sw(resume_row + 6u));
        }
        // 做的那一遍走到第一项位置不在这一片里（不在 high 之前）的那一项就停：之后各项的位置都不比它小
        var stopped_before_the_end_of_the_round = false;
        for (var device_index = first_device_index; device_index < device_count && !stopped_before_the_end_of_the_round;
            device_index = device_index + 1u) {
            if (!records_are_counted
                || (span_reading == C_SPAN_READING_AS_THE_UNIT_CLASSES
                    && (sw(C_ACCOUNTING_CARRY_DECISION_OF_EACH_DEVICE + device_index) & C_ACCOUNTING_DECISION_COUNTED_AGAIN_AS_THE_UNIT_CLASSES) == 0u)) {
                continue;
            }
            let device = hdr(12u + device_index);
            var bounds = c_accounting_carry_bounds_load(device_index, span_reading);
            let resumes_in_this_device = device_index == first_device_index;
            for (var item_kind = select(0u, first_item_kind, resumes_in_this_device);
                item_kind < C_ACCOUNTING_ITEM_KINDS && !stopped_before_the_end_of_the_round; item_kind = item_kind + 1u) {
                if (item_kind == C_ACCOUNTING_ITEM_RECORD_DISJOINT_FROM_THE_RECORDS_AFTER_IT && span_reading == C_SPAN_READING_AS_RECORDED) {
                    continue;
                }
                let is_a_walked_reference = item_kind == C_ACCOUNTING_ITEM_SLOTS_OF_A_WALKED_REFERENCE;
                let item_count = select(c_alloc_record_count, w_reference_count[0], is_a_walked_reference);
                for (var item_index = select(0u, first_item_index, resumes_in_this_device && item_kind == first_item_kind);
                    item_index < item_count; item_index = item_index + 1u) {
                    let row = select(C_ALLOC_RECORDS + item_index * C_ALLOC_RECORD_WORDS, C_REFERENCES0 + item_index * C_REFERENCE_WORDS,
                        is_a_walked_reference);
                    if (sw(row) != device) {
                        continue;
                    }
                    let start = weight_before;
                    if (does_the_items && !lt64(shl64(start, C_ACCOUNTING_SLICE_POSITION_BITS), high_scaled)) {
                        c_accounting_note_where_the_next_slice_resumes(resume_row, device_index, item_kind, item_index, start);
                        stopped_before_the_end_of_the_round = true;
                        break;
                    }
                    weight_before = add64(weight_before, c_weight_of_an_accounting_item(item_kind, item_index, row, span_reading,
                        reclaim_threshold_computed_by_the_checker, reclaim_threshold_on_the_newest_root));
                    if (!does_the_items || !c_accounting_item_falls_in_the_slice(start, low_scaled, high_scaled)) {
                        continue;
                    }
                    if (item_kind == C_ACCOUNTING_ITEM_SLOTS_OF_A_RECORD) {
                        bounds = c_count_the_slots_of_a_record_into_the_bounds(bounds, device, item_index, reclaim_threshold_computed_by_the_checker,
                            reclaim_threshold_on_the_newest_root, span_reading);
                    } else if (is_a_walked_reference) {
                        if (bounds.every_walked_slot_is_recorded
                            && !c_slots_of_a_walked_reference_are_recorded(device, item_index, reclaim_threshold_computed_by_the_checker, span_reading)) {
                            bounds.every_walked_slot_is_recorded = false;
                        }
                    } else if (bounds.every_unreleased_record_is_disjoint_from_the_other_records
                        && !c_unit_class_reading_of_the_record_is_disjoint_from_the_records_after_it(device, item_index)) {
                        bounds.every_unreleased_record_is_disjoint_from_the_other_records = false;
                    }
                }
            }
            if (does_the_items) {
                c_accounting_carry_bounds_store(device_index, span_reading, bounds);
            }
        }
        if (records_compared != NONE && !stopped_before_the_end_of_the_round) {
            for (var reference_index = select(0u, first_item_index, first_device_index == device_count);
                reference_index < w_reference_count[0]; reference_index = reference_index + 1u) {
                let start = weight_before;
                if (does_the_items && !lt64(shl64(start, C_ACCOUNTING_SLICE_POSITION_BITS), high_scaled)) {
                    c_accounting_note_where_the_next_slice_resumes(resume_row, device_count, 0u, reference_index, start);
                    stopped_before_the_end_of_the_round = true;
                    break;
                }
                weight_before = add64(weight_before, u64_of(records_compared + 1u));
                if (does_the_items && c_accounting_item_falls_in_the_slice(start, low_scaled, high_scaled)) {
                    c_judge_the_start_of_a_walked_reference_against_the_records_of_the_newest_root(reference_index, records_compared,
                        reclaim_threshold_computed_by_the_checker);
                }
            }
        }
        if (!does_the_items) {
            weight_total = weight_before;
            sws(resume_row, weight_total.x);
            sws(resume_row + 1u, weight_total.y);
        } else if (!stopped_before_the_end_of_the_round) {
            // 这一轮的项做完了：之后的片（罩到 C_ACCOUNTING_SLICE_POSITIONS 之前的那几片）从最后一条引用之后接着，一项都不做
            c_accounting_note_where_the_next_slice_resumes(resume_row, device_count, 0u, w_reference_count[0], weight_before);
        }
    }
}

// 这块盘（第 device_index 块）遍历走到的槽数加第一遍（全部走过的版本）隔离豁免的槽数：I-3.1 上沿那一份的「遍历的槽 + 隔离豁免」
fn c_walked_and_exempted_slots_of_the_device(device: u32, device_index: u32) -> vec2<u32> {
    return add64(c_slots_referenced(device, w_reference_count[0]), u64_of(sw(C_ACCOUNTING_CARRY_EXEMPTED_AGAINST_EVERY_WALKED_VERSION + device_index)));
}

// 决断那一趟（按记录自述那一轮分片罩满之后）：有记账、最新根的分配记录读全且比得上时逐块盘照按记录自述那一份界判三判（记下成不成立）；
// 不成立而这块盘上有 I-3.18 判红的记录（c_note_the_unit_class_slots_of_the_records_the_span_invariant_reddens 把那几条记录改记的槽数写进
// C_ALLOC_RECORDS 第八个字，只改这块盘的记录、按记录自述那一份不读那几位）就记下按单元那一类那一轮要再数一份（Q9 的条件），
// 那一份界清成初值。两块盘先都按自述数完、再逐盘记单元那一类，与原来逐盘交替做的结果相同（两种读法都只数这块盘上的记录）
fn accounting_decide() {
    if (!w_accounting_seen || sw(C_ACCOUNTING_CARRY_RECORDS_ARE_COUNTED) == 0u) {
        return;
    }
    let device_count = hdr(3u);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        var allocated = vec2<u32>(0u, 0u);
        let have_allocated = c_accounting_row(STATISTIC_ALLOCATED_BYTES, device, &allocated);
        let bounds = c_accounting_carry_bounds_load(d, C_SPAN_READING_AS_RECORDED);
        let lies_within_its_bounds = have_allocated
            && c_allocated_statistic_lies_within_its_bounds(allocated, bounds, c_walked_and_exempted_slots_of_the_device(device, d))
            && bounds.every_slot_counted_at_the_root_threshold_is_walked_or_deferred;
        var decision = select(0u, C_ACCOUNTING_DECISION_LIES_WITHIN_THE_BOUNDS_AS_RECORDED, lies_within_its_bounds);
        if (!lies_within_its_bounds && have_allocated && c_note_the_unit_class_slots_of_the_records_the_span_invariant_reddens(device)) {
            decision = decision | C_ACCOUNTING_DECISION_COUNTED_AGAIN_AS_THE_UNIT_CLASSES;
            c_accounting_carry_bounds_store(d, C_SPAN_READING_AS_THE_UNIT_CLASSES, c_initial_allocated_statistic_bounds());
        }
        sws(C_ACCOUNTING_CARRY_DECISION_OF_EACH_DEVICE + d, decision);
    }
}

// 收尾那一趟里逐块盘那几判：I-3.1（两轮罩满之后：按记录自述那一份成立就判成立；不成立而按单元那一类那一份整份过三判、另加逐槽的上沿
// （walk.rs allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading），配不上的那块全落在 I-3.18 判红的单元里，记射程里判不了，不判红）、
// I-5.2、I-3.20 (a)、I-3.11
fn c_judge_the_allocated_statistic_and_the_sums_of_each_device() {
    let device_count = hdr(3u);
    let records_are_counted = sw(C_ACCOUNTING_CARRY_RECORDS_ARE_COUNTED) != 0u;
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        var allocated = vec2<u32>(0u, 0u);
        let have_allocated = c_accounting_row(1u, device, &allocated);
        if (records_are_counted) {
            let decision = sw(C_ACCOUNTING_CARRY_DECISION_OF_EACH_DEVICE + d);
            let lies_within_its_bounds = (decision & C_ACCOUNTING_DECISION_LIES_WITHIN_THE_BOUNDS_AS_RECORDED) != 0u;
            var the_misfit_lies_in_units_the_span_invariant_reddens = false;
            if ((decision & C_ACCOUNTING_DECISION_COUNTED_AGAIN_AS_THE_UNIT_CLASSES) != 0u) {
                let bounds = c_accounting_carry_bounds_load(d, C_SPAN_READING_AS_THE_UNIT_CLASSES);
                the_misfit_lies_in_units_the_span_invariant_reddens = have_allocated
                    && c_allocated_statistic_lies_within_its_bounds(allocated, bounds, c_walked_and_exempted_slots_of_the_device(device, d))
                    && bounds.every_slot_counted_at_the_root_threshold_is_walked_or_deferred;
            }
            if (the_misfit_lies_in_units_the_span_invariant_reddens) {
                mark_part_of_the_range_not_judged(INVARIANT_I_3_1);
            } else {
                judge(INVARIANT_I_3_1, lies_within_its_bounds);
            }
        } else {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_1);
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
        // I-3.20 (a)（walk.rs judge_free_allocated_and_unreclaimable_of_a_device_sum_to_the_capacity，C70）：不可回收（统计量 3）那一行在、是 0；
        // 空闲 + 已分配 + 不可回收 = 单元区容量（哪一行缺、加起来溢出、单元区起点越过盘末都判红）
        var unreclaimable = vec2<u32>(0u, 0u);
        let have_unreclaimable = c_accounting_row(3u, device, &unreclaimable);
        judge(INVARIANT_I_3_20, have_unreclaimable && is_zero64(unreclaimable));
        var sums_to_the_capacity = have_free && have_allocated && have_unreclaimable && le64(handoff_unit_area_start_slot(), slots)
            && !add64_overflows(free, allocated);
        if (sums_to_the_capacity) {
            sums_to_the_capacity = !add64_overflows(add64(free, allocated), unreclaimable)
                && eq64(add64(add64(free, allocated), unreclaimable), shl64(sub64(slots, handoff_unit_area_start_slot()), SLOT_SHIFT));
        }
        judge(INVARIANT_I_3_20, sums_to_the_capacity);
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

// 收尾那一趟：两轮分片都罩满了没有（没罩满这个状态记判不了、交 CPU）；有记账时逐块盘那几判、I-3.20 (b)、I-3.16（walk.rs 的次序；没有记账时连
// I-3.20 (b)、I-3.16 一起不判，同 walk.rs），再是 I-3.12（每条根各自那张实例表，原来扫描那一截排在记账之后的那一判）
fn accounting_final() {
    if (sw(C_ACCOUNTING_CARRY_SLICES_COVERED_OF_EACH_ROUND) != C_ACCOUNTING_SLICE_POSITIONS
        || sw(C_ACCOUNTING_CARRY_SLICES_COVERED_OF_EACH_ROUND + 1u) != C_ACCOUNTING_SLICE_POSITIONS) {
        undecidable = C_SITE_ACCOUNTING_SLICES_DID_NOT_COVER_THEIR_ROUND;
    }
    if (w_accounting_seen) {
        let newest_root = checker_root_of(c_newest_index);
        let written_under_effective_floor = eq64(newest_root.floor, c_effective_floor);
        c_judge_the_allocated_statistic_and_the_sums_of_each_device();
        // I-3.20 (b)（walk.rs judge_the_committed_reservation_against_its_closed_form，C70）：walk.rs 在 I-3.16 之前判
        judge_the_committed_reservation_against_its_closed_form(newest_root.base);
        // I-3.16（walk.rs 在 I-3.1 那一格之后判，同一个条件：有记账、最新根的记账按 F 生效值记的）
        if (written_under_effective_floor) {
            judge_fragmentation_and_empty_segment_statistics();
        }
    }
    judge_instance_table_rows_against_the_roots_referencing_them();
}

// I-3.20 (b) 要的一棵容器树（墓碑树、退役树）的内部层数：树表 tt 里种类 kind 的条目恰一条或没有（没有取 0），根指针全零取 0，
// 否则读根节点（按这一版的中央映射认位置）取层级减一（不到 1 取 0）。条目多于一条、根读不出交回 NONE（判不了）
fn c_internal_levels_of_the_container_tree_of_kind(tt: Node, kind: u32) -> u32 {
    var entries_of_the_kind = 0u;
    var root_pointer = zero_pointer();
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == kind) {
            entries_of_the_kind = entries_of_the_kind + 1u;
            root_pointer = parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
        }
    }
    if (entries_of_the_kind == 0u || (entries_of_the_kind == 1u && root_pointer.all_zero)) {
        return 0u;
    }
    if (entries_of_the_kind > 1u) {
        return NONE;
    }
    let root = r_read_located_index_node(root_pointer);
    if (!root.ok) {
        return NONE;
    }
    return max(root.level, 1u) - 1u;
}

// I-3.20 (b)：最新根记账里的标签 6 等于 D3（空间分配） 已定项 2 的闭式
// 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)；树表 0 条取 0。副本数取记账里已分配（统计量 1）那一行的盘数。
// 树表、两棵容器树的根、inode 树（恰一棵）读不全，树表非空而没有 deleted_inodes 条目：判不了（CPU 报部分没判）。乘出来过 2^64 判红（CPU 闭式交回 None）
fn judge_the_committed_reservation_against_its_closed_form(newest_base: u32) {
    var recorded = vec2<u32>(0u, 0u);
    let have_recorded = c_accounting_row(6u, NONE, &recorded);
    var replicas = 0u;
    for (var k = 0u; k < w_accounting_row_count; k = k + 1u) {
        if (sw(C_ACCOUNTING_ROWS + k * 4u) == 1u) {
            replicas = replicas + 1u;
        }
    }
    let tt = g_tree_table_of_the_version(newest_base, G_ROOT_RECORD_TREE_TABLE_POINTER);
    if (!tt.ok) {
        mark_part_of_the_range_not_judged(INVARIANT_I_3_20);
        return;
    }
    var has_a_deleted_inodes_entry = false;
    var inode_tree_entries = 0u;
    var inode_tree_root = zero_pointer();
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        let kind = rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
        if (kind == TREE_KIND_DELETED_INODES) {
            has_a_deleted_inodes_entry = true;
        }
        if (kind == TREE_KIND_INODE) {
            inode_tree_entries = inode_tree_entries + 1u;
            inode_tree_root = parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
        }
    }
    // 树表非空而一条 deleted_inodes 都没有（条目缺了或种类码没登记；I-9.16 ④ 判红的那块畸形）：这一版取 0 还是照闭式算认不出，判不了
    // （walk.rs committed_reservation_basis_of_a_version_without_judging 交回说明，CPU 报部分没判、点名 I-9.16）
    if (!has_a_deleted_inodes_entry && tt.entry_count > 0u) {
        mark_part_of_the_range_not_judged(INVARIANT_I_3_20);
        return;
    }
    if (!has_a_deleted_inodes_entry) {
        judge(INVARIANT_I_3_20, have_recorded && is_zero64(recorded));
        return;
    }
    s_use_the_central_mapping_named_at(newest_base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
    let tombstone_tree_internal_levels = c_internal_levels_of_the_container_tree_of_kind(tt, TREE_KIND_TOMBSTONE);
    let retirement_tree_internal_levels = c_internal_levels_of_the_container_tree_of_kind(tt, TREE_KIND_RETIREMENT);
    if (tombstone_tree_internal_levels == NONE || retirement_tree_internal_levels == NONE || inode_tree_entries != 1u
        || !d_read_inode_records(inode_tree_root)) {
        mark_part_of_the_range_not_judged(INVARIANT_I_3_20);
        return;
    }
    var live_objects = 0u;
    for (var r = 0u; r < d_inode_record_count; r = r + 1u) {
        if (d_inode_record_nlink(D_INODE_RECORDS + r * D_INODE_RECORD_WORDS) != 0u) {
            live_objects = live_objects + 1u;
        }
    }
    // 2 是一个开放容器的槽数（32768 ÷ 16384），1 是树根那一槽；41 = ⌈32768 ÷ 815⌉、35 = ⌈32768 ÷ 959⌉（每对象先取整再乘）
    // 层数是码 2 头里的一字节、记录数是表里的行数：常驻槽数、两项乘积与它们的和都装得进 u64，只有乘副本数那一步可能过 2^64
    let open_container_slots = DATA_UNIT_BYTES >> SLOT_SHIFT;
    let resident_slots = u64_of(open_container_slots + tombstone_tree_internal_levels + 1u + open_container_slots + retirement_tree_internal_levels + 1u);
    let tombstone_bytes_per_live_object = (DATA_UNIT_BYTES + DELETION_TOMBSTONE_CONTAINER_RECORDS - 1u) / DELETION_TOMBSTONE_CONTAINER_RECORDS;
    let retirement_bytes_per_inode_record = (DATA_UNIT_BYTES + DELETION_RETIREMENT_CONTAINER_RECORDS - 1u) / DELETION_RETIREMENT_CONTAINER_RECORDS;
    let one_replica = add64(add64(shl64(resident_slots, SLOT_SHIFT), mul32x32(live_objects, tombstone_bytes_per_live_object)),
        mul32x32(d_inode_record_count, retirement_bytes_per_inode_record));
    let expected = mul64_32(one_replica, replicas);
    judge(INVARIANT_I_3_20, have_recorded && !expected.overflow && eq64(recorded, expected.value));
}
