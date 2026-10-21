// ───────────────────────── 恢复与 oracle ─────────────────────────

struct Recovery {
    outcome: u32,            // 走读结局；择不到根、重放停下也算 FAILED
    has_effective: bool,
    effective_instance: u32,
    effective_txg: vec2<u32>,
    content_matches: bool,
    failure: u32,
};
fn recover(consult: bool, cfg_choice: SysChoice) -> Recovery {
    var out: Recovery;
    out.outcome = OUTCOME_FAILED;
    out.has_effective = false;
    out.effective_instance = 0u;
    out.effective_txg = vec2<u32>(0u, 0u);
    out.content_matches = false;
    out.failure = FAILURE_POINT_NONE;
    if (!cfg_choice.ok) {
        out.failure = cfg_choice.failure;
        return out;
    }
    let chosen = choose_root(cfg_choice.cfg);
    if (!chosen.found) {
        out.failure = FAILURE_POINT_NO_VALID_ROOT;
        return out;
    }
    var effective = chosen.root;
    if (consult) {
        // 看 journal 那一遍：先扫环（recovery::scan_journal），再从所选根接着重放（此前漏了这一句，重放的记录表恒空，
        // 两遍恢复落到的是同一条根；红位上看不出来，对拍比落到的根与结局才抓到）
        scan_journal(cfg_choice.cfg);
        let replayed = replay_journal(chosen.root, cfg_choice.cfg.ring_bytes);
        if (!replayed.ok) {
            out.failure = replayed.failure;
            return out;
        }
        effective = replayed.root;
    }
    out.has_effective = true;
    out.effective_instance = effective.instance;
    out.effective_txg = effective.txg;
    // 走读失败、看 journal 的那一遍再走一次退回见证到的那一版（recovery::recover 的 F2）：一个调用点走两遍，内核展开只多一份判法、不多一份走读
    var root_to_walk = effective;
    for (var walk_round = 0u; walk_round < 2u; walk_round = walk_round + 1u) {
        let walked = walk_to_file(root_to_walk, cfg_choice.cfg, expected_version_index(root_to_walk.instance, root_to_walk.txg));
        // 第一遍照报；退回的那一版走不通时照报第一遍的失败（落到的根也不换）
        if (walk_round == 0u || walked.outcome != OUTCOME_FAILED) {
            out.effective_instance = root_to_walk.instance;
            out.effective_txg = root_to_walk.txg;
            out.outcome = walked.outcome;
            out.failure = walked.failure;
            out.content_matches = walked.content_matches;
        }
        if (walk_round == 1u || walked.outcome != OUTCOME_FAILED || !consult) {
            break;
        }
        let fallback = newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(cfg_choice.cfg, chosen.root, effective);
        if (!fallback.found) {
            break;
        }
        root_to_walk = fallback.root;
    }
    return out;
}

// recovery::system_configuration_slots_of_one_device_read_once 收进 self_verified 的那一判：SystemConfiguration::parse_slot 交 Ok
// （magic、整槽校验和、incompat 位认得、每区槽数在 [4, 16]；读者收不收那几个池级值不在这一判里）且 fsid 与所选系统配置的相同
fn system_configuration_slot_self_verified_with_the_fsid(s: Slice, fsid: vec4<u32>) -> bool {
    if (s.version == NONE || s.length < SYSTEM_CONFIGURATION_SLOT_BYTES || rd32(s.base, 0u) != MAGIC_SFSB || s.c0 != 1u) {
        return false;
    }
    if (!incompat_bits_are_recognized(s.base)) {
        return false;
    }
    let slots_per_region = rd8(s.base, 362u);
    return slots_per_region >= 4u && slots_per_region <= 16u && fsid_matches(s.base, 102u, fsid);
}
// 系统配置槽里 journal tail 的偏移（system_configuration.rs 的 TAIL_OFFSET）
const SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET: u32 = 469u;
// 内存池按扇区对齐判读请求（memory_pool 的 check_request_against_the_device_contract）：槽距不按 512 对齐的槽 1 读不到、算设备报错
const SECTOR_BYTES_OF_THE_MEMORY_POOL: u32 = 512u;

struct FallbackVersion {
    found: bool,
    root: RootFields,
};
// 择根的序（recovery 里 order_of_the_root_choice：checkpoint_txg 为主、实例代号破平局）上 (earlier_txg, earlier_instance) 排在 (later_txg, later_instance) 之前
fn precedes_in_the_root_choice_order(earlier_txg: vec2<u32>, earlier_instance: u32, later_txg: vec2<u32>, later_instance: u32) -> bool {
    return lt64(earlier_txg, later_txg) || (eq64(earlier_txg, later_txg) && earlier_instance < later_instance);
}
// recovery::instance_table_chain_of_root 读出的整张表的行，写进 R_ROWS（abandoned_by_rows 照它判）：沿链读法与走读判表读不读得出的
// instance_table_chain_readable 同一份（unit::parse_packed_unit、InstanceTablePage::parse，链指针头部按走读那一种判），只多把每片的行记下来。
// 一片读不出、解不开交 false；行多过 R_ROWS 装得下的、链长过 256 片的记判不了（CPU 不设这两个上限，那几个状态交 CPU 判）
fn load_rows_of_the_instance_table_chain(root: RootFields) -> bool {
    row_count = 0u;
    var pointer = root.instance_table;
    var page_index = vec2<u32>(0u, 0u);
    for (var page = 0u; page < 256u; page = page + 1u) {
        let page_read = read_unit_via_locations(pointer, DATA_UNIT_BYTES);
        if (page_read.version == NONE) {
            return false;
        }
        let page_unit = parse_packed_unit(page_read);
        let parsed_page = parse_instance_table_page(page_unit, page_index);
        if (parsed_page.verdict == 2u) {
            return false;
        }
        for (var row_index = 0u; row_index + 1u < page_unit.record_count; row_index = row_index + 1u) {
            if (row_count >= MAX_ROWS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                return false;
            }
            let row_offset = packed_record_offset(page_unit, row_index);
            sws(R_ROWS + row_count * 3u, rd32(page_unit.base, row_offset + 1u));
            let row_txg = rd64(page_unit.base, row_offset + 5u);
            sws(R_ROWS + row_count * 3u + 1u, row_txg.x);
            sws(R_ROWS + row_count * 3u + 2u, row_txg.y);
            row_count = row_count + 1u;
        }
        if (parsed_page.verdict == 0u) {
            return true;
        }
        pointer = parsed_page.next;
        page_index = add64(page_index, u64_of(1u));
    }
    undecidable = UNDECIDABLE_TOO_DEEP;
    return false;
}
// recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to，F2_NEWEST 那一形（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md`
// 第二节 K1 / K2 W4 H4 行与第三节第 2 条，用户 2026-10-08 定「四形都修」）：所选那一版（一条记录都没施加的所选根本身）走读失败、系统配置没见证它时，
// 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（R_ROOTS 按根环次序记着 choose_root 自证过的根；同为最新的几条取根环次序上
// 最后那条，同 max_by_key）；所选根的实例表读不出、解不开时只认 (txg, 实例) 与见证到的那一版恰好相等的那条。「见证到的那一版」是计数器 = c_见证 的那条记录
// 所属那一版。见证判不出、见证过、那条记录读不出、c_见证 = 0 都不退：c_见证 = 0 时环里读得出计数器 0 的记录（写者的计数器从 1 起，那一条只能是
// 坏盘或伪造的）也不退，与 CPU 同一个守卫（D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」）。退回的那一版不重放、直接走。
// 「见证过」：所选那一版带末条标志的记录读得出 ⇒ 它的计数器 ≤ c_见证；读不出而 c_见证 那条记录读得出 ⇒ 那条记录的 (txg, 实例) 不比所选那一版旧；
// 两条都读不出当见证过。c_见证 落在所选那次发布自己的某条记录上时下界就是所选那一版、区间是空的，同样不退
fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(cfg: SysConfig, chosen: RootFields, selected: RootFields) -> FallbackVersion {
    var out: FallbackVersion;
    out.found = false;
    out.root = invalid_root();
    if (selected.instance != chosen.instance || !eq64(selected.txg, chosen.txg)) {
        return out;
    }
    // c_见证：每块盘两个系统配置槽（偏移 0 与所选系统配置的槽距）里自证过的 journal tail 取最大；见证判不出（R2）：有槽读不到、又没有一块盘两槽都自证过
    let device_count = hdr(3u);
    var witnessed_counter = vec2<u32>(0u, 0u);
    var some_slot_reports_a_read_error = false;
    var some_device_has_both_slots_self_verified = false;
    for (var d = 0u; d < device_count; d = d + 1u) {
        var self_verified_slots = 0u;
        for (var slot = 0u; slot < 2u; slot = slot + 1u) {
            let offset = select(0u, cfg.spacing, slot == 1u);
            // 内存池另把越过盘尾的读报成读错（check_request_against_the_device_contract 的 OutOfRange），这里不另判那一支：走到这一步时所选根已经
            // 从根环里读出来了（根环起点 ROOT_RING_BASE_BYTES = 1 MiB，CPU 读得出就说明盘至少这么大，池里每块盘一样大），读者收下的槽距不超过
            // 1 MiB − 4096（parse_system_configuration 的 largest_spacing），两槽 [0, 4096)、[槽距, 槽距 + 4096) 恒在盘内（判决
            // `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 V1 行「建模差两处」之一：在走得到的状态上两边同判）
            if ((offset % SECTOR_BYTES_OF_THE_MEMORY_POOL) != 0u) {
                some_slot_reports_a_read_error = true;
                continue;
            }
            let s = read_at(hdr(12u + d), u64_of(offset), SYSTEM_CONFIGURATION_SLOT_BYTES);
            if (!system_configuration_slot_self_verified_with_the_fsid(s, cfg.fsid)) {
                continue;
            }
            self_verified_slots = self_verified_slots + 1u;
            let tail = rd64(s.base, SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET);
            if (lt64(witnessed_counter, tail)) {
                witnessed_counter = tail;
            }
        }
        if (self_verified_slots == 2u) {
            some_device_has_both_slots_self_verified = true;
        }
    }
    if (some_slot_reports_a_read_error && !some_device_has_both_slots_self_verified) {
        return out;
    }
    var has_last_record_of_the_selected_version = false;
    var last_record_counter = vec2<u32>(0u, 0u);
    var has_witnessed_record = false;
    var witnessed_record_txg = vec2<u32>(0u, 0u);
    var witnessed_record_instance = 0u;
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = record_at(i);
        if (r.instance == selected.instance && eq64(r.txg, selected.txg) && r.last
            && (!has_last_record_of_the_selected_version || lt64(last_record_counter, r.counter))) {
            has_last_record_of_the_selected_version = true;
            last_record_counter = r.counter;
        }
        if (eq64(r.counter, witnessed_counter)
            && (!has_witnessed_record || lt64(witnessed_record_txg, r.txg)
                || (eq64(witnessed_record_txg, r.txg) && witnessed_record_instance < r.instance))) {
            has_witnessed_record = true;
            witnessed_record_txg = r.txg;
            witnessed_record_instance = r.instance;
        }
    }
    let witnessed_record_is_older_than_the_selected_version = lt64(witnessed_record_txg, selected.txg)
        || (eq64(witnessed_record_txg, selected.txg) && witnessed_record_instance < selected.instance);
    var is_witnessed = true;
    if (has_last_record_of_the_selected_version) {
        is_witnessed = le64(last_record_counter, witnessed_counter);
    } else if (has_witnessed_record) {
        is_witnessed = !witnessed_record_is_older_than_the_selected_version;
    }
    if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {
        return out;
    }
    let chosen_instance_table_is_readable = load_rows_of_the_instance_table_chain(chosen);
    for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
        var root_read: Slice = no_slice();
        root_read.version = sw(R_ROOTS + root_index * 2u);
        root_read.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, root_read.version * VERSION_WORDS + 1u);
        root_read.length = cfg.physical_block_size;
        root_read.c0 = tab(T_VERSIONS, root_read.version * VERSION_WORDS + 2u);
        let root = parse_root(root_read, cfg.fsid);
        let is_the_witnessed_version = root.instance == witnessed_record_instance && eq64(root.txg, witnessed_record_txg);
        var is_admissible = is_the_witnessed_version;
        if (chosen_instance_table_is_readable) {
            is_admissible = !abandoned_by_rows(root.instance, root.txg);
        }
        let is_in_the_range = !precedes_in_the_root_choice_order(root.txg, root.instance, witnessed_record_txg, witnessed_record_instance)
            && precedes_in_the_root_choice_order(root.txg, root.instance, chosen.txg, chosen.instance);
        if (is_in_the_range && is_admissible
            && (!out.found || !precedes_in_the_root_choice_order(root.txg, root.instance, out.root.txg, out.root.instance))) {
            out.found = true;
            out.root = root;
        }
    }
    // 不跳过一次末条读得出、根读不出、不被所选根的实例表抛弃的发布（CPU 同名函数里那一判，攻方 `research/prompts/m3-batch37-43-code-r1-opus-output.md`
    // 第二节 Y4）：退到的那一条与所选根之间（择根的序，两头都不含）有一条带末条标志的记录，R_ROOTS 里没有 (txg, 实例) 与它相等的根，
    // 所选根的实例表读得出时它的那次发布不被表抛弃（读不出时算不被抛弃）——就不退。R_ROOTS 里的根 choose_root 都自证过，只取实例代号与 txg 那两段比
    if (out.found) {
        for (var i = 0u; i < record_count; i = i + 1u) {
            let r = record_at(i);
            if (!r.last
                || !precedes_in_the_root_choice_order(out.root.txg, out.root.instance, r.txg, r.instance)
                || !precedes_in_the_root_choice_order(r.txg, r.instance, chosen.txg, chosen.instance)
                || (chosen_instance_table_is_readable && abandoned_by_rows(r.instance, r.txg))) {
                continue;
            }
            var root_of_the_publish_reads = false;
            for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
                let root_base = B[T_VERSION_BYTES] + tab(T_VERSIONS, sw(R_ROOTS + root_index * 2u) * VERSION_WORDS + 1u);
                if (rd32(root_base, 24u) == r.instance && eq64(rd64(root_base, 28u), r.txg)) {
                    root_of_the_publish_reads = true;
                }
            }
            if (!root_of_the_publish_reads) {
                out.found = false;
                out.root = invalid_root();
                return out;
            }
        }
    }
    return out;
}
struct NewestRoot {
    found: bool,
    txg: vec2<u32>,
    instance: u32,
};
// memory_pool::newest_persisted_root：持久的根槽写里 (txg, 实例) 最大的
fn newest_persisted_root() -> NewestRoot {
    var out: NewestRoot;
    out.found = false;
    out.txg = vec2<u32>(0u, 0u);
    out.instance = 0u;
    let count = hdr(7u);
    for (var i = 0u; i < count; i = i + 1u) {
        let root_write = tab(T_PUBLISHES, i * PUBLISH_WORDS);
        if (!is_persisted(root_write)) {
            continue;
        }
        let instance = tab(T_PUBLISHES, i * PUBLISH_WORDS + 1u);
        let txg = vec2<u32>(tab(T_PUBLISHES, i * PUBLISH_WORDS + 2u), tab(T_PUBLISHES, i * PUBLISH_WORDS + 3u));
        if (!out.found || lt64(out.txg, txg) || (eq64(out.txg, txg) && out.instance < instance)) {
            out.found = true;
            out.txg = txg;
            out.instance = instance;
        }
    }
    return out;
}
// 期望版本表第 5 字（crash_judge_tables::EXPECTED_VERSION_WITH_A_FILE / EXPECTED_VERSION_WITHOUT_A_FILE）：这一版有文件，还是文件从这一版起没有了
const EXPECTED_VERSION_WITH_A_FILE: u32 = 0u;
const EXPECTED_VERSION_WITHOUT_A_FILE: u32 = 1u;

// crash::classified_oracle_violation_for_the_version_table：落到的根是带文件的一版，读回的得是它的内容；是「文件从这一版起没有了」的一版，
// 只许报没有文件；不在表里，报没有文件时看比它旧的最新那一项（按 (txg, 实例)）——那一项有文件就是违例，没有文件、或没有更旧的项不是
// （文件从不消失的表上与「任一更旧版有文件」相同）；不在表里却读出了内容一律是违例
fn classify_oracle(recovery: Recovery, newest: NewestRoot) -> u32 {
    if (!recovery.has_effective) {
        return ORACLE_NO_ROOT_CHOSEN;
    }
    if (newest.found) {
        if (lt64(recovery.effective_txg, newest.txg) || (eq64(recovery.effective_txg, newest.txg) && recovery.effective_instance < newest.instance)) {
            return ORACLE_RECOVERED_TO_AN_OLDER_ROOT;
        }
    }
    let version = expected_version_index(recovery.effective_instance, recovery.effective_txg);
    var version_has_a_file = false;
    if (version != NONE) {
        version_has_a_file = tab(T_EXPECTED, version * EXPECTED_WORDS + 5u) == EXPECTED_VERSION_WITH_A_FILE;
    }
    if (recovery.outcome == OUTCOME_FAILED) {
        return ORACLE_READ_FAILED;
    }
    if (recovery.outcome == OUTCOME_NO_FILE) {
        if (version == NONE) {
            // 比它旧的最新那一项下面有文件？
            let count = hdr(8u);
            var newest_older = NONE;
            var newest_older_txg = vec2<u32>(0u, 0u);
            var newest_older_instance = 0u;
            for (var i = 0u; i < count; i = i + 1u) {
                let instance = tab(T_EXPECTED, i * EXPECTED_WORDS);
                let txg = vec2<u32>(tab(T_EXPECTED, i * EXPECTED_WORDS + 1u), tab(T_EXPECTED, i * EXPECTED_WORDS + 2u));
                let is_older = lt64(txg, recovery.effective_txg) || (eq64(txg, recovery.effective_txg) && instance < recovery.effective_instance);
                let is_newer_than_the_newest_older = newest_older == NONE || lt64(newest_older_txg, txg)
                    || (eq64(newest_older_txg, txg) && newest_older_instance < instance);
                if (is_older && is_newer_than_the_newest_older) {
                    newest_older = i;
                    newest_older_txg = txg;
                    newest_older_instance = instance;
                }
            }
            if (newest_older != NONE && tab(T_EXPECTED, newest_older * EXPECTED_WORDS + 5u) == EXPECTED_VERSION_WITH_A_FILE) {
                return ORACLE_NO_FILE_WHERE_AN_OLDER_ROOT_HAS_ONE;
            }
            return ORACLE_NONE;
        }
        if (!version_has_a_file) {
            return ORACLE_NONE;
        }
        return ORACLE_NO_FILE_WHERE_THIS_ROOT_HAS_ONE;
    }
    if (version == NONE || !version_has_a_file) {
        return ORACLE_FILE_READ_WHERE_THIS_ROOT_HAS_NONE;
    }
    if (!recovery.content_matches) {
        return ORACLE_WRONG_CONTENT;
    }
    return ORACLE_NONE;
}

// ───────────────────────── 记录核对器 ─────────────────────────

// 一次写还在盘上：它落点的版本在它的每个扇区上都是它的字节
fn write_in_place(write_index: u32) -> bool {
    let location = tab(T_WRITES, write_index * WRITE_WORDS + 5u);
    if (location == NONE) {
        return false;
    }
    let version = version_of_location(location);
    if (version == NONE) {
        return false;
    }
    let writes_first = location_word(location, 5u);
    let writes_count = location_word(location, 6u);
    var position = NONE;
    for (var i = 0u; i < writes_count; i = i + 1u) {
        if (tab(T_LOCATION_WRITES, writes_first + i) == write_index) {
            position = i;
        }
    }
    if (position == NONE) {
        return false;
    }
    let equal_first = tab(T_VERSIONS, version * VERSION_WORDS + 6u);
    let mask = vec2<u32>(tab(T_EQUAL_MASKS, (equal_first + position) * 2u), tab(T_EQUAL_MASKS, (equal_first + position) * 2u + 1u));
    let sectors = tab(T_WRITES, write_index * WRITE_WORDS + 3u) >> 9u;
    for (var s = 0u; s < sectors; s = s + 1u) {
        var bit = 0u;
        if (s < 32u) {
            bit = (mask.x >> s) & 1u;
        } else {
            bit = (mask.y >> (s - 32u)) & 1u;
        }
        if (bit == 0u) {
            return false;
        }
    }
    return true;
}
// crash::unit_copy_is_missing_under_the_persisted_set
fn unit_copy_is_missing(copy: u32) -> bool {
    let location = tab(T_WRITES, copy * WRITE_WORDS + 5u);
    let version = version_of_location(location);
    if (version == NONE) {
        return true;
    }
    let writes_first = location_word(location, 5u);
    let writes_count = location_word(location, 6u);
    var position = NONE;
    for (var i = 0u; i < writes_count; i = i + 1u) {
        if (tab(T_LOCATION_WRITES, writes_first + i) == copy) {
            position = i;
        }
    }
    if (position == NONE) {
        return true;
    }
    let equal_first = tab(T_VERSIONS, version * VERSION_WORDS + 6u);
    let mask = vec2<u32>(tab(T_EQUAL_MASKS, (equal_first + position) * 2u), tab(T_EQUAL_MASKS, (equal_first + position) * 2u + 1u));
    let sectors = tab(T_WRITES, copy * WRITE_WORDS + 3u) >> 9u;
    let laters_first = tab(T_WRITES, copy * WRITE_WORDS + 10u);
    let laters_count = tab(T_WRITES, copy * WRITE_WORDS + 11u);
    for (var s = 0u; s < sectors; s = s + 1u) {
        var bit = 0u;
        if (s < 32u) {
            bit = (mask.x >> s) & 1u;
        } else {
            bit = (mask.y >> (s - 32u)) & 1u;
        }
        if (bit == 1u) {
            continue;
        }
        var explained = false;
        for (var l = 0u; l < laters_count; l = l + 1u) {
            let later = tab(T_LATERS, (laters_first + l) * LATER_WORDS);
            let legal = tab(T_LATERS, (laters_first + l) * LATER_WORDS + 1u);
            let covered = vec2<u32>(tab(T_LATERS, (laters_first + l) * LATER_WORDS + 2u), tab(T_LATERS, (laters_first + l) * LATER_WORDS + 3u));
            var covers = 0u;
            if (s < 32u) {
                covers = (covered.x >> s) & 1u;
            } else {
                covers = (covered.y >> (s - 32u)) & 1u;
            }
            if (is_persisted(later) && covers == 1u && legal == 1u) {
                explained = true;
                break;
            }
        }
        if (!explained) {
            return true;
        }
    }
    return false;
}
// 恢复落到的那一版的实例表（写表里最后一次写出这个根身份的根槽写 → 它的实例表指针 → 沿链读，只读不判的读法）；行写进 R_ROWS
fn load_instance_table_of_the_effective_root(instance: u32, txg: vec2<u32>) -> bool {
    row_count = 0u;
    let write_count = hdr(1u);
    var root_bytes = NONE;
    for (var w = 0u; w < write_count; w = w + 1u) {
        let i = write_count - 1u - w;
        if (tab(T_WRITES, i * WRITE_WORDS + 4u) != WRITE_ROOT) {
            continue;
        }
        if (tab(T_WRITES, i * WRITE_WORDS + 6u) == instance && tab(T_WRITES, i * WRITE_WORDS + 7u) == txg.x && tab(T_WRITES, i * WRITE_WORDS + 8u) == txg.y) {
            root_bytes = tab(T_WRITES, i * WRITE_WORDS + 9u);
            break;
        }
    }
    if (root_bytes == NONE) {
        return false;
    }
    var pointer = parse_node_pointer(B[T_ROOT_BYTES] + root_bytes, 170u);
    var page_index = vec2<u32>(0u, 0u);
    for (var page = 0u; page < 256u; page = page + 1u) {
        // read_unit_without_judging：整单元 CRC 与位置条目相符
        let s = read_unit_via_locations(pointer, DATA_UNIT_BYTES);
        if (s.version == NONE) {
            return false;
        }
        // check_unit == Ok(3)：magic、flags、类 3、两道校验和、版本、预留位
        let base = s.base;
        if (rd32(base, 0u) != MAGIC_SFSU || rd8(base, 7u) != 0u || rd8(base, 6u) != 3u || (s.c2 & 11u) != 11u || rd16(base, 4u) != 1u) {
            return false;
        }
        if (!is_zero64(rd64(base, 43u)) || rd16(base, 51u) != 4u || !eq64(rd64(base, 53u), page_index) || !is_zero64(rd64(base, 61u))) {
            return false;
        }
        let count = rd16(base, 69u);
        let width = rd16(base, 71u);
        let declared = rd16(base, 8u);
        if (width != 88u || count * width > declared || declared > DATA_UNIT_BYTES - 136u) {
            return false;
        }
        if (count == 0u) {
            return false;
        }
        for (var r = 0u; r + 1u < count; r = r + 1u) {
            let o = 136u + r * 88u;
            // 行的 flags（偏移 21）非 0 同样解不开（InstanceTableOfRootRecord::read 同判；D18 已定项 11「第一版恒 0、非 0 拒收」）
            if (rd8(base, o) != 0u || rd8(base, o + 21u) != 0u) {
                return false;
            }
            if (row_count >= MAX_ROWS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                return false;
            }
            sws(R_ROWS + row_count * 3u, rd32(base, o + 1u));
            let row_txg = rd64(base, o + 5u);
            sws(R_ROWS + row_count * 3u + 1u, row_txg.x);
            sws(R_ROWS + row_count * 3u + 2u, row_txg.y);
            row_count = row_count + 1u;
        }
        let last = 136u + (count - 1u) * 88u;
        if (rd8(base, last) != 1u) {
            return false;
        }
        let next = parse_node_pointer(base, last + 2u);
        let flag = rd8(base, last + 1u);
        if (flag == 0u && next.all_zero) {
            return true;
        }
        // 有下一片还要位置指针头部 extent 偏移及之前那几段都是第一版的取值（Ptr 的 head_ok 与 extent_offset_zero；walk.rs chain_record_view 同判）：
        // 有一段不过是 I-3.14 第五形，这一片解不开、读不出整张表（用户 2026-10-06 定 K2-1 甲，第 10u 批）
        if (flag == 1u && !next.all_zero && next.head_ok && next.extent_offset_zero) {
            pointer = next;
            page_index = add64(page_index, u64_of(1u));
            continue;
        }
        return false;
    }
    undecidable = UNDECIDABLE_TOO_DEEP;
    return false;
}
fn abandoned_by_rows(instance: u32, txg: vec2<u32>) -> bool {
    for (var i = 0u; i < row_count; i = i + 1u) {
        if (sw(R_ROWS + i * 3u) == instance) {
            let row_txg = vec2<u32>(sw(R_ROWS + i * 3u + 1u), sw(R_ROWS + i * 3u + 2u));
            if (lt64(row_txg, txg)) {
                return true;
            }
        }
    }
    return false;
}
struct RecordVerdict {
    root_without_record: bool,
    claimed_state_missing_unit: bool,
    claimed_version_not_landed_on_by_the_recovery: bool,
};
// crash::check_records（崩溃点重放：两份镜像同一份，一条录制流）。恢复自称的那一版：给定状态表带了那一栏（T_CLAIMED_VERSIONS 不空，
// crash_judge_tables::JudgeStateSource::GivenPersistedSets 的 claimed_versions）而这个状态写了的，照写的那一版（崩溃注入第二截挂载之后的池：
// 挂载与那次发布写出的最新那条根，crash_given_states::judge_a_given_state_on_the_cpu 同取法），写的那一版与看 journal 那一遍落到的不同另判红；
// 没写的照看 journal 那一遍落到的根
fn check_records(consulted: Recovery) -> RecordVerdict {
    var out: RecordVerdict;
    out.root_without_record = false;
    out.claimed_state_missing_unit = false;
    out.claimed_version_not_landed_on_by_the_recovery = false;
    var claimed_has_a_root = consulted.has_effective;
    var claimed_instance = consulted.effective_instance;
    var claimed_txg = consulted.effective_txg;
    if (table_length(T_CLAIMED_VERSIONS) != 0u) {
        let claimed_at = given_state_ordinal * CLAIMED_VERSION_WORDS;
        let claimed_code = tab(T_CLAIMED_VERSIONS, claimed_at);
        if (claimed_code != CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY) {
            claimed_has_a_root = claimed_code == CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITH_A_ROOT;
            claimed_instance = tab(T_CLAIMED_VERSIONS, claimed_at + 1u);
            claimed_txg = vec2<u32>(tab(T_CLAIMED_VERSIONS, claimed_at + 2u), tab(T_CLAIMED_VERSIONS, claimed_at + 3u));
            if (claimed_has_a_root != consulted.has_effective) {
                out.claimed_version_not_landed_on_by_the_recovery = true;
            } else if (claimed_has_a_root
                && (claimed_instance != consulted.effective_instance
                    || claimed_txg.x != consulted.effective_txg.x
                    || claimed_txg.y != consulted.effective_txg.y)) {
                out.claimed_version_not_landed_on_by_the_recovery = true;
            }
        }
    }
    var have_table = false;
    if (claimed_has_a_root) {
        have_table = load_instance_table_of_the_effective_root(claimed_instance, claimed_txg);
    }
    let count = hdr(7u);
    for (var i = 0u; i < count; i = i + 1u) {
        let root_write = tab(T_PUBLISHES, i * PUBLISH_WORDS);
        let instance = tab(T_PUBLISHES, i * PUBLISH_WORDS + 1u);
        let txg = vec2<u32>(tab(T_PUBLISHES, i * PUBLISH_WORDS + 2u), tab(T_PUBLISHES, i * PUBLISH_WORDS + 3u));
        let records_first = tab(T_PUBLISHES, i * PUBLISH_WORDS + 4u);
        let records_count = tab(T_PUBLISHES, i * PUBLISH_WORDS + 5u);
        let groups_first = tab(T_PUBLISHES, i * PUBLISH_WORDS + 6u);
        let groups_count = tab(T_PUBLISHES, i * PUBLISH_WORDS + 7u);
        if (records_count > 0u && write_in_place(root_write)) {
            var any_record = false;
            for (var r = 0u; r < records_count; r = r + 1u) {
                if (write_in_place(tab(T_PUBLISH_RECORDS, records_first + r))) {
                    any_record = true;
                    break;
                }
            }
            if (!any_record) {
                out.root_without_record = true;
            }
        }
        if (claimed_has_a_root && le64(txg, claimed_txg)) {
            var abandoned = false;
            if (have_table) {
                abandoned = abandoned_by_rows(instance, txg);
            }
            if (!abandoned) {
                for (var g = 0u; g < groups_count; g = g + 1u) {
                    let copies_first = tab(T_COPY_GROUPS, (groups_first + g) * 2u);
                    let copies_count = tab(T_COPY_GROUPS, (groups_first + g) * 2u + 1u);
                    var all_missing = true;
                    for (var c = 0u; c < copies_count; c = c + 1u) {
                        if (!unit_copy_is_missing(tab(T_COPIES, copies_first + c))) {
                            all_missing = false;
                            break;
                        }
                    }
                    if (all_missing) {
                        out.claimed_state_missing_unit = true;
                    }
                }
            }
        }
    }
    return out;
}

// ───────────────────────── 入口 ─────────────────────────

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    // 判
    let cfg_choice = choose_system_configuration();
    let consulted = recover(true, cfg_choice);
    let ignored = recover(false, cfg_choice);
    let newest = newest_persisted_root();
    let consulted_oracle = classify_oracle(consulted, newest);
    let ignored_oracle = classify_oracle(ignored, newest);
    let records = check_records(consulted);
    var bits = 0u;
    if (consulted_oracle != ORACLE_NONE) {
        bits = bits | (1u << (BIT_CONSULTED_ORACLE + consulted_oracle));
    }
    if (ignored_oracle != ORACLE_NONE) {
        bits = bits | (1u << (BIT_IGNORED_ORACLE + ignored_oracle));
    }
    if (records.root_without_record) {
        bits = bits | (1u << BIT_ROOT_WITHOUT_RECORD);
    }
    if (records.claimed_state_missing_unit) {
        bits = bits | (1u << BIT_CLAIMED_STATE_MISSING_UNIT);
    }
    if (records.claimed_version_not_landed_on_by_the_recovery) {
        bits = bits | (1u << BIT_CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY);
    }
    if (undecidable != 0u) {
        bits = bits | (1u << BIT_UNDECIDABLE);
    }
    let out = local * VERDICT_WORDS;
    verdicts[out] = bits;
    // 判得了时第 1 字本来恒 0：借它带出看 journal 那一遍重放停在哪一道（诊断，`replay_stop_reason`；高 16 位给 record_count）
    if (undecidable != 0u) {
        verdicts[out + 1u] = undecidable;
    } else {
        verdicts[out + 1u] = replay_stop_reason | (record_count << 16u);
    }
    // 两遍各自落到的根：没落到根（择不到系统配置或根、重放在施加任何记录之前停下）时两个字都是 NONE（CPU 的 effective_root 为 None）
    var consulted_instance = NONE;
    var consulted_txg = NONE;
    if (consulted.has_effective) {
        consulted_instance = consulted.effective_instance;
        consulted_txg = consulted.effective_txg.x;
    }
    var ignored_instance = NONE;
    var ignored_txg = NONE;
    if (ignored.has_effective) {
        ignored_instance = ignored.effective_instance;
        ignored_txg = ignored.effective_txg.x;
    }
    verdicts[out + CONSULTED_PASS_LANDED_INSTANCE_WORD] = consulted_instance;
    verdicts[out + CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = consulted_txg;
    verdicts[out + CONSULTED_PASS_OUTCOME_WORD] = consulted.outcome;
    verdicts[out + CONSULTED_PASS_FAILURE_POINT_WORD] = consulted.failure;
    verdicts[out + IGNORED_PASS_OUTCOME_WORD] = ignored.outcome;
    verdicts[out + IGNORED_PASS_FAILURE_POINT_WORD] = ignored.failure;
    verdicts[out + IGNORED_PASS_LANDED_INSTANCE_WORD] = ignored_instance;
    verdicts[out + IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = ignored_txg;
}
