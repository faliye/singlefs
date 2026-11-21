use super::*;

#[test]
fn window_rates_turn_end_offsets_into_states_per_second() {
    let rates = window_states_per_second(&[2_000_000, 3_000_000, 7_000_000], 65_536);
    assert_eq!(rates, vec![32_768_000.0, 65_536_000.0, 16_384_000.0]);
    assert_eq!(parse_window_offsets("W end_offsets_microseconds=10,20"), vec![10, 20]);
    assert!(parse_window_offsets("W end_offsets_microseconds=none").is_empty());
}

#[test]
fn throughput_judgement_separates_two_measurements_only_at_a_threshold_between_them() {
    assert!(network_throughput_flips(1_000.0, 1_500.0));
    assert!(!network_throughput_flips(2_000.0, 1_500.0));
    assert!(!network_throughput_flips(1_500.0, 1_500.0), "等于门槛不翻（登记：< R_B 才翻）");
    let mut output = ProductLines::new();
    assert_eq!(run_discrimination(&mut output, &["1000".to_string(), "3000".to_string()]), 0);
    assert_eq!(run_discrimination(&mut output, &["1000".to_string(), "1000".to_string()]), 1, "两个一样的数分不开");
}

#[test]
fn rate_control_needs_the_slow_pass_at_most_two_hundred_and_the_fast_pass_above() {
    assert!(rate_control_seen(201.0, 200.0));
    assert!(!rate_control_seen(200.0, 150.0), "快的那遍不到 200 以上");
    assert!(!rate_control_seen(900.0, 200.5), "慢的那遍超过 200");
}

#[test]
fn disconnect_paths_need_a_half_frame_a_resend_and_a_second_library_connection() {
    let run_with = |half_frames: u64, resent: u64, connections: u64| NetworkCellRun {
        sender_summary: format!("S ending=complete resent_frames={resent}"),
        library_network: format!("N receive_mode=registered connections={connections} half_frames_dropped={half_frames}"),
        ..NetworkCellRun::default()
    };
    assert!(disconnect_paths_reached(&run_with(3, 64, 51)));
    assert!(!disconnect_paths_reached(&run_with(0, 64, 51)), "没丢过半帧");
    assert!(!disconnect_paths_reached(&run_with(3, 0, 51)), "没重发过");
    assert!(!disconnect_paths_reached(&run_with(3, 64, 1)), "重发都没进库：库进程只收过一条连接");
    assert!(disconnect_paths_reached(&run_with(1, 1, 2)), "三样各自刚过下沿");
}
