//! 驱动方记的调用（calls.tsv）。

use super::*;

// ───────────────────────────── 驱动方记的调用（calls.tsv） ─────────────────────────────

/// 驱动方调的一次 API：标签、它发出的第一条操作的流序下标、返回点（返回时录制流的长度），都从 mkfs 之后数。
/// 失败的调用在这个装置里一律 `expect` 停下，写进表的只有 `ok`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RecordedCall {
    pub(super) label: &'static str,
    pub(super) first_operation: usize,
    pub(super) return_point: usize,
}

/// 调 `run` 并记一行：调之前与返回时各读一次流长。
pub(super) fn record_call<Output>(
    calls: &mut Vec<RecordedCall>,
    stream: &SharedStream,
    operations_before_the_recording: usize,
    label: &'static str,
    run: impl FnOnce() -> Output,
) -> Output {
    let first_operation = stream.operation_count() - operations_before_the_recording;
    let output = run();
    let return_point = stream.operation_count() - operations_before_the_recording;
    calls.push(RecordedCall {
        label,
        first_operation,
        return_point,
    });
    output
}
