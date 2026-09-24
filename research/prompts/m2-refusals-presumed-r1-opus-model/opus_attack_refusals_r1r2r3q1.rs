//! 云端攻方腿（m2-refusals-presumed-r1）R1 / R2 / R3 / Q1 的探针。**只在副本上跑，不进仓。**
//! 每条用例都打印出实际拿到的错误成员，判定靠 assert。

mod common;

use common::{
    build_pool, disk_snapshot, file_content, format_pool, parameters, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_of_root, readable_roots, recover, scan_journal,
    JournalPolicy, PoolReader, RecoveryOutcome,
};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

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

/// R1 / R2 合起来的那一格：只做过 mkfs 的池可写挂载一次、不写文件、**进程正常退出**（零故障），
/// 第二次可写挂载被 `InstanceRowsOnVersionWithoutFileUnsupported` 拒；再把环里每一条可读根逐个当回退目标试一遍，
/// 全被 `RollbackToVersionWithoutFileUnsupported` 拒 ⇒ 这个池再没有任何一条可写的出路。只读挂载照旧。
#[test]
fn opus_r1_second_writable_mount_of_a_formatted_pool_is_refused_and_no_rollback_target_is_an_exit() {
    let mut formatted = format_pool("opus-r1-second-mount");
    {
        let mut devices = formatted.reopen_recorded();
        let mounted = mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
        assert_eq!(mounted.output.instance, InstanceGeneration(1));
        println!(
            "[R1] 第一次可写挂载成功：实例 {}，写行那次 txg {}，暖机 {} 次",
            mounted.output.instance.0,
            mounted.output.row_publish.root().checkpoint_txg.0,
            mounted.output.warm_up_publishes.len()
        );
        formatted.devices = Some(devices);
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    let mut devices = formatted.reopen_recorded();
    let refused = mount_writable(&parameters(), &mut devices);
    formatted.devices = Some(devices);
    println!("[R1] 第二次可写挂载：{:?}", refused.as_ref().err());
    assert!(
        matches!(
            refused,
            Err(MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. })
        ),
        "第二次可写挂载要给实例 1 写行"
    );
    assert_eq!(
        disk_snapshot(&formatted.memory_pool(), &formatted.stream),
        before,
        "[R1] 拒得早：一个写都没发"
    );

    let image = formatted.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let roots = readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    println!("[R2] 环里可读根 {} 条", roots.len());
    for root in roots {
        let target = RollbackTarget {
            instance: root.instance,
            checkpoint_txg: root.checkpoint_txg,
        };
        let mut devices = formatted.reopen_recorded();
        let refused = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        formatted.devices = Some(devices);
        println!(
            "[R2] 回退到 ({}, {})：{:?}",
            root.instance.0,
            root.checkpoint_txg.0,
            refused.as_ref().err()
        );
        assert!(
            matches!(
                refused,
                Err(MountError::RollbackToVersionWithoutFileUnsupported(_))
            ),
            "环里每一条根都是树表 0 条"
        );
    }
    let cold = formatted.reopen_cold();
    let read_only = mount_read_only(&cold as &dyn PoolReader);
    println!(
        "[R1] 砖住之后的只读挂载：ok = {}，err = {:?}",
        read_only.is_ok(),
        read_only.as_ref().err()
    );
    drop(cold);
    drop(read_only);
    // 对照：一次都没挂载过的、刚 mkfs 的池，只读挂载给的是同一个结局（空池本来就读不出文件）。
    let mut baseline = format_pool("opus-r1-readonly-baseline");
    let baseline_cold = baseline.reopen_cold();
    let baseline_read_only = mount_read_only(&baseline_cold as &dyn PoolReader);
    println!(
        "[R1] 对照：刚 mkfs、没挂载过的池只读挂载：ok = {}，err = {:?}",
        baseline_read_only.is_ok(),
        baseline_read_only.as_ref().err()
    );
}

/// R2 单独那一格（零故障、池里有文件）：mkfs → 取号 1 → 暖机 txg 1、2 → 文件版本 A（txg 3）→ 重开取号 2（写行、暖机）；
/// 管理员回退到实例 1 的暖机根 (1, 1) / (1, 2)——它们在回退候选集里（不报候选排除），却被
/// `RollbackToVersionWithoutFileUnsupported` 拒；同一次挂载回退到 (1, 3) 成功，说明拒的只是「那一版树表 0 条」。
#[test]
fn opus_r2_rollback_to_a_warm_up_root_is_refused_though_it_is_inside_the_candidate_set() {
    let mut pool = build_pool("opus-r2-warm-up-target");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开取号 2");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("A 之后重开，现行那一版带文件");
    println!(
        "[R2] 第二次挂载写的行：{:?}",
        mounted
            .output
            .rows_written
            .iter()
            .map(|row| (
                row.instance.0,
                row.selected_root_txg.0,
                row.applied_transaction_high_water
            ))
            .collect::<Vec<_>>()
    );
    for txg in [1u64, 2] {
        let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
        let target = RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(txg),
        };
        let mut devices = pool.reopen_recorded();
        let refused = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        pool.devices = Some(devices);
        println!("[R2] 回退到 (1, {txg})：{:?}", refused.as_ref().err());
        assert!(
            matches!(
                refused,
                Err(MountError::RollbackToVersionWithoutFileUnsupported(_))
            ),
            "暖机根在候选集里、被「第一版不支持」拒"
        );
        assert_eq!(
            disk_snapshot(&pool.memory_pool(), &pool.stream),
            before,
            "[R2] 拒得早：一个写都没发"
        );
    }
    let mut devices = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    );
    pool.devices = Some(devices);
    println!("[R2] 对照：回退到 (1, 3) 成功 = {}", rolled_back.is_ok());
    assert!(rolled_back.is_ok(), "同一条路回退到有文件的根做得成");
}

/// R3：把两块盘上前 16 条记录槽清零（介质故障，不是崩溃——记一次「这一族要几次故障」的量），
/// 可写挂载报 `FileVersionWithoutAnyJournalRecord`；同一份镜像上冷启动读回照样把文件读出来。
#[test]
fn opus_r3_wiping_every_record_refuses_the_writable_mount_while_the_file_is_still_readable() {
    let mut pool = build_pool("opus-r3-wipe-ring");
    let content = file_content();
    drop(pool.devices.take());
    let mut wiped = 0usize;
    for path in &pool.paths {
        use std::io::{Seek, SeekFrom, Write};
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect("打开镜像");
        for counter in 1..=16u64 {
            let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
            file.seek(SeekFrom::Start(offset.0)).expect("seek");
            file.write_all(&vec![0u8; 4096]).expect("写零");
            wiped += 1;
        }
        file.sync_all().expect("sync");
    }
    println!("[R3] 抹掉的记录槽：{wiped} 个（两块盘各 16）");
    let cold = pool.reopen_cold();
    let system_configuration = choose_system_configuration(&cold as &dyn PoolReader).expect("系统配置");
    let records = scan_journal(&cold as &dyn PoolReader, &system_configuration);
    println!("[R3] 环里自证过的记录：{} 条", records.len());
    assert_eq!(records.len(), 0, "一条记录都读不出");
    let report = recover(&cold as &dyn PoolReader, JournalPolicy::Consult);
    match &report.outcome {
        RecoveryOutcome::FileRead { root, content: got } => {
            println!(
                "[R3] 冷启动仍读回 ({}, {}) 的内容 {} 字节，与写进去的一致 = {}",
                root.0 .0,
                root.1 .0,
                got.len(),
                *got == content
            );
            assert_eq!(*got, content, "文件原样读得回");
        }
        other => panic!("[R3] 冷启动没读回文件：{other:?}"),
    }
    drop(cold);
    let mut devices = pool.reopen_recorded();
    let refused = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(devices);
    println!("[R3] 可写挂载：{:?}", refused.as_ref().err());
    assert!(
        matches!(refused, Err(MountError::FileVersionWithoutAnyJournalRecord)),
        "有文件的根 + 环里 0 条记录"
    );
}

/// Q1：抬 F 的那道拒绝不只在「第一个事务刚发完」那一刻——mkfs 同一个进程里再覆盖写三次（txg 4、5、6）之后照样拒，
/// 而同一版的根指针指着的实例表在盘上读得出、解得开（拒的是「`units` 里没有」，不是「盘上没有」）。
#[test]
fn opus_q1_raising_the_floor_is_refused_after_in_process_overwrites_though_the_table_is_on_disk() {
    let mut pool = build_pool("opus-q1-raise-after-overwrites");
    for seed in [3usize, 5, 7] {
        overwrite_in_process(&mut pool, &content_of(4100, seed), InstanceGeneration(1));
    }
    println!(
        "[Q1] 现行版本 txg {}，units 里的角色：{:?}",
        pool.output.root.checkpoint_txg.0,
        pool.output
            .units
            .iter()
            .map(|unit| format!("{:?}", unit.identity))
            .collect::<Vec<_>>()
    );
    assert!(
        !pool
            .output
            .units
            .iter()
            .any(|unit| unit.identity == TransactionUnit::InstanceTable),
        "[Q1] 覆盖写之后现行版本里仍然没有实例表单元"
    );
    let image = pool.memory_pool();
    let table = instance_table_of_root(&image, &pool.output.root);
    println!(
        "[Q1] 同一版的根指针指着的实例表读得出、解得开 = {}，行数 {:?}",
        table.is_some(),
        table.as_ref().map(|table| table.rows.len())
    );
    assert!(table.is_some(), "实例表在盘上读得出");
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let refused = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(3),
        ShadowLedger::On,
    );
    println!("[Q1] 抬 F 到 3：{:?}", refused.as_ref().err());
    assert!(
        matches!(
            refused,
            Err(MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion)
        ),
        "三次覆盖写之后抬 F 照样被拒"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "[Q1] 拒得早：一个写都没发"
    );
}

/// R4 自己那个错误成员的一条零故障历史：在**同一对镜像**上用**同一个 fsid** 再做一次 mkfs
/// （今天的 mkfs 不把 768 MiB 的 journal 环写 0，见里程碑「并行线四」），旧池的记录原样留在环里、fsid 对得上 ⇒
/// 新池的第一次可写挂载算出的 first_txg / jsn 不是 1，被 `FormattedPoolMountNotShapedLikeTheFirstTransaction` 拒。
#[test]
fn opus_r4_second_mkfs_with_the_same_fsid_on_the_same_images_is_refused_on_the_very_first_mount() {
    let mut pool = build_pool("opus-r4-second-mkfs");
    let mut devices = pool.reopen_recorded();
    let genesis = singlefs_core::make_filesystem::make_filesystem(&parameters(), &mut devices)
        .expect("第二次 mkfs");
    println!(
        "[R4] 第二次 mkfs 做完：第 0 代根 txg {}，实例 {}",
        genesis.root.checkpoint_txg.0, genesis.root.instance.0
    );
    let system_configuration =
        choose_system_configuration(&devices as &dyn PoolReader).expect("系统配置");
    let records = scan_journal(&devices as &dyn PoolReader, &system_configuration);
    println!(
        "[R4] 新池环里仍自证得过的旧记录：{} 条，(实例, jsn) = {:?}",
        records.len(),
        records.keys().map(|(instance, counter)| (instance.0, *counter)).collect::<Vec<_>>()
    );
    let mounted = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(devices);
    match &mounted {
        Ok(mounted) => println!(
            "[R4] 新池第一次可写挂载做成了：实例 {}，写行那次 txg {}——这条形状没打中",
            mounted.output.instance.0,
            mounted.output.row_publish.root().checkpoint_txg.0
        ),
        Err(error) => println!("[R4] 新池第一次可写挂载：{error:?}"),
    }
    assert_eq!(
        records.len(),
        0,
        "mkfs 今天把 journal 环整环清零（C484，2026-09-22 定案）：同 fsid 重来也没有旧记录顶着"
    );
    assert!(
        mounted.is_ok(),
        "这条形状没打中：R4 那个错误成员在这条历史上够不着"
    );
}
