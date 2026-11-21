#![allow(
    dead_code,
    reason = "照抄的片段把 pool 之后用不到的变量也建了出来"
)]

use super::experiment_pool::{build_pool, geometry};
use singlefs_verification::memory_pool::{writes_and_segments, MemoryPool, RetainedWrite};
use singlefs_verification::segments::StepKind;

pub struct PreparedNewPool {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub root_index: usize,
}

pub fn prepare_through_the_first_file(tag: &str) -> PreparedNewPool {
// COPIED-BEGIN new_pool_prepare_body crates/singlefs-verification/tests/crash_enumeration_new_pool_file_creation_stream.rs:859-877
    let pool = build_pool(tag);
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2],
        "取号写两轮、轮后各一道池屏障，各成一段；C577 之后暖机第二次的系统配置槽轮换过了发布末尾那道屏障，不再与 A 的 24 个单元写同段"
    );
    assert_eq!(
        writes.len(),
        43,
        "取号两轮各 2 + 暖机两次各 5 + A 29（24 个单元写、两份记录、根槽、两块盘的系统配置槽轮换）"
    );
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽那一条");
// COPIED-END new_pool_prepare_body
    PreparedNewPool {
        base,
        writes,
        segments,
        root_index,
    }
}
