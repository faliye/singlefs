//! 副本上的攻方腿实验（m2-step3-code-r1，Opus 攻方）。不进原仓。
mod common;

use common::{build_pool, image_path, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, FileBackedBlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
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
    let parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写")
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// X1-A：所选根自己那条记录读不出（两份都撕了）+ 紧接着的一条也读不出 ⇒ 代码把更后面那条接上，断号即止失效。
#[test]
fn x1_gap_is_skipped_when_the_chosen_roots_own_record_is_also_unreadable() {
    let mut pool = build_pool("opus-x1-gap");
    let second = overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    pool.output = second;
    let third = overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(1));
    pool.output = third;
    let fourth = overwrite_in_process(&mut pool, &content_of(3300, 17), InstanceGeneration(1));
    assert_eq!(
        (fourth.root.checkpoint_txg, fourth.record.counter),
        (CheckpointTxg(6), 6)
    );
    let mut image = pool.memory_pool();
    // txg 5 / 6 的根槽坏掉 ⇒ 所选根是 (1, 4)。
    for txg in [5u64, 6] {
        let target = target_for_publish(CheckpointTxg(txg));
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    // 所选根自己那条记录（jsn 4）与紧接着的 jsn 5 两份都坏掉。
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for counter in [4u64, 5] {
            image.flip_byte(device, record_offset(counter, JOURNAL_RING_DEFAULT_BYTES), 300);
        }
    }
    let report = recover(&image, JournalPolicy::Consult);
    println!("X1-A report = {report:?}");
    println!(
        "X1-A effective_root = {:?} prefix_applied = {} above_water = {}",
        report.effective_root, report.journal.prefix_applied, report.journal.above_water
    );
    match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => {
            println!(
                "X1-A 读回的是 root = {root:?}，内容长度 {}（second=4100 third=2500 fourth=3300）",
                content.len()
            );
        }
        other => println!("X1-A outcome = {other:?}"),
    }
}

/// X2-A：只 mkfs、没发过文件版本的池，可写挂载报什么。
#[test]
fn x2_fresh_pool_cannot_be_mounted_writable() {
    let parameters = parameters();
    let mut paths = Vec::new();
    let mut devices: Vec<(DeviceIdentity, FileBackedBlockDevice)> = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path("opus-x2-fresh", device_number);
        let file = FileBackedBlockDevice::open_or_create(
            &path,
            common::IMAGE_BYTES,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像");
        devices.push((DeviceIdentity(device_number), file));
        paths.push(path);
    }
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let outcome = mount_writable(&parameters, &mut devices);
    match outcome {
        Ok(_) => println!("X2-A 可写挂载成功"),
        Err(MountError::NoPublishedVersion) => println!("X2-A NoPublishedVersion"),
        Err(other) => println!("X2-A 别的错：{other:?}"),
    }
    for path in paths {
        let _ = std::fs::remove_file(path);
    }
}

/// X1-B / X4-A：同一段历史上做可写挂载 —— 写出的行是 (1, T, W) 里的哪一对。
#[test]
fn x1b_mount_after_the_gap_writes_a_row_that_is_not_a_prefix() {
    use singlefs_core::block_device::WriteDurability;
    let mut pool = build_pool("opus-x1b-gap-mount");
    let second = overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    pool.output = second;
    let third = overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(1));
    pool.output = third;
    let fourth = overwrite_in_process(&mut pool, &content_of(3300, 17), InstanceGeneration(1));
    assert_eq!(
        (fourth.root.checkpoint_txg, fourth.record.transaction),
        (CheckpointTxg(6), 4)
    );
    let mut devices = pool.reopen_recorded();
    let mut damage = |device: DeviceIdentity, offset: singlefs_core::address::DeviceOffsetInBytes| {
        let (_, handle) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == device)
            .expect("盘在池里");
        let mut garbage = vec![0u8; 4096];
        garbage[0] = 0xff;
        handle
            .write_at(offset, &garbage, WriteDurability::Plain)
            .expect("改坏");
    };
    for txg in [5u64, 6] {
        let target = target_for_publish(CheckpointTxg(txg));
        damage(region_device(txg), slot_offset(target, 4096));
    }
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for counter in [4u64, 5] {
            damage(device, record_offset(counter, JOURNAL_RING_DEFAULT_BYTES));
        }
    }
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    println!(
        "X1-B chosen={:?} effective={:?} prefix_applied={} W={} rows={:?}",
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg
        ),
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        mounted.output.journal.prefix_applied,
        mounted.output.journal.maximum_applied_transaction,
        mounted.output.rows_written
    );
    pool.devices = Some(devices);
}
