//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 9 行：照今天的做法写成条款的三格，各一条会红的用例
//! （三方判决 `research/prompts/m2-presumed-clauses-r1-main-verification.md` 第七节；P6 归并行线一，不在这里）。
//!
//! - C497（实例表在 bump 次序里排第几没有条款）：D3（空间分配） 已定项 10 ⑤「一次发布重写实例表单元时（写行、暖机、回退那几次），
//!   实例表单元最前」——写行那次发布（带文件的一版上、树表 0 条的一版上）里，实例表的落点在这次全部提交内生块之前，
//!   树 0 里的出生序号也排在它们之前。管理员回退改成挂着时的一次向前发布之后，回退那次发布照带现行那一版的实例表、不重写它
//!   （D23（journal 的角色与格式） 已定项 14），那一格没有对象。
//! - C498（后续挂载与崩了再挂的暖机次数没有条款）：D16（发布语义） 已定项 8「后续挂载、回退之后与挂载中途崩了再挂」——
//!   写行 txg 取 max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1，之后每次空发布加一，
//!   按根环落点看落哪块盘，推到本实例的根覆盖每块盘为止，至多 R 次（写行 txg ≡ 2 (mod 3) 时白推一次，认下）。
//! - C499（干净关闭标记有没有没有条款）：D23（journal 的角色与格式） 已定项 14「第一版没有干净关闭标记」——
//!   干净关闭再挂载与崩溃后再挂载走同一条恢复路径（扫环、逐条验证、择根、按前缀口径施加）。扫环读哪些槽看盘答不答 journal 提示，
//!   两种盘各一条：不答的（像真盘）两条历史都逐槽扫全环，答的（只记写过的扇区的内存盘）两条历史都恰好读写过的记录槽。

mod common;

use std::collections::BTreeSet;

use common::{
    build_pool, crash_state_devices, file_content, format_pool, geometry, parameters,
    publish_overwrite_in_process, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, Mounted, RollbackTarget,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput, TransactionUnit,
    TREE_IDENTIFIER_NONE,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::read_tally::{
    BlockDeviceReadLog, JournalHintAnswer, JournalRingRegion, ReadLoggingBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// C497：实例表单元在 bump 次序里最前
// ---------------------------------------------------------------------------------------------

/// 带文件的一版上一次重写实例表的发布：实例表的落点低于这次每一个别的提交内生块的落点（bump 从开放段里单调往上取），
/// 树 0 里实例表的出生序号低于树表单元的。`occasion` 只进断言消息。
fn assert_the_instance_table_is_bumped_first_in_a_file_version_publish(
    publish: &TransactionOutput,
    occasion: &str,
) {
    assert!(
        publish.rewritten.contains(&TransactionUnit::InstanceTable),
        "{occasion}：这次重写实例表：{:?}",
        publish.rewritten
    );
    let instance_table_slot = publish.unit(TransactionUnit::InstanceTable).slot;
    let later_commit_generated_slots: Vec<(TransactionUnit, SlotNumber)> = publish
        .rewritten
        .iter()
        .filter(|identity| {
            **identity != TransactionUnit::InstanceTable
                && !matches!(identity, TransactionUnit::Data(_))
        })
        .map(|identity| (*identity, publish.unit(*identity).slot))
        .collect();
    assert!(
        !later_commit_generated_slots.is_empty(),
        "{occasion}：这次发布还重写了别的提交内生块"
    );
    for (identity, slot) in &later_commit_generated_slots {
        assert!(
            instance_table_slot < *slot,
            "{occasion}：实例表（槽 {}）要排在 {identity:?}（槽 {}）之前 bump",
            instance_table_slot.0,
            slot.0
        );
    }
    let instance_table = publish.root.instance_table;
    let tree_table = publish.root.tree_table;
    assert_eq!(
        (
            instance_table.head.birth_tree.0,
            tree_table.head.birth_tree.0
        ),
        (TREE_IDENTIFIER_NONE, TREE_IDENTIFIER_NONE),
        "{occasion}：实例表与树表单元都归树 0"
    );
    assert!(
        instance_table.birth_sequence.0 < tree_table.birth_sequence.0,
        "{occasion}：树 0 里实例表先发出生序号（{}），树表单元在后（{}）",
        instance_table.birth_sequence.0,
        tree_table.birth_sequence.0
    );
}

/// C497：D3（空间分配） 已定项 10 ⑤ 的「实例表单元最前」，两个重写实例表的场合各核一遍——
/// ① 带文件的一版上写行（第一个文件之后重开可写挂载，写行 txg 4）；② 树表 0 条的一版上写行（只做过 mkfs 的池第二次可写挂载，
/// 这次只写实例表与一片分配记录节点）。旧形态的 ③（挂载时回退那次写回退行的发布）随回退改成向前发布删掉：向前回退不重写实例表。
/// 判别力自证：把实例表挪到树表单元之后取落点（带文件的一版那条路）、把两个落点的先后对调（树表 0 条那条路），各自红（`crates/mutations.tsv`）。
#[test]
fn c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block(
) {
    let mut pool = build_pool("c497-instance-table-first");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    pool.devices = Some(devices);
    let row_publish = mounted
        .output
        .row_publish
        .file_version()
        .expect("第一个文件之后重开：写行那次发布是带文件的一版");
    assert_the_instance_table_is_bumped_first_in_a_file_version_publish(
        row_publish,
        "带文件的一版上写行",
    );

    let mut formatted = format_pool("c497-instance-table-first-without-file");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let row_written = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    formatted.devices = Some(second);
    let PoolVersion::WithoutFile(row_publish_without_file) = &row_written.output.row_publish else {
        panic!("树表 0 条的一版上写行，写出来的仍是没有文件版本的一版")
    };
    let instance_table = row_publish_without_file.root.instance_table;
    let allocation_record_node = row_publish_without_file.root.allocation_record_tree_root;
    assert!(
        instance_table.locations[0].slot < allocation_record_node.locations[0].slot,
        "树表 0 条的一版上写行：实例表（槽 {}）排在分配记录节点（槽 {}）之前 bump",
        instance_table.locations[0].slot.0,
        allocation_record_node.locations[0].slot.0
    );
    assert!(
        instance_table.birth_sequence.0 < allocation_record_node.birth_sequence.0,
        "树表 0 条的一版上写行：两片都归树 0，实例表先发出生序号"
    );
}

// ---------------------------------------------------------------------------------------------
// C498：后续挂载、回退之后、挂载中途崩了再挂的暖机次数现算
// ---------------------------------------------------------------------------------------------

/// 条款给的暖机 txg（D16（发布语义） 已定项 8）：从写行 txg 之后逐个加一，按根环落点看落哪块盘——区域 = txg mod 3
/// （D22（单元原子性怎么合成） 已定项 2），区域归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7）——推到本实例的根覆盖每块盘为止，
/// 至多 R = 3 次。在用例里照条款另算一遍，不调实现的 `warm_up_publish_txgs`。
fn warm_up_txgs_by_the_clause(row_publish_txg: CheckpointTxg) -> Vec<CheckpointTxg> {
    let root_ring_regions = 3u64;
    let device_of = |txg: u64| {
        parameters().region_devices[usize::try_from(txg % root_ring_regions).expect("区域号")]
    };
    let mut covered: BTreeSet<DeviceIdentity> = BTreeSet::from([device_of(row_publish_txg.0)]);
    let mut warm_up_txgs = Vec::new();
    let mut latest = row_publish_txg.0;
    while covered.len() < BOTH_DEVICES.len()
        && u64::try_from(warm_up_txgs.len()).expect("次数") < root_ring_regions
    {
        latest += 1;
        covered.insert(device_of(latest));
        warm_up_txgs.push(CheckpointTxg(latest));
    }
    warm_up_txgs
}

/// 一次挂载（可写或回退）实际推的：写行 txg 与暖机 txg。
fn row_and_warm_up_txgs(mounted: &Mounted) -> (CheckpointTxg, Vec<CheckpointTxg>) {
    (
        mounted.output.row_publish.root().checkpoint_txg,
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| publish.root().checkpoint_txg)
            .collect(),
    )
}

/// 这次挂载的写行 txg 是条款给的那个下界、暖机 txg 与条款逐个相等，暖机次数是 `expected_warm_up_count`（手算的那一格，与模型互相核）。
fn assert_warm_up_follows_the_clause(
    mounted: &Mounted,
    expected_row_publish_txg: CheckpointTxg,
    expected_warm_up_count: usize,
    history: &str,
) {
    let (row_publish_txg, warm_up_txgs) = row_and_warm_up_txgs(mounted);
    assert_eq!(
        row_publish_txg, expected_row_publish_txg,
        "{history}：写行 txg = max(根环, 环里记录) + 1"
    );
    assert_eq!(
        warm_up_txgs,
        warm_up_txgs_by_the_clause(row_publish_txg),
        "{history}：暖机 txg 按条款现算"
    );
    assert_eq!(
        warm_up_txgs.len(),
        expected_warm_up_count,
        "{history}：写行 txg {} ≡ {} (mod 3)",
        row_publish_txg.0,
        row_publish_txg.0 % 3
    );
}

fn reopen_and_mount(pool: &mut BuiltPool) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    mounted
}

/// C498：D16（发布语义） 已定项 8 的「后续挂载、回退之后与挂载中途崩了再挂：次数现算」，一条历史串起来——
/// 第一个文件（txg 3）之后三次关闭再挂载（写行 txg 4、6、8：暖机 1、1、2 次，txg 8 ≡ 2 那次白推一次）→ 同一个会话里发一版数据（txg 11）→
/// 挂着的时候回退到 (1, 3)（一次向前发布 txg 12，不写行、不暖机）→ 回退之后关闭再挂载（写行 txg 13：暖机 1 次）→ 再挂载一次、
/// 崩在第一次暖机的根槽写上（写行 txg 15 已落盘，暖机 txg 16 的记录落盘、根没落）→ 再挂载（施加 txg 16 那条记录，写行 txg 17：
/// 暖机 2 次，txg 17 ≡ 2 那次白推一次）。每一次的写行 txg 与暖机 txg 与条款现算的逐个相等。
/// 旧形态（挂载时回退：回退那次挂载写行 txg 12、暖机 1 次）下回退之后的几次挂载 txg 不同，见实三报告的钉死值表。
/// 判别力自证：把暖机次数改成恒按 R 次推，第一次挂载就红（`crates/mutations.tsv`）。
#[test]
fn c498_warm_up_publishes_after_remounts_rollback_and_a_crash_in_the_middle_of_a_mount_are_counted_by_the_clause(
) {
    let mut pool = build_pool("c498-warm-up-count");
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(3));

    let first_remount = reopen_and_mount(&mut pool);
    assert_warm_up_follows_the_clause(&first_remount, CheckpointTxg(4), 1, "第一次关闭再挂载");
    let second_remount = reopen_and_mount(&mut pool);
    assert_warm_up_follows_the_clause(&second_remount, CheckpointTxg(6), 1, "第二次关闭再挂载");
    let mut third_remount = reopen_and_mount(&mut pool);
    assert_warm_up_follows_the_clause(
        &third_remount,
        CheckpointTxg(8),
        2,
        "第三次关闭再挂载（txg 9 落在已覆盖的盘 0 上，白推一次）",
    );

    // 同一个会话里发一版数据（txg 11），再在同一个会话里回退。
    let current = third_remount
        .current
        .file_version()
        .expect("带文件的一版")
        .clone();
    let data_version = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut third_remount.allocator,
            &current,
            FirstFile {
                content: &content_of(2900, 29),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 300,
            },
            third_remount.output.instance,
        )
        .expect("发一版数据")
    };
    assert_eq!(data_version.root.checkpoint_txg, CheckpointTxg(11));
    let mut current_after_the_data_version = data_version;
    {
        let rollback_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &rollback_parameters,
            devices,
            &mut third_remount.allocator,
            &mut current_after_the_data_version,
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
            },
        )
        .expect("挂着的时候回退到 (1, 3)");
    }
    assert_eq!(
        current_after_the_data_version.root.checkpoint_txg,
        CheckpointTxg(12),
        "回退是一次向前发布：txg 12，不写行、不暖机"
    );
    let after_rollback = reopen_and_mount(&mut pool);
    assert_warm_up_follows_the_clause(&after_rollback, CheckpointTxg(13), 1, "回退之后关闭再挂载");

    // 挂载中途崩了：写行 txg 15 整次落盘，第一次暖机（txg 16）的根槽写失败——那次的记录已经落盘（记录之后那道屏障已完成），
    // 根槽与系统配置槽都没有。挂载报错、进程退出。
    let crashed_warm_up_txg = CheckpointTxg(16);
    let crashed_root_target = target_for_publish(
        crashed_warm_up_txg,
        parameters().geometry.root_ring_slots_per_region,
    );
    let crashed_root_device =
        parameters().region_devices[usize::try_from(crashed_root_target.region).expect("区域号")];
    let plan = SharedFaultPlan::armed(
        geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::OnlyDevice(crashed_root_device),
            placement: FaultPlacement::OffsetExactly(slot_offset(
                crashed_root_target,
                parameters().geometry.fixed_structure_slot_spacing,
            )),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
        },
    );
    let mut failing_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Recorded>)> = pool
        .reopen_recorded()
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    let crashed = mount_writable(&parameters(), &mut failing_devices);
    assert!(crashed.is_err(), "第一次暖机的根槽写失败，这次挂载没做成");
    assert_eq!(plan.fired_count(), 1, "注入只打在 txg 16 那个根槽写上");
    drop(failing_devices);
    let after_the_crash = reopen_and_mount(&mut pool);
    assert_eq!(
        (
            after_the_crash.output.chosen_root.checkpoint_txg,
            after_the_crash.output.effective_root.checkpoint_txg,
            after_the_crash.output.journal.prefix_applied
        ),
        (CheckpointTxg(15), CheckpointTxg(16), 1),
        "挂载中途崩了再挂：所选根是写行 txg 15，txg 16 那条暖机记录被施加"
    );
    assert_warm_up_follows_the_clause(
        &after_the_crash,
        CheckpointTxg(17),
        2,
        "挂载中途崩了再挂（txg 18 落在已覆盖的盘 0 上，白推一次）",
    );
}

// ---------------------------------------------------------------------------------------------
// C499：第一版没有干净关闭标记
// ---------------------------------------------------------------------------------------------

/// 两条历史共用的那种盘：只记写过的扇区的内存盘，外面包录制器（`crash_state_devices` 交回的）。
type MemoryDisk = RecordingBlockDevice<SparseBlockDevice>;

/// 两条历史的盘面与各自写过的记录槽（盘号、槽起点）。两条都从 A（txg 3）之后发布 B（txg 4）起：
/// - 干净关闭：B 整次落盘，进程退出；
/// - 崩溃：B 的单元与记录落盘、根槽与系统配置槽没落就崩。
struct CleanCloseAndCrashHistories {
    clean_close_devices: Vec<(DeviceIdentity, MemoryDisk)>,
    crash_devices: Vec<(DeviceIdentity, MemoryDisk)>,
    clean_close_written_record_slots: BTreeSet<(DeviceIdentity, DeviceOffsetInBytes)>,
    crash_written_record_slots: BTreeSet<(DeviceIdentity, DeviceOffsetInBytes)>,
    second_version_content: Vec<u8>,
}

/// 这几条写里 journal 记录那几条的落点（盘号、槽起点）；记录写都是一整条记录宽、落在环里。
fn record_slots_written_by<'write>(
    writes: impl Iterator<Item = &'write RetainedWrite>,
) -> BTreeSet<(DeviceIdentity, DeviceOffsetInBytes)> {
    let ring = journal_ring();
    writes
        .filter(|write| write.kind == StepKind::JournalRecord)
        .map(|write| {
            assert!(
                write.bytes().is_some() && write.length_in_bytes() == JOURNAL_RECORD_BYTES,
                "journal 记录写是一整条记录宽的普通写：{:?} 偏移 {}",
                write.device,
                write.offset.0
            );
            assert!(
                ring.contains(write.offset)
                    && (write.offset.0 - ring.start.0).is_multiple_of(JOURNAL_RECORD_BYTES),
                "journal 记录写落在环里的记录槽起点上：{:?} 偏移 {}",
                write.device,
                write.offset.0
            );
            (write.device, write.offset)
        })
        .collect()
}

/// 第一版的 journal 环：标准起点、默认长度（`parameters()` 的几何）。
fn journal_ring() -> JournalRingRegion {
    assert_eq!(
        parameters().geometry.journal_ring_bytes,
        JOURNAL_RING_DEFAULT_BYTES,
        "这个文件的池用默认环长"
    );
    JournalRingRegion::starting_at_the_standard_slot(JOURNAL_RING_DEFAULT_BYTES)
}

/// 建池、发布 B，造出两条历史的内存盘。
fn build_the_clean_close_and_the_crash_histories() -> CleanCloseAndCrashHistories {
    let mut pool = build_pool("c499-no-clean-close-marker");
    let operations_before_the_second_version = pool.stream.operation_count();
    let first_version = pool.output.clone();
    let second_version_content = content_of(4100, 3);
    let second_version = publish_overwrite_in_process(
        &mut pool,
        &first_version,
        &second_version_content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("发布 B");
    assert_eq!(
        (
            second_version.root.checkpoint_txg,
            second_version.record.counter
        ),
        (CheckpointTxg(4), 4)
    );

    let operations = pool.retained_operations();
    let mut base = MemoryPool::with_devices(&BOTH_DEVICES, IMAGE_BYTES);
    base.apply(&operations[..operations_before_the_second_version]);
    let (writes_before_the_second_version, _segments_before_the_second_version) =
        writes_and_segments(
            &operations[..operations_before_the_second_version],
            &geometry(),
        );
    let (writes, _segments) = writes_and_segments(
        &operations[operations_before_the_second_version..],
        &geometry(),
    );
    let first_root_write = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("B 有根槽写");
    assert!(
        writes[..first_root_write]
            .iter()
            .any(|write| write.kind == StepKind::JournalRecord),
        "根槽写之前 B 的记录已经写了"
    );
    let every_write_persisted = vec![true; writes.len()];
    // 崩溃那条历史的盘面：B 的写里根槽写之前的全部持久（单元与记录），根槽与系统配置槽一个都没有。
    let persisted_before_the_first_root_write: Vec<bool> = (0..writes.len())
        .map(|write_index| write_index < first_root_write)
        .collect();
    let record_slots_before_the_second_version =
        record_slots_written_by(writes_before_the_second_version.iter());
    let mut clean_close_written_record_slots = record_slots_before_the_second_version.clone();
    clean_close_written_record_slots.extend(record_slots_written_by(writes.iter()));
    let mut crash_written_record_slots = record_slots_before_the_second_version;
    crash_written_record_slots.extend(record_slots_written_by(writes[..first_root_write].iter()));
    CleanCloseAndCrashHistories {
        clean_close_devices: crash_state_devices(
            &base,
            &writes,
            &every_write_persisted,
            &SharedStream::retaining_contents(),
        ),
        crash_devices: crash_state_devices(
            &base,
            &writes,
            &persisted_before_the_first_root_write,
            &SharedStream::retaining_contents(),
        ),
        clean_close_written_record_slots,
        crash_written_record_slots,
        second_version_content,
    }
}

/// 一次可写挂载写出的东西、挂载完那一刻每块盘上的读（冷恢复的读不在里面），与挂载完仍开着的设备（冷恢复接着读它们）。
struct MountWithReadLogs<Device: BlockDevice> {
    mounted: Mounted,
    read_logs: Vec<(DeviceIdentity, BlockDeviceReadLog)>,
    devices: Vec<(DeviceIdentity, ReadLoggingBlockDevice<Device>)>,
}

impl<Device: BlockDevice> MountWithReadLogs<Device> {
    /// 挂载里起点落在 journal 环里的读读到的（盘号、偏移），去重。
    fn journal_ring_read_slots(&self) -> BTreeSet<(DeviceIdentity, DeviceOffsetInBytes)> {
        self.read_logs
            .iter()
            .flat_map(|(device, read_log)| {
                read_log
                    .journal_ring_read_offsets
                    .iter()
                    .map(|offset| (*device, *offset))
            })
            .collect()
    }

    /// 挂载里起点落在 journal 环里的读一共几次（同一槽读几次算几次）。
    fn journal_ring_reads(&self) -> usize {
        self.read_logs
            .iter()
            .map(|(_, read_log)| read_log.journal_ring_read_offsets.len())
            .sum()
    }
}

/// 一次可写挂载，每块盘外面包一层记读的装置，journal 提示照 `journal_hint_answer` 答。
fn mount_logging_reads<Device: BlockDevice>(
    devices: Vec<(DeviceIdentity, Device)>,
    journal_hint_answer: JournalHintAnswer,
) -> MountWithReadLogs<Device> {
    let mut logged: Vec<(DeviceIdentity, ReadLoggingBlockDevice<Device>)> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                ReadLoggingBlockDevice::new(device, journal_ring(), journal_hint_answer),
            )
        })
        .collect();
    let mounted = mount_writable(&parameters(), &mut logged).expect("可写挂载");
    let read_logs = logged
        .iter()
        .map(|(identity, device)| (*identity, device.read_log()))
        .collect();
    MountWithReadLogs {
        mounted,
        read_logs,
        devices: logged,
    }
}

/// 挂载之后冷恢复读回的内容与施加之后的根。
fn cold_read_back<Device: BlockDevice>(
    mount: &MountWithReadLogs<Device>,
) -> (RecoveryOutcome, Option<(InstanceGeneration, CheckpointTxg)>) {
    let report = recover(&mount.devices, JournalPolicy::Consult);
    (report.outcome, report.effective_root)
}

/// C499 的后一半，两条用例共用：两条历史照同一套规则恢复——都看到环里四条记录，干净关闭那次择到 B 的根、一条都不用施加，
/// 崩溃那次择到 A 的根、验过点名单元施加 B 那一条，施加之后的根都是 (1, 4)；之后写行 txg 5、jsn 5、暖机 txg 6、7 逐项相同，
/// 冷恢复都读回 B 的内容。`case` 只进断言消息。
fn assert_the_two_histories_recover_by_the_same_rules<Device: BlockDevice>(
    clean_close: &MountWithReadLogs<Device>,
    crash: &MountWithReadLogs<Device>,
    second_version_content: &[u8],
    case: &str,
) {
    for (history, mounted) in [
        ("干净关闭再挂载", &clean_close.mounted),
        ("崩溃后再挂载", &crash.mounted),
    ] {
        assert_eq!(
            mounted.output.journal.valid_records, 4,
            "{case}，{history}：恢复扫环看到 jsn 1–4 四条"
        );
        assert_eq!(
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg
            ),
            (InstanceGeneration(1), CheckpointTxg(4)),
            "{case}，{history}：施加之后的根是 B 那一版"
        );
        assert_eq!(
            row_and_warm_up_txgs(mounted),
            (CheckpointTxg(5), vec![CheckpointTxg(6), CheckpointTxg(7)]),
            "{case}，{history}：写行 txg 5、暖机 txg 6、7"
        );
        assert_eq!(
            mounted.output.row_publish.record().counter,
            5,
            "{case}，{history}：写行 jsn 5"
        );
    }
    assert_eq!(
        (
            clean_close.mounted.output.chosen_root.checkpoint_txg,
            clean_close.mounted.output.journal.above_water,
            clean_close.mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(4), 0, 0),
        "{case}，干净关闭：择到 B 的根，水位之上没有记录"
    );
    assert_eq!(
        (
            crash.mounted.output.chosen_root.checkpoint_txg,
            crash.mounted.output.journal.above_water,
            crash.mounted.output.journal.verification_passed,
            crash.mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(3), 1, 1, 1),
        "{case}，崩溃：择到 A 的根，验过点名单元施加 B 那一条"
    );
    for (history, read_back) in [
        ("干净关闭再挂载", cold_read_back(clean_close)),
        ("崩溃后再挂载", cold_read_back(crash)),
    ] {
        assert_eq!(
            read_back,
            (
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(7)),
                    content: second_version_content.to_vec()
                },
                Some((InstanceGeneration(2), CheckpointTxg(7)))
            ),
            "{case}，{history}：冷恢复读回 B 的内容"
        );
    }
    assert_ne!(
        second_version_content,
        file_content(),
        "B 的内容与 A 不同，读回哪一版分得出"
    );
}

/// C499：D23（journal 的角色与格式） 已定项 14「第一版没有干净关闭标记：每次挂载一律走恢复（全环扫描、逐条验证、择根，
/// 按六条口径与注 1–4 施加），不按上一次是不是干净关闭分支」。第一版没有关闭动作：干净关闭 = 最后一次发布整次落盘之后进程退出。
/// 这一条是盘不答 journal 提示（像文件后端与真设备，`JournalHintAnswer::AnswerUnknownLikeARealDevice`）的那一格：
/// 两次挂载都逐槽扫了全环——每块盘上起点落在环里的读不少于整环的记录槽数 196 608，读到的是环里每一个记录槽，两条历史读的槽相同；
/// 再照同一套规则恢复（`assert_the_two_histories_recover_by_the_same_rules`）。
/// 判别力自证：给挂载加一道「上一次干净关闭就只取 tail 那一条、不扫全环」的分支，干净关闭那一条历史当场红（`crates/mutations.tsv`）。
#[test]
fn c499_remount_on_devices_that_do_not_answer_the_journal_hint_scans_the_whole_ring_after_a_clean_close_and_after_a_crash_and_recovers_by_the_same_rules(
) {
    let histories = build_the_clean_close_and_the_crash_histories();
    let clean_close = mount_logging_reads(
        histories.clean_close_devices,
        JournalHintAnswer::AnswerUnknownLikeARealDevice,
    );
    let crash = mount_logging_reads(
        histories.crash_devices,
        JournalHintAnswer::AnswerUnknownLikeARealDevice,
    );

    let ring = journal_ring();
    let full_ring_scan_reads_per_device = ring.full_scan_reads_per_device();
    let every_record_slot_of_the_ring: BTreeSet<DeviceOffsetInBytes> = (0
        ..full_ring_scan_reads_per_device)
        .map(|record_index| DeviceOffsetInBytes(ring.start.0 + record_index * JOURNAL_RECORD_BYTES))
        .collect();
    for (history, mount) in [("干净关闭再挂载", &clean_close), ("崩溃后再挂载", &crash)]
    {
        assert_eq!(
            mount
                .read_logs
                .iter()
                .map(|(device, _)| *device)
                .collect::<Vec<_>>(),
            BOTH_DEVICES.to_vec(),
            "{history}：两块盘都记了读"
        );
        for (device, read_log) in &mount.read_logs {
            let reads_inside_the_ring =
                u64::try_from(read_log.journal_ring_read_offsets.len()).expect("读次数装得进 u64");
            assert!(
                reads_inside_the_ring >= full_ring_scan_reads_per_device,
                "{history}：盘 {} 上起点落在环里的读只有 {reads_inside_the_ring} 次，不够扫一遍整环（{full_ring_scan_reads_per_device} 个记录槽）",
                device.0
            );
            let slots_read: BTreeSet<DeviceOffsetInBytes> =
                read_log.journal_ring_read_offsets.iter().copied().collect();
            let slots_never_read = every_record_slot_of_the_ring
                .difference(&slots_read)
                .count();
            let offsets_off_the_record_slots = slots_read
                .difference(&every_record_slot_of_the_ring)
                .count();
            assert!(
                slots_never_read == 0 && offsets_off_the_record_slots == 0,
                "{history}：盘 {} 上读到的是环里每一个记录槽：没读到的槽 {slots_never_read} 个，不在记录槽起点上的读 {offsets_off_the_record_slots} 个",
                device.0
            );
        }
    }
    let clean_close_slots = clean_close.journal_ring_read_slots();
    let crash_slots = crash.journal_ring_read_slots();
    assert!(
        clean_close_slots == crash_slots,
        "两条历史读的记录槽相同：只有干净关闭读了的 {} 个，只有崩溃读了的 {} 个",
        clean_close_slots.difference(&crash_slots).count(),
        crash_slots.difference(&clean_close_slots).count()
    );
    assert_the_two_histories_recover_by_the_same_rules(
        &clean_close,
        &crash,
        &histories.second_version_content,
        "盘不答提示",
    );
}

/// C499 盘答 journal 提示（只记写过的扇区的内存盘，`JournalHintAnswer::PassTheInnerDeviceAnswerThrough`）的那一格：
/// 恢复扫环只读盘答的记录槽（`singlefs_core::recovery::scan_journal`），所以「走了恢复」量成：两次挂载读的记录槽都恰好是盘答的那几个、
/// 也恰好是这条历史写过的记录槽（A 那几次与 B 的记录，崩溃那条在根槽写之前已经落了 B 的记录，所以两条相同）；逐盘钉两样读：
/// - 恢复扫环：每块盘先读这块盘写过的每个记录槽，每个槽恰好一次；
/// - 恢复扫环之后：只有取号之前逐盘核的那一读。D18（块里携带什么信息） 已定项 11 落后支缺单元那一半照系统配置第三轮判决
///   （`research/prompts/m3-close-sysconf-r3-main-verification.md`「四、落地」）改成「本盘环槽里有那一版末条记录就不算落后」，
///   这块盘最新那份系统配置的 tail 落后时，在它自己的环里读一次所选那一版末条记录那一槽（`singlefs_core::mount` 的
///   `last_record_of_the_version_in_its_ring_slot_on_the_device`，读盘、不走读阶段的缓存）。崩溃那条两块盘的 tail 都停在 B 之前，
///   各读一次 B 的末条记录（jsn 4）那一槽；干净关闭那条 B 之后的系统配置落了、tail 不落后，一读都没有。
///   这一读来自落后判定，不按上一次是不是干净关闭分支，与 C499 的条款不冲突。
///
/// 再照同一套规则恢复（`assert_the_two_histories_recover_by_the_same_rules`）。
/// 判别力自证：内存盘答的提示漏掉一个写过的记录槽，当场红；取号之前逐盘核不读那一槽，崩溃那条当场红（`crates/mutations.tsv`）。
#[test]
fn c499_remount_on_devices_that_answer_the_journal_hint_reads_exactly_the_written_record_slots_after_a_clean_close_and_after_a_crash_and_recovers_by_the_same_rules(
) {
    let histories = build_the_clean_close_and_the_crash_histories();
    let ring = journal_ring();
    let hinted_record_slots = |devices: &[(DeviceIdentity, MemoryDisk)]| {
        devices
            .iter()
            .flat_map(|(identity, device)| {
                device
                    .journal_record_offsets_hint(ring.start, ring.length_in_bytes)
                    .expect("只记写过的扇区的内存盘答得出提示")
                    .into_iter()
                    .map(|offset| (*identity, offset))
            })
            .collect::<BTreeSet<(DeviceIdentity, DeviceOffsetInBytes)>>()
    };
    let clean_close_hinted_slots = hinted_record_slots(&histories.clean_close_devices);
    let crash_hinted_slots = hinted_record_slots(&histories.crash_devices);
    let clean_close = mount_logging_reads(
        histories.clean_close_devices,
        JournalHintAnswer::PassTheInnerDeviceAnswerThrough,
    );
    let crash = mount_logging_reads(
        histories.crash_devices,
        JournalHintAnswer::PassTheInnerDeviceAnswerThrough,
    );

    // B 那次发布（txg 4）的末条记录 jsn 4 在环里的那一槽：取号之前逐盘核读的就是它。
    let last_record_slot_of_the_second_version =
        singlefs_core::journal::record_offset(4, JOURNAL_RING_DEFAULT_BYTES);
    for (
        history,
        mount,
        hinted_slots,
        written_record_slots,
        reads_per_device_of_the_check_before_the_acquisition,
    ) in [
        (
            "干净关闭再挂载",
            &clean_close,
            &clean_close_hinted_slots,
            &histories.clean_close_written_record_slots,
            0usize,
        ),
        (
            "崩溃后再挂载",
            &crash,
            &crash_hinted_slots,
            &histories.crash_written_record_slots,
            1usize,
        ),
    ] {
        let slots_read = mount.journal_ring_read_slots();
        assert_eq!(
            &slots_read, hinted_slots,
            "{history}：恢复读的记录槽恰好是盘答的那几个"
        );
        assert_eq!(
            &slots_read, written_record_slots,
            "{history}：恢复读的记录槽恰好是这条历史写过的记录槽"
        );
        assert_eq!(
            mount.journal_ring_reads(),
            written_record_slots.len()
                + reads_per_device_of_the_check_before_the_acquisition * BOTH_DEVICES.len(),
            "{history}：起点落在环里的读恰好是写过的记录槽数（恢复扫环每个槽读一次）加上取号之前逐盘核每块盘的那几读"
        );
        for (device, read_log) in &mount.read_logs {
            let written_on_this_device: BTreeSet<DeviceOffsetInBytes> = written_record_slots
                .iter()
                .filter(|(written_device, _)| written_device == device)
                .map(|(_, offset)| *offset)
                .collect();
            let offsets = &read_log.journal_ring_read_offsets;
            let (reads_of_the_recovery_scan, reads_after_the_recovery_scan) =
                offsets.split_at(written_on_this_device.len().min(offsets.len()));
            let slots_of_the_recovery_scan: BTreeSet<DeviceOffsetInBytes> =
                reads_of_the_recovery_scan.iter().copied().collect();
            assert_eq!(
                (
                    reads_of_the_recovery_scan.len(),
                    &slots_of_the_recovery_scan
                ),
                (written_on_this_device.len(), &written_on_this_device),
                "{history}：盘 {} 上恢复扫环读的恰好是这块盘写过的记录槽，每个槽一次",
                device.0
            );
            assert_eq!(
                reads_after_the_recovery_scan,
                vec![
                    last_record_slot_of_the_second_version;
                    reads_per_device_of_the_check_before_the_acquisition
                ]
                .as_slice(),
                "{history}：盘 {} 上恢复扫环之后的环内读只有取号之前逐盘核读 B 末条记录那一槽",
                device.0
            );
        }
    }
    assert_eq!(
        clean_close.journal_ring_read_slots(),
        crash.journal_ring_read_slots(),
        "两条历史读的记录槽相同"
    );
    assert_the_two_histories_recover_by_the_same_rules(
        &clean_close,
        &crash,
        &histories.second_version_content,
        "盘答提示",
    );
}
