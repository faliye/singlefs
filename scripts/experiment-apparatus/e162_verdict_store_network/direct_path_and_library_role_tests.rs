use super::*;

#[test]
fn sender_arguments_round_trip_through_the_sender_parser() {
    let settings = SenderSettings {
        endpoint: SenderEndpoint::ConnectTcp("127.0.0.1:4242".to_string()),
        block_count: 50_000,
        states_per_block: 65_536,
        window_frames: 256,
        pace_between_first_transmissions: Some(Duration::from_millis(100)),
        acknowledgement_stall_limit: Duration::from_secs(120),
        spinning_thread_count: 3,
    };
    let arguments = sender_arguments(&settings);
    assert_eq!(arguments[0], "send");
    let parsed = SenderSettings::parse(&arguments[1..]).expect("自己写的参数认得出");
    assert_eq!(parsed.endpoint, settings.endpoint);
    assert_eq!(parsed.block_count, 50_000);
    assert_eq!(parsed.states_per_block, 65_536);
    assert_eq!(parsed.window_frames, 256);
    assert_eq!(parsed.pace_between_first_transmissions, Some(Duration::from_millis(100)));
    assert_eq!(parsed.acknowledgement_stall_limit, Duration::from_secs(120));
    assert_eq!(parsed.spinning_thread_count, 3);
}
