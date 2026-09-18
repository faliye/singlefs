//! 云端攻方腿 m2-step45-code-r2 的四条攻击用例（副本，不入库）。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RaisedFloor, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::recovery::{
    choose_superblock, readable_roots, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};

type Recorded = common::Recorded;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
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

fn build_through_rollback(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    let mut reopened = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut reopened,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退");
    pool.devices = Some(reopened);
    pool.allocator = rolled_back.allocator;
    pool.output = rolled_back.current;
    pool
}

fn four_overwrites_after_the_rollback(pool: &mut BuiltPool) -> Vec<TransactionOutput> {
    [17usize, 19, 23, 29]
        .iter()
        .map(|seed| {
            overwrite_in_process(pool, &content_of(3000 + seed, *seed), InstanceGeneration(3))
        })
        .collect()
}

fn raise_floor(pool: &mut BuiltPool, new_floor: CheckpointTxg) -> Result<RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
    );
    pool.output = current;
    raised
}

/// 一个字节翻掉某个 txg 的根槽（根槽不镜像，一次发布只写一个槽 ⇒ 一个故障）。
fn damage_root_slot(devices: &mut [(DeviceIdentity, Recorded)], txg: u64) {
    let target = target_for_publish(CheckpointTxg(txg));
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
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

fn roots_of(pool: &BuiltPool) -> Vec<RootRecord> {
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
}

/// X3-A：抬 F 生效、回收、E 合法复用 50178 之后，翻掉盘 1 上唯一带 F = 11 的根（txg 16）一个字节。
/// 重开时 F_生效 回到 0，新实例的根按 `establish_instance` 把 `rollback_floor: effective_floor` 写成 0
/// ⇒ checker 的候选集（按最新根自己的 F 收）重新含进第 0 代根与 A，而它们引用的 mkfs 树表单元已被 E 合法盖掉 ⇒ I-2.1 红。
#[test]
fn x3_losing_the_only_floor_carrier_after_a_legal_reuse_writes_the_floor_back_to_zero_and_the_checker_goes_red(
) {
    let mut pool = build_through_rollback("opus-r2-x3");
    four_overwrites_after_the_rollback(&mut pool);
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg.0)
            .collect::<Vec<_>>(),
        vec![15, 16]
    );
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(
        reuse.data_pointer.locations[0].slot,
        SlotNumber(50178),
        "E 合法复用 mkfs 树表那一槽（释放代 3 ≤ F = 11）"
    );
    let before = check_pool_image(&pool.memory_pool());
    let violated_before: Vec<&str> = before
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert!(violated_before.is_empty(), "复用之后 checker 全绿：{violated_before:?}");

    let mut devices = pool.reopen_recorded();
    damage_root_slot(&mut devices, 16);
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted.current.clone();
    let written_floor = mounted.output.row_publish.root.rollback_floor;
    let after = check_pool_image(&pool.memory_pool());
    let violated_after: Vec<&str> = after
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    println!(
        "X3-A 一个字节之后：新实例的根写 F = {}；违例 {:?}",
        written_floor.0, violated_after
    );
    for (name, verdict) in after
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
    {
        println!("  {name} {verdict:?}");
    }
    assert_eq!(written_floor, CheckpointTxg(0), "新实例的根把 F 写回 0");
    assert!(violated_after.contains(&"I-2.1"), "{violated_after:?}");
    assert!(violated_after.contains(&"I-3.1"), "{violated_after:?}");
}

/// X2-A：被抛弃根（C，(2, 8)）的树表单元两份都坏掉之后，`rebuilt_allocator` 的影子账那一步 `allocation_records_under_root`
/// 返回 Err ⇒ 整个可写挂载开不了。当前时间线（A 与它之后）一个字节没坏，只读恢复照常读得回文件。
/// 第一轮之前影子账只在 `mount_rollback` 里算，这条路是 S2「每次挂载都算」引进来的。
#[test]
fn x2_a_corrupting_an_abandoned_roots_tree_table_makes_every_writable_mount_fail() {
    let mut pool = build_through_rollback("opus-r2-x2a");
    let abandoned_root = roots_of(&pool)
        .into_iter()
        .find(|root| root.checkpoint_txg == CheckpointTxg(8))
        .expect("C 的根在环里");
    let locations = abandoned_root.tree_table.locations;
    let mut devices = pool.reopen_recorded();
    for location in &locations {
        let (_, recorded) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("那块盘");
        let offset = location.slot.to_device_offset();
        let mut bytes = vec![0u8; 16384];
        recorded.read_at(offset, &mut bytes).expect("读树表单元");
        bytes[200] ^= 0xff;
        recorded
            .write_at(offset, &bytes, WriteDurability::Plain)
            .expect("改坏树表单元");
    }
    println!(
        "X2-A 改坏 {:?}（被抛弃根 (2, 8) 的树表单元两份）",
        locations
            .iter()
            .map(|location| (location.device.0, location.slot.0))
            .collect::<Vec<_>>()
    );
    let refused = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(devices);
    let error = match refused {
        Ok(mounted) => panic!("挂载居然成功了：实例 {:?}", mounted.output.instance),
        Err(error) => error,
    };
    println!("X2-A 可写挂载失败：{error:?}");
    // 只读恢复照常：当前时间线一个字节没坏。
    let report = recover(&pool.memory_pool(), JournalPolicy::Consult);
    println!("X2-A 只读恢复：{:?}", match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => format!("根 {root:?}、内容 {} 字节", content.len()),
        other => format!("{other:?}"),
    });
    assert!(
        matches!(report.outcome, RecoveryOutcome::FileRead { .. }),
        "只读恢复仍读得回文件：{:?}",
        report.outcome
    );
}

/// X2-B：保守读法把 R_old 也引用的槽一起隔离。这些槽被 R_old 之后的发布释放、抬 F 之后被回收 ⇒ 它们离开「已分配」、
/// 进「空闲」，却仍被隔离、永远发不出去；D28 已定项 1 第九项「被抛弃根独占量 = 被抛弃根的已分配 − R_old 的已分配」
/// 按定义扣不到它们（两边都有的在差里相消）⇒ 可用被高估正好那几槽。
#[test]
fn x2_b_isolated_slots_that_r_old_also_referenced_are_counted_as_free_after_reclaim() {
    let mut pool = build_through_rollback("opus-r2-x2b");
    four_overwrites_after_the_rollback(&mut pool);
    raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    for device in &pool.allocator.devices {
        let start = 50176u64;
        let end = start + device.unit_area_slots();
        let really_free = (start..end)
            .filter(|slot| device.is_free(SlotNumber(*slot)))
            .count() as u64;
        let bookkept_free = device.free_slots();
        println!(
            "X2-B 盘 {:?}：记账空闲 {bookkept_free}、位图上真能发的 {really_free}、隔离 {}、差 {}",
            device.device,
            device.isolated_slots(),
            bookkept_free - really_free
        );
        assert!(
            bookkept_free > really_free,
            "记账空闲比真能发的多：{bookkept_free} vs {really_free}"
        );
        // 50176–50177 是 mkfs 实例表：A 也引用它，D 把它放掉（释放代 9）、抬 F 到 11 时回收了，却仍被隔离。
        assert!(!device.is_free(SlotNumber(50176)));
        assert!(!device.is_free(SlotNumber(50177)));
    }
}

/// X1-A：P2 把被抛弃的记录原样留在环里。同实例连着发三版 B、B2、B3（txg 4、5、6，jsn 4、5、6），回退到 A (1, 3)
/// ⇒ 新实例 2 的 D 落在 jsn 7（P2），jsn 4–6 三条原样在环里。翻掉 txg 4、5、6、7、8 五个根槽（各一个字节、各一块盘）之后，
/// 恢复择到 A 的根，前缀规则从 A 自己那条记录（jsn 3）往后接：jsn 4、5、6 同实例、连号、txg 更大 ⇒ 三条全施加，
/// 管理员的回退被静默撤销、读回的是被抛弃的第四版。前缀第五条救不了（回退行写在实例 2 的表里，所选根是实例 1 的旧根）。
#[test]
fn x1_a_after_p2_a_recovery_that_falls_back_to_r_old_replays_the_whole_abandoned_run() {
    let mut pool = build_pool("opus-r2-x1a");
    let second = content_of(4100, 3);
    let third = content_of(2500, 11);
    let fourth = content_of(2600, 13);
    overwrite_in_process(&mut pool, &second, InstanceGeneration(1));
    overwrite_in_process(&mut pool, &third, InstanceGeneration(1));
    overwrite_in_process(&mut pool, &fourth, InstanceGeneration(1));
    let mut reopened = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut reopened,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退到 A");
    pool.devices = Some(reopened);
    pool.allocator = rolled_back.allocator.clone();
    pool.output = rolled_back.current.clone();
    println!(
        "X1-A 回退：新实例 {:?}、D 的 (txg, jsn) = ({}, {})、暖机 {:?}",
        rolled_back.output.instance,
        rolled_back.output.row_publish.root.checkpoint_txg.0,
        rolled_back.output.row_publish.record.counter,
        rolled_back
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| (publish.root.checkpoint_txg.0, publish.record.counter))
            .collect::<Vec<_>>()
    );
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let records = singlefs_core::recovery::scan_journal(&image, &superblock);
    println!(
        "X1-A 环里的记录 (实例, jsn, txg)：{:?}",
        records
            .values()
            .map(|record| (
                record.instance.0,
                record.counter,
                record.checkpoint_txg.0
            ))
            .collect::<Vec<_>>()
    );
    let before = recover(&image, JournalPolicy::Consult);
    println!(
        "X1-A 零故障恢复：{}",
        match &before.outcome {
            RecoveryOutcome::FileRead { root, content } =>
                format!("根 {root:?}、{} 字节、= 第一版 {}", content.len(), *content == common::file_content()),
            other => format!("{other:?}"),
        }
    );

    let mut devices = pool.reopen_recorded();
    for txg in [4u64, 5, 6, 7, 8] {
        damage_root_slot(&mut devices, txg);
    }
    pool.devices = Some(devices);
    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    println!(
        "X1-A 五个故障之后：所选根 {:?}、施加 {} 条、{}",
        report.effective_root.map(|root| (root.0 .0, root.1 .0)),
        report.journal.prefix_applied,
        match &report.outcome {
            RecoveryOutcome::FileRead { root, content } => format!(
                "读回 根 {root:?}、{} 字节、= 第四版 {}",
                content.len(),
                *content == fourth
            ),
            other => format!("{other:?}"),
        }
    );
    assert_eq!(report.journal.prefix_applied, 3, "jsn 4、5、6 三条全施加");
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: fourth
        },
        "所选根报的是回退目标 A，读回的却是被抛弃的第四版"
    );
}

/// X3-B：`effective_rollback_floor` 只把「有可读根的盘」算进 min ⇒ 故障越多反而越安全。
/// 只翻掉盘 1 上那一条带 F = 11 的根（1 个字节）⇒ F_生效 = min(11, 0) = 0；把盘 1 上全部 6 条根都翻掉（6 个字节）
/// ⇒ 盘 1 不进 min，F_生效 = 11。同一个 checker 在前一格红、后一格绿。
#[test]
fn x3_b_damaging_only_the_carrier_is_worse_than_damaging_every_root_on_that_device() {
    for (label, damaged) in [
        ("只坏载体（1 个故障）", vec![16u64]),
        ("盘 1 全部根都坏（6 个故障）", vec![1u64, 4, 7, 10, 13, 16]),
    ] {
        let mut pool = build_through_rollback("opus-r2-x3b");
        four_overwrites_after_the_rollback(&mut pool);
        raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
        let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
        assert_eq!(reuse.data_pointer.locations[0].slot, SlotNumber(50178));
        let mut devices = pool.reopen_recorded();
        for txg in &damaged {
            damage_root_slot(&mut devices, *txg);
        }
        let mounted = mount_writable(&parameters(), &mut devices).expect("重开");
        pool.devices = Some(devices);
        pool.allocator = mounted.allocator.clone();
        pool.output = mounted.current.clone();
        let violated: Vec<&str> = check_pool_image(&pool.memory_pool())
            .into_iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| name)
            .collect();
        println!(
            "X3-B {label}：新实例的根写 F = {}、违例 {:?}",
            mounted.output.row_publish.root.rollback_floor.0, violated
        );
    }
}

/// X3-C：F_生效 回落之后，第 0 代根与 A 又回到回退候选集，而它们引用的单元已被 E 合法盖掉——退过去会怎样。
#[test]
fn x3_c_rolling_back_onto_a_root_whose_units_were_legally_reused() {
    for target in [
        RollbackTarget {
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
        },
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    ] {
        let mut pool = build_through_rollback("opus-r2-x3c");
        four_overwrites_after_the_rollback(&mut pool);
        raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
        overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
        let mut devices = pool.reopen_recorded();
        damage_root_slot(&mut devices, 16);
        let attempt = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        pool.devices = Some(devices);
        println!(
            "X3-C 退到 ({}, {})：{}",
            target.instance.0,
            target.checkpoint_txg.0,
            match &attempt {
                Ok(mounted) => format!("成功，新实例 {:?}", mounted.output.instance),
                Err(error) => format!("{error:?}"),
            }
        );
    }
}

/// X3-D：F 生效期间多复用一步（E 拿 50178、再发一版拿 50180 = A 的数据单元），然后翻掉盘 1 的载体。
/// F_生效 回到 0 ⇒ A 回到候选集，而 A 的数据单元已经被合法复用。
#[test]
fn x3_d_rolling_back_to_a_root_whose_data_unit_was_already_reused() {
    let mut pool = build_through_rollback("opus-r2-x3d");
    four_overwrites_after_the_rollback(&mut pool);
    raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    let e = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    let f = overwrite_in_process(&mut pool, &content_of(2100, 37), InstanceGeneration(3));
    println!(
        "X3-D E 落 {}、下一版落 {}",
        e.data_pointer.locations[0].slot.0, f.data_pointer.locations[0].slot.0
    );
    let before: Vec<&str> = check_pool_image(&pool.memory_pool())
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| name)
        .collect();
    println!("X3-D 翻字节之前的违例：{before:?}");
    let mut devices = pool.reopen_recorded();
    damage_root_slot(&mut devices, 16);
    let attempt = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    );
    pool.devices = Some(devices);
    match &attempt {
        Ok(mounted) => {
            pool.allocator = mounted.allocator.clone();
            pool.output = mounted.current.clone();
            let report = recover(&pool.memory_pool(), JournalPolicy::Consult);
            println!(
                "X3-D 退到 A 成功，新实例 {:?}；冷启动读回 {}",
                mounted.output.instance,
                match &report.outcome {
                    RecoveryOutcome::FileRead { root, content } => format!(
                        "根 {root:?}、{} 字节、= A 的内容 {}",
                        content.len(),
                        *content == common::file_content()
                    ),
                    other => format!("{other:?}"),
                }
            );
        }
        Err(error) => println!("X3-D 退到 A 被拒：{error:?}"),
    }
}

/// X3-B 的对照：同样翻掉盘 1 全部 6 条根，但不做 E 那次复用——I-3.1 还红不红（追 X3-B 第二格那条违例的来历）。
#[test]
fn x3_b_control_damaging_every_root_on_device_one_without_any_reuse() {
    let mut pool = build_through_rollback("opus-r2-x3b-control");
    four_overwrites_after_the_rollback(&mut pool);
    raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    let mut devices = pool.reopen_recorded();
    for txg in [1u64, 4, 7, 10, 13, 16] {
        damage_root_slot(&mut devices, txg);
    }
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted.current.clone();
    let violated: Vec<(&str, InvariantVerdict)> = check_pool_image(&pool.memory_pool())
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .collect();
    println!(
        "X3-B 对照（无复用、6 个故障）：新实例的根写 F = {}、违例 {:?}",
        mounted.output.row_publish.root.rollback_floor.0, violated
    );
}
