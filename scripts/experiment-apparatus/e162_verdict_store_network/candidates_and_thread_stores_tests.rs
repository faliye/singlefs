use super::*;

#[test]
fn loopback_run_with_five_forwarder_kills_resends_and_keeps_every_block() {
    let directory = TestDirectory::new("five-kills");
    let cell = small_loopback_cell("five-kills", 128, 600, PRIMARY_STATES_PER_BLOCK, ReceiveMode::Registered, 5);
    let run = run_loopback_cell(Candidate::RocksDatabase, &cell, &directory.path);
    assert_clean_run(&run, &cell, Candidate::RocksDatabase);
    assert_eq!(run.interruptions.len(), 5);
    assert!(run.interruptions.iter().all(|interruption| interruption.forwarder_restarted));
    // 每次杀转发进程至少逼出一次重连；被杀的进程退出时连接先断、监听口后关，送块进程可能在这几十微秒里
    // 又连上那个正在退出的监听口、重发一段之后再断一次（推的：单测读到过重连用时几十微秒、同一确认数紧接一次
    // 正常重连的 R 行），所以一次中断记一到两次重连。
    let reconnections = number_field(&run.sender_summary, "reconnections");
    assert!(
        (5..=10).contains(&reconnections),
        "{} | {:?} | {:?} | {}",
        run.sender_summary,
        run.sender_reconnections,
        run.interruptions,
        run.library_network
    );
    assert!(number_field(&run.sender_summary, "resent_frames") >= 5, "{}", run.sender_summary);
    assert!(number_field(&run.library_network, "connections") >= 6, "{}", run.library_network);
}

#[test]
fn drop_control_run_loses_exactly_the_frames_the_library_recorded() {
    let directory = TestDirectory::new("drop-run");
    let cell = small_loopback_cell("drop", 0, 2_000, SMALL_STATES_PER_BLOCK, ReceiveMode::DropOnePerThousand, 0);
    let run = run_loopback_cell(Candidate::HardenedFile, &cell, &directory.path);
    assert_eq!(number_field(&run.library_network, "controlled_drops"), 2);
    assert_eq!(run.verification.lost_confirmed.len(), 2, "PC3-drop：丢块数恰好 = 库进程记的数");
    assert_eq!(run.verification.silently_corrupted.len(), 0);
    assert_eq!(run.verification.enumerated_key_count, 1_998);
}
