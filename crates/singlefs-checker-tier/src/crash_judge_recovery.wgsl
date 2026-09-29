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
    out.failure = 0u;
    if (!cfg_choice.ok) {
        out.failure = 1u;
        return out;
    }
    let chosen = choose_root(cfg_choice.cfg);
    if (!chosen.found) {
        out.failure = 2u;
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
    let expected = expected_version_index(effective.instance, effective.txg);
    let walked = walk_to_file(effective, cfg_choice.cfg, expected);
    out.outcome = walked.outcome;
    out.failure = walked.failure;
    out.content_matches = walked.content_matches;
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
// crash::classified_oracle_violation_for_versions
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
    if (recovery.outcome == OUTCOME_FAILED) {
        return ORACLE_READ_FAILED;
    }
    if (recovery.outcome == OUTCOME_NO_FILE) {
        if (version == NONE) {
            // 更旧的根下面有文件？
            let count = hdr(8u);
            for (var i = 0u; i < count; i = i + 1u) {
                let instance = tab(T_EXPECTED, i * EXPECTED_WORDS);
                let txg = vec2<u32>(tab(T_EXPECTED, i * EXPECTED_WORDS + 1u), tab(T_EXPECTED, i * EXPECTED_WORDS + 2u));
                if (lt64(txg, recovery.effective_txg) || (eq64(txg, recovery.effective_txg) && instance < recovery.effective_instance)) {
                    return ORACLE_NO_FILE_WHERE_AN_OLDER_ROOT_HAS_ONE;
                }
            }
            return ORACLE_NONE;
        }
        return ORACLE_NO_FILE_WHERE_THIS_ROOT_HAS_ONE;
    }
    if (version == NONE) {
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
            if (rd8(base, o) != 0u) {
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
        if (flag == 1u && !next.all_zero) {
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
};
// crash::check_records（层 0：两份镜像同一份，一条录制流）
fn check_records(consulted: Recovery) -> RecordVerdict {
    var out: RecordVerdict;
    out.root_without_record = false;
    out.claimed_state_missing_unit = false;
    var have_table = false;
    if (consulted.has_effective) {
        have_table = load_instance_table_of_the_effective_root(consulted.effective_instance, consulted.effective_txg);
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
        if (consulted.has_effective && le64(txg, consulted.effective_txg)) {
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
    if (undecidable != 0u) {
        bits = bits | (1u << BIT_UNDECIDABLE);
    }
    let out = local * 12u;
    verdicts[out] = bits;
    // 判得了时第 1 字本来恒 0：借它带出看 journal 那一遍重放停在哪一道（诊断，`replay_stop_reason`；高 16 位给 record_count）
    if (undecidable != 0u) {
        verdicts[out + 1u] = undecidable;
    } else {
        verdicts[out + 1u] = replay_stop_reason | (record_count << 16u);
    }
    var effective_instance = NONE;
    var effective_txg = NONE;
    if (consulted.has_effective) {
        effective_instance = consulted.effective_instance;
        effective_txg = consulted.effective_txg.x;
    }
    verdicts[out + 2u] = effective_instance;
    verdicts[out + 3u] = effective_txg;
    verdicts[out + 4u] = consulted.outcome;
    verdicts[out + 5u] = consulted.failure;
    verdicts[out + 6u] = ignored.outcome;
    verdicts[out + 7u] = ignored.failure;
}
