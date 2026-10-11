// 攻方腿模型（m3-run-compare-split-r1）：P14 并进恢复内核、再加 P1「拿录下的 Rust 落点再跑一遍记录核对器」那一处，展开量是多少。
// 入口照恢复内核 main 的调用点（crash_judge_recovery.wgsl 第 597 行起），再接 p14_probe_entry.wgsl 那几段（择系统配置复用 main 那一次的结果），
// 再多一个 check_records 调用点。只为量展开量；接在 crash_judge_common.wgsl + crash_judge_recovery.wgsl 之后解析。
@compute @workgroup_size(64)
fn merged_probe(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    let cfg_choice = choose_system_configuration();
    let consulted = recover(true, cfg_choice);
    let ignored = recover(false, cfg_choice);
    let newest = newest_persisted_root();
    let consulted_oracle = classify_oracle(consulted, newest);
    let ignored_oracle = classify_oracle(ignored, newest);
    let records = check_records(consulted);
    // P1：按录下的 Rust 落点再跑一遍记录核对器（落点从表里读，这里拿 ignored 代它，只为多一个调用点）
    let records_at_the_recorded_landing = check_records(ignored);
    // P14：判据复算要的几段解析
    let chosen = choose_root(cfg_choice.cfg);
    scan_journal(cfg_choice.cfg);
    let fallback = newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(cfg_choice.cfg, chosen.root, chosen.root);
    let rows_loaded = load_rows_of_the_instance_table_chain(chosen.root);
    let chain_readable = instance_table_chain_readable(chosen.root);
    let unit = read_unit_via_locations(chosen.root.instance_table, 16384u);
    var bits = consulted_oracle + ignored_oracle;
    if (records.root_without_record || records_at_the_recorded_landing.root_without_record) { bits = bits | 1u; }
    if (fallback.found) { bits = bits | 2u; }
    if (rows_loaded && chain_readable) { bits = bits | 4u; }
    if (unit.length != 0u) { bits = bits | 8u; }
    verdicts[local * VERDICT_WORDS] = bits;
}
