//! 重开可写挂载时取号写两轮与写行那次发布的单元写之间的屏障（D22（单元原子性怎么合成） 已定项 8 取号写两轮，
//! D23（journal 的角色与格式） 已定项 16「取号那一步的屏障」）：每轮逐盘写一次系统配置槽、轮后一道池屏障，
//! 第二轮之后那道屏障把取号与写行那次发布的单元写隔开。
//!
//! 流同 checker 档 `crash_enumeration_acquisition_barrier.rs`：mkfs → 取号 1 → 暖机 → A → B（同一个进程）→ 进程退出、
//! 重开可写挂载（取号 2 → 写行 → 暖机）。这里只按崩溃点重放的切段规则切录制流、钉段的形状，不枚举崩溃状态：
//! 少了每轮之后那道屏障，取号两轮与写行的单元写并成一段，崩溃点重放就摆得出「新实例的单元已持久、两块盘的系统配置还是旧号」的状态。
//! 那一族状态由 checker 档那条用例按 I-7.7（系统配置实例代号不低于根环） 判；并段之后那条用例展开的域从二十几万态涨到两千多万态，
//! 改坏之后一小时跑不完，这一条在 harness 档几秒内先判出段并了。

mod common;

use common::{
    build_pool, geometry, parameters, publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::mount::mount_writable;
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::segments::StepKind;

/// B 的内容：与第一个文件不同长、不同字节（同 checker 档那条用例的第二次写）。
const SECOND_FILE_BYTES: usize = 4100;

/// 取号写一轮的系统配置槽写数：两块盘各一次。
const SYSTEM_CONFIGURATION_WRITES_OF_ONE_ACQUISITION_ROUND: usize = 2;

/// 取号写的轮数（D22（单元原子性怎么合成） 已定项 8 落地）。
const ACQUISITION_ROUNDS: usize = 2;

/// 写行那次发布的单元写数（checker 档那条用例钉的展开段 `[2, 2, 18]` 的末一段）。
const UNIT_WRITES_OF_THE_ROW_PUBLISH: usize = 18;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 判别力（`crates/mutations.tsv`「步 3 验收：取号之后没有屏障」那一行）：去掉每轮之后那道屏障，取号两轮的四次系统配置槽写
/// 与写行那次发布的 18 个单元写并成一段，取号只落在一段里、那一段也是写行单元写所在的段，这条红。
#[test]
fn each_acquisition_round_of_a_remount_is_its_own_segment_and_the_unit_writes_of_the_row_publish_start_another(
) {
    let mut pool = build_pool("acquisition-rounds-separated-by-barriers");
    let previous = pool.output.clone();
    let second = publish_overwrite_in_process(
        &mut pool,
        &previous,
        &second_content(),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 B");
    pool.output = second;
    let operations_before_the_remount = pool.stream.operations().len();
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开之后可写挂载");
    pool.devices = Some(devices);
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "重开取号 2");

    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let earliest_stream_index_of_the_remount =
        operations_before_the_remount - pool.mkfs_operation_count;
    let remount_writes: Vec<usize> = (0..writes.len())
        .filter(|write_index| stream_indexes[*write_index] >= earliest_stream_index_of_the_remount)
        .collect();
    let acquisition_write_count =
        ACQUISITION_ROUNDS * SYSTEM_CONFIGURATION_WRITES_OF_ONE_ACQUISITION_ROUND;
    assert_eq!(
        remount_writes[..acquisition_write_count]
            .iter()
            .map(|write_index| writes[*write_index].kind)
            .collect::<Vec<StepKind>>(),
        vec![StepKind::SystemConfigurationSlot; acquisition_write_count],
        "重开之后头四个写是取号那四次系统配置槽写（两轮，每轮两块盘各一次）"
    );
    let segment_of = |write_index: usize| {
        segments
            .iter()
            .position(|segment| segment.contains(&write_index))
            .expect("每个写都在某一段里")
    };
    let acquisition_segments: Vec<usize> = remount_writes[..acquisition_write_count]
        .iter()
        .map(|write_index| segment_of(*write_index))
        .fold(Vec::new(), |mut segment_indexes, segment_index| {
            if !segment_indexes.contains(&segment_index) {
                segment_indexes.push(segment_index);
            }
            segment_indexes
        });
    let first_unit_write_of_the_remount = *remount_writes
        .iter()
        .find(|write_index| writes[**write_index].kind == StepKind::UnitWrite)
        .expect("写行那次发布写单元");
    let first_unit_segment = segment_of(first_unit_write_of_the_remount);
    let kinds_of_the_segment = |segment_index: usize| {
        segments[segment_index]
            .iter()
            .map(|write_index| writes[*write_index].kind)
            .collect::<Vec<StepKind>>()
    };

    assert_eq!(
        acquisition_segments
            .iter()
            .map(|segment_index| kinds_of_the_segment(*segment_index))
            .collect::<Vec<Vec<StepKind>>>(),
        vec![
            vec![
                StepKind::SystemConfigurationSlot;
                SYSTEM_CONFIGURATION_WRITES_OF_ONE_ACQUISITION_ROUND
            ];
            ACQUISITION_ROUNDS
        ],
        "取号两轮各自成一段，每段只有那一轮两块盘各一次的系统配置槽写：段 {acquisition_segments:?}，写行单元写在段 {first_unit_segment}"
    );
    assert_eq!(
        kinds_of_the_segment(first_unit_segment),
        vec![StepKind::UnitWrite; UNIT_WRITES_OF_THE_ROW_PUBLISH],
        "写行那次发布的单元写另成一段，里面没有取号的系统配置槽写"
    );
    assert_eq!(
        (acquisition_segments, first_unit_segment),
        (vec![16, 17], 18),
        "取号两轮与写行单元写是相邻的三段：与 checker 档那条用例展开的段号相同"
    );
}
