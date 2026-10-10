//! 层 0 枚举域的规模：几段代表性的删除历史（录制流从 mkfs 之后起），按 `closed_form_state_count` 算全量状态数、列各段写数。
//! 原型规则：一段历史全量超过约 10⁶ 个状态的不跑；这里只数，不枚举（数出来每段都超过，见报告）。
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments_with_stream_indexes};

use crate::badimages::intent;
use crate::common::{file_content, geometry, FIXED_WRITE_TIME_SECONDS};
use crate::common_deletion_in_progress as del;
use crate::pool::SparsePool;

pub fn run(_from: usize, _to: usize, _threads: usize) {
    let mut cases: Vec<(&str, SparsePool, usize)> = Vec::new();
    // 每一种：前缀照建，录制流里意图之前那一段也在流里（层 0 枚举的是 mkfs 之后整条流）。
    let mut one = SparsePool::build(4 << 30, &file_content());
    let before = one.stream.operation_count();
    intent(&mut one, 1);
    one.steps_until_empty(144);
    cases.push(("一个单元的文件：意图 + 最后一批", one, before));
    let mut empty = SparsePool::build(4 << 30, &file_content());
    empty.new_inodes(1, FIXED_WRITE_TIME_SECONDS);
    let before = empty.stream.operation_count();
    intent(&mut empty, 2);
    empty.steps_until_empty(1);
    cases.push(("size 0 的 2 号：意图 + 最后一批", empty, before));
    let mut three = SparsePool::build(4 << 30, &file_content());
    three.sequential_write(&del::content_of_three_data_units(), FIXED_WRITE_TIME_SECONDS + 60);
    let before = three.stream.operation_count();
    intent(&mut three, 1);
    three.steps_until_empty(1);
    cases.push(("H1 形：三个单元、批大小 1 删到空", three, before));
    let mut h3 = SparsePool::build(4 << 30, &file_content());
    h3.new_inodes(1, FIXED_WRITE_TIME_SECONDS);
    h3.sequential_write(&del::content_of_three_data_units(), FIXED_WRITE_TIME_SECONDS + 60);
    let before = h3.stream.operation_count();
    let time = crate::badimages::record_time(&h3, 1) + 60;
    h3.intent(1, time).expect("意图");
    h3.steps_until_empty(1);
    cases.push(("H3 形：先建 size 0 的 2 号、三个单元、批大小 1 删到空", h3, before));
    for (name, pool, before_intent) in &cases {
        let operations = pool.stream.retained_operations();
        let (_, segments, stream_indexes) = writes_and_segments_with_stream_indexes(&operations[pool.mkfs_operation_count..], &geometry());
        let whole = closed_form_state_count(&segments);
        // 只数意图那次发布起的段（段里第一次写在流里的下标不早于意图之前录制流的长度）。
        let from_intent: Vec<Vec<usize>> = segments
            .iter()
            .filter(|segment| segment.first().is_some_and(|write| stream_indexes[*write] + pool.mkfs_operation_count >= *before_intent))
            .cloned()
            .collect();
        println!(
            "L0 case=「{name}」 segments={} closed_form_state_count_whole_stream={whole} segments_from_the_intent={} closed_form_state_count_from_the_intent={} longest_segments_from_the_intent={:?} runs=no（超过约 10⁶，原型规则不跑）",
            segments.len(),
            from_intent.len(),
            closed_form_state_count(&from_intent),
            {
                let mut lengths: Vec<usize> = from_intent.iter().map(Vec::len).collect();
                lengths.sort_unstable_by(|a, b| b.cmp(a));
                lengths.truncate(6);
                lengths
            }
        );
    }
}
