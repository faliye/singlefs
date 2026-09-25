//! 回退见证（D23（journal 的角色与格式） 已定项 14「回退见证」）在层 0 的三条流（判决 `research/prompts/m2-witness-r1-main-verification.md`
//! 第四节「层 0 流」那一行、攻方报告 `m2-witness-r1-opus-output.md` 三.5）：
//!
//! 1. **回退**：A（txg 3）→ B（4）→ C（5），实例 1；进程退出、重开走回退到 (1, 3)：取号 2、写行（txg 6，回退行）、暖机一次（txg 7）。
//!    见证 (2, 1, 3) 随写行那次发布的系统配置轮换写（写序 post），之后暖机的轮换照带。比已有的回退流（`second_transaction_step_zero_layer0.rs`
//!    的 `RollbackToFirstVersion`：回退之前隔着一次普通挂载，中间实例写 (2, 0, 0)）多罩的：回退紧接在被抛弃的那个实例之后（没有中间实例），
//!    写行那次的轮换是这条流上第一次带非空见证表的系统配置写——写到一半的状态（一块盘新、一块盘旧）每个都跑恢复、checker（I-7.10、I-7.11）。
//! 2. **第二次回退（表越过 512）**：流 1 之后覆盖写 D（txg 8，实例 2）、进程退出、再回退到 (2, 7)：取号 3、写行（txg 9）与暖机。
//!    见证表两条 (2, 1, 3)、(3, 2, 7)，第二条落在槽内 [498, 514)，越过 512。多罩的：非空的旧表被整张带着、又加一条的那几次系统配置写，
//!    与 D 这条被第二次回退抛弃的根在环里的每个状态。
//! 3. **跨挂载**：A、B、C 之后回退，挂载崩在写行那次发布的系统配置轮换刚写完（见证两盘都落了、暖机一个字节都没写）；
//!    重开一次可写挂载，回退实例的根（txg 6）一直读不出——择根跳过被见证抛弃的 B、C，落回 R_old（1, 3），取号 3、接在 A 上写行与暖机；
//!    再撤掉读故障重开一次可写挂载（取号 4）。多罩的：带读故障那一次挂载写出的整段流（它的根比回退实例的根新、实例表判回退实例被抛弃），
//!    与撤故障之后回退实例那条根重新读得出的状态——层 0 已有的流都没有「一次挂载在读故障下写出来」的段。
//!
//! 三条都只保证编得过（用户 2026-09-24 定：全量崩溃最后统一跑）：快的那条（不展开 10 写以上的段）与全量那条都在这里，全量标 ignored，
//! 提交时由 crash-verifier 跑。段序列与状态数没有写死：流是新的，登记表里还没有它们的数，第一次全量跑出来再登记。

mod common;

use common::{
    build_pool, file_content, geometry, parameters, publish_overwrite_in_process, BuiltPool,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{mount_rollback, mount_writable, Mounted, RollbackTarget, ShadowLedger};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{PoolVersion, TransactionOutput};
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions, enumerate_layer0_versions,
    writes_and_segments, Layer0Tally, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, NamedRootRingSlots, RootRingSlotTarget,
    SharedFaultPlan,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedOperationKind, RetainedOperation, SharedStream};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    /// 被判的那次根槽 FUA 写：流里最后一次发布的根。
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn content_of(seed: usize) -> Vec<u8> {
    (0..2600 + seed)
        .map(|index| u8::try_from((index * 7 + seed * 11) % 239).expect("小于 256"))
        .collect()
}

fn version(instance: u32, txg: u64, content: &[u8]) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(txg),
        content: content.to_vec(),
    }
}

/// 一次挂载写出的每一版（写行那次与暖机那几次），内容都是 `content`。
fn versions_of_the_mount(mounted: &Mounted, content: &[u8]) -> Vec<PublishedVersion> {
    std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|published: &PoolVersion| PublishedVersion {
            instance: published.root().instance,
            checkpoint_txg: published.root().checkpoint_txg,
            content: content.to_vec(),
        })
        .collect()
}

/// A、B、C（实例 1），交回池、C 那一版与三版的内容。
fn pool_through_the_third_version(
    tag: &str,
) -> (BuiltPool, TransactionOutput, Vec<PublishedVersion>) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content = content_of(1);
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &second_content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    let third_content = content_of(2);
    let third = publish_overwrite_in_process(
        &mut pool,
        &second,
        &third_content,
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("C");
    pool.output = third.clone();
    let versions = vec![
        version(1, 3, &file_content()),
        version(1, 4, &second_content),
        version(1, 5, &third_content),
    ];
    (pool, third, versions)
}

fn rollback(pool: &mut BuiltPool, target: RollbackTarget) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let rolled_back =
        mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On).expect("回退");
    pool.devices = Some(devices);
    rolled_back
}

/// mkfs 之后的整条流切成写表与段，被判的根取最后一次根槽写。
fn prepared_from(
    base: MemoryPool,
    operations_after_mkfs: &[RetainedOperation],
    versions: Vec<PublishedVersion>,
) -> Prepared {
    let (writes, segments) = writes_and_segments(operations_after_mkfs, &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("流里至少一次根槽写");
    Prepared {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

/// 流 1：回退。
fn prepare_rollback(tag: &str) -> Prepared {
    let (mut pool, _third, mut versions) = pool_through_the_third_version(tag);
    let rolled_back = rollback(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    assert_eq!(
        rolled_back.output.rollback_witness_written.entries().len(),
        1,
        "见证表一条 (2, 1, 3)"
    );
    versions.extend(versions_of_the_mount(&rolled_back, &file_content()));
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    prepared_from(base, &operations[pool.mkfs_operation_count..], versions)
}

/// 流 2：第二次回退（见证表两条、越过 512）。
fn prepare_second_rollback(tag: &str) -> Prepared {
    let (mut pool, _third, mut versions) = pool_through_the_third_version(tag);
    let first_rollback = rollback(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    versions.extend(versions_of_the_mount(&first_rollback, &file_content()));
    let target_of_the_second_rollback = first_rollback
        .output
        .warm_up_publishes
        .last()
        .map_or(*first_rollback.output.row_publish.root(), |published| {
            *published.root()
        });
    pool.allocator = first_rollback.allocator;
    let current = first_rollback
        .current
        .into_file_version()
        .expect("回退到 A，现行那一版带文件");
    let fourth_content = content_of(3);
    let fourth = publish_overwrite_in_process(
        &mut pool,
        &current,
        &fourth_content,
        FIXED_WRITE_TIME_SECONDS + 180,
        first_rollback.output.instance,
    )
    .expect("D");
    versions.push(PublishedVersion {
        instance: fourth.root.instance,
        checkpoint_txg: fourth.root.checkpoint_txg,
        content: fourth_content,
    });
    let second_rollback = rollback(
        &mut pool,
        RollbackTarget {
            instance: target_of_the_second_rollback.instance,
            checkpoint_txg: target_of_the_second_rollback.checkpoint_txg,
        },
    );
    assert_eq!(
        second_rollback
            .output
            .rollback_witness_written
            .entries()
            .len(),
        2,
        "见证表两条：第一次回退那一条照留（环里还住着实例 1 的根），加第二次这一条"
    );
    let second_entry_end = 481 + 1 + 2 * 16;
    assert!(
        second_entry_end > 512
            && u64::try_from(second_entry_end).expect("字节数") <= SYSTEM_CONFIGURATION_SLOT_BYTES,
        "第二条越过 512"
    );
    versions.extend(versions_of_the_mount(&second_rollback, &file_content()));
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    prepared_from(base, &operations[pool.mkfs_operation_count..], versions)
}

/// 流 3：回退挂载崩在半路 → 带读故障的可写挂载 → 撤故障的挂载。
fn prepare_cross_mount(tag: &str) -> Prepared {
    let (mut pool, _third, mut versions) = pool_through_the_third_version(tag);
    let operations_before_the_rollback = pool.retained_operations().len();
    let rolled_back = rollback(
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    let row_publish_root = *rolled_back.output.row_publish.root();
    versions.push(PublishedVersion {
        instance: row_publish_root.instance,
        checkpoint_txg: row_publish_root.checkpoint_txg,
        content: file_content(),
    });
    // 崩在写行那次发布的系统配置轮换刚写完：回退挂载里第一次根槽 FUA 写之后的两次系统配置槽写是那次轮换（两盘各一次），
    // 截到第二次为止；暖机一个字节都没写。
    let operations = pool.retained_operations();
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let row_root_write = operations_before_the_rollback
        + operations[operations_before_the_rollback..]
            .iter()
            .position(|retained| {
                retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess
            })
            .expect("写行那次发布写过根槽");
    let system_configuration_writes_after_the_row_root: Vec<usize> = operations[row_root_write..]
        .iter()
        .enumerate()
        .filter(|(_, retained)| {
            retained.operation.kind == RecordedOperationKind::Write
                && (retained.operation.offset.0 == 0 || retained.operation.offset.0 == spacing)
                && retained.operation.length == SYSTEM_CONFIGURATION_SLOT_BYTES
        })
        .map(|(position, _)| row_root_write + position)
        .take(2)
        .collect();
    let crash_after = *system_configuration_writes_after_the_row_root
        .last()
        .expect("写行那次发布的轮换两盘各写一次");
    let mut crashed_image = MemoryPool::with_devices(&DISKS, IMAGE_BYTES);
    crashed_image.apply(&operations[..=crash_after]);

    // 重开可写挂载，回退实例写行那次的根（txg 6）一直读不出。
    let stream = SharedStream::retaining_contents();
    let devices = common::crash_state_devices(&crashed_image, &[], &[], &stream);
    let unreadable = RootRingSlotTarget {
        named_slots: NamedRootRingSlots::naming(&[target_for_publish(
            row_publish_root.checkpoint_txg,
            parameters().geometry.root_ring_slots_per_region,
        )]),
        region_devices: parameters().region_devices,
        fixed_structure_slot_spacing: parameters().geometry.fixed_structure_slot_spacing,
    };
    let plan = SharedFaultPlan::armed(
        geometry(),
        FaultSchedule::every_read_of_named_root_ring_slots_fails(unreadable),
    );
    let mut failing_devices: Vec<_> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    let mounted_under_faults =
        mount_writable(&parameters(), &mut failing_devices).expect("带读故障的可写挂载");
    assert_eq!(
        (
            mounted_under_faults.output.effective_root.instance,
            mounted_under_faults.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(3)),
        "择根跳过被见证抛弃的 B、C，落回 R_old"
    );
    versions.extend(versions_of_the_mount(
        &mounted_under_faults,
        &file_content(),
    ));

    // 撤掉读故障，再重开可写挂载。
    let mut devices_without_faults: Vec<_> = failing_devices
        .into_iter()
        .map(|(identity, device)| (identity, device.into_inner()))
        .collect();
    let mounted_without_faults =
        mount_writable(&parameters(), &mut devices_without_faults).expect("撤故障之后的可写挂载");
    versions.extend(versions_of_the_mount(
        &mounted_without_faults,
        &file_content(),
    ));

    let base = pool.memory_pool_after_mkfs();
    let mut operations_after_mkfs: Vec<RetainedOperation> =
        operations[pool.mkfs_operation_count..=crash_after].to_vec();
    operations_after_mkfs.extend(stream.retained_operations());
    prepared_from(base, &operations_after_mkfs, versions)
}

fn assert_clean(tally: &Layer0Tally) {
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0);
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例的状态数必须是 0：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    for must_evaluate in ["I-7.10", "I-7.11"] {
        assert!(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0)
                > 0,
            "{must_evaluate} 至少在一个状态上真被评估过：{}",
            tally.checker_counts_by_invariant()
        );
    }
}

/// 快的那一份：10 写以上的段不展开（只以整段持久进入后面的状态），其余每段任意子集。
fn fast_enumeration(prepared: &Prepared) -> Layer0Tally {
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
    let tally = enumerate_layer0_selecting_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .filter(|segment| segment.len() < 10)
        .cloned()
        .collect();
    assert_eq!(
        tally.states,
        closed_form_state_count(&expanded),
        "展开的段按闭式数"
    );
    tally
}

fn full_enumeration(prepared: &Prepared) -> Layer0Tally {
    let closed_form = closed_form_state_count(&prepared.segments);
    let tally = enumerate_layer0_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
    );
    println!(
        "LAYER0-WITNESS states={} closed_form={closed_form} exhaustive={} violations={} failed={} {} states_by_publish=[{}] first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.failed_states,
        tally.checker_counts_by_invariant(),
        tally.states_by_publish_text(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
    tally
}

#[test]
fn every_crash_state_of_the_rollback_stream_outside_the_unit_segments_is_clean() {
    assert_clean(&fast_enumeration(&prepare_rollback(
        "witness-layer0-rollback-fast",
    )));
}

#[test]
#[ignore = "全量：回退流的全部崩溃状态、每个两遍恢复 + checker；提交时由 crash-verifier 在 release 下跑"]
fn full_enumeration_of_the_rollback_stream_is_exhaustive_and_clean() {
    assert_clean(&full_enumeration(&prepare_rollback(
        "witness-layer0-rollback-full",
    )));
}

#[test]
fn every_crash_state_of_the_second_rollback_stream_outside_the_unit_segments_is_clean() {
    assert_clean(&fast_enumeration(&prepare_second_rollback(
        "witness-layer0-second-rollback-fast",
    )));
}

#[test]
#[ignore = "全量：第二次回退流的全部崩溃状态、每个两遍恢复 + checker；提交时由 crash-verifier 在 release 下跑"]
fn full_enumeration_of_the_second_rollback_stream_is_exhaustive_and_clean() {
    assert_clean(&full_enumeration(&prepare_second_rollback(
        "witness-layer0-second-rollback-full",
    )));
}

#[test]
fn every_crash_state_of_the_cross_mount_stream_outside_the_unit_segments_is_clean() {
    assert_clean(&fast_enumeration(&prepare_cross_mount(
        "witness-layer0-cross-mount-fast",
    )));
}

#[test]
#[ignore = "全量：跨挂载流（回退崩在半路 → 带读故障的挂载 → 撤故障的挂载）的全部崩溃状态；提交时由 crash-verifier 在 release 下跑"]
fn full_enumeration_of_the_cross_mount_stream_is_exhaustive_and_clean() {
    assert_clean(&full_enumeration(&prepare_cross_mount(
        "witness-layer0-cross-mount-full",
    )));
}
