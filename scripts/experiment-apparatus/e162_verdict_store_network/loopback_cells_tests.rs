use super::*;

const ALL_ARMS: [Candidate; 4] = [Candidate::RedbQuickRepair, Candidate::RedbDefault, Candidate::RocksDatabase, Candidate::HardenedFile];

#[test]
fn every_arm_keeps_both_paths_through_a_small_loopback_run() {
    let directory = TestDirectory::new("every-arm");
    for arm in ALL_ARMS {
        let cell = small_loopback_cell("small", 96, 128, SMALL_STATES_PER_BLOCK, ReceiveMode::Registered, 0);
        let run = run_loopback_cell(arm, &cell, &directory.path);
        assert_clean_run(&run, &cell, arm);
        assert_eq!(run.sender_exit_code, 0);
        assert_eq!(run.library_exit_code, 0);
        assert_eq!(number_field(&run.library_network, "commits"), 128);
    }
}
