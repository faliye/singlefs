//! 里程碑「第二个事务」增补 2 收口表第 9 行：用户 2026-09-23 照今天的做法写成条款的三格，各一条会红的用例
//! （三方判决 `research/prompts/m2-presumed-clauses-r1-main-verification.md` 第七节；P6 归并行线一，不在这里）。
//!
//! - C497（实例表在 bump 次序里排第几没有条款）：D3（空间分配） 已定项 10 ⑤「一次发布重写实例表单元时（写行、暖机、回退那几次），
//!   实例表单元最前」——写行那次发布（带文件的一版上、树表 0 条的一版上）与回退那次发布里，实例表的落点在这次全部提交内生块之前，
//!   树 0 里的出生序号也排在它们之前。
//! - C498（后续挂载与回退之后的暖机次数没有条款）：D16（发布语义） 已定项 8「后续挂载、回退之后与挂载中途崩了再挂」——
//!   写行 txg 取 max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1，之后每次空发布加一，
//!   按根环落点看落哪块盘，推到本实例的根覆盖每块盘为止，至多 R 次（写行 txg ≡ 2 (mod 3) 时白推一次，认下）。
//! - C499（干净关闭标记有没有没有条款）：D23（journal 的角色与格式） 已定项 14「第一版没有干净关闭标记」——
//!   干净关闭再挂载与崩溃后再挂载走同一条恢复路径（全环扫描、逐条验证、择根、按前缀口径施加）。

mod common;

use std::collections::BTreeSet;

use common::{
    build_pool, crash_state_devices, file_content, format_pool, geometry, parameters,
    publish_overwrite_in_process, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::{mount_rollback, mount_writable, Mounted, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput, TransactionUnit,
    TREE_IDENTIFIER_NONE,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::crash::{writes_and_segments, MemoryPool};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

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

/// C497：D3（空间分配） 已定项 10 ⑤ 的「实例表单元最前」，三个重写实例表的场合各核一遍——
/// ① 带文件的一版上写行（第一个文件之后重开可写挂载，写行 txg 4）；② 树表 0 条的一版上写行（只做过 mkfs 的池第二次可写挂载，
/// 这次只写实例表与一片分配记录节点）；③ 回退（① 之后退回 (1, 3)，回退行那次发布）。
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

    let mut rollback_devices = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut rollback_devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退到 (1, 3)");
    pool.devices = Some(rollback_devices);
    assert!(
        rolled_back
            .output
            .rows_written
            .iter()
            .any(|row| row.is_rollback),
        "这次写的是回退行"
    );
    let rollback_publish = rolled_back
        .output
        .row_publish
        .file_version()
        .expect("回退到带文件的一版：回退行那次发布是带文件的一版");
    assert_the_instance_table_is_bumped_first_in_a_file_version_publish(rollback_publish, "回退");

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
/// 第一个文件（txg 3）之后三次关闭再挂载（写行 txg 4、6、8：暖机 1、1、2 次，txg 8 ≡ 2 那次白推一次）→ 发一版数据（txg 11）→
/// 回退到 (1, 3)（写行 txg 12：暖机 1 次）→ 回退之后关闭再挂载（txg 14：2 次）→ 再挂载一次、崩在第一次暖机的根槽写上
/// （写行 txg 17 已落盘，暖机 txg 18 的记录落盘、根没落）→ 再挂载（施加 txg 18 那条记录，写行 txg 19：1 次）。
/// 每一次的写行 txg 与暖机 txg 与条款现算的逐个相等。
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

    // 同一个会话里发一版数据（txg 11），让回退那次的写行 txg 落到 ≡ 0 的一格。
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

    let mut devices = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退到 (1, 3)");
    pool.devices = Some(devices);
    assert_warm_up_follows_the_clause(&rolled_back, CheckpointTxg(12), 1, "回退");
    let after_rollback = reopen_and_mount(&mut pool);
    assert_warm_up_follows_the_clause(&after_rollback, CheckpointTxg(14), 2, "回退之后关闭再挂载");

    // 挂载中途崩了：写行 txg 17 整次落盘，第一次暖机（txg 18）的根槽写失败——那次的记录已经落盘（记录之后那道屏障已完成），
    // 根槽与系统配置槽都没有。挂载报错、进程退出。
    let crashed_warm_up_txg = CheckpointTxg(18);
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
    assert_eq!(plan.fired_count(), 1, "注入只打在 txg 18 那个根槽写上");
    drop(failing_devices);
    let after_the_crash = reopen_and_mount(&mut pool);
    assert_eq!(
        (
            after_the_crash.output.chosen_root.checkpoint_txg,
            after_the_crash.output.effective_root.checkpoint_txg,
            after_the_crash.output.journal.prefix_applied
        ),
        (CheckpointTxg(17), CheckpointTxg(18), 1),
        "挂载中途崩了再挂：所选根是写行 txg 17，txg 18 那条暖机记录被施加"
    );
    assert_warm_up_follows_the_clause(&after_the_crash, CheckpointTxg(19), 1, "挂载中途崩了再挂");
}

// ---------------------------------------------------------------------------------------------
// C499：第一版没有干净关闭标记
// ---------------------------------------------------------------------------------------------

/// 一次可写挂载写出的东西、它在每块盘上读了几次（按 `BOTH_DEVICES` 的次序），与挂载完仍开着的设备（冷恢复接着读它们）。
struct MountWithReadCounts<Device: BlockDevice> {
    mounted: Mounted,
    reads_per_device: Vec<u64>,
    devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Device>)>,
}

/// 一次可写挂载，数它在每块盘上读了几次（注入计划不开，只数）。
fn mount_counting_reads<Device: BlockDevice>(
    devices: Vec<(DeviceIdentity, Device)>,
) -> MountWithReadCounts<Device> {
    let plan = SharedFaultPlan::unarmed(geometry());
    let mut counted: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Device>)> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    let mounted = mount_writable(&parameters(), &mut counted).expect("可写挂载");
    let reads_per_device = BOTH_DEVICES
        .iter()
        .map(|device| plan.counts_of_device(*device).reads)
        .collect();
    MountWithReadCounts {
        mounted,
        reads_per_device,
        devices: counted,
    }
}

/// 挂载之后冷恢复读回的内容与施加之后的根。
fn cold_read_back<Device: BlockDevice>(
    mount: &MountWithReadCounts<Device>,
) -> (RecoveryOutcome, Option<(InstanceGeneration, CheckpointTxg)>) {
    let report = recover(&mount.devices, JournalPolicy::Consult);
    (report.outcome, report.effective_root)
}

/// C499：D23（journal 的角色与格式） 已定项 14「第一版没有干净关闭标记：每次挂载一律走恢复（全环扫描、逐条验证、择根，
/// 按六条口径与注 1–4 施加），不按上一次是不是干净关闭分支」。第一版没有关闭动作：干净关闭 = 最后一次发布整次落盘之后进程退出。
/// 两条历史都从 A（txg 3）之后发布 B（txg 4）起：
/// - 干净关闭：B 整次落盘，进程退出，重开可写挂载；
/// - 崩溃：B 的单元与记录落盘、根槽与系统配置槽没落就崩，重开可写挂载。
///
/// 两次挂载都做了一遍全环扫描（每块盘读的次数不少于整环的记录槽数 196 608）、都看到环里全部四条记录；
/// 干净关闭那次择到 B 的根、一条都不用施加，崩溃那次择到 A 的根、验过点名单元施加 B 那一条——施加之后的根都是 (1, 4)；
/// 之后写行 txg 5、jsn 5、暖机 txg 6、7 逐项相同，冷恢复都读回 B 的内容。
/// 判别力自证：给挂载加一道「上一次干净关闭就只取 tail 那一条、不扫全环」的分支，干净关闭那一条历史当场红（`crates/mutations.tsv`）。
#[test]
fn c499_remount_after_a_clean_close_and_after_a_crash_both_scan_the_whole_ring_and_recover_by_the_same_rules(
) {
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
    pool.output = second_version;

    // 崩溃那条历史的盘面：B 的写里根槽写之前的全部持久（单元与记录），根槽与系统配置槽一个都没有。
    let operations = pool.retained_operations();
    let mut base = MemoryPool::with_devices(&BOTH_DEVICES, IMAGE_BYTES);
    base.apply(&operations[..operations_before_the_second_version]);
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
    let persisted: Vec<bool> = (0..writes.len())
        .map(|write_index| write_index < first_root_write)
        .collect();
    let crash_devices = crash_state_devices(
        &base,
        &writes,
        &persisted,
        &SharedStream::retaining_contents(),
    );

    let full_ring_scan_reads_per_device = JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES;
    let clean_close = mount_counting_reads(pool.reopen_recorded());
    let crash = mount_counting_reads(crash_devices);
    for (history, mounted, reads) in [
        (
            "干净关闭再挂载",
            &clean_close.mounted,
            &clean_close.reads_per_device,
        ),
        ("崩溃后再挂载", &crash.mounted, &crash.reads_per_device),
    ] {
        for (device, reads_on_device) in BOTH_DEVICES.iter().zip(reads.iter()) {
            assert!(
                *reads_on_device >= full_ring_scan_reads_per_device,
                "{history}：盘 {} 上只读了 {reads_on_device} 次，不够扫一遍整环（{full_ring_scan_reads_per_device} 个记录槽）",
                device.0
            );
        }
        assert_eq!(
            mounted.output.journal.valid_records, 4,
            "{history}：全环扫描看到 jsn 1–4 四条"
        );
        assert_eq!(
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg
            ),
            (InstanceGeneration(1), CheckpointTxg(4)),
            "{history}：施加之后的根是 B 那一版"
        );
        assert_eq!(
            row_and_warm_up_txgs(mounted),
            (CheckpointTxg(5), vec![CheckpointTxg(6), CheckpointTxg(7)]),
            "{history}：写行 txg 5、暖机 txg 6、7"
        );
        assert_eq!(
            mounted.output.row_publish.record().counter,
            5,
            "{history}：写行 jsn 5"
        );
    }
    assert_eq!(
        (
            clean_close.mounted.output.chosen_root.checkpoint_txg,
            clean_close.mounted.output.journal.above_water,
            clean_close.mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(4), 0, 0),
        "干净关闭：择到 B 的根，水位之上没有记录"
    );
    assert_eq!(
        (
            crash.mounted.output.chosen_root.checkpoint_txg,
            crash.mounted.output.journal.above_water,
            crash.mounted.output.journal.verification_passed,
            crash.mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(3), 1, 1, 1),
        "崩溃：择到 A 的根，验过点名单元施加 B 那一条"
    );
    for (history, read_back) in [
        ("干净关闭再挂载", cold_read_back(&clean_close)),
        ("崩溃后再挂载", cold_read_back(&crash)),
    ] {
        assert_eq!(
            read_back,
            (
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(7)),
                    content: second_version_content.clone()
                },
                Some((InstanceGeneration(2), CheckpointTxg(7)))
            ),
            "{history}：冷恢复读回 B 的内容"
        );
    }
    assert_ne!(
        second_version_content,
        file_content(),
        "B 的内容与 A 不同，读回哪一版分得出"
    );
}
