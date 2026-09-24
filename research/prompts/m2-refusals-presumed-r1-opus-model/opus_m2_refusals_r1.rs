//! 云端攻方腿（轮 m2-refusals-presumed-r1）的装置：只在副本上跑，不入库。
//! R1 / R2 / R3 / R4 / Q1 五格各造一条「零故障可达或只带一次普通崩溃」的历史，看今天的 `crates/` 会不会拒。

mod common;

use common::{
    build_pool, disk_snapshot, file_content, format_pool, parameters, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{choose_system_configuration, readable_roots, scan_journal};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES};

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

/// R1 第一条历史：mkfs 之后第一次可写挂载崩在取号的两次系统配置写之后（一次普通崩溃，盘上没有任何实例 1 写出的根、记录或单元）。
/// 之后每一次可写挂载都被 `InstanceRowsOnVersionWithoutFileUnsupported` 拒，盘上逐字节不变——这个池永远挂不上了。
#[test]
fn r1_a_crash_right_after_the_very_first_acquisition_bricks_a_brand_new_pool() {
    let mut formatted = format_pool("opus-r1-brick");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        assert_eq!(
            acquire_instance(&mut writer).expect("取号"),
            InstanceGeneration(1)
        );
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    // 崩溃点上盘面的事实：环里一条记录都没有，根环里只有 mkfs 的第 0 代根——实例 1 什么都没写出来。
    {
        let image = formatted.memory_pool();
        let system_configuration = choose_system_configuration(&image).expect("系统配置");
        assert!(
            scan_journal(&image, &system_configuration).is_empty(),
            "实例 1 一条记录都没写"
        );
        let roots = readable_roots(
            &image,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
        );
        assert!(
            roots
                .iter()
                .all(|root| root.instance == InstanceGeneration(0)),
            "根环里只有 mkfs 的第 0 代根：{:?}",
            roots
                .iter()
                .map(|root| (root.instance, root.checkpoint_txg))
                .collect::<Vec<_>>()
        );
    }
    for attempt in 1..=3 {
        let mut devices = formatted.reopen_recorded();
        let refused = mount_writable(&parameters(), &mut devices);
        formatted.devices = Some(devices);
        assert!(
            matches!(
                refused,
                Err(MountError::InstanceRowsOnVersionWithoutFileUnsupported {
                    chosen_root: RollbackTarget {
                        instance: InstanceGeneration(0),
                        checkpoint_txg: CheckpointTxg(0)
                    },
                    first_row_instance: InstanceGeneration(1),
                    instance_to_acquire: InstanceGeneration(2),
                })
            ),
            "第 {attempt} 次可写挂载：{:?}",
            refused.as_ref().err()
        );
        assert_eq!(
            disk_snapshot(&formatted.memory_pool(), &formatted.stream),
            before,
            "第 {attempt} 次被拒之后盘上逐字节不变"
        );
    }
    // 只读挂载这条退路走得通吗：报出来，给报告用。
    let image = formatted.memory_pool();
    let read_only = mount_read_only(&image);
    println!(
        "R1 只读挂载: {}",
        match &read_only {
            Ok(_) => "Ok".to_string(),
            Err(failure) => format!("{failure:?}"),
        }
    );
}

/// R4：只做过 mkfs 的池可写挂载一次、不写文件、进程正常退出（零故障），下一次可写挂载被拒——拒它的是 R1 那个成员，不是形状判定。
#[test]
fn r4_the_writable_mount_of_a_formatted_pool_is_honoured_only_once_and_the_refusal_is_r1() {
    let mut formatted = format_pool("opus-r4-second-mount");
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(1));
    formatted.devices = Some(devices);
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    for attempt in 1..=2 {
        let mut devices = formatted.reopen_recorded();
        let refused = mount_writable(&parameters(), &mut devices);
        formatted.devices = Some(devices);
        assert!(
            matches!(
                refused,
                Err(MountError::InstanceRowsOnVersionWithoutFileUnsupported {
                    chosen_root: RollbackTarget {
                        instance: InstanceGeneration(1),
                        checkpoint_txg: CheckpointTxg(2)
                    },
                    first_row_instance: InstanceGeneration(1),
                    instance_to_acquire: InstanceGeneration(2),
                })
            ),
            "第 {attempt} 次重开：{:?}",
            refused.as_ref().err()
        );
        assert_eq!(
            disk_snapshot(&formatted.memory_pool(), &formatted.stream),
            before,
            "第 {attempt} 次被拒之后盘上逐字节不变"
        );
    }
}

/// R4 的边界：同一个池，第一次可写挂载之后**不退出进程**、接着发第一个文件版本，此后重开就一切正常。
/// 拒与不拒的差别只在「进程有没有活到写出第一个文件版本」。
#[test]
fn r4_the_same_pool_is_fine_when_the_first_file_version_lands_in_the_mounting_process() {
    let mut formatted = format_pool("opus-r4-first-file-in-process");
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
    let instance = mounted.output.instance;
    let last_record_bytes = mounted.current.record_bytes().to_vec();
    let mut allocator = mounted.allocator;
    let content = file_content();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &formatted.genesis.root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &last_record_bytes,
        )
        .expect("第一个文件版本");
    }
    formatted.devices = Some(devices);
    let mut devices = formatted.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices);
    formatted.devices = Some(devices);
    assert!(
        remounted.is_ok(),
        "写出第一个文件版本之后重开就挂得上：{:?}",
        remounted.as_ref().err().map(|error| format!("{error:?}"))
    );
}

/// Q1：mkfs 与第一个事务在同一个进程里（零故障），之后这个进程一直活着、连着覆盖写六次——每一次抬 F 都被
/// `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion` 拒。拒的理由是「现行版本的内存表示里没有实例表单元」，
/// 而盘上的实例表一直在（根记录的 `instance_table` 指针指着 mkfs 写的那一片）。
#[test]
fn q1_raising_the_floor_is_refused_for_the_whole_life_of_the_pool_creating_process() {
    let mut pool = build_pool("opus-q1-raise-forever");
    let refused_at_once = {
        let mut current = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let raised = raise_rollback_floor(
            &parameters(),
            devices,
            &mut pool.allocator,
            &mut current,
            CheckpointTxg(1),
            ShadowLedger::On,
        );
        pool.output = current;
        raised
    };
    assert!(
        matches!(
            refused_at_once,
            Err(MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion)
        ),
        "第一个事务之后立刻抬 F：{:?}",
        refused_at_once.as_ref().err()
    );
    // 盘上的实例表在：根记录指着 mkfs 写的那一片（诞生 txg 0）。
    assert_eq!(
        pool.output.root.instance_table.head.birth_txg,
        CheckpointTxg(0),
        "现行那条根仍指着 mkfs 写的实例表"
    );
    for seed in [3usize, 5, 7, 11, 13, 17] {
        overwrite_in_process(&mut pool, &content_of(3000 + seed, seed), InstanceGeneration(1));
    }
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(9),
        "六次覆盖写之后 txg 9"
    );
    for floor in [CheckpointTxg(1), CheckpointTxg(4), CheckpointTxg(8)] {
        let refused = {
            let mut current = pool.output.clone();
            let devices = pool.devices.as_mut().expect("镜像还开着");
            let raised = raise_rollback_floor(
                &parameters(),
                devices,
                &mut pool.allocator,
                &mut current,
                floor,
                ShadowLedger::On,
            );
            pool.output = current;
            raised
        };
        assert!(
            matches!(
                refused,
                Err(MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion)
            ),
            "抬到 {floor:?}：{:?}",
            refused.as_ref().err()
        );
    }
    // 同一个池，重开一次（写行那次发布重写实例表）之后，同一个目标抬得动。
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("重开之后现行那一版带文件");
    for seed in [19usize, 23, 29] {
        overwrite_in_process(&mut pool, &content_of(2500 + seed, seed), InstanceGeneration(2));
    }
    let raised = {
        let mut current = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let raised = raise_rollback_floor(
            &parameters(),
            devices,
            &mut pool.allocator,
            &mut current,
            CheckpointTxg(8),
            ShadowLedger::On,
        );
        pool.output = current;
        raised
    };
    println!(
        "Q1 重开之后抬 F 到 8: {}",
        match &raised {
            Ok(raised) => format!(
                "Ok(ceiling={:?}, publishes={}, reclaimed={})",
                raised.ceiling,
                raised.publishes.len(),
                raised.reclaimed.len()
            ),
            Err(error) => format!("{error:?}"),
        }
    );
    assert!(
        !matches!(
            raised,
            Err(MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion)
        ),
        "重开之后这一格不再拒"
    );
}

/// R2：一条零故障的历史上回退到树表 0 条的三条根（mkfs 的第 0 代根、第一个事务的两条暖机根）各被拒成什么，
/// 以及被拒之后盘上逐字节变没变。
#[test]
fn r2_rolling_back_to_the_mkfs_root_or_a_warm_up_root_is_refused() {
    let mut pool = build_pool("opus-r2-rollback");
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("重开之后现行那一版带文件");
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    for target in [
        RollbackTarget {
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
        },
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(1),
        },
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(2),
        },
    ] {
        let mut devices = pool.reopen_recorded();
        let refused = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        pool.devices = Some(devices);
        println!(
            "R2 回退到 {target:?}: {}",
            match &refused {
                Ok(_) => "Ok".to_string(),
                Err(error) => format!("{error:?}"),
            }
        );
        assert!(refused.is_err(), "回退到 {target:?} 没被拒");
        assert_eq!(
            disk_snapshot(&pool.memory_pool(), &pool.stream),
            before,
            "回退到 {target:?} 被拒之后盘上逐字节不变"
        );
    }
}

/// R3：所选根有文件、而环里一条记录都读不出要几处独立故障——把两块盘上的记录槽逐条清零。
/// 留一条时照样挂得上，全抹光才撞上 `FileVersionWithoutAnyJournalRecord`。
#[test]
fn r3_how_many_independent_faults_it_takes_to_reach_the_refusal() {
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let erase = |pool: &mut BuiltPool, counters: &[u64]| {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for (_, device) in devices.iter_mut() {
            for counter in counters {
                device
                    .write_zeroes_at(
                        DeviceOffsetInBytes(ring_start + (counter - 1) * JOURNAL_RECORD_BYTES),
                        JOURNAL_RECORD_BYTES,
                    )
                    .expect("清零");
            }
            device.barrier().expect("屏障");
        }
    };
    let counters_of = |pool: &BuiltPool| -> Vec<u64> {
        let image = pool.memory_pool();
        let system_configuration = choose_system_configuration(&image).expect("系统配置");
        scan_journal(&image, &system_configuration)
            .values()
            .map(|record| record.counter)
            .collect()
    };

    // 臂一：留最后一条，其余全抹——挂得上。
    let mut pool = build_pool("opus-r3-keep-one");
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let counters = counters_of(&pool);
    println!("R3 环里自证过的记录: {counters:?}");
    assert!(counters.len() >= 2);
    let all_but_last: Vec<u64> = counters[..counters.len() - 1].to_vec();
    erase(&mut pool, &all_but_last);
    let mut devices = pool.reopen_recorded();
    let kept_one = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(devices);
    println!(
        "R3 抹掉 {} 条记录（{} 处独立故障）、留一条之后可写挂载: {}",
        all_but_last.len(),
        all_but_last.len() * 2,
        match &kept_one {
            Ok(_) => "Ok".to_string(),
            Err(error) => format!("{error:?}"),
        }
    );

    // 臂二：全抹光。
    let mut pool = build_pool("opus-r3-erase-all");
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let counters = counters_of(&pool);
    erase(&mut pool, &counters);
    let mut devices = pool.reopen_recorded();
    let erased_all = mount_writable(&parameters(), &mut devices);
    pool.devices = Some(devices);
    println!(
        "R3 抹掉全部 {} 条记录（{} 处独立故障）之后可写挂载: {}",
        counters.len(),
        counters.len() * 2,
        match &erased_all {
            Ok(_) => "Ok".to_string(),
            Err(error) => format!("{error:?}"),
        }
    );
    assert!(
        matches!(erased_all, Err(MountError::FileVersionWithoutAnyJournalRecord)),
        "全抹光才撞上那个拒绝：{:?}",
        erased_all.as_ref().err().map(|error| format!("{error:?}"))
    );
}

/// 探针（两种跑法都跑）：只做过 mkfs 的池挂第二次会发生什么。未打补丁时它撞 R1 的拒绝；
/// 把 R1 与 R4 那两个拒绝关掉（副本上的补丁 `two-refusals-off.patch`）再跑，看真正挡住第二次挂载的是谁。
#[test]
fn probe_what_actually_blocks_the_second_mount_of_a_formatted_pool() {
    let mut formatted = format_pool("opus-probe-second-mount");
    let mut devices = formatted.reopen_recorded();
    let first = mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
    println!(
        "probe 第一次挂载 Ok: instance={:?} row_publish_txg={:?} warm_ups={}",
        first.output.instance,
        first.output.row_publish.root().checkpoint_txg,
        first.output.warm_up_publishes.len()
    );
    formatted.devices = Some(devices);
    let mut devices = formatted.reopen_recorded();
    let second = mount_writable(&parameters(), &mut devices);
    match &second {
        Ok(mounted) => println!(
            "probe 第二次挂载 Ok: instance={:?} row_publish_txg={:?} warm_ups={} rows={:?}",
            mounted.output.instance,
            mounted.output.row_publish.root().checkpoint_txg,
            mounted.output.warm_up_publishes.len(),
            mounted.output.rows_written
        ),
        Err(error) => println!("probe 第二次挂载 {error:?}"),
    }
    if let Ok(mounted) = second {
        let instance = mounted.output.instance;
        let last_record_bytes = mounted.current.record_bytes().to_vec();
        let mut allocator = mounted.allocator;
        let content = file_content();
        let published = {
            let publish_parameters = parameters();
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_first_file(
                &mut writer,
                &mut allocator,
                &formatted.genesis.root,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &last_record_bytes,
            )
        };
        println!(
            "probe 第二次挂载之后发第一个文件版本: {}",
            match &published {
                Ok(output) => format!("Ok(txg={:?})", output.root.checkpoint_txg),
                Err(error) => format!("{error:?}"),
            }
        );
    }
    formatted.devices = Some(devices);
    let verdicts = check_pool_image(&formatted.memory_pool());
    let violations: Vec<String> = verdicts
        .iter()
        .filter(|(_, verdict)| {
            !matches!(
                verdict,
                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)
            )
        })
        .map(|(invariant, verdict)| format!("{invariant}:{verdict:?}"))
        .collect();
    println!("probe checker 违例: {violations:?}");
}

/// R1 的池还有没有别的门：mkfs 之后崩在取号之后的那个池，回退到 mkfs 的第 0 代根同样被拒（R2 那个成员）。
#[test]
fn r1_the_bricked_pool_has_no_rollback_door_either() {
    let mut formatted = format_pool("opus-r1-no-rollback-door");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        acquire_instance(&mut writer).expect("取号");
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    let mut devices = formatted.reopen_recorded();
    let refused_mount = mount_writable(&parameters(), &mut devices);
    let refused_rollback = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(0),
            checkpoint_txg: CheckpointTxg(0),
        },
        ShadowLedger::On,
    );
    formatted.devices = Some(devices);
    println!(
        "R1 两扇门: 可写挂载 {:?} / 回退到 (0, 0) {:?}",
        refused_mount.as_ref().err().map(|error| format!("{error:?}")),
        refused_rollback
            .as_ref()
            .err()
            .map(|error| format!("{error:?}"))
    );
    assert!(refused_mount.is_err() && refused_rollback.is_err());
    assert_eq!(
        disk_snapshot(&formatted.memory_pool(), &formatted.stream),
        before,
        "两扇门都在任何写之前拒"
    );
}

/// Q1 的探针（两种跑法都跑）：创建池的那个进程里抬 F。未打补丁时撞 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`；
/// 把「实例表从现行根的指针上读」那个改法打上（副本补丁 `raise-reads-table-from-root.patch`）再跑，看抬得动抬不动、checker 红不红。
#[test]
fn probe_q1_raising_the_floor_in_the_pool_creating_process() {
    let mut pool = build_pool("opus-probe-q1-raise");
    for seed in [3usize, 5, 7, 11, 13, 17] {
        overwrite_in_process(
            &mut pool,
            &content_of(3000 + seed, seed),
            InstanceGeneration(1),
        );
    }
    println!(
        "probe Q1 现行 txg={:?}，根指着的实例表诞生 txg={:?}",
        pool.output.root.checkpoint_txg, pool.output.root.instance_table.head.birth_txg
    );
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let raised = {
        let mut current = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let raised = raise_rollback_floor(
            &parameters(),
            devices,
            &mut pool.allocator,
            &mut current,
            CheckpointTxg(4),
            ShadowLedger::On,
        );
        pool.output = current;
        raised
    };
    let after = disk_snapshot(&pool.memory_pool(), &pool.stream);
    println!(
        "probe Q1 抬 F 前后盘上快照相等: {}（录制流步数 {} → {}）",
        after == before,
        before.recorded_operations,
        after.recorded_operations
    );
    println!(
        "probe Q1 抬 F 到 4: {}",
        match &raised {
            Ok(raised) => format!(
                "Ok(ceiling={:?}, publishes={}, reclaimed={})",
                raised.ceiling,
                raised.publishes.len(),
                raised.reclaimed.len()
            ),
            Err(error) => format!("{error:?}"),
        }
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    let violations: Vec<String> = verdicts
        .iter()
        .filter(|(_, verdict)| {
            !matches!(
                verdict,
                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)
            )
        })
        .map(|(invariant, verdict)| format!("{invariant}:{verdict:?}"))
        .collect();
    println!("probe Q1 checker 违例: {violations:?}");
}
