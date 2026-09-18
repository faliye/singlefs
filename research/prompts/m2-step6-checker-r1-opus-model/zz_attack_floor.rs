//! 攻方腿 Y1 / Y5（副本专用，不入库）：checker 的候选集用最新根自己带的 F，
//! 而 D16 已定项 1 的 F_生效 = 各幸存盘所带 F 最大值的最小值。
//! 只有一块盘落上带新 F 的根时，两者不同：条款仍护着 F 之下的根，checker 已经不看它们。
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
use singlefs_core::recovery::{choose_superblock, effective_rollback_floor, readable_roots};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};

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

fn raise_floor(pool: &mut BuiltPool, new_floor: CheckpointTxg) -> Result<RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    );
    pool.output = current;
    raised
}

#[test]
fn the_checker_stops_looking_at_roots_that_are_still_rollback_candidates_when_only_one_device_carries_the_new_floor(
) {
    let mut pool = build_through_rollback("zz-floor");
    for seed in [17usize, 19, 23, 29] {
        overwrite_in_process(&mut pool, &content_of(3000 + seed, seed), InstanceGeneration(3));
    }
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    // 合法的崩溃状态：第二条带新 F 的根那一槽没写成（层 0 枚举里「根槽写到一半」的一格）。
    let mut devices = pool.reopen_recorded();
    let target = target_for_publish(second_carrier);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = slot_offset(target, 4096);
    {
        let (_, recorded) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == device)
            .expect("那块盘");
        let mut bytes = vec![0u8; 4096];
        recorded.read_at(offset, &mut bytes).expect("读根槽");
        bytes[100] ^= 0xff;
        recorded
            .write_at(offset, &bytes, WriteDurability::Plain)
            .expect("打掉第二条载体根");
    }
    pool.devices = Some(devices);
    // 再发布一次：E 落回 50178——roots txg 0 / 1 / 2 的树表就在那一槽。
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(reuse.data_pointer.locations[0].slot, SlotNumber(50178));
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let effective = effective_rollback_floor(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let roots = readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let newest = roots
        .iter()
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .expect("根");
    println!(
        "F_生效 = {:?}；最新根 txg {:?} 自己带的 F = {:?}",
        effective, newest.checkpoint_txg, newest.rollback_floor
    );
    let below: Vec<u64> = roots
        .iter()
        .filter(|root| root.checkpoint_txg < newest.rollback_floor)
        .map(|root| root.checkpoint_txg.0)
        .collect();
    println!("txg 低于最新根 F 的根：{below:?}（按 F_生效 它们都还是回退候选）");
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    println!("判违例的：{violated:?}");
    for (invariant, verdict) in &verdicts {
        println!("  {invariant}: {verdict:?}");
    }
    // 证明那些根真是回退候选：退到 txg 2（暖机根，实例 1，不被回退行抛弃）。
    let mut reopened = pool.reopen_recorded();
    let rolled = mount_rollback(
        &parameters(),
        &mut reopened,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(2),
        },
        ShadowLedger::On,
    );
    pool.devices = Some(reopened);
    println!("退到 (1, 2) 的结果：{:?}", rolled.map(|done| done.output.instance));
}

/// 对照：同一段历史、不打掉第二条载体根（F_生效 = 11，条款与 checker 的候选集一致）。
#[test]
fn control_without_damaging_the_second_carrier() {
    let mut pool = build_through_rollback("zz-floor-control");
    for seed in [17usize, 19, 23, 29] {
        overwrite_in_process(&mut pool, &content_of(3000 + seed, seed), InstanceGeneration(3));
    }
    raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(reuse.data_pointer.locations[0].slot, SlotNumber(50178));
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    println!(
        "对照：F_生效 = {:?}",
        effective_rollback_floor(
            &image,
            &superblock.region_devices,
            &superblock.geometry,
            &superblock.filesystem_identifier,
        )
    );
    for (invariant, verdict) in check_pool_image(&image) {
        if verdict != InvariantVerdict::Holds {
            println!("  对照 {invariant}: {verdict:?}");
        }
    }
    println!("对照跑完");
}

/// Y2 探针：回退 + 四次覆盖写之后（F = 0，环里每条根都是候选），哪几条根共用同一个树表单元。
#[test]
fn probe_shared_tree_tables_after_rollback() {
    let mut pool = build_through_rollback("zz-shared");
    for seed in [17usize, 19, 23, 29] {
        overwrite_in_process(&mut pool, &content_of(3000 + seed, seed), InstanceGeneration(3));
    }
    let image = pool.memory_pool();
    let chosen = singlefs_checker::image::chosen_superblocks(&image);
    let geometry = chosen[0].1.clone().expect("超级块").1;
    for (region, slot, root) in singlefs_checker::image::valid_roots(&image, &geometry) {
        let tree_table = singlefs_checker::image::parse_node_pointer(&root.record_bytes[36..122]);
        println!(
            "区域 {region} 槽 {slot}：txg {} 实例 {} F {} 树表槽 {} 校验和 {:#x}",
            root.checkpoint_txg,
            root.instance,
            root.rollback_floor,
            tree_table.locations[0].slot,
            tree_table.locations[0].checksum
        );
    }
}
