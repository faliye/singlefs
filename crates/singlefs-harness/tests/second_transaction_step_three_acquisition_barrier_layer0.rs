//! 里程碑「第二个事务」步 3 验收的变异「取号之后没有屏障 ⇒ 层 0 里出现「系统配置已是 2、单元却先持久」的状态，
//! I-7.7（系统配置实例代号不低于根环） 那一族判红」要红在的那一条层 0 用例。
//!
//! 流：mkfs → 取号 1 → 暖机 → A → B（同一个进程）→ 进程退出、重开可写挂载（取号 2 → 写行 → 暖机）。
//! 取号那两次系统配置槽写与 B 的两次系统配置槽轮换同一段（进程退出与重开之间没有屏障，登记表八那条 ⚠️），
//! 取号那道屏障把它们与写行那次发布的单元写隔开（D23（journal 的角色与格式） 已定项 16「取号那一步的屏障」）。
//! 平时展开的只有这两段：取号所在的那一段、取号之后第一个带单元写的那一段——少了取号那道屏障，两段并成一段，
//! 枚举就摆得出「实例 2 的单元已持久、两块盘的系统配置还都是 1」的状态。
//! 展开哪几段按写在录制流里的位置挑（重开之后的第一个写起），不按段的写数挑：并段之后段变长，按写数挑会把并出来的那一段漏掉。
//!
//! 这条流的段序列就是第二条流（`second_transaction_step_zero_layer0.rs`）发 C 之前那一截：前 22 段逐段相同，
//! 末段是暖机第二次的系统配置槽轮换（第二条流里它与 C 的单元写并成 18 写一段）。展开的这两段在第二条流里是第 13、14 段，
//! 全量（门禁 54 号 `--full`）已经罩着；这一条不多罩崩溃状态，多的是在平时的 `cargo test` 里展开它们、按 I-7.7 判——
//! 第二条流的快用例只展开写数小于 10 的段，写行那一段（10 写）与并段之后的 14 写一段都不展开。

mod common;

use common::{
    build_pool, file_content, geometry, parameters, publish_overwrite_in_process,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::mount_writable;
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions,
    writes_and_segments_with_stream_indexes, PublishedVersion,
};
use singlefs_harness::segments::StepKind;

/// B 的内容：与第一个文件不同长、不同字节（同 `second_transaction_step_one_overwrite.rs` 的第二次写）。
const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// I-7.7（系统配置实例代号不低于根环） 在池级 checker 里的名字。
const SYSTEM_CONFIGURATION_INSTANCE_INVARIANT: &str = "I-7.7";

#[test]
fn no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount(
) {
    let mut pool = build_pool("acquisition-barrier-layer0");
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

    let mut versions = vec![
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
            content: second_content(),
        },
    ];
    for version in
        std::iter::once(&mounted.output.row_publish).chain(mounted.output.warm_up_publishes.iter())
    {
        versions.push(PublishedVersion {
            instance: version.root().instance,
            checkpoint_txg: version.root().checkpoint_txg,
            content: second_content(),
        });
    }

    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let first_stream_index_of_the_remount =
        operations_before_the_remount - pool.mkfs_operation_count;
    let remount_writes: Vec<usize> = (0..writes.len())
        .filter(|write_index| stream_indexes[*write_index] >= first_stream_index_of_the_remount)
        .collect();
    let first_remount_write = *remount_writes.first().expect("重开之后发过写");
    assert_eq!(
        remount_writes[..2]
            .iter()
            .map(|write_index| writes[*write_index].kind)
            .collect::<Vec<StepKind>>(),
        vec![StepKind::SystemConfigurationSlot; 2],
        "重开之后头两个写是取号那两次系统配置槽写（两块盘各一次）"
    );
    let first_unit_write_of_the_remount = *remount_writes
        .iter()
        .find(|write_index| writes[**write_index].kind == StepKind::UnitWrite)
        .expect("写行那次发布写单元");
    let segment_of = |write_index: usize| {
        segments
            .iter()
            .position(|segment| segment.contains(&write_index))
            .expect("每个写都在某一段里")
    };
    let acquisition_segment = segment_of(first_remount_write);
    let first_unit_segment = segment_of(first_unit_write_of_the_remount);
    let expand = |segment_index: usize, _segment: &[usize]| {
        segment_index == acquisition_segment || segment_index == first_unit_segment
    };
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽写");
    let tally = enumerate_layer0_selecting_versions(
        &base,
        &writes,
        &segments,
        judged_root_index,
        &versions,
        &expand,
    );
    let expanded: Vec<Vec<usize>> = segments
        .iter()
        .enumerate()
        .filter(|(segment_index, segment)| expand(*segment_index, segment))
        .map(|(_, segment)| segment.clone())
        .collect();
    println!(
        "LAYER0_ACQUISITION_BARRIER states={} expanded_segment_sizes={:?} checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        expanded.iter().map(Vec::len).collect::<Vec<usize>>(),
        tally.checker_counts_by_invariant()
    );

    // 变异「取号之后没有屏障」要红在这一条上：新实例的单元先于新号持久，I-7.7 ① 判红。
    assert_eq!(
        tally
            .checker_violated_states
            .get(SYSTEM_CONFIGURATION_INSTANCE_INVARIANT)
            .copied()
            .unwrap_or(0),
        0,
        "取号那一段与写行的第一段展开之后，出现了新实例的写先于新号持久的状态：{:?}",
        tally
            .checker_first_violation
            .get(SYSTEM_CONFIGURATION_INSTANCE_INVARIANT)
    );
    assert_eq!(
        tally
            .checker_evaluated_states
            .get(SYSTEM_CONFIGURATION_INSTANCE_INVARIANT)
            .copied()
            .unwrap_or(0),
        tally.states,
        "I-7.7 在展开出来的每个状态上都真被评估过（阴性结果与「代码没跑到」分开）：{}",
        tally.checker_counts_by_invariant()
    );
    assert_eq!(
        tally.violations, 0,
        "多版本 oracle 的第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    let checker_violations: u64 = tally.checker_violated_states.values().sum();
    assert_eq!(
        checker_violations, 0,
        "别的不变量也不许判红：{:?}",
        tally.checker_first_violation
    );

    // 状态数钉绝对值：取号那一段是 B 的两次系统配置槽轮换并上取号两次（4 写，15 个真子集），
    // 写行那次发布的单元写一段（实例表 + 四个固定点单元，各两盘：10 写，1023 个），再加全部持久那一个。
    assert_ne!(
        acquisition_segment, first_unit_segment,
        "取号那一段与写行的第一段之间隔着一道屏障"
    );
    assert_eq!(
        expanded.iter().map(Vec::len).collect::<Vec<usize>>(),
        vec![4, 10]
    );
    assert_eq!(tally.states, closed_form_state_count(&expanded));
    assert_eq!(tally.states, 1 + 15 + 1023);
    // 与第二条流发 C 之前那一截逐段相同（门禁 52 号核的那个数组的前 22 段），末段 2 写是暖机第二次的系统配置槽轮换。
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 2],
        "段序列是第二条流发 C 之前那一截"
    );
    assert_eq!(
        (acquisition_segment, first_unit_segment),
        (12, 13),
        "展开的是第二条流的第 13、14 段（从 1 数）"
    );
}
