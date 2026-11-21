//! 一条流在服务里的键与调度要的两样事实。
//!
//! 键是（打包判表的摘要、三份内核源码的摘要）：同一份表照两套源码编出来的判器不同，是两条流；工作线程的判器按这把键缓存。键只在这个进程里认流，不写进判定库、
//! 不进任何判定键（判定库里判表的键是判表摘要与旋钮摘要，`crate::packed_tables_family::PackedTablesName`）。
//! 表的摘要与判定库存判表用的是同一个（`sha256(PackedJudgeTables::to_bytes())`），从库里按摘要读回判表时不另算；源码的摘要是这个文件里的 `kernel_sources_digest`。

use std::fmt;

use singlefs_verification::crash_judge_dispatch::PackedJudgeTables;
use singlefs_verification::crash_judge_gpu::JudgeKernelSources;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::sha256::{sha256_digest, DIGEST_BYTES};

/// 三份内核源码合起来的摘要：每份前面带它的标签与字节数，三份之间的边界挪了摘要跟着变。
fn kernel_sources_digest(kernel_sources: &JudgeKernelSources) -> [u8; DIGEST_BYTES] {
    let mut message = Vec::new();
    for (label, source) in [
        ("recovery", &kernel_sources.recovery),
        ("checker-walk", &kernel_sources.checker_walk),
        ("checker-scan", &kernel_sources.checker_scan),
    ] {
        message.extend_from_slice(format!("{label}\n{}\n", source.len()).as_bytes());
        message.extend_from_slice(source.as_bytes());
    }
    sha256_digest(&message)
}

/// 流的键：批次调度、待判池、工作线程的判器缓存都按它认流。比较按字节序，调度里「同样入池先后时按键排」照它定。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamKey {
    pub tables_digest: [u8; DIGEST_BYTES],
    pub kernel_sources_digest: [u8; DIGEST_BYTES],
}

impl fmt::Debug for StreamKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "StreamKey(tables={}, kernel_sources={})",
            hexadecimal_text(&self.tables_digest[..6]),
            hexadecimal_text(&self.kernel_sources_digest[..6])
        )
    }
}

impl StreamKey {
    /// 一份打包判表照一套内核源码判的那条流。
    #[must_use]
    pub fn of_the_tables_under_the_kernel_sources(
        tables: &PackedJudgeTables,
        kernel_sources: &JudgeKernelSources,
    ) -> Self {
        Self {
            tables_digest: sha256_digest(&tables.to_bytes()),
            kernel_sources_digest: kernel_sources_digest(kernel_sources),
        }
    }
}

/// 批次调度要的一条流的事实：打包判表上卡占多少显存（`PackedJudgeTables::mebibytes`）、一共几个态（序号从 0 起）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamFacts {
    pub table_mebibytes: u64,
    pub state_count: u64,
}

impl StreamFacts {
    #[must_use]
    pub fn of_the_tables(tables: &PackedJudgeTables) -> Self {
        Self {
            table_mebibytes: tables.mebibytes(),
            state_count: tables.state_count(),
        }
    }
}
