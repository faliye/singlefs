//! 几份测试共用的辅助：每个单测的临时目录、中继目标的环境变量、小回环格与它的干净核对。

use super::*;

/// 跨机格单测里的中继：库进程（本身是带着 CHILD_ROLE_ENVIRONMENT 起的子进程）经 `sh -c` 起的孙进程，
/// 另用这个环境变量带中继的目标，免得与 CHILD_ROLE_ENVIRONMENT 的 `\u{1f}` 分隔嵌在一起。
pub(super) const RELAY_TARGET_ENVIRONMENT: &str = "E162_S3_TEST_RELAY_TARGET";

/// 每个单测一个自己的临时目录，测完删掉。
pub(super) struct TestDirectory {
    pub(super) path: PathBuf,
}

impl TestDirectory {
    pub(super) fn new(name: &str) -> TestDirectory {
        let path = std::env::temp_dir().join(format!("e162-s3-test-{}-{name}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path).expect("删得了上一次的临时目录");
        }
        fs::create_dir_all(&path).expect("建得了临时目录");
        TestDirectory { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _cleanup_outcome = fs::remove_dir_all(&self.path);
    }
}

pub(super) fn small_loopback_cell(label: &'static str, direct_blocks: u64, network_blocks: u64, states_per_block: u64, receive_mode: ReceiveMode, interruption_count: usize) -> LoopbackCell {
    LoopbackCell { label, direct_blocks, network_blocks, states_per_block, window_frames: PRIMARY_WINDOW_FRAMES, receive_mode, interruption_count, is_timed: false }
}

pub(super) fn assert_clean_run(run: &NetworkCellRun, cell: &LoopbackCell, arm: Candidate) {
    let context = format!("臂 {} 格 {}：{} | {} | {}", arm.label(), cell.label, run.sender_summary, run.library_network, run.setup_problem);
    assert_eq!(run.setup_problem, "", "{context}");
    assert_eq!(field_value(&run.sender_summary, "ending"), Some("complete"), "{context}");
    assert_eq!(number_field(&run.sender_summary, "acknowledged"), cell.network_blocks, "{context}");
    assert_eq!(number_field(&run.library_direct, "confirmed"), cell.direct_blocks, "{context}");
    assert!(!run.verification.verification_missing, "{context}");
    assert!(!run.verification.open_failed, "{context}");
    assert_eq!(run.verification.lost_confirmed.len(), 0, "{context}");
    assert_eq!(run.verification.silently_corrupted.len(), 0, "{context}");
    assert_eq!(run.read_errors(), 0, "{context}");
    assert_eq!(run.verification.phantom_keys.len(), 0, "{context}");
    assert_eq!(run.verification.enumerated_key_count, cell.direct_blocks + cell.network_blocks, "{context}");
    assert_eq!(run.verification.enumerated_value_bytes, (cell.direct_blocks + cell.network_blocks) * cell.states_per_block, "{context}");
}
