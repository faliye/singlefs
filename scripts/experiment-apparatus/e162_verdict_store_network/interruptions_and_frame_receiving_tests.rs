use super::*;

#[test]
fn interruption_plan_pins_first_thresholds_and_pauses_and_is_sorted() {
    let plan = interruption_plan(REGISTERED_INTERRUPTIONS, PATH_BLOCK_COUNT);
    assert_eq!(plan.len(), 50);
    let thresholds: Vec<u64> = plan.iter().map(|interruption| interruption.acknowledgement_threshold).collect();
    assert!(thresholds.windows(2).all(|pair| pair[0] <= pair[1]), "a_k 排好序");
    assert!(thresholds.iter().all(|&threshold| (1..50_000).contains(&threshold)), "a_k 在 [1, 5 × 10⁴)");
    assert_eq!(thresholds[..5], PINNED_FIRST_THRESHOLDS);
    assert_eq!(thresholds[49], PINNED_LAST_THRESHOLD);
    assert_eq!(plan[0].pause_fraction, PINNED_FIRST_PAUSE_FRACTION);
    let escalated = interruption_plan(ESCALATED_INTERRUPTIONS, PATH_BLOCK_COUNT);
    assert_eq!(escalated.len(), 200);
    let small = interruption_plan(5, 400);
    assert!(small.iter().all(|interruption| (1..400).contains(&interruption.acknowledgement_threshold)));
}

#[test]
fn drop_control_picks_one_offset_below_a_thousand_per_window_and_pins_the_first_three() {
    let picks = drop_control_picks(52);
    assert_eq!(picks.len(), 52);
    assert!(picks.iter().all(|&pick| pick < 1_000));
    assert_eq!(picks[..3], PINNED_FIRST_DROP_PICKS);
}

fn frame_for(network_position: u64, states_per_block: u64) -> (BlockKey, Vec<u8>, Vec<u8>) {
    let block_index = network_path_block_index(network_position);
    let key = interleaved_block_key(block_index, states_per_block, &input_fingerprint());
    let value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, states_per_block);
    let frame = sender_role::encode_block_frame(&key, &value);
    (key, value, frame)
}

fn receiver_on_file_library(directory: &Path, receive_mode: ReceiveMode, network_blocks: u64) -> FrameReceiver {
    let mut stores = open_thread_stores(Candidate::HardenedFile, &directory.join("library"), 1).expect("开得了 F 库");
    FrameReceiver::new(receive_mode, SMALL_STATES_PER_BLOCK, network_blocks, stores.pop().expect("一个句柄"))
}

fn acknowledged_keys(acknowledgements: &[u8]) -> Vec<BlockKey> {
    acknowledgements
        .chunks(sender_role::ACKNOWLEDGEMENT_FRAME_BYTES)
        .map(|chunk| sender_role::decode_acknowledgement_frame(chunk.try_into().expect("40 字节")).expect("确认帧对"))
        .collect()
}

#[test]
fn registered_receiver_drops_a_bad_checksum_frame_and_a_half_frame_and_acknowledges_the_rest() {
    let directory = TestDirectory::new("registered-receiver");
    let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::Registered, 10);
    let (first_key, first_value, first_frame) = frame_for(0, SMALL_STATES_PER_BLOCK);
    let (second_key, _second_value, mut corrupted_frame) = frame_for(1, SMALL_STATES_PER_BLOCK);
    corrupted_frame[BLOCK_FRAME_HEADER_BYTES + 100] ^= 0x5A;
    let (third_key, third_value, third_frame) = frame_for(2, SMALL_STATES_PER_BLOCK);
    let (half_key, _half_value, half_frame) = frame_for(3, SMALL_STATES_PER_BLOCK);
    let mut stream = Vec::new();
    stream.extend_from_slice(&first_frame);
    stream.extend_from_slice(&corrupted_frame);
    stream.extend_from_slice(&third_frame);
    stream.extend_from_slice(&half_frame[..half_frame.len() / 2]);
    let mut acknowledgements = Vec::new();
    let end = receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
    assert_eq!(end, ConnectionEnd::HalfFrame);
    assert_eq!(acknowledged_keys(&acknowledgements), vec![first_key, third_key]);
    assert_eq!(receiver.counters.complete_frames, 3);
    assert_eq!(receiver.counters.checksum_failures, 1);
    assert_eq!(receiver.counters.half_frames_dropped, 1);
    assert_eq!(receiver.counters.half_frames_written, 0);
    assert_eq!(receiver.counters.commits, 2);
    assert_eq!(receiver.store.read_block(&first_key), ReadOutcome::Found(first_value));
    assert_eq!(receiver.store.read_block(&second_key), ReadOutcome::NotFound, "CRC 不对的帧不写");
    assert_eq!(receiver.store.read_block(&third_key), ReadOutcome::Found(third_value));
    assert_eq!(receiver.store.read_block(&half_key), ReadOutcome::NotFound, "半帧不写");
}

#[test]
fn registered_receiver_rewrites_a_resent_frame_with_the_same_value_and_counts_it_as_already_stored() {
    let directory = TestDirectory::new("resent-frame");
    let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::Registered, 10);
    let (key, value, frame) = frame_for(5, SMALL_STATES_PER_BLOCK);
    let mut stream = frame.clone();
    stream.extend_from_slice(&frame);
    let mut acknowledgements = Vec::new();
    let end = receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
    assert_eq!(end, ConnectionEnd::ClosedAtFrameBoundary);
    assert_eq!(acknowledged_keys(&acknowledgements), vec![key, key]);
    assert_eq!(receiver.counters.commits, 2, "同键再来一次就同值再写一遍");
    assert_eq!(receiver.counters.frames_already_stored, 1);
    assert_eq!(receiver.store.read_block(&key), ReadOutcome::Found(value));
}

#[test]
fn torn_accepting_receiver_writes_a_zero_padded_half_frame_and_then_skips_the_resent_frame() {
    let directory = TestDirectory::new("torn-receiver");
    let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::TornAccepting, 10);
    let (key, value, frame) = frame_for(7, SMALL_STATES_PER_BLOCK);
    let cut = BLOCK_FRAME_HEADER_BYTES + 300;
    let mut acknowledgements = Vec::new();
    let end = receiver.serve_connection(&mut &frame[..cut], &mut acknowledgements, None);
    assert_eq!(end, ConnectionEnd::HalfFrame);
    assert_eq!(receiver.counters.half_frames_written, 1);
    let end = receiver.serve_connection(&mut frame.as_slice(), &mut acknowledgements, None);
    assert_eq!(end, ConnectionEnd::ClosedAtFrameBoundary);
    assert_eq!(acknowledged_keys(&acknowledgements), vec![key], "重发的帧键已在：跳过不写、照回确认");
    assert_eq!(receiver.counters.frames_skipped_as_already_stored, 1);
    let mut padded = value[..300].to_vec();
    padded.resize(value.len(), 0);
    assert_eq!(receiver.store.read_block(&key), ReadOutcome::Found(padded), "库里留着补 0 的半帧");
}

#[test]
fn drop_control_receiver_acknowledges_without_writing_exactly_the_picked_frames() {
    let directory = TestDirectory::new("drop-receiver");
    let network_blocks = 2_000u64;
    let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::DropOnePerThousand, network_blocks);
    let mut stream = Vec::new();
    for network_position in 0..network_blocks {
        stream.extend_from_slice(&frame_for(network_position, SMALL_STATES_PER_BLOCK).2);
    }
    let mut acknowledgements = Vec::new();
    receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
    assert_eq!(acknowledged_keys(&acknowledgements).len(), 2_000);
    assert_eq!(receiver.counters.controlled_drops, 2);
    assert_eq!(receiver.counters.commits, 1_998);
    let picks = drop_control_picks(2);
    for (window, pick) in picks.iter().enumerate() {
        let dropped_position = u64::try_from(window).expect("窗号") * 1_000 + pick;
        let (key, _value, _frame) = frame_for(dropped_position, SMALL_STATES_PER_BLOCK);
        assert_eq!(receiver.store.read_block(&key), ReadOutcome::NotFound, "挑中的第 {dropped_position} 帧不写");
    }
}

#[test]
fn acknowledge_without_writing_receiver_leaves_the_library_empty() {
    let directory = TestDirectory::new("raw-receiver");
    let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::AcknowledgeWithoutWriting, 10);
    let mut stream = Vec::new();
    for network_position in 0..5 {
        stream.extend_from_slice(&frame_for(network_position, SMALL_STATES_PER_BLOCK).2);
    }
    let mut acknowledgements = Vec::new();
    receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
    assert_eq!(acknowledged_keys(&acknowledgements).len(), 5);
    assert_eq!(receiver.counters.commits, 0);
    assert_eq!(receiver.store.enumerate_keys_with_value_lengths().expect("枚举得了").len(), 0);
}

#[test]
// wallclock-assert-load-safe: 只比 20 帧至少花了 100 ms（下界，每帧先睡 5 ms），负载再高只会更久
fn sleep_before_commit_receiver_spends_at_least_five_milliseconds_per_frame() {
    let directory = TestDirectory::new("sleep-receiver");
    let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::SleepBeforeCommit, 20);
    let mut stream = Vec::new();
    for network_position in 0..20 {
        stream.extend_from_slice(&frame_for(network_position, SMALL_STATES_PER_BLOCK).2);
    }
    let mut acknowledgements = Vec::new();
    let started = Instant::now();
    receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
    assert!(started.elapsed() >= Duration::from_millis(100), "20 帧每帧先睡 5 ms");
    assert_eq!(receiver.counters.commits, 20);
}

#[test]
fn stop_after_acknowledgements_returns_interruption_due_right_after_that_acknowledgement() {
    let directory = TestDirectory::new("stop-after");
    let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::Registered, 10);
    let mut stream = Vec::new();
    for network_position in 0..6 {
        stream.extend_from_slice(&frame_for(network_position, SMALL_STATES_PER_BLOCK).2);
    }
    let mut acknowledgements = Vec::new();
    let end = receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, Some(4));
    assert_eq!(end, ConnectionEnd::InterruptionDue);
    assert_eq!(acknowledged_keys(&acknowledgements).len(), 4);
}

// 下面几个数由另写的 python（另起一份 SplitMix64，脚本与原样输出抄在登记第十二节「修订」第三段那几行里）独立算出，
// 不取自这个文件的输出。
const PINNED_FIRST_THRESHOLDS: [u64; 5] = [18, 114, 463, 950, 1132];
const PINNED_LAST_THRESHOLD: u64 = 49_349;
const PINNED_FIRST_PAUSE_FRACTION: f64 = 0.5665882453311935;
const PINNED_FIRST_DROP_PICKS: [u64; 3] = [541, 117, 338];
