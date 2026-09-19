MUTATIONS = [
    ("已知红第 1 条认错操作种类", "crates/singlefs-harness/src/history.rs",
     "observation.operation_kind == Some(HistoryOperationKind::RaiseRollbackFloor)",
     "observation.operation_kind == Some(HistoryOperationKind::PublishOverwrite)", "second_transaction_supplement_three_random_history"),
    ("已知红第 1 条不排除转过圈的", "crates/singlefs-harness/src/history.rs",
     "        && !observation.root_ring_has_turned()\n        && only_allocated_statistic_above_walked(observation)",
     "        && only_allocated_statistic_above_walked(observation)", "second_transaction_supplement_three_random_history"),
]
