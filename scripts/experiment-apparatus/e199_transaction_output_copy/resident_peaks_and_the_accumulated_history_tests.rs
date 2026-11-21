use super::*;

#[test]
fn ding_history_over_256_publishes() {
    let geometries = ding_history_geometries();
    let mut jia = HistoryTotals::default();
    for geometry in &geometries {
        jia.add_publish(geometry, ArmBytes::default());
    }
    assert_eq!((jia.jia.copied_before, jia.jia.copied_after), (103456147732, 20796884553));
    let expected = [
        (1, 20948865498, (0, 0, 0)),
        (2, 10474451876, (1, 128, 0)),
        (3, 6955701571, (2, 171, 1)),
        (4, 5237228498, (3, 192, 0)),
        (8, 2618633376, (7, 224, 0)),
        (64, 327333652, (63, 252, 0)),
        (1024, 0, (256, 256, 256)),
    ];
    for (period, total, layers) in expected {
        let history = ding_history(&geometries, period, Compaction::WholeNewBase);
        assert_eq!(history.compaction_copied_total, total, "p = {period}");
        assert_eq!((history.layer_peak, history.layer_positive_publishes, history.layer_final), layers, "p = {period}");
        assert_eq!(ding_history(&geometries, period, Compaction::FoldInPlace).compaction_copied_total, 0);
    }
}

// ── 10⁶ 次 unlink 累计：每次 unlink 两次发布，与线程数无关 ──

#[test]
fn cumulative_unlinks_count_the_intent_and_the_final_batch() {
    let totals = cumulative_unlinks_between(0..3, DING_MAIN_PERIOD);
    assert_eq!(totals.publishes, 6);
    assert_eq!((totals.jia.copied_before, totals.jia.copied_after), (6665160, 2602316));
    let intent_at_zero = main_with(|knobs| knobs.tombstone_records = 0);
    assert_eq!((jia_before(&intent_at_zero), jia_after(&intent_at_zero)), (1107132, 408842));
}

#[test]
fn cumulative_unlinks_compact_ding_every_period_publishes() {
    // 0..64 这 64 次 unlink 共 128 次发布，第 64 次是 u = 32 上的最后一批，第 128 次是 u = 64 上的最后一批。
    let totals = cumulative_unlinks_between(0..64, DING_MAIN_PERIOD);
    let compacted: u64 = [32, 64]
        .iter()
        .map(|&records| version_bytes_after(&main_with(|knobs| (knobs.tombstone_records, knobs.tombstones_added) = (records, 0))))
        .sum();
    assert_eq!(totals.ding.copied_after, compacted);
    assert!(compacted > 0);
}

#[test]
fn cumulative_unlinks_do_not_depend_on_the_thread_count() {
    assert_eq!(cumulative_unlinks(300, 1), cumulative_unlinks(300, 7));
    assert_eq!(cumulative_unlinks(300, 1), cumulative_unlinks_between(0..300, DING_MAIN_PERIOD));
}

#[test]
fn cumulative_ratio_pair_names_the_arm_whose_bytes_it_divides_by() {
    // 0..64 这 64 次 unlink 共 128 次发布，丁在第 64、128 次压实，整份新基准入后的照抄不为 0，与丙的不同。
    assert_eq!(DING_MAIN_PERIOD, 64);
    let lines = cumulative_lines(&cumulative_unlinks_between(0..64, DING_MAIN_PERIOD));
    let field = |line: &str, key: &str| -> String {
        line.split(' ').find_map(|word| word.strip_prefix(&format!("{key}=")).map(str::to_string)).unwrap_or_else(|| panic!("{key} is not in {line}"))
    };
    let arm_line = |name: &str| -> String {
        let found: Vec<&String> = lines.iter().filter(|line| line.starts_with("name=cumulative_arm ") && field(line.as_str(), "arm") == name).collect();
        assert_eq!(found.len(), 1, "one cumulative_arm line named {name}");
        found[0].clone()
    };
    let bytes = |name: &str, key: &str| -> u64 { field(arm_line(name).as_str(), key).parse().unwrap() };
    let ratio_lines: Vec<&String> = lines.iter().filter(|line| line.starts_with("name=cumulative_ratio ")).collect();
    let pairs: Vec<String> = ratio_lines.iter().map(|line| field(line.as_str(), "pair")).collect();
    assert_eq!(
        pairs,
        [
            "jia_after_patch_over_yi2_persistent",
            "jia_after_patch_over_yi2_persistent",
            "jia_after_patch_over_bing_by_value",
            "jia_after_patch_over_bing_by_value",
            "jia_after_patch_over_ding_period_64_whole_new_base",
            "jia_after_patch_over_ding_period_64_whole_new_base",
        ]
    );
    assert!(bytes("ding_period_64_whole_new_base", "copied_after") > 0);
    assert_ne!(bytes("ding_period_64_whole_new_base", "copied_after"), bytes("bing_by_value", "copied_after"));
    for line in ratio_lines {
        let pair = field(line.as_str(), "pair");
        let denominator = pair.strip_prefix("jia_after_patch_over_").unwrap();
        let (copied_key, recomputed_key) = match field(line.as_str(), "segment").as_str() {
            "before_admission" => ("copied_before", "recomputed_before"),
            "after_admission" => ("copied_after", "recomputed_after"),
            other => panic!("segment {other}"),
        };
        let copied = ratio_text(bytes("jia_after_patch", copied_key), bytes(denominator, copied_key));
        let recomputed = ratio_text(bytes("jia_after_patch", recomputed_key), bytes(denominator, recomputed_key));
        assert_eq!(field(line.as_str(), "copied_without_installed"), copied, "{line}");
        assert_eq!(field(line.as_str(), "recomputed"), recomputed, "{line}");
    }
}

#[test]
fn bing_resident_peak_counts_only_the_replaced_containers_at_815000_tombstones() {
    // 第四份第九节：被换下的旧字节取 k_old，不取 k_old + k_new（a_T = 815000 上 k_new = 1000，G0 上 k_new = 0）。
    let geometry = main_with(|knobs| knobs.tombstones_added = 815_000);
    assert_eq!(geometry.tombstones.new_containers, 1000);
    assert_eq!(bing_resident_peak(&geometry), 149447442);
    // G0 上被换下的是墓碑树开放那一片与两个索引节点：32768 + 2 × 16384 = 65536。
    let main = main_with(|_| {});
    let replaced: Vec<(u64, u64)> = main.container_trees().iter().map(|tree| (tree.open_container_rewritten, tree.rewritten_index_nodes)).collect();
    assert_eq!(replaced, [(1, 2), (0, 0), (0, 0)]);
    assert_eq!(bing_resident_peak(&main), 82623711);
}

#[test]
fn cumulative_unlinks_prefix_of_thirty_two_unlinks_starts_at_zero_records() {
    // 第四份第五节：第 j 次 unlink（j 从 0 起）发布序号 2j + 1、2j + 2；丁在序号 64（j = 31 的最后一批，u = 32）压实一次。
    let totals = cumulative_unlinks_between(0..32, DING_MAIN_PERIOD);
    assert_eq!(totals.publishes, 64);
    assert_eq!((totals.jia.copied_before, totals.jia.copied_after), (71410560, 27796704));
    assert_eq!((totals.yi2.copied_before, totals.yi2.copied_after), (6291984, 5865360));
    assert_eq!(totals.bing.copied_after, 189688);
    assert_eq!(totals.ding.copied_after, 822204);
    assert_eq!(version_bytes_after(&main_with(|knobs| (knobs.tombstone_records, knobs.tombstones_added) = (32, 0))), 822204);
}
