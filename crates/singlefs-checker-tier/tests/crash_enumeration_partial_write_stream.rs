//! checker 档模块：crash、crash_state_enumeration
//! 部分写的三条崩溃流（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R7「崩溃流三条进 checker 档」）：都接在多记录流
//! 「取号 → 暖机 × 2 → A → B → C」之后（C 是第一个文件三个数据单元、两个半净荷；前缀与多记录流、截断流同一份录法），C 之后：
//! - 中间单元部分写：在单元 1 的偏移 100 写 4 KiB，单元 1、2 一起重写（R4「写到 o 时把 o 之后的单元一起重写」），一次发布两个数据单元、两个事务；
//! - 跨末尾再写进洞里：跨过 size 写 100 字节（单元 2 读回、size 变长，R5）→ 变长到四个半净荷（单元 2 补满，之后是洞，标签 3）
//!   → 在洞里的单元 3 写 100 字节（单元 3 新加、读残留，size 不变、洞还在）；
//! - 快照下部分写：建快照 S → 在单元 0 的偏移 100 写 4 KiB，单元 0、1、2 一起重写，被快照引用的三个旧单元进 deadlist（R7、截断判决 K4）。
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态：每个状态跑恢复 + 多版本 oracle（A、B、C 与 C 之后每一版有文件：崩溃之后读到的只能是
//! 某一版的全旧或全新，一个字节新一个字节旧都不许）、池级 checker（I-1.13 扫描重建与遍历方向同一份索引、I-2.3 补齐为零、I-4.6 单元数对标签对 size、
//! I-9.39 截断字段之外的单元不早于 T，每一条都判）、记录核对器；另逐状态记恢复落到哪一版（只记计数）。
//!
//! **比已有的流多罩了什么**：按偏移的部分写那一次发布（一次发布重写一段单元、各一个事务，o 之后照抄不了；新加单元的残留读；写进洞里换单元数而
//! 标签照旧）第一次被枚举到；快照下部分写换下的数据单元进 deadlist 那一段与截断流的意图不同（这次换下三个、没有截断条目）。
//! 多跑的步：崩溃点重放那一路没有（每个状态两遍恢复、池级 checker、记录核对器，同截断流）。崩溃放量那一路这一份没接（观察者登记表在
//! checker 档 `src/` 里，这一批不改它）。
//!
//! 快档只展开写数小于 10 的段；全量标 ignored，登记成崩溃枚举用例。这一份写成之后实现员没有跑（checker 档归提交时的门禁）：段形、闭式与各步的
//! 计数不钉字面值，打印出来由第一次跑的人钉。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_point_observers;
mod common_crash_points;
#[path = "../../singlefs-harness/tests/common_snapshot_prefix/mod.rs"]
mod common_snapshot_prefix;
#[path = "../../singlefs-harness/tests/common_truncation/mod.rs"]
mod common_truncation;
mod common_truncation_crash_streams;

use common::{build_pool, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use common_crash_points::{content_needing, sequential_write};
use common_truncation_crash_streams::{
    assert_clean, digest_of_the_landed_versions, expansion_of_the_segments_shorter_than_ten,
    observe_the_landed_version, LANDED_BEFORE_THE_STEPS_COUNTER,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_expansion, CrashPointReplayItem, CrashPointReplayObserver,
};
use singlefs_core::address::{CheckpointTxg, FileOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::{
    publish_partial_write, PartialWriteBytes, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes, CrashImage, MemoryPool,
    PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_partial_write_stream.rs";

/// 多记录流建池那一次的实例（`build_pool` 取号取到的）。
const INSTANCE_OF_THE_STREAM: InstanceGeneration = InstanceGeneration(1);

fn payload_in_bytes() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// 三条流。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PartialWriteCrashStream {
    InTheMiddleUnit,
    AcrossTheEndThenIntoTheHole,
    UnderASnapshot,
}

impl PartialWriteCrashStream {
    /// 这条流在来历表里的流名（崩溃点重放合表的项各占一行，三条流不共用一个流名）。
    const fn flow_name(self) -> &'static str {
        match self {
            Self::InTheMiddleUnit => "partial_write_in_the_middle_unit",
            Self::AcrossTheEndThenIntoTheHole => "partial_write_across_the_end_then_into_the_hole",
            Self::UnderASnapshot => "partial_write_under_a_snapshot",
        }
    }
}

/// C 之后的一步（按计划定死）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PartialWriteStreamStep {
    CreateSnapshot,
    Extension { new_size_in_bytes: u64 },
    WriteAtOffset { offset: u64, length_in_bytes: u64 },
}

impl PartialWriteStreamStep {
    /// 落到这一步那一版时加一的计数名。
    fn landed_counter_name(self) -> &'static str {
        match self {
            PartialWriteStreamStep::CreateSnapshot => "landed_on_snapshot_creation",
            PartialWriteStreamStep::Extension { .. } => "landed_on_extension",
            PartialWriteStreamStep::WriteAtOffset { .. } => "landed_on_partial_write",
        }
    }

    /// 这一步写的字节（每一步按偏移算，彼此不同）。
    fn bytes_written(offset: u64, length_in_bytes: u64) -> Vec<u8> {
        (offset..offset + length_in_bytes)
            .map(|position| u8::try_from((position * 13 + 5) % 251).expect("小于 256"))
            .collect()
    }

    /// 接着池的现行那一版发这一步，池接到它那一版上。
    fn publish(self, pool: &mut BuiltPool) -> TransactionOutput {
        match self {
            PartialWriteStreamStep::CreateSnapshot => {
                let created = common_snapshot_prefix::create_snapshot(pool);
                pool.output = created.clone();
                created
            }
            PartialWriteStreamStep::Extension { new_size_in_bytes } => {
                common_truncation::change_the_size_of_the_first_file_and_advance(
                    pool,
                    new_size_in_bytes,
                )
            }
            PartialWriteStreamStep::WriteAtOffset {
                offset,
                length_in_bytes,
            } => {
                let bytes = PartialWriteStreamStep::bytes_written(offset, length_in_bytes);
                let publish_parameters = parameters();
                let previous = pool.output.clone();
                let devices = pool.devices.as_mut().expect("镜像还开着");
                let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
                let published = publish_partial_write(
                    &mut writer,
                    &mut pool.allocator,
                    &previous,
                    InodeNumber(FIRST_INODE_NUMBER),
                    FileOffsetInBytes(offset),
                    PartialWriteBytes::new(&bytes).expect("这几条流写的字节不是空的"),
                    FIXED_WRITE_TIME_SECONDS + 120,
                    INSTANCE_OF_THE_STREAM,
                )
                .expect("部分写发成");
                pool.output = published.clone();
                published
            }
        }
    }

    /// 这一步之后第一个文件读出的内容（前一版是 `content_before`）。
    fn content_after(self, content_before: &[u8]) -> Vec<u8> {
        match self {
            PartialWriteStreamStep::CreateSnapshot => content_before.to_vec(),
            PartialWriteStreamStep::Extension { new_size_in_bytes } => {
                let mut content = content_before.to_vec();
                content.resize(usize::try_from(new_size_in_bytes).expect("长度"), 0);
                content
            }
            PartialWriteStreamStep::WriteAtOffset {
                offset,
                length_in_bytes,
            } => {
                let start = usize::try_from(offset).expect("偏移");
                let end = usize::try_from(offset + length_in_bytes).expect("终点");
                let mut content = content_before.to_vec();
                if content.len() < end {
                    content.resize(end, 0);
                }
                content[start..end].copy_from_slice(&PartialWriteStreamStep::bytes_written(
                    offset,
                    length_in_bytes,
                ));
                content
            }
        }
    }
}

/// 这一条流接在 C 之后的各步。
fn steps_of(stream: PartialWriteCrashStream) -> Vec<PartialWriteStreamStep> {
    let payload = payload_in_bytes();
    match stream {
        PartialWriteCrashStream::InTheMiddleUnit => vec![PartialWriteStreamStep::WriteAtOffset {
            offset: payload + 100,
            length_in_bytes: 4096,
        }],
        PartialWriteCrashStream::AcrossTheEndThenIntoTheHole => vec![
            PartialWriteStreamStep::WriteAtOffset {
                offset: 2 * payload + payload / 2 - 50,
                length_in_bytes: 100,
            },
            PartialWriteStreamStep::Extension {
                new_size_in_bytes: 4 * payload + payload / 2,
            },
            PartialWriteStreamStep::WriteAtOffset {
                offset: 3 * payload + payload / 2,
                length_in_bytes: 100,
            },
        ],
        PartialWriteCrashStream::UnderASnapshot => vec![
            PartialWriteStreamStep::CreateSnapshot,
            PartialWriteStreamStep::WriteAtOffset {
                offset: 100,
                length_in_bytes: 4096,
            },
        ],
    }
}

/// 崩溃点重放那一路要的几样：起点镜像、写表、段、被判的根槽写、版本表、落点表，与录流时每一步的输出。
struct PreparedPartialWriteCrashStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    landed_versions: Vec<((InstanceGeneration, CheckpointTxg), &'static str)>,
    steps: Vec<(PartialWriteStreamStep, TransactionOutput)>,
    /// 录流的池：录制流在它里面，镜像文件随它删。
    pool: BuiltPool,
}

impl PreparedPartialWriteCrashStream {
    fn version_table(&self) -> CrashPointReplayVersionTable<'_> {
        CrashPointReplayVersionTable::with_a_file_only(&self.versions)
    }

    fn segments_shorter_than(&self, writes_below: usize) -> Vec<Vec<usize>> {
        self.segments
            .iter()
            .filter(|segment| segment.len() < writes_below)
            .cloned()
            .collect()
    }
}

/// 录一条流（不录崩溃点）：`build_pool`（取号、暖机两次、A）、B、C，再按计划发 C 之后的各步；切段。
fn prepare(tag: &str, stream: PartialWriteCrashStream) -> PreparedPartialWriteCrashStream {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let second = sequential_write(&mut pool, &second_content);
    let third_content = content_needing(3, 2);
    let third = sequential_write(&mut pool, &third_content);
    assert_eq!(third.data_pointers.len(), 3, "C 之后第一个文件三个数据单元");
    let version = |output: &TransactionOutput, content: Vec<u8>| PublishedVersion {
        instance: output.root.instance,
        checkpoint_txg: output.root.checkpoint_txg,
        content,
    };
    let mut versions = vec![
        version(&first, common::file_content()),
        version(&second, second_content),
        version(&third, third_content.clone()),
    ];
    let mut content = third_content;
    let mut steps = Vec::new();
    for step in steps_of(stream) {
        let output = step.publish(&mut pool);
        content = step.content_after(&content);
        versions.push(version(&output, content.clone()));
        steps.push((step, output));
    }
    let landed_versions = steps
        .iter()
        .map(|(step, output)| {
            (
                (output.root.instance, output.root.checkpoint_txg),
                step.landed_counter_name(),
            )
        })
        .collect();
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("流里有根槽写");
    PreparedPartialWriteCrashStream {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        landed_versions,
        steps,
        pool,
    }
}

/// 录流的形状（判器判完之后再核：改坏实现时先红在判器上）：每一步之后第一个文件的单元数，与各单元的出生 txg 随偏移不降（R4 的前提）；
/// 快照下那一条另核部分写换下的单元进了 deadlist。
fn assert_the_shape_of_the_steps(
    stream: PartialWriteCrashStream,
    prepared: &PreparedPartialWriteCrashStream,
) {
    let expected_data_units: Vec<usize> = match stream {
        PartialWriteCrashStream::InTheMiddleUnit => vec![3],
        // 跨末尾那一写还在单元 2 里；变长不加单元；写进洞里的单元 3 新加（之后到 size 仍是洞）
        PartialWriteCrashStream::AcrossTheEndThenIntoTheHole => vec![3, 3, 4],
        PartialWriteCrashStream::UnderASnapshot => vec![3, 3],
    };
    let found: Vec<usize> = prepared
        .steps
        .iter()
        .map(|(_step, output)| output.data_pointers.len())
        .collect();
    assert_eq!(
        found, expected_data_units,
        "{stream:?} 每一步之后的数据单元数"
    );
    let (_last_step, last_output) = prepared.steps.last().expect("每条流至少一步");
    assert!(
        prepared.pool.output == *last_output,
        "录流的池停在最后一步那一版上"
    );
    for (step, output) in &prepared.steps {
        let births: Vec<u64> = output
            .data_pointers
            .iter()
            .map(|pointer| pointer.head.birth_txg.0)
            .collect();
        assert!(
            births.windows(2).all(|pair| pair[0] <= pair[1]),
            "{step:?}：出生 txg 随偏移不降 {births:?}"
        );
    }
    if stream == PartialWriteCrashStream::UnderASnapshot {
        let (_snapshot_step, after_the_snapshot) = &prepared.steps[0];
        let (_write_step, after_the_write) = &prepared.steps[1];
        assert!(
            after_the_write.deadlist_entries.len() > after_the_snapshot.deadlist_entries.len(),
            "部分写换下的、被快照引用的单元进 deadlist"
        );
    }
}

/// 落点观察者：逐状态记恢复落到哪一版（只记计数）。
fn landed_version_observer<'prepared>(
    prepared: &'prepared PreparedPartialWriteCrashStream,
    observe: &'prepared (dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut CrashPointReplayObserverCounts)
                    + Sync),
) -> CrashPointReplayObserver<'prepared> {
    CrashPointReplayObserver {
        name: "landed_version",
        extra_inputs_digest: digest_of_the_landed_versions(&prepared.landed_versions),
        observe,
    }
}

/// 每个状态都看过；落到每一步那一版的、落到 C 或更早的都有状态，加起来是状态数。
fn assert_every_state_was_observed(
    prepared: &PreparedPartialWriteCrashStream,
    tally: &CrashPointReplayTally,
) {
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    let counts = &tally.observer_counts;
    let mut counters: Vec<&'static str> = vec![LANDED_BEFORE_THE_STEPS_COUNTER];
    for (_version, counter) in &prepared.landed_versions {
        if !counters.contains(counter) {
            counters.push(counter);
        }
    }
    assert_eq!(
        counters
            .iter()
            .map(|counter| counts.count_named(counter))
            .sum::<u64>(),
        tally.states,
        "每个状态都落到流里的某一版、C 或更早：{counts:?}"
    );
    for counter in counters {
        assert!(
            counts.count_named(counter) > 0,
            "{counter} 的状态真跑到了：{counts:?}"
        );
    }
}

/// 按 `expansion` 枚举一条流的崩溃状态，每个状态交落点观察者。
fn enumerate(
    prepared: &PreparedPartialWriteCrashStream,
    item_name: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let landed_versions = prepared.landed_versions.as_slice();
    let observe = |_ordinal: u64,
                   _image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version(landed_versions, report, counts);
    };
    enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: item_name,
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        prepared.version_table(),
        expansion,
        &[landed_version_observer(prepared, &observe)],
    )
    .judged()
}

/// 快档：写数不小于 10 的段不展开，其余每段任意真子集；判器先判，形状与计数在它之后核。
fn run_the_quick_tier(tag: &str, stream: PartialWriteCrashStream, count_line: &str) {
    let prepared = prepare(tag, stream);
    let item_name = format!("{}::quick_tier", stream.flow_name());
    let Some(tally) = enumerate(
        &prepared,
        &item_name,
        &expansion_of_the_segments_shorter_than_ten,
    ) else {
        return;
    };
    println!(
        "{count_line}_FAST states={} segment_sizes={:?} closed_form_of_the_expanded_segments={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        prepared.segments.iter().map(Vec::len).collect::<Vec<_>>(),
        closed_form_state_count(&prepared.segments_shorter_than(10)),
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
    assert_the_shape_of_the_steps(stream, &prepared);
    assert_eq!(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &expansion_of_the_segments_shorter_than_ten,
        ),
        "展开的段按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_every_state_was_observed(&prepared, &tally);
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 记落点；打印的那一行里 `exhaustive=true` 由门禁认。
fn run_the_full_enumeration(tag: &str, stream: PartialWriteCrashStream, count_line: &str) {
    let prepared = prepare(tag, stream);
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let item_name = format!("{}::full", stream.flow_name());
    let Some(tally) = enumerate(&prepared, &item_name, &full_expansion) else {
        return;
    };
    println!(
        "{count_line} states={} closed_form={closed_form} exhaustive={} violations={} two_state_closed_form={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        closed_form_state_count(&prepared.segments),
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_clean(&tally);
    assert_eq!(tally.states, closed_form);
    assert_the_shape_of_the_steps(stream, &prepared);
    assert_every_state_was_observed(&prepared, &tally);
}

/// 中间单元部分写那一条的快档。
#[test]
fn every_crash_state_of_a_partial_write_in_the_middle_unit_reads_one_whole_version_and_stays_clean()
{
    run_the_quick_tier(
        "partial-write-in-the-middle-unit-crash-point-replay-fast",
        PartialWriteCrashStream::InTheMiddleUnit,
        "CRASH_POINT_REPLAY_PARTIAL_WRITE_IN_THE_MIDDLE_UNIT",
    );
}

/// 跨末尾再写进洞里那一条的快档。
#[test]
fn every_crash_state_of_a_partial_write_across_the_end_then_into_the_hole_reads_one_whole_version_and_stays_clean(
) {
    run_the_quick_tier(
        "partial-write-across-the-end-then-into-the-hole-crash-point-replay-fast",
        PartialWriteCrashStream::AcrossTheEndThenIntoTheHole,
        "CRASH_POINT_REPLAY_PARTIAL_WRITE_ACROSS_THE_END_THEN_INTO_THE_HOLE",
    );
}

/// 快照下部分写那一条的快档。
#[test]
fn every_crash_state_of_a_partial_write_under_a_snapshot_reads_one_whole_version_and_stays_clean() {
    run_the_quick_tier(
        "partial-write-under-a-snapshot-crash-point-replay-fast",
        PartialWriteCrashStream::UnderASnapshot,
        "CRASH_POINT_REPLAY_PARTIAL_WRITE_UNDER_A_SNAPSHOT",
    );
}

/// 中间单元部分写那一条的全量。
#[test]
#[ignore = "全量（每段都展开、每个状态两遍恢复 + checker，状态数以用例打出的为准）；门禁 checker-tier-crash-point-replay --full 在 release 下跑"]
fn full_enumeration_of_the_partial_write_in_the_middle_unit_stream_is_exhaustive_and_clean() {
    run_the_full_enumeration(
        "partial-write-in-the-middle-unit-crash-point-replay-full",
        PartialWriteCrashStream::InTheMiddleUnit,
        "CRASH_POINT_REPLAY_PARTIAL_WRITE_IN_THE_MIDDLE_UNIT",
    );
}

/// 跨末尾再写进洞里那一条的全量。
#[test]
#[ignore = "全量（每段都展开、每个状态两遍恢复 + checker，状态数以用例打出的为准）；门禁 checker-tier-crash-point-replay --full 在 release 下跑"]
fn full_enumeration_of_the_partial_write_across_the_end_then_into_the_hole_stream_is_exhaustive_and_clean(
) {
    run_the_full_enumeration(
        "partial-write-across-the-end-then-into-the-hole-crash-point-replay-full",
        PartialWriteCrashStream::AcrossTheEndThenIntoTheHole,
        "CRASH_POINT_REPLAY_PARTIAL_WRITE_ACROSS_THE_END_THEN_INTO_THE_HOLE",
    );
}

/// 快照下部分写那一条的全量。
#[test]
#[ignore = "全量（每段都展开、每个状态两遍恢复 + checker，状态数以用例打出的为准）；门禁 checker-tier-crash-point-replay --full 在 release 下跑"]
fn full_enumeration_of_the_partial_write_under_a_snapshot_stream_is_exhaustive_and_clean() {
    run_the_full_enumeration(
        "partial-write-under-a-snapshot-crash-point-replay-full",
        PartialWriteCrashStream::UnderASnapshot,
        "CRASH_POINT_REPLAY_PARTIAL_WRITE_UNDER_A_SNAPSHOT",
    );
}
