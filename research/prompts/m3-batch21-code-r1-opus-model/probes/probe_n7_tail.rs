
/// 攻方探针（m3-batch21-code-r1 攻方腿）：用例三点之外的规模，打印每批换下数与写调用最多的那次。
#[test]
#[ignore = "攻方探针，只在副本里点名跑"]
fn probe_points_outside_the_sampled_sizes() {
    let sizes: Vec<usize> = std::env::var("N7_PROBE_SIZES")
        .unwrap_or_else(|_| "4609,9216".to_string())
        .split(',')
        .map(|one| one.parse().expect("size"))
        .collect();
    let one_data_unit = writes_of_every_publish_of_an_unlink_in_product_batches(1);
    let bound = write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file();
    println!("N7_PROBE one_unit_largest={} bound_add={}", largest_write_calls_of_one_publish(&one_data_unit), bound);
    for size in sizes {
        let writes = writes_of_every_publish_of_an_unlink_in_product_batches(size);
        let released = data_units_released_by_every_publish(&writes);
        let clause = data_units_released_by_every_publish_as_the_clause_gives(size);
        println!(
            "N7_PROBE size={size} publishes={} first_released={:?} clause_first={:?} released_equals_clause={} largest_write_calls={} largest_written_bytes={}",
            writes.len(),
            released.iter().take(4).collect::<Vec<_>>(),
            clause.iter().take(4).collect::<Vec<_>>(),
            released == clause,
            largest_write_calls_of_one_publish(&writes),
            writes.iter().map(|w| w.written_bytes).max().unwrap_or(0)
        );
    }
}
