//! 里程碑「覆盖写、释放、回退与复用」步 3 的验收：发布 B 之后进程退出、重开两个镜像走可写挂载——恢复、从盘上重建上一版与分配器、
//! 取实例代号 2、给实例 1 写行 (1, 4, 0)、写行发布 txg 5、暖机两次（txg 6 落盘 0 白费、txg 7 落盘 1）——再发布 C；
//! 冷启动择实例 2 的根读回第三次的内容；实例表读回那一行；池级 checker 全绿（含 I-3.8）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/writable_mount_takes_a_second_instance.rs"]
mod scenarios_of_writable_mount_takes_a_second_instance;

use common::{build_pool, disk_snapshot, parameters, Recorded};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_writable_mount_takes_a_second_instance::{
    build_publish_second_version_and_remount, content_of, overwrite_in_process, region_device,
    second_content, third_content, RollbackFloorHandedToTheWriter,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_writable, InstanceRow, MountError, NewerPublishWitness, RollbackTarget,
    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

/// 同一实例再发的第四版（txg 6、jsn 6）：字节数与种子都与前两版不同，读回哪一版看得出。
const FOURTH_FILE_BYTES: usize = 3300;

const FOURTH_FILE_CONTENT_SEED: usize = 17;

fn fourth_content() -> Vec<u8> {
    content_of(FOURTH_FILE_BYTES, FOURTH_FILE_CONTENT_SEED)
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_takes_a_second_instance::remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version()
{
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_takes_a_second_instance::remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version,
    );
}

/// 验收第二条的反面（D16 已定项 8 买的东西）：把实例 2 在盘 0 上的两个根槽（txg 5、6）都改坏，冷启动仍择到盘 1 上 txg 7 那个根、
/// 内容还是第二次的；暖机只推一次的话盘 1 上没有实例 2 的根，恢复退回实例 1。
#[test]
fn damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device() {
    let (pool, _second, mounted) =
        build_publish_second_version_and_remount("step-three-one-device-damaged");
    let mut image = pool.memory_pool();
    for txg in [5u64, 6] {
        let target = target_for_publish(
            CheckpointTxg(txg),
            parameters().geometry.root_ring_slots_per_region,
        );
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(7))),
        "盘 0 上实例 2 的根都坏了，盘 1 上 txg 7 那个还在"
    );
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(7)),
            content: second_content()
        }
    );
    assert_eq!(mounted.output.warm_up_publishes.len(), 2);
}

/// 验收第三条：预置一条实例 1 的记录（jsn 5、txg 5、事务号 3，点名的单元也在盘上），把它的根槽改坏 ⇒ 重开那一刻所选根是 B 的根、
/// 同一个实例 ⇒ 恢复必须施加它，行写 (1, 5, 3)，写行发布的 txg 从 6 起。
/// 那一版把 F 抬到 4（B 的根带 F = 0）：施加出来的根的 F 取记录新根段里的 4、不是所选根的 0（已定项 15）。
#[test]
fn stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row(
) {
    let mut pool = build_pool("step-three-stray-record");
    let second = overwrite_in_process(
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
    );
    pool.output = second;
    let stray = overwrite_in_process(
        &mut pool,
        &third_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::RaisedTo(CheckpointTxg(4)),
    );
    assert_eq!(
        (
            stray.root.checkpoint_txg,
            stray.record.counter,
            stray.record.transaction
        ),
        (CheckpointTxg(5), 5, 3)
    );
    let mut devices: Vec<(DeviceIdentity, Recorded)> = pool.reopen_recorded();
    // 把 txg 5 的根槽改坏：只留记录与单元，模拟「根槽没持久」。
    let target = target_for_publish(
        CheckpointTxg(5),
        parameters().geometry.root_ring_slots_per_region,
    );
    let device = region_device(5);
    let offset = slot_offset(target, 4096);
    let (_, damaged_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("区域归属的盘在池里");
    let mut garbage = vec![0u8; 4096];
    garbage[0] = 0xff;
    damaged_device
        .write_at(offset, &garbage, WriteDurability::Plain)
        .expect("改坏根槽");
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4))
    );
    assert_eq!(mounted.output.journal.prefix_applied, 1, "jsn 5 被施加");
    assert_eq!(mounted.output.journal.maximum_applied_transaction, 3);
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(5),
        "施加之后的根是第 5 代"
    );
    assert_eq!(
        (
            mounted.output.chosen_root.rollback_floor,
            stray.record.new_rollback_floor
        ),
        (CheckpointTxg(0), CheckpointTxg(4)),
        "取样点：所选根（B 的根）带 F = 0，那条记录的新根段带抬过的 F = 4"
    );
    assert_eq!(
        mounted.output.effective_root.rollback_floor, stray.record.new_rollback_floor,
        "施加出来的根的 F 取记录新根段里的那一份，不是所选根的"
    );
    assert_eq!(
        mounted.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(5),
            applied_transaction_high_water: 3,
        }]
    );
    assert_eq!(
        mounted.output.row_publish.root().checkpoint_txg,
        CheckpointTxg(6),
        "新实例的第一次发布 = max(根环 4, 记录 5) + 1"
    );
    assert_eq!(mounted.output.row_publish.record().counter, 6);
    pool.devices = Some(devices);
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), mounted.current.root().checkpoint_txg),
            content: third_content()
        },
        "施加了那条记录之后照抄的文件角色就是第三次的内容"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_takes_a_second_instance::checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_takes_a_second_instance::checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root,
    );
}

/// 前缀链首接在所选根自己那条记录之后（D23 已定项 14 第 1 条「链从所选根覆盖的最后一条记录之后接」）：同一实例里再发两版（txg 5、6），
/// 把两个根槽改坏、再把 jsn 5 的记录两份都改坏——所选根 (1, 4) 自己那条 jsn 4 读得出，链首该是 jsn 5，jsn 5 读不出 ⇒ 断号即止，
/// jsn 6 一条都不施加，读回第二次的内容；从「水位之上最小的那条」接的写法会施加 jsn 6、读出第四版。
#[test]
fn one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact(
) {
    let mut pool = build_pool("step-three-chain-start");
    let second = overwrite_in_process(
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
    );
    pool.output = second;
    let third = overwrite_in_process(
        &mut pool,
        &third_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
    );
    pool.output = third;
    let fourth = overwrite_in_process(
        &mut pool,
        &fourth_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
    );
    assert_eq!(
        (fourth.root.checkpoint_txg, fourth.record.counter),
        (CheckpointTxg(6), 6)
    );
    let mut image = pool.memory_pool();
    for txg in [5u64, 6] {
        let target = target_for_publish(
            CheckpointTxg(txg),
            parameters().geometry.root_ring_slots_per_region,
        );
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.flip_byte(device, record_offset(5, JOURNAL_RING_DEFAULT_BYTES), 300);
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: second_content()
        },
        "jsn 5 读不出、jsn 6 不许接上：{:?}",
        report.journal
    );
    assert_eq!(report.journal.valid_records, 5, "jsn 1–4、6");
    assert_eq!(report.journal.prefix_applied, 0);
}

/// 三方第一轮攻方腿打中的一格：所选根自己那条记录读不出时链首没有锚点，不许把水位之上最小的那条无条件接上——同一实例再发三版
/// （txg 4、5、6），改坏 txg 5、6 的根槽让所选根退到 (1, 4)，再改坏 jsn 4 与 jsn 5 两份镜像：jsn 6 的 txg 是 6、不是 4 + 1 ⇒ 一条都不施加、
/// 读回第二版；只改坏 jsn 4 时 jsn 5 的 txg 正是 5 ⇒ 接上、再顺着 jsn 6 施加到第四版，可写挂载写行 (1, 6, 4)。
/// 改坏 jsn 4 与 jsn 5 那一格的可写挂载被 C554 乙拒：系统配置见证过 jsn 6（第四版那次发布轮换写的 tail），所选那一版 (1, 4) 的末条
/// jsn 4 读不出、计数器等于见证值的 jsn 6 读得出、它的 (1, 6) 比 (1, 4) 新，判据为真、重读一次仍为真 ⇒
/// `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { 见证 6、按 jsn 6 那条 (1, 6) 比 })`，
/// 拒在任何写之前、盘上逐字节不变（原来钉的是写行 (1, 4, 0)，乙合入之后那一段不可达）。恢复那一段（链首没锚点、jsn 6 不接上）照旧。
#[test]
fn torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_the_next_record_is_torn_too(
) {
    torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg(
        AnchorTear::TheAnchorAndTheNextRecord,
    );
}

#[test]
fn torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_only_the_anchor_is_torn(
) {
    torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg(
        AnchorTear::OnlyTheAnchor,
    );
}

/// 改坏哪几份 journal 记录：锚点 jsn 4 连同下一条 jsn 5，还是只有 jsn 4。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnchorTear {
    TheAnchorAndTheNextRecord,
    OnlyTheAnchor,
}

fn torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg(tear: AnchorTear) {
    let (tear_jsn_five, expected_effective_root, expected_content, expected_applied) = match tear {
        AnchorTear::TheAnchorAndTheNextRecord => (true, CheckpointTxg(4), second_content(), 0),
        AnchorTear::OnlyTheAnchor => (false, CheckpointTxg(6), fourth_content(), 2),
    };
    let mut pool = build_pool("step-three-torn-anchor");
    pool.output = overwrite_in_process(
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
    );
    pool.output = overwrite_in_process(
        &mut pool,
        &third_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
    );
    // 第四版把 F 抬到 5（所选根 (1, 4) 带 F = 0）：只撕锚点那一格施加到 jsn 6，施加出来的根的 F 取 jsn 6 新根段里的 5（已定项 15）。
    let fourth = overwrite_in_process(
        &mut pool,
        &fourth_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::RaisedTo(CheckpointTxg(5)),
    );
    assert_eq!(fourth.record.counter, 6);
    let mut image = pool.memory_pool();
    for txg in [5u64, 6] {
        let target = target_for_publish(
            CheckpointTxg(txg),
            parameters().geometry.root_ring_slots_per_region,
        );
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    let torn: Vec<u64> = if tear_jsn_five { vec![4, 5] } else { vec![4] };
    for jsn in &torn {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            image.flip_byte(device, record_offset(*jsn, JOURNAL_RING_DEFAULT_BYTES), 300);
        }
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        (report.outcome, report.effective_root),
        (
            RecoveryOutcome::FileRead {
                root: (InstanceGeneration(1), CheckpointTxg(4)),
                content: expected_content
            },
            Some((InstanceGeneration(1), expected_effective_root))
        ),
        "撕掉 {torn:?}（所选根 (1, 4)）：{:?}",
        report.journal
    );
    assert_eq!(report.journal.prefix_applied, expected_applied);
    // 同一段历史做可写挂载：写出的行 W 只罩住真被施加的前缀。
    let mut devices = pool.reopen_recorded();
    for txg in [5u64, 6] {
        let target = target_for_publish(
            CheckpointTxg(txg),
            parameters().geometry.root_ring_slots_per_region,
        );
        let device = region_device(txg);
        let offset = slot_offset(target, 4096);
        let (_, recorded) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == device)
            .expect("那块盘");
        let mut bytes = vec![0u8; 4096];
        recorded.read_at(offset, &mut bytes).expect("读根槽");
        bytes[100] ^= 0xff;
        recorded
            .write_at(offset, &bytes, WriteDurability::Plain)
            .expect("改坏根槽");
    }
    for jsn in &torn {
        for (_, recorded) in devices.iter_mut() {
            let offset = record_offset(*jsn, JOURNAL_RING_DEFAULT_BYTES);
            let mut bytes = vec![0u8; 4096];
            recorded.read_at(offset, &mut bytes).expect("读记录");
            bytes[300] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏记录");
        }
    }
    pool.devices = Some(devices);
    match tear {
        AnchorTear::TheAnchorAndTheNextRecord => {
            let operations_before_the_mount = pool.stream.operation_count();
            let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
            let refused = mount_writable(&parameters(), pool.devices.as_mut().expect("镜像还开着"))
                .expect_err("见证 jsn 6 的那一版比所选根新：可写挂载拒");
            let reading = SelectedVersionAgainstTheWitness {
                selected_version: RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(4),
                },
                witness: NewerPublishWitness {
                    witnessed_journal_counter: 6,
                    comparison: WitnessedCounterComparison::AgainstTheRecordAtTheWitnessedCounter {
                        record: RollbackTarget {
                            instance: InstanceGeneration(1),
                            checkpoint_txg: CheckpointTxg(6),
                        },
                    },
                },
            };
            assert!(
                matches!(
                    &refused,
                    MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                        if **still_unreadable
                            == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                                first_read: reading,
                                reread: reading,
                            }
                ),
                "撕掉 {torn:?}：C554 乙按 jsn 6 那条 (1, 6) 判、重读仍为真：{refused:?}"
            );
            assert_eq!(
                pool.stream.operation_count(),
                operations_before_the_mount,
                "拒在取号之前：录制流一个写、一道屏障都没多"
            );
            assert_eq!(
                disk_snapshot(&pool.memory_pool(), &pool.stream),
                before,
                "两块盘逐字节不变"
            );
        }
        AnchorTear::OnlyTheAnchor => {
            let mounted = mount_writable(&parameters(), pool.devices.as_mut().expect("镜像还开着"))
                .expect("可写挂载");
            assert_eq!(
                mounted
                    .output
                    .rows_written
                    .iter()
                    .map(|row| (row.selected_root_txg, row.applied_transaction_high_water))
                    .collect::<Vec<_>>(),
                vec![(CheckpointTxg(6), 4)],
                "撕掉 {torn:?} 之后写的行"
            );
            assert_eq!(
                (
                    mounted.output.chosen_root.rollback_floor,
                    fourth.record.new_rollback_floor
                ),
                (CheckpointTxg(0), CheckpointTxg(5)),
                "取样点：所选根 (1, 4) 带 F = 0，jsn 6 的新根段带抬过的 F = 5"
            );
            assert_eq!(
                mounted.output.effective_root.rollback_floor, fourth.record.new_rollback_floor,
                "施加到 jsn 6：施加出来的根的 F 取它新根段里的那一份，不是所选根的"
            );
        }
    }
}
