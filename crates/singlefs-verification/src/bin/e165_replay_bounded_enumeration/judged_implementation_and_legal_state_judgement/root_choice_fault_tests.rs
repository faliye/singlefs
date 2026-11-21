use super::*;

/// J8c 手造（登记 r5 9.2「J8c 只比卸载记号」那一行的取样点）：一条没施加、返回根的实例表指针被改过，J8c 记一条违例；原样交回的不记。
/// 另一格：投影不在生成器真相里的返回根记「无出处」、不判 J8a。
#[test]
fn j8c_counts_a_returned_root_that_kept_the_projection_but_changed_the_instance_table() {
    let root_fields_in_records_profile =
        RunProfile::root_fields_in_records(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    let chosen_root = adversarial_chosen_root(root_fields_in_records_profile);
    let truth = adversarial_record_truth(
        1,
        &adversarial_record_alphabet(),
        &chosen_root,
        root_fields_in_records_profile,
    );
    let judge = |returned_root: &RootRecord| {
        let mut tally = Tally::new();
        judge_the_fields_beyond_the_projection_against_the_generated_records(
            &chosen_root,
            returned_root,
            &truth,
            None,
            &|| "hand_made".to_string(),
            &mut tally,
        );
        tally
    };
    let untouched = judge(&chosen_root);
    assert_eq!(
        (
            untouched.count_of(Counter::J8cJudged),
            untouched.count_of(Counter::J8cViolations),
            untouched.count_of(Counter::J8bViolations)
        ),
        (1, 0, 0),
        "原样交回所选根：J8c 判到、不违例"
    );
    let mut changed_instance_table = chosen_root;
    changed_instance_table.instance_table =
        node_pointer_at_slot(INSTANCE_TABLE_SLOT_OF_THE_FIRST_VERSION + 5, 1);
    let changed = judge(&changed_instance_table);
    assert_eq!(
        (
            changed.count_of(Counter::J8cJudged),
            changed.count_of(Counter::J8cViolations),
            changed.count_of(Counter::J8aJudged)
        ),
        (1, 1, 0),
        "一条没施加而实例表指针变了：J8c 记一条"
    );
    let mut without_a_source = chosen_root;
    without_a_source.tree_identifier_watermark = OFF_TIMELINE_WATERMARK_BASE + 999_999;
    let unsourced = judge(&without_a_source);
    assert_eq!(
        (
            unsourced.count_of(Counter::NewRootSegmentFieldsWithoutASource),
            unsourced.count_of(Counter::J8aJudged),
            unsourced.count_of(Counter::J8cJudged)
        ),
        (1, 0, 0),
        "投影不在生成器真相里：记无出处，J8a、J8c 都不判"
    );
}
