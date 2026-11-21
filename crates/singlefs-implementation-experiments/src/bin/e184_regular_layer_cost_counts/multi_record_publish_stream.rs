#![allow(
    dead_code,
    reason = "照抄的测试代码里发布边界观察者另读的输入（publish_boundary_inputs）在这个装置里没人读"
)]

use super::experiment_pool::{build_pool, file_content, geometry};
use super::multi_record_steps::{
    content_needing, multi_record_publish, sequential_write, MultiRecordPublishBoundaryInputs,
};
use singlefs_verification::memory_pool::{
    writes_and_segments, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_verification::segments::StepKind;

// COPIED-BEGIN multi_record_publish crates/singlefs-verification/tests/crash_enumeration_multi_record_publish_stream.rs:62-152
struct PreparedMultiRecordPublishStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    /// 发布边界观察者另读的输入（公共模块 `MultiRecordPublishBoundaryInputs`）：B、C 两次多条记录的发布。
    publish_boundary_inputs: MultiRecordPublishBoundaryInputs,
}

fn prepare_the_multi_record_publish_stream(tag: &str) -> PreparedMultiRecordPublishStream {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let second = sequential_write(&mut pool, &second_content);
    assert_eq!(
        second.earlier_records_of_this_publish.len() + 1,
        2,
        "B：两个数据单元两条记录"
    );
    assert_eq!(
        second.rewritten.len(),
        2 + 2 + 2 + 5 + 3,
        "B：两个数据单元 + extent 下段根兼叶与上段根 + inode 叶容器与根 + 分配记录树五个节点 + 记账树、中央映射树、树表 = 14 个单元\
             （验收第 3 条的「≤ 10」今天不成立，见文件头）：{:?}",
        second.rewritten
    );
    let third_content = content_needing(3, 2);
    let third = sequential_write(&mut pool, &third_content);
    assert_eq!(
        third.earlier_records_of_this_publish.len() + 1,
        3,
        "C：三个数据单元三条记录"
    );
    assert_eq!(
        third.rewritten.len(),
        3 + 2 + 2 + 5 + 3,
        "C：三个数据单元 + 与 B 同样的 12 个单元 = 15 个单元（「≤ 10」今天不成立，见文件头）：{:?}",
        third.rewritten
    );

    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2],
        "取号两轮各 2、暖机两次、上一次的系统配置槽轮换 2 与 A 的 24 个单元写各自成一段；A 的轮换 2、B 的 28 个单元写、两条记录 4 写；\
             B 的轮换 2、C 的 30 个单元写、三条记录 6 写（C577 之后轮换之后有屏障，改之前轮换并进下一次的单元段：26、30、32）"
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 5, "暖机两次、A、B、C 各一条根槽写");
    let versions = vec![
        PublishedVersion {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
            content: file_content(),
        },
        PublishedVersion {
            instance: second.root.instance,
            checkpoint_txg: second.root.checkpoint_txg,
            content: second_content.clone(),
        },
        PublishedVersion {
            instance: third.root.instance,
            checkpoint_txg: third.root.checkpoint_txg,
            content: third_content,
        },
    ];
    let publish_boundary_inputs = MultiRecordPublishBoundaryInputs {
        publishes: vec![
            multi_record_publish(&writes, &second, &first, file_content()),
            multi_record_publish(&writes, &third, &second, second_content),
        ],
    };
    PreparedMultiRecordPublishStream {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        versions,
        publish_boundary_inputs,
    }
}
// COPIED-END multi_record_publish

pub fn prepare_through_the_third_publish(tag: &str) -> (MemoryPool, Vec<RetainedWrite>, Vec<Vec<usize>>, usize, Vec<PublishedVersion>) {
    let prepared = prepare_the_multi_record_publish_stream(tag);
    (prepared.base, prepared.writes, prepared.segments, prepared.judged_root_index, prepared.versions)
}
