
#[test]
fn probe_p2_older_candidate_root_with_units_of_its_own_all_zero() {
    // 覆盖写之后：A（txg 3）的数据单元只被 A 引用；A 是回退候选集里的更早的根
    let mut image = image_after_the_overwrite("probe-p2-older-with-units");
    matrix_row("F1c.clean(after overwrite)", &image);
    rewrite_the_root_slot_of(&mut image, CheckpointTxg(3), |bytes| { bytes[ROOT_RECORD_TREE_TABLE_POINTER_RANGE].fill(0); });
    matrix_row("F1c.older_candidate_root_txg3_all_zero", &image);
}
