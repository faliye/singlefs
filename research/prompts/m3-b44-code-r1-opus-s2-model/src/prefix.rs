//! 换几何的取样（照 c834 第三轮攻方原型的前缀写法，这一腿重写一份）：R 之前的前缀是一串用户动作（o 覆盖写一次、i 建一个 inode、w 顺序写两个数据单元），
//! 让 R 的单元在单元区里挪位，看 M1（入库 `urJ` 那一形）的写落到 R 的树表、映射根两个指针槽上的是什么。
//! 录法逐行照入库搭建 `common_unwitnessed_publish::record_one_publish_after_overwrites`，只把前缀换成这一串。
use crate::common_unwitnessed_publish::{
    device_bytes_of_the_384_slot_small_devices, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, PoolUnderTest, PublishRecordedAsR, RecordedPublish,
    OVERWRITE_BYTES,
};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 起池（384 槽那一档、第一个文件），依次做 `prefix` 里的动作，再经会话录一次 `publish`；前缀或 R 被拒交回 `Err`（原样拒因）。
pub fn record_after_prefix(prefix: &str, publish: PublishRecordedAsR) -> Result<RecordedPublish, String> {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    );
    for step in prefix.chars() {
        match step {
            'o' => pool.overwrite(OVERWRITE_BYTES).map(|_| ()),
            'i' => pool.create_one_inode().map(|_| ()),
            'w' => pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity()).map(|_| ()),
            other => panic!("认不出前缀动作 {other}"),
        }
        .map_err(|refusal| format!("prefix_refused:{step}:{refusal:?}").chars().take(160).collect::<String>())?;
    }
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    match publish {
        PublishRecordedAsR::Overwrite => pool.overwrite(OVERWRITE_BYTES),
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits => {
            pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity() + 1000)
        }
    }
    .map_err(|refusal| format!("r_refused:{refusal:?}").chars().take(160).collect::<String>())?;
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    if root_indices.len() != 1 {
        return Err(format!("r_roots={}", root_indices.len()));
    }
    Ok(RecordedPublish {
        content: pool.content_of_the_current_version.clone(),
        pool,
        base,
        writes,
        root_index: root_indices[0],
        key: (root.instance, root.checkpoint_txg),
        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
        previous_content,
    })
}
