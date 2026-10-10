
// ── 第二个探针：合法历史上新九条（I-9.17–I-9.24、I-9.29）有没有落到 Judgements::into_report 的缺省理由上（N3 的正推一格）
// 每个合法历史上、每一条要么 Holds，要么 NotApplicable 且理由是 walk 里为它写的某一句专门理由；缺省理由或别的理由、Violated、PartiallyJudged 都标出来。
fn zs_specific_reasons() -> Vec<&'static str> {
    use singlefs_checker::walk::*;
    vec![
        NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
        NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD,
        NO_TOMBSTONE_IN_ANY_WALKED_VERSION,
        NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE,
        NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER,
        NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY,
        PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB,
        NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB,
        NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
        NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    ]
}

fn zs_report_on_the_nine(name: &str, entry: &str, verdicts: &[(&'static str, InvariantVerdict)]) -> usize {
    let specific = zs_specific_reasons();
    let mut flagged = 0usize;
    for invariant in [
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
    ] {
        let found = verdicts.iter().find(|(identifier, _)| *identifier == invariant).expect("清单里有");
        let (kind, flag) = match &found.1 {
            InvariantVerdict::Holds => ("Holds".to_string(), false),
            InvariantVerdict::NotApplicable(reason) if specific.contains(reason) => (format!("NA-specific:{}", reason.chars().take(14).collect::<String>()), false),
            InvariantVerdict::NotApplicable(reason) if *reason == "走读到的每一版树表里都没有 deleted_inodes 条目（树表 0 条）：没有队列可判" => ("NA-I-9.24-own:树表0条".to_string(), false),
            InvariantVerdict::NotApplicable(reason) => (format!("NA-OTHER:{reason}"), true),
            InvariantVerdict::Violated(detail) => (format!("VIOLATED:{}", detail.chars().take(40).collect::<String>()), true),
            InvariantVerdict::PartiallyJudged(detail) => (format!("PARTIAL:{}", detail.chars().take(40).collect::<String>()), true),
        };
        if flag {
            flagged += 1;
        }
        println!("ZS_REASON {name} entry={entry} {} {kind}{}", found.0, if flag { " FLAGGED" } else { "" });
    }
    flagged
}

#[test]
fn zs_new_nine_never_fall_to_the_default_reason_on_the_legal_histories() {
    let mut flagged = 0usize;
    let mut cases = 0usize;
    let mut judge = |name: &str, image: &MemoryPool, knob: bool| {
        let verdicts = if knob { check_pool_image_under_the_tombstone_knob(image) } else { check_pool_image(image) };
        let violated = violated_invariants(&verdicts);
        println!("ZS_RED {name} knob={knob} reds={violated:?}");
        flagged += zs_report_on_the_nine(name, if knob { "knob" } else { "old" }, &verdicts);
        cases += 1;
    };
    judge("one-file", &build_pool("zs-r-one-file").memory_pool(), false);
    judge("three-unit-file-after-the-intent", &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("zs-r-intent"), false);
    judge("three-unit-file-after-first-batch", &common_deletion_in_progress::image_after_the_first_batch_of_a_three_unit_file("zs-r-batch"), false);
    judge("empty-inode-after-the-intent", &common_deletion_in_progress::image_after_the_intent_of_an_empty_inode("zs-r-empty-intent"), false);
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file("zs-r-unlinked");
    common_deletion_in_progress::unlink_to_the_end(&mut pool, common_deletion_in_progress::FIRST_FILE.0);
    judge("three-unit-file-unlinked-to-the-end", &pool.memory_pool(), false);
    judge("inode-three-unlinked-ring-turned-over", &common_deletion_in_progress::pool_with_inode_three_unlinked_and_the_root_ring_turned_over("zs-r-ring").memory_pool(), false);
    judge("inode-two-unlinked-rolled-back-ring-turned-over", &common_deletion_in_progress::pool_with_inode_two_unlinked_then_rolled_back_before_the_intent_and_the_root_ring_turned_over("zs-r-rb").memory_pool(), false);
    judge("rolled-back-to-the-first-root", &image_after_rolling_back_to_the_first_root("zs-r-lineage").image, false);
    judge("two-inodes-one-tombstone-per-container-old-entry", &pool_with_empty_inodes_unlinked_under_the_tombstone_knob("zs-r-knob2-old", &[2, 3]).memory_pool(), false);
    judge("two-inodes-one-tombstone-per-container", &pool_with_empty_inodes_unlinked_under_the_tombstone_knob("zs-r-knob2", &[2, 3]).memory_pool(), true);
    judge("three-inodes-level-two", &pool_with_empty_inodes_unlinked_under_the_tombstone_knob("zs-r-knob3", &[2, 3, 4]).memory_pool(), true);
    judge("one-inode-under-the-knob", &pool_with_empty_inodes_unlinked_under_the_tombstone_knob("zs-r-knob1", &[2]).memory_pool(), true);
    println!("ZS_REASON_SUMMARY cases={cases} flagged={flagged}");
}
