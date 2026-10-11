// 攻方腿模型（m3-run-compare-split-r1）：P14（拒绝合法性与 W 的判据复算）上 GPU 时最少要调的那几段解析，每段一个调用点。
// 只为量展开量（kernel-size 的数法），不判对错；接在 crash_judge_common.wgsl + crash_judge_recovery.wgsl 之后解析。
// 对应 crates/singlefs-harness/src/mount_transparency.rs 第 2352–2720 行那几条判据里调到的：择系统配置、择根、扫 journal、
// 「见证到的那一版起往回找根」、实例表链、命名单元读一份。判据本身的比较逻辑不在里面（只会更大）。
@compute @workgroup_size(64)
fn p14_probe(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    let choice = choose_system_configuration();
    let chosen = choose_root(choice.cfg);
    scan_journal(choice.cfg);
    let fallback = newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(choice.cfg, chosen.root, chosen.root);
    let rows_loaded = load_rows_of_the_instance_table_chain(chosen.root);
    let chain_readable = instance_table_chain_readable(chosen.root);
    let unit = read_unit_via_locations(chosen.root.instance_table, 16384u);
    var bits = 0u;
    if (fallback.found) { bits = bits | 1u; }
    if (rows_loaded) { bits = bits | 2u; }
    if (chain_readable) { bits = bits | 4u; }
    if (unit.length != 0u) { bits = bits | 8u; }
    verdicts[local * VERDICT_WORDS] = bits;
}
