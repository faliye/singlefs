//! E153 S1 对拍（跑前登记第十一节停机条款 S1）：只打印，不断言（写范围只在草稿目录的仓副本，不改工作区 crates/）。
//! 四段历史 (a)(b)(c)(d) 照登记第十一节 S1 列的四段，照
//! `second_transaction_step_four_rollback.rs` / `second_transaction_step_five_reuse.rs` / `common/mod.rs`
//! 的调用顺序写；把它们已有的私有辅助函数原样抄一份（每个测试文件的辅助函数互相私有，抄的理由与那两个文件相同）。
//! 打印用 `E153S1` 开头的固定格式，供执行员逐项与模型比对。

mod common;

use common::{build_pool, disk_snapshot, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::Placement;
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{choose_superblock, readable_roots};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};

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

fn accounting_line(tag: &str, pool: &BuiltPool) {
    for device in &pool.allocator.devices {
        println!(
            "E153S1 tag={tag} device={:?} txg={} allocated={} free={} deferred={}",
            device.device,
            pool.output.root.checkpoint_txg.0,
            device.allocated_slots(),
            device.free_slots(),
            device.deferred_slots()
        );
    }
}

/// (a) 第一个事务的落点与 txg 3 那一行记账三项（K1、A6）。
#[test]
fn e153_s1_segment_a_first_transaction() {
    let pool = build_pool("e153-s1-a");
    println!(
        "E153S1 tag=a txg={} instance={} data_unit_slot={} extent_root_slot={} inode_leaf_slot={} inode_root_slot={}",
        pool.output.root.checkpoint_txg.0,
        pool.output.root.instance.0,
        pool.output.data_pointer.locations[0].slot.0,
        pool.output.unit(TransactionUnit::ExtentRoot).slot.0,
        pool.output.unit(TransactionUnit::InodeLeaf).slot.0,
        pool.output.unit(TransactionUnit::InodeRoot).slot.0
    );
    accounting_line("a", &pool);
}

/// (b) 回退段：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）、重开回退到 A 的根 (1,3)。
/// isolated_slots_per_device、回退 W（写行发布）的 txg、写的行。
#[test]
fn e153_s1_segment_b_rollback() {
    let mut pool = build_pool("e153-s1-b");
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1)); // B, txg4
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开取号 2");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    println!(
        "E153S1 tag=b_remount instance={} write_row_txg={} warm_up_count={}",
        mounted.output.instance.0,
        mounted.output.row_publish.root().checkpoint_txg.0,
        mounted.output.warm_up_publishes.len()
    );
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2)); // C, txg8
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
    let output = &rolled_back.output;
    println!(
        "E153S1 tag=b_rollback instance={} write_row_txg={} rows_written={:?}",
        output.instance.0,
        output.row_publish.root().checkpoint_txg.0,
        output
            .rows_written
            .iter()
            .map(|row| (row.instance.0, row.selected_root_txg.0, row.applied_transaction_high_water, row.is_rollback))
            .collect::<Vec<_>>()
    );
    for (device, isolated) in &output.isolated_slots_per_device {
        println!("E153S1 tag=b_isolated device={device:?} isolated={isolated}");
    }
    pool.devices = Some(reopened);
}

/// (c) 在 (b) 的回退之后再覆盖写四次（txg 11-14）、抬 F 到第一次释放代（txg 11）、复用发布。
/// 回收的落点、扣住期间发出的落点、之后复用的落点、每次发布的记账三项。
#[test]
fn e153_s1_segment_c_raise_floor_and_reuse() {
    let mut pool = build_pool("e153-s1-c");
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开取号 2");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
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
    .expect("回退到 A");
    pool.devices = Some(reopened);
    pool.allocator = rolled_back.allocator.clone();
    pool.output = rolled_back
        .current
        .file_version()
        .expect("回退到 A，现行那一版带文件")
        .clone();

    for seed in [17usize, 19, 23, 29] {
        let output = overwrite_in_process(&mut pool, &content_of(3000 + seed, seed), InstanceGeneration(3));
        println!("E153S1 tag=c_overwrite txg={}", output.root.checkpoint_txg.0);
        accounting_line("c_overwrite", &pool);
    }

    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(11),
        ShadowLedger::On,
    )
    .expect("抬 F 到 11");
    pool.output = current;
    println!(
        "E153S1 tag=c_raise_floor ceiling={} publish_count={} reclaimed_count={}",
        raised.ceiling.0,
        raised.publishes.len(),
        raised.reclaimed.len()
    );
    let reclaimed_slots: std::collections::BTreeSet<u64> = raised
        .reclaimed
        .iter()
        .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
        .collect();
    println!("E153S1 tag=c_reclaimed_slots slots={reclaimed_slots:?}");
    for publish in &raised.publishes {
        let withheld_during: Vec<u64> = publish
            .placements()
            .iter()
            .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
            .collect();
        println!("E153S1 tag=c_withhold_publish txg={} placements={withheld_during:?}", publish.root.checkpoint_txg.0);
    }
    accounting_line("c_after_raise", &pool);

    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    println!(
        "E153S1 tag=c_reuse txg={} data_unit_slot={}",
        reuse.root.checkpoint_txg.0,
        reuse.data_pointer.locations[0].slot.0
    );
    accounting_line("c_reuse", &pool);
}

/// (d) H1 基准格前 N+6 次发布（一次挂载之内转过一圈）：每次发布的记账三项与 checker 的 I-3.1/I-5.2 判定。
/// 基准格 S=8（`SUPERBLOCK`/根环参数是格式常量，不受这里控制；此段只做 N+6=30 次连续覆盖写，N 取根环槽位总数 24）。
#[test]
fn e153_s1_segment_d_steady_state() {
    let mut pool = build_pool("e153-s1-d"); // A: txg 3
    accounting_line("d", &pool);
    let image_now = pool.memory_pool();
    let superblock = choose_superblock(&image_now).expect("超级块");
    let verdicts = check_pool_image(&image_now);
    let i31 = verdicts.iter().find(|(name, _)| *name == "I-3.1").map(|(_, verdict)| verdict.clone());
    let i52 = verdicts.iter().find(|(name, _)| *name == "I-5.2").map(|(_, verdict)| verdict.clone());
    println!("E153S1 tag=d_checker txg={} i31={i31:?} i52={i52:?}", pool.output.root.checkpoint_txg.0);
    drop(superblock);

    // N + 6 = 30 次连续覆盖写（N = 24，第一版根环 3 区域 × 8 槽）。A 本身算第 1 次，这里再做 29 次。
    for step in 1..30u64 {
        let seed = 37 + step as usize;
        let output = overwrite_in_process(&mut pool, &content_of(1500 + seed, seed), InstanceGeneration(1));
        accounting_line("d", &pool);
        let diagnostic_image = pool.memory_pool();
        let diagnostic_superblock = choose_superblock(&diagnostic_image).expect("超级块");
        let mut diagnostic_roots: Vec<u64> = readable_roots(
            &diagnostic_image,
            &diagnostic_superblock.region_devices,
            &diagnostic_superblock.geometry,
            &diagnostic_superblock.filesystem_identifier,
        )
        .iter()
        .map(|root| root.checkpoint_txg.0)
        .collect();
        diagnostic_roots.sort_unstable();
        println!("E153S1 tag=d_readable_roots txg={} roots={diagnostic_roots:?}", output.root.checkpoint_txg.0);
        let image_now = pool.memory_pool();
        let verdicts = check_pool_image(&image_now);
        let i31 = verdicts.iter().find(|(name, _)| *name == "I-3.1").map(|(_, verdict)| verdict.clone());
        let i52 = verdicts.iter().find(|(name, _)| *name == "I-5.2").map(|(_, verdict)| verdict.clone());
        println!(
            "E153S1 tag=d_checker txg={} i31={i31:?} i52={i52:?}",
            output.root.checkpoint_txg.0
        );
        if i31 != Some(InvariantVerdict::Holds) {
            println!("E153S1 tag=d_i31_violation txg={} detail={i31:?}", output.root.checkpoint_txg.0);
        }
        if i52 != Some(InvariantVerdict::Holds) {
            println!("E153S1 tag=d_i52_violation txg={} detail={i52:?}", output.root.checkpoint_txg.0);
        }
    }
    let final_readable = readable_roots(
        &pool.memory_pool(),
        &choose_superblock(&pool.memory_pool()).expect("超级块").region_devices,
        &choose_superblock(&pool.memory_pool()).expect("超级块").geometry,
        &choose_superblock(&pool.memory_pool()).expect("超级块").filesystem_identifier,
    );
    println!("E153S1 tag=d_final_readable_root_count count={}", final_readable.len());
    let _ = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let _ = file_content();
    let _ = DeviceIdentity(0);
    let _placement_probe: Option<Placement> = None;
    let _ = _placement_probe;
}
