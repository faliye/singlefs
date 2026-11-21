use super::*;

#[test]
fn acknowledged_ranges_expand_back_to_the_same_flags() {
    let flags = vec![true, true, false, true, false, false, true];
    assert_eq!(expand_true_positions(&compress_true_positions(&flags), 7), Some(flags));
    assert_eq!(expand_true_positions("none", 2), Some(vec![false, false]));
    assert_eq!(expand_true_positions("0-7", 7), None, "越界的区间认不出");
    assert_eq!(expand_true_positions("3-1", 7), None, "倒过来的区间认不出");
    let everything = expand_true_positions("0-49999", 50_000).expect("认得出");
    assert_eq!(everything.iter().filter(|&&flag| flag).count(), 50_000);
}

#[test]
fn classification_table_matches_the_registered_table() {
    use BlockClass::{Confirmed, InFlight, NotStarted};
    use BlockVerdict::{ConfirmedReadError, Correct, LostConfirmed, Phantom, SilentlyCorrupted, UnconfirmedReadError};
    use ReadKind::{DifferentBytes, MatchingBytes, NotFound, ReadError};
    let expected = [
        (Confirmed, [Correct, SilentlyCorrupted, LostConfirmed, ConfirmedReadError]),
        (InFlight, [Correct, SilentlyCorrupted, Correct, UnconfirmedReadError]),
        (NotStarted, [Phantom, Phantom, Correct, UnconfirmedReadError]),
    ];
    for (class, verdicts) in expected {
        for (read, verdict) in [MatchingBytes, DifferentBytes, NotFound, ReadError].into_iter().zip(verdicts) {
            assert_eq!(classify_block(class, read), verdict, "{class:?} × {read:?}");
        }
    }
}

#[test]
fn verification_counts_a_flipped_byte_a_missing_confirmed_block_and_a_phantom_key() {
    let directory = TestDirectory::new("verification");
    let library = directory.path.join("library");
    let mut stores = open_thread_stores(Candidate::HardenedFile, &library, 1).expect("开得了 F 库");
    let mut store = stores.pop().expect("一个句柄");
    let fingerprint = input_fingerprint();
    let states = SMALL_STATES_PER_BLOCK;
    for direct_position in 0..4u64 {
        let block_index = 2 * direct_position;
        let mut value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, states);
        if direct_position == 2 {
            value[500] ^= 0x5A;
        }
        store.put_block(&interleaved_block_key(block_index, states, &fingerprint), &value).expect("写得进");
    }
    for network_position in [0u64, 2] {
        let block_index = network_path_block_index(network_position);
        let value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, states);
        store.put_block(&interleaved_block_key(block_index, states, &fingerprint), &value).expect("写得进");
    }
    let phantom = interleaved_block_key(9_999, states, &fingerprint);
    store.put_block(&phantom, &[1u8; 1_024]).expect("写得进");
    let plan = PathVerificationPlan {
        states_per_block: states,
        direct_blocks: 4,
        direct_confirmed: 4,
        direct_started: 4,
        network_blocks: 4,
        network_acknowledged: vec![true, true, true, false],
        network_sent: 4,
    };
    let report = verify_library(Candidate::HardenedFile, &library, &plan);
    assert!(!report.open_failed);
    assert_eq!(report.silently_corrupted, vec![4], "第 2 个直写块 j = 4 翻了一个字节");
    assert_eq!(report.lost_confirmed, vec![3], "网络第 1 块 j = 3 确认过而不在");
    assert_eq!(report.phantom_keys.len(), 1);
    assert_eq!(report.confirmed_checked, 7);
    assert_eq!(report.in_flight_checked, 1);
    assert_eq!(report.enumerated_key_count, 7);
    let decoded = decode_verification_line(&report.encode());
    assert_eq!(decoded.lost_confirmed, vec![3]);
    assert_eq!(decoded.silently_corrupted, vec![4]);
    assert_eq!(decoded.phantom_keys.len(), 1);
    assert_eq!(decoded.enumerated_key_count, 7);
}

#[test]
fn peer_channel_run_with_five_relay_kills_keeps_every_block() {
    let directory = TestDirectory::new("peer-channel");
    let socket_path = directory.path.join("s.sock");
    let network_blocks = 300u64;
    let sender_settings = SenderSettings {
        endpoint: SenderEndpoint::ListenUnix(socket_path.clone()),
        block_count: network_blocks,
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        window_frames: PRIMARY_WINDOW_FRAMES,
        pace_between_first_transmissions: None,
        acknowledgement_stall_limit: Duration::from_secs(20),
        spinning_thread_count: 0,
    };
    let sender = std::thread::spawn(move || {
        let mut output = Vec::new();
        sender_role::run_sender(&sender_settings, &mut output)
    });
    let test_binary = std::env::current_exe().expect("找得到测试二进制");
    let relay_command = format!(
        "{RELAY_TARGET_ENVIRONMENT}='unix:{}' exec '{}' --exact tests::child_process_role_entry --nocapture --test-threads=1 --quiet 3>&1 1>/dev/null",
        socket_path.display(),
        test_binary.display()
    );
    let library = directory.path.join("library");
    let library_settings = LibrarySettings {
        candidate: Candidate::HardenedFile,
        library: library.clone(),
        receive_mode: ReceiveMode::Registered,
        direct_blocks: 64,
        network_blocks,
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        channel: ReceiveChannel::PeerCommand {
            command_words: vec!["sh".to_string(), "-c".to_string()],
            relay_command,
            interruptions: interruption_plan(5, network_blocks),
        },
    };
    let mut library_process = spawn_running_child(child_command(&library_settings.to_arguments()), true);
    let report = sender.join().expect("送块线程没 panic");
    drop(library_process.input.take());
    let lines: Vec<String> = library_process.lines.iter().collect();
    library_process.child.wait().expect("收得了库进程");
    let network_line = lines.iter().find(|line| line.starts_with("N ")).cloned().unwrap_or_default();
    let direct_line = lines.iter().find(|line| line.starts_with("D ")).cloned().unwrap_or_default();
    let interruption_lines = lines.iter().filter(|line| line.starts_with("I ")).count();
    assert_eq!(report.ending, sender_role::SenderEnding::Complete, "{network_line}");
    assert_eq!(report.acknowledged_count, network_blocks);
    assert_eq!(report.reconnections.len(), 5, "{network_line}");
    assert_eq!(interruption_lines, 5, "库进程回出第 a_k 个确认之后杀数据通道，五次");
    let plan = PathVerificationPlan {
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        direct_blocks: 64,
        direct_confirmed: number_field(&direct_line, "confirmed"),
        direct_started: number_field(&direct_line, "started"),
        network_blocks,
        network_acknowledged: expand_true_positions(&report.acknowledged_ranges, network_blocks).expect("区间认得出"),
        network_sent: report.sent_positions,
    };
    let verification = verify_library(Candidate::HardenedFile, &library, &plan);
    assert_eq!(verification.lost_confirmed.len(), 0);
    assert_eq!(verification.silently_corrupted.len(), 0);
    assert_eq!(verification.phantom_keys.len(), 0);
    assert_eq!(verification.enumerated_key_count, 364);
}
