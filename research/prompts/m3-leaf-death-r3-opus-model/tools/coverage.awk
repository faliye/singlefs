# 按变体把计数行相加（两种形状合起来；*_peak 取最大），只列走到的形状那几项。输入：sweep 的输出行。
BEGIN { split("publishes crash_points crash_states_enumerated crash_points_skipped_more_writes_than_k_max rollbacks recoveries lost_root_one lost_root_one_readable_but_unusable lost_root_three drops absorbs borrows records_with_data_moved_to_another_leaf truncates truncates_to_zero truncates_of_a_record_moved_to_another_leaf_before regrows_after_a_truncate zero_entry_versions retirement_records units_tombstone w2_damage_events w2_copies_repaired w2_copies_lost_for_good w2_damage_refused_last_copy_of_a_ring_unit w2_units_readable_only_on_disk1_peak sweep_rows_deleted sweep_units_erased sweep_rounds_skipped_for_a_bad_copy sweep_rows_deleted_without_erasing", K, " ") }
/name=counts/ {
    delete v
    for (i = 1; i <= NF; i++) { n = index($i, "="); if (n > 0) v[substr($i, 1, n - 1)] = substr($i, n + 1) }
    g = v["variant"]; h[g]++
    for (j in K) { k = K[j]; if (k ~ /_peak$/) { if (v[k] + 0 > c[g, k]) c[g, k] = v[k] + 0 } else c[g, k] += v[k] }
}
END { for (g in h) { line = "variant=" g " histories=" h[g]; for (j = 1; j <= length(K); j++) line = line " " K[j] "=" c[g, K[j]] + 0; print line } }
