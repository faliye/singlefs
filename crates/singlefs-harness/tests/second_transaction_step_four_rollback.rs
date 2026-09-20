//! 里程碑「第二个事务」步 4 的验收：发布 C 之后进程退出、重开走管理员回退到 A 的根 (1, 3)——不施加 A 之后的任何记录、
//! 取实例代号 3、在 A 那一版实例表上写回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)、发布 D（txg 9，jsn 接在环里最大的 jsn 8 之后 = 9，被抛弃的记录一条不盖）、
//! 暖机一次（txg 10 落盘 1）——冷启动择实例 3 的根读回第一次的内容；只被被抛弃根引用的槽由影子账隔离；池级 checker 全绿。

mod common;

use common::{
    build_pool, disk_snapshot, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_rollback, mount_writable, InstanceRow, MountError, Mounted, RollbackCandidateExclusion,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_root, choose_superblock, recover, replay_journal, scan_journal, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};
use std::collections::BTreeSet;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
fn build_through_third_publish(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    pool
}

/// 回退到树表 0 条的根（第一个事务里 txg 2 的暖机根），第一版不支持（设计没定），在任何写之前拒绝：第一个事务之后进程退出、重开回退到 (1, 2)
/// ⇒ 返回 `RollbackToVersionWithoutFileUnsupported`（它在回退候选集里，不报候选排除；增补 3 第 2 件代码三方第二轮判决第三节第 1 条）；
/// 两盘系统配置槽逐字节不变、根环没有新根、录制流一步都没多。
#[test]
fn rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write() {
    let mut pool = build_pool("step-four-rollback-to-warm-up-root");
    let warm_up_root = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(2),
    };
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mut devices = pool.reopen_recorded();
    let refused = mount_rollback(&parameters(), &mut devices, warm_up_root, ShadowLedger::On);
    pool.devices = Some(devices);
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackToVersionWithoutFileUnsupported(target)) if target == warm_up_root
        ),
        "暖机根下面没有文件版本：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "盘上逐字节不变"
    );
}

fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
}

/// 进程退出、重开走回退到 A 的根；回来的可写态装回 pool。
fn rollback_to_first_root(pool: &mut BuiltPool, shadow_ledger: ShadowLedger) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let rolled_back =
        mount_rollback(&parameters(), &mut devices, first_root(), shadow_ledger).expect("回退");
    pool.devices = Some(devices);
    pool.allocator = rolled_back.allocator.clone();
    pool.output = rolled_back
        .current
        .file_version()
        .expect("回退到 A，现行那一版带文件")
        .clone();
    rolled_back
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 一版账里这块盘上占着的每个槽（记录按跨度展开）。
fn slots_of(output: &TransactionOutput, device: DeviceIdentity) -> BTreeSet<u64> {
    output
        .allocation_records
        .iter()
        .filter(|record| record.device == device)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// 验收第一条：回退行 (1, 3, 0)、中间实例行 (2, 0, 0)；D 的根 (3, 9)、jsn 9（接在环里最大的 jsn 8 之后，C340 取 P2）、事务号 0、反向链 0，
/// 重写实例表 + 四个固定点单元；暖机一次落到另一块盘；一条记录都不施加；冷启动读回第一次的内容；被抛弃根独占的槽逐盘 34 个、
/// D 与暖机一个都不落在上面；checker 全绿（I-3.1 的并集按实例表把被抛弃的根排除，I-3.8 看见回退行）。
#[test]
fn rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content(
) {
    let mut pool = build_through_third_publish("step-four-rollback");
    let third = pool.output.clone();
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    let output = &rolled_back.output;
    assert_eq!(output.instance, InstanceGeneration(3));
    assert_eq!(
        output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(3),
                applied_transaction_high_water: 0,
                is_rollback: true,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            },
        ],
        "回退行与中间实例行"
    );
    assert_eq!(output.journal.prefix_applied, 0, "A 之后的记录一条都不施加");
    assert_eq!(
        (
            output.effective_root.instance,
            output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(3))
    );
    let rollback_publish = output
        .row_publish
        .file_version()
        .expect("回退到带文件的 A：D 是带文件的一版");
    assert_eq!(
        (
            rollback_publish.root.instance,
            rollback_publish.root.checkpoint_txg,
            rollback_publish.record.counter,
            rollback_publish.record.transaction,
            rollback_publish.record.back_chain
        ),
        (InstanceGeneration(3), CheckpointTxg(9), 9, 0, 0),
        "D：txg = max(根环 8, 记录 8) + 1；jsn 接在环里最大的 8 之后（C340 取 P2，被抛弃的记录一条不盖）；本实例第一条反向链 0"
    );
    assert_eq!(
        rollback_publish.rewritten,
        vec![
            TransactionUnit::InstanceTable,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]
    );
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "D 落盘 0、txg 10 落盘 1，一次就够"
    );
    let warm_up = output.warm_up_publishes[0]
        .file_version()
        .expect("带文件的一版上的暖机");
    assert_eq!(warm_up.root.checkpoint_txg, CheckpointTxg(10));
    assert_ne!(region_device(9), region_device(10));
    assert_eq!(warm_up.record.counter, 10);

    // 影子账：被抛弃的根 B、(2, 5)、(2, 6)、(2, 7)、C 引用而 A 不引用的槽——B 10、写行 6、暖机 4 + 4、C 10 = 34 个槽，逐盘。
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let abandoned: BTreeSet<u64> = slots_of(&third, device)
            .difference(&slots_of(rollback_publish, device))
            .copied()
            .collect();
        assert_eq!(abandoned.len(), 34, "盘 {device:?} 上只被被抛弃根引用的槽");
        // 影子账按窄读法只隔离这 34 个：mkfs 实例表那 2 个槽 A（候选）与 B 都引用，不在其内。
        assert!(
            output.isolated_slots_per_device.contains(&(device, 34)),
            "隔离的槽数 = 只被被抛弃根引用的 34 个 {:?}",
            output.isolated_slots_per_device
        );
        for publish in std::iter::once(rollback_publish).chain(
            output
                .warm_up_publishes
                .iter()
                .map(|publish| publish.file_version().expect("带文件的一版上的暖机")),
        ) {
            for placement in publish.placements() {
                for slot in placement.slot.0..placement.slot.0 + placement.span {
                    assert!(
                        !abandoned.contains(&slot),
                        "txg {} 的落点 {slot} 落在被抛弃根引用的槽上",
                        publish.root.checkpoint_txg.0
                    );
                }
            }
        }
    }

    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(10)),
            content: file_content()
        },
        "{:?}",
        report.journal
    );
    assert_eq!(
        report.journal.valid_records, 10,
        "jsn 1–8 原样在（P2 一条不盖）、9 是 D、10 是暖机"
    );
    assert_eq!(
        report.journal.prefix_applied, 0,
        "(3, 10) 之后 jsn 11 一条都没有"
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在回退之后的镜像上要成立"
        );
    }
    for must_hold in ["I-3.1", "I-3.8", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}

/// 回退候选集（D23 已定项 14）：实例表里有行 (i, Ti, Wi) 的实例，只有 T ≤ Ti 的根可选——B 的根 (1, 4) 与 C 的根 (2, 8) 都是被抛弃时间线的
/// （F 还是 0，不低于 F），报 `OnAbandonedTimeline`；根环里没有的 (1, 42) 报 `NotInRing`。调用方按这个字段分流，不看文字。
#[test]
fn rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused() {
    let mut pool = build_through_third_publish("step-four-refused");
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    let missing = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(42),
    };
    for (target, expected) in [
        (
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            },
            RollbackCandidateExclusion::OnAbandonedTimeline,
        ),
        (
            RollbackTarget {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(8),
            },
            RollbackCandidateExclusion::OnAbandonedTimeline,
        ),
        (missing, RollbackCandidateExclusion::NotInRing),
    ] {
        let mut devices = pool.reopen_recorded();
        let refused = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        pool.devices = Some(devices);
        match refused {
            Err(MountError::RollbackTargetNotACandidate {
                target: reported,
                exclusion,
            }) => assert_eq!((reported, exclusion), (target, expected)),
            other => panic!(
                "{target:?} 该被拒：{:?}",
                other.map(|mounted| mounted.output.instance)
            ),
        }
    }
}

/// C314（回退可以复用被抛弃的根引用的单元） 那一格的必红，影子账开关强制进入：关掉影子账，回退之后再发两版文件，
/// 数据单元落回 B 与 C 的数据槽（50182、50184）；把实例 3 的四个根槽都改坏，恢复挂上 C 的根 (2, 8)，它的数据单元已被盖掉，读不出第三次的内容。
/// 影子账开着：两版数据落 50186、50188，同样改坏四个根槽之后恢复挂上 C 的根、第三次的内容原样读回。
#[test]
fn without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit(
) {
    for (shadow_ledger, expected_slots, expect_third_content_readable) in [
        (ShadowLedger::Off, [50182, 50184], false),
        (ShadowLedger::On, [50186, 50188], true),
    ] {
        let mut pool = build_through_third_publish("step-four-shadow");
        let rolled_back = rollback_to_first_root(&mut pool, shadow_ledger);
        let expected_isolated = if shadow_ledger == ShadowLedger::On {
            34
        } else {
            0
        };
        assert!(
            rolled_back
                .output
                .isolated_slots_per_device
                .iter()
                .all(|(_, isolated)| *isolated == expected_isolated),
            "{shadow_ledger:?}：{:?}",
            rolled_back.output.isolated_slots_per_device
        );
        let fourth = overwrite_in_process(&mut pool, &content_of(3000, 5), InstanceGeneration(3));
        let fifth = overwrite_in_process(&mut pool, &content_of(3100, 9), InstanceGeneration(3));
        assert_eq!(
            [
                fourth.data_pointer.locations[0].slot.0,
                fifth.data_pointer.locations[0].slot.0
            ],
            expected_slots,
            "{shadow_ledger:?} 下回退之后两版的数据落点"
        );
        let mut image = pool.memory_pool();
        for txg in [9u64, 10, 11, 12] {
            let target = target_for_publish(CheckpointTxg(txg));
            image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
        }
        let report = recover(&image, JournalPolicy::Consult);
        if expect_third_content_readable {
            assert_eq!(
                report.outcome,
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(8)),
                    content: third_content()
                },
                "影子账开着：C 引用的单元一个没被盖，回到 C 读第三次的内容"
            );
        } else {
            assert_ne!(
                report.outcome,
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(8)),
                    content: third_content()
                },
                "影子账关着：C 的数据单元已被第五版盖掉，第三次的内容读不回来"
            );
        }
    }
}

/// 前缀第五条（D23 已定项 14）：所选根的实例有回退行时，该实例的记录只施加到回退行的 W 为止——直接喂 `replay_journal`：
/// 到 B 为止的镜像上选 A 的根 (1, 3)，不带回退行施加 B 那条（事务号 2）；W = 0 一条都不施加；W = 2 施加到 B；W = 1 停在 B 之前。
#[test]
fn the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water() {
    let mut pool = build_pool("step-four-cap");
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let newest = choose_root(&image, &superblock).expect("B 的根");
    assert_eq!(newest.checkpoint_txg, CheckpointTxg(4));
    let records = scan_journal(&image, &superblock);
    let roots = singlefs_core::recovery::readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let first = roots
        .iter()
        .find(|root| root.checkpoint_txg == CheckpointTxg(3))
        .copied()
        .expect("A 的根在环里");
    for (high_water, expected_applied, expected_txg) in [
        (None, 1, 4u64),
        (Some(0), 0, 3),
        (Some(1), 0, 3),
        (Some(2), 1, 4),
    ] {
        let (report, effective) = replay_journal(
            &image,
            &first,
            superblock.geometry.journal_ring_bytes,
            &records,
            true,
            high_water,
        );
        assert_eq!(
            (report.prefix_applied, effective.checkpoint_txg.0),
            (expected_applied, expected_txg),
            "回退行 W = {high_water:?}"
        );
    }
}

/// C340 取 P2：回退之后新实例的第一条 jsn 接在环里最大的 jsn 之后，被抛弃发布的记录一条不盖——B 的记录已提交、根还没落盘时发起回退，
/// 崩在回退生效之前那次恢复才仍「与没发起回退时相同」（步 4 / 步 5 代码三方第一轮云端攻方腿打中 P1 盖掉 B 那条）。
/// 影子账只住内存，所以每次挂载都重算：回退之后普通重开一次，被抛弃根引用的槽照样隔离（同一轮攻方腿打中重开后隔离归零）。
#[test]
fn rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation() {
    let mut pool = build_through_third_publish("step-four-p2-remount");
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    assert_eq!(
        rolled_back.output.row_publish.record().counter,
        9,
        "D 的 jsn 接在 C 的 8 之后"
    );
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let records = scan_journal(&image, &superblock);
    for (instance, counter) in [(1u32, 4u64), (2, 5), (2, 8), (3, 9), (3, 10)] {
        assert!(
            records.contains_key(&(InstanceGeneration(instance), counter)),
            "记录 ({instance}, jsn {counter}) 该原样在环里：{:?}",
            records.keys().collect::<Vec<_>>()
        );
    }
    let mut devices = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("回退之后普通重开");
    pool.devices = Some(devices);
    pool.allocator = remounted.allocator.clone();
    pool.output = remounted
        .current
        .file_version()
        .expect("回退之后重开，现行那一版带文件")
        .clone();
    assert_eq!(remounted.output.instance, InstanceGeneration(4));
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 34), (DeviceIdentity(1), 34)],
        "按 D 那一版实例表判被抛弃的根（B、实例 2 的四条）引用的槽，普通重开照样隔离"
    );
    assert_eq!(remounted.output.abandoned_roots_unreadable, 0);
    // 被抛弃根引用的槽：B 的数据 50182–50183、C 的数据 50184–50185 与它们的节点（隔离），以及 mkfs 实例表 50176–50177
    // （A 也引用、不隔离，但 D 释放它之后还在 defer 里）——重开后的发布一个都不该落上去。
    let abandoned: BTreeSet<u64> = (50176..50178).chain(50182..50186).collect();
    let next = overwrite_in_process(&mut pool, &content_of(2100, 41), InstanceGeneration(4));
    for placement in next.placements() {
        for slot in placement.slot.0..placement.slot.0 + placement.span {
            assert!(
                !abandoned.contains(&slot),
                "重开后的发布落到了被抛弃根引用的槽 {slot}"
            );
        }
    }
}

/// 被抛弃根 C 的树表单元在两块盘上都改坏：影子账罩不到 C（读不出就没法知道它引用谁），挂载照样成功、只计数一条读不出的被抛弃根，
/// 别的被抛弃根引用的槽照旧隔离（步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
#[test]
fn torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount() {
    let mut pool = build_through_third_publish("step-four-torn-abandoned");
    let third = pool.output.clone();
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    let mut devices = pool.reopen_recorded();
    for location in &third.root.tree_table.locations {
        let (_, recorded) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("C 的树表所在的盘");
        let offset = location.slot.to_device_offset();
        let mut bytes = vec![0u8; usize::try_from(singlefs_format::NODE_BYTES).expect("16384")];
        recorded.read_at(offset, &mut bytes).expect("读 C 的树表");
        bytes[200] ^= 0xff;
        recorded
            .write_at(offset, &bytes, WriteDurability::Plain)
            .expect("改坏 C 的树表");
    }
    let remounted =
        mount_writable(&parameters(), &mut devices).expect("被抛弃根的树表撕裂不拒绝挂载");
    assert_eq!(remounted.output.abandoned_roots_unreadable, 1, "C 读不出");
    // C 独占的槽（它的数据 2 + 它的节点）罩不到；B、写行与暖机那些照旧隔离。
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 24), (DeviceIdentity(1), 24)],
        "少了只被 C 引用的 10 个槽"
    );
}
