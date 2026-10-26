//! 量具（只在草稿副本里，I-7.18 旧槽第一轮正推腿）：环洞池（S = 4、环长 3S = 12）上逐个 N 造「最新根 N 之后那一槽（N+1 的落点）读不出自证过的根」的镜像，
//! 以及把最新根的回收门槛 t 与钉槽 p 改成给定值并重封的错 t 镜像，照 I718_VARIANT 开关选中的判法（基线 / 甲 / 甲含种子边 / 乙）在池级 checker 上跑，
//! 每个镜像打一行 `ROW …`。不判对错：对错由 rerun.sh 打出的汇总与报告里的比对来读。
#![allow(dead_code, unused_imports)]
mod common_root_ring_holes;

use common_root_ring_holes::{
    content_of_the_version, fixed_geometry_of_the_ring_hole_pools, parameters_of_the_ring_hole_pools,
    pool_of, recording_of, RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{choose_system_configuration, readable_roots_with_ring_slots, PoolReader};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
const ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET: usize = 428;
const ROOT_RECORD_PINNED_RING_SLOT_OFFSET: usize = 436;
const SLOT_BYTES: usize = 512;

struct Through {
    pool: MemoryPool,
    allocator: PoolAllocator,
    current: TransactionOutput,
    instance: InstanceGeneration,
}

/// mkfs → 可写挂载（实例 1）→ 第一个文件（txg 3）→ 覆盖写到 txg `last`，每次都完整落盘；连同分配器与最后一版一起交回。
fn through(last: u64) -> Through {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let instance = output.instance;
    let mut current: TransactionOutput = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
            },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while current.root.checkpoint_txg.0 < last {
        let salt = current.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            instance,
        )
        .expect("覆盖写");
    }
    Through {
        pool: pool_of(&devices),
        allocator,
        current,
        instance,
    }
}

/// 下一次覆盖写（txg N+1）录下来的写，与 N+1 那次根槽写在写表里的下标。
fn next_publish(through: &Through) -> (Vec<RetainedWrite>, usize, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&through.pool, &stream);
    let mut allocator = through.allocator.clone();
    let salt = through.current.root.checkpoint_txg.0 + 1;
    {
        let mut writer = PoolWriter::new(&parameters, recording.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &through.current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            through.instance,
        )
        .expect("下一次覆盖写");
    }
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let root_index = writes
        .iter()
        .position(|write| {
            write.kind == StepKind::RootRecordFua
                && write
                    .bytes()
                    .and_then(|bytes| {
                        RootRecord::parse_slot(bytes, &parameters.filesystem_identifier)
                    })
                    .is_some_and(|root| root.checkpoint_txg.0 == salt)
        })
        .expect("N+1 的根槽写");
    (writes, root_index, salt)
}

fn after(base: &MemoryPool, writes: &[RetainedWrite]) -> MemoryPool {
    let mut image = base.clone();
    image.apply_writes(writes);
    image
}

fn overwrite_slot(image: &mut MemoryPool, like: &RetainedWrite, bytes: Vec<u8>) {
    image.apply_writes(&[RetainedWrite {
        device: like.device,
        kind: like.kind,
        is_force_unit_access: like.is_force_unit_access,
        offset: like.offset,
        contents: WrittenContents::Bytes(bytes),
    }]);
}

fn short(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "成立",
        InvariantVerdict::Violated(_) => "红",
        InvariantVerdict::NotApplicable(_) => "不适用",
        InvariantVerdict::PartiallyJudged(_) => "部分没判",
    }
}

struct Judged {
    i718: &'static str,
    i719: &'static str,
    red_others: Vec<&'static str>,
    detail718: String,
}

fn judged(image: &MemoryPool) -> Judged {
    let verdicts = check_pool_image(image);
    let find = |name: &str| verdicts.iter().find(|(n, _)| *n == name).map(|(_, v)| v.clone());
    let v718 = find("I-7.18").expect("判了 I-7.18");
    let v719 = find("I-7.19").expect("判了 I-7.19");
    Judged {
        i718: short(&v718),
        i719: short(&v719),
        red_others: verdicts
            .iter()
            .filter(|(n, v)| {
                matches!(v, InvariantVerdict::Violated(_)) && *n != "I-7.18" && *n != "I-7.19"
            })
            .map(|(n, _)| *n)
            .collect(),
        detail718: match v718 {
            InvariantVerdict::Violated(detail) => detail,
            _ => String::new(),
        },
    }
}

fn newest_root_and_ring(image: &MemoryPool) -> (RootRecord, Vec<u64>, Vec<(RootRingSlot, RootRecord)>) {
    let configuration = choose_system_configuration(image).expect("系统配置");
    let mut roots = readable_roots_with_ring_slots(
        image,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    );
    roots.sort_by_key(|(_, root)| (root.checkpoint_txg, root.instance));
    let newest = roots.last().expect("环里有根").1;
    let txgs = roots.iter().map(|(_, root)| root.checkpoint_txg.0).collect();
    (newest, txgs, roots)
}

fn slot_position(
    ring_slot: RootRingSlot,
) -> (DeviceIdentity, singlefs_core::address::DeviceOffsetInBytes) {
    let parameters = parameters_of_the_ring_hole_pools();
    (
        parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
        slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing),
    )
}

fn zero_the_slot(image: &mut MemoryPool, ring_slot: RootRingSlot) {
    let (device, offset) = slot_position(ring_slot);
    image.apply_writes(&[RetainedWrite {
        device,
        kind: StepKind::RootRecordFua,
        is_force_unit_access: true,
        offset,
        contents: WrittenContents::Bytes(vec![0u8; SLOT_BYTES]),
    }]);
}

/// 最新根（txg `txg`）的 t、p 改成给定值并重封。
fn with_t_and_p(image: &MemoryPool, txg: u64, t: u64, p: u8) -> MemoryPool {
    let parameters = parameters_of_the_ring_hole_pools();
    let ring_slot = target_for_publish(
        CheckpointTxg(txg),
        parameters.geometry.root_ring_slots_per_region,
    );
    let (device, offset) = slot_position(ring_slot);
    let mut bytes = PoolReader::read(image, device, offset, SLOT_BYTES).expect("读根槽");
    bytes[ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET..ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET + 8]
        .copy_from_slice(&t.to_le_bytes());
    bytes[ROOT_RECORD_PINNED_RING_SLOT_OFFSET] = p;
    let slot_bytes = bytes.len();
    let digest = wide_checksum_with_field_zeroed(&bytes, slot_bytes, ROOT_RECORD_CHECKSUM_OFFSET);
    bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    let mut rewritten = image.clone();
    rewritten.apply_writes(&[RetainedWrite {
        device,
        kind: StepKind::RootRecordFua,
        is_force_unit_access: true,
        offset,
        contents: WrittenContents::Bytes(bytes),
    }]);
    rewritten
}

fn row(group: &str, label: &str, image: &MemoryPool) {
    let (newest, txgs, _) = newest_root_and_ring(image);
    let judged = judged(image);
    println!(
        "ROW group={group} label={label} newest_txg={} F={} t={} p={} ring_min={} ring_len={} I-7.18={} I-7.19={} other_red={:?}{}",
        newest.checkpoint_txg.0,
        newest.rollback_floor.0,
        newest.reclaim_threshold.0,
        newest.pinned_ring_slot.0,
        txgs.iter().min().copied().unwrap_or(u64::MAX),
        txgs.len(),
        judged.i718,
        judged.i719,
        judged.red_others,
        if judged.detail718.is_empty() {
            String::new()
        } else {
            format!(" detail718={}", judged.detail718)
        }
    );
}

/// 第一组：N 从 3 到 26，N+1 那次发布的根槽写落在环里最旧那条根的槽上（N ≥ 11）或还没写过的槽上（N < 11）：
/// 旧根完好（根槽写之前崩）、根槽写成全零、写一半（新根前半 + 旧根后半）、写完整，四种盘面各打一行。
#[test]
fn group_next_slot_states() {
    for last in 3..=26u64 {
        let built = through(last);
        let (writes, root_index, salt) = next_publish(&built);
        let before = after(&built.pool, &writes[..root_index]);
        let root_write = &writes[root_index];
        let new_slot = root_write.bytes().expect("根槽写带字节").to_vec();
        let old_slot = PoolReader::read(&built.pool, root_write.device, root_write.offset, SLOT_BYTES)
            .expect("旧字节");
        row("next", &format!("N={last} 根槽写之前（旧根完好）"), &before);
        let mut zeroed = before.clone();
        overwrite_slot(&mut zeroed, root_write, vec![0u8; SLOT_BYTES]);
        row("next", &format!("N={last} N+1={salt} 的根槽写成全零"), &zeroed);
        let mut torn = before.clone();
        let half: Vec<u8> = new_slot[..SLOT_BYTES / 2]
            .iter()
            .chain(old_slot[SLOT_BYTES / 2..].iter())
            .copied()
            .collect();
        overwrite_slot(&mut torn, root_write, half);
        row("next", &format!("N={last} N+1={salt} 的根槽写一半"), &torn);
        row(
            "next",
            &format!("N={last} N+1={salt} 的根槽写完整"),
            &after(&built.pool, &writes[..=root_index]),
        );
    }
}

/// 第二组（纯掉电）：满环上 N+1 那次发布的写表，崩在每一个前缀上的盘面（根槽 512 字节在一个扇区里，写要么落要么不落）。
#[test]
fn group_every_prefix_of_the_publish_on_a_full_ring() {
    for last in [11u64, 12, 14, 15] {
        let built = through(last);
        let (writes, _root_index, _salt) = next_publish(&built);
        for prefix in 0..=writes.len() {
            let image = after(&built.pool, &writes[..prefix]);
            row(
                "prefix",
                &format!("N={last} 前缀 {prefix}/{}", writes.len()),
                &image,
            );
        }
    }
}

/// 第三组（错 t 镜像）：N = 14 的满环。三个盘面——N+1 的根槽写成全零（目标态）、旧根完好（对照）、第二旧的槽（N+2−3S，txg 4）写成全零（对照）——
/// 各把最新根 txg 14 的 t 扫 0..=16，p 取 255、3、4、5、11、12（12 = 环长，还原不成槽）。
#[test]
fn group_wrong_t_and_p_mirrors() {
    let built = through(14);
    let (writes, root_index, _salt) = next_publish(&built);
    let before = after(&built.pool, &writes[..root_index]);
    let root_write = &writes[root_index];
    let mut target = before.clone();
    overwrite_slot(&mut target, root_write, vec![0u8; SLOT_BYTES]);
    let mut second_oldest_zeroed = before.clone();
    let (_, _, roots) = newest_root_and_ring(&before);
    let second_oldest = roots
        .iter()
        .find(|(_, root)| root.checkpoint_txg.0 == 4)
        .expect("txg 4 在环里")
        .0;
    zero_the_slot(&mut second_oldest_zeroed, second_oldest);
    for (group, base) in [
        ("sweep_target", &target),
        ("sweep_intact", &before),
        ("sweep_second_oldest_zeroed", &second_oldest_zeroed),
    ] {
        for t in 0..=16u64 {
            for p in [255u8, 3, 4, 5, 11, 12] {
                let image = with_t_and_p(base, 14, t, p);
                row(group, &format!("t={t} p={p}"), &image);
            }
        }
    }
}

/// 第四组（年轻的环里最旧那条根的槽事后读坏）：N = 3、5、7、9、10：环里 0（种子）、1…N 都在，环没满；把环里最旧那条根的槽写成全零。
#[test]
fn group_young_ring_oldest_root_lost_afterwards() {
    for last in [3u64, 5, 7, 9, 10] {
        let built = through(last);
        let (_, _, roots) = newest_root_and_ring(&built.pool);
        let oldest = roots.first().expect("环里有根");
        let mut image = built.pool.clone();
        zero_the_slot(&mut image, oldest.0);
        row(
            "young",
            &format!("N={last} 环里最旧的 txg {} 的槽读坏", oldest.1.checkpoint_txg.0),
            &image,
        );
        row("young", &format!("N={last} 对照：没坏"), &built.pool);
    }
}

/// 同 `through`，但偶数 txg 发的是空发布（文件不变、只换根与记账），奇数 txg 覆盖写。
fn through_with_empty_publishes_on_even_txgs(last: u64) -> Through {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let instance = output.instance;
    let mut current: TransactionOutput = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
            },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while current.root.checkpoint_txg.0 < last {
        let salt = current.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = if salt % 2 == 0 {
            common_root_ring_holes::publish_empty(&mut writer, &mut allocator, &current, instance)
                .expect("空发布")
        } else {
            publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &content_of_the_version(salt),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
                },
                instance,
            )
            .expect("覆盖写")
        };
    }
    Through {
        pool: pool_of(&devices),
        allocator,
        current,
        instance,
    }
}

/// 第五组（空发布的历史）：偶数 txg 是空发布。N = 14（环里最旧的 txg 3 是覆盖写、N+1−3S+1 = 4 是空发布）与 N = 15（反过来）的目标态，t 扫 0..=8、p = 255。
#[test]
fn group_wrong_t_mirrors_on_a_history_with_empty_publishes() {
    for last in [14u64, 15] {
        let built = through_with_empty_publishes_on_even_txgs(last);
        let (writes, root_index, _salt) = next_publish(&built);
        let before = after(&built.pool, &writes[..root_index]);
        let mut target = before.clone();
        overwrite_slot(&mut target, &writes[root_index], vec![0u8; SLOT_BYTES]);
        row("empty_target", &format!("N={last} 真写者的 t"), &target);
        for t in 0..=8u64 {
            row(
                "empty_target_sweep",
                &format!("N={last} t={t} p=255"),
                &with_t_and_p(&target, last, t, 255),
            );
        }
    }
}

/// 第六组（掉一块盘的降级镜像）：满环 N = 14、15、16、17，拿掉盘 0 或盘 1（根环三个区域住在盘 0、盘 1、盘 0 上），最新根以及 N+1 的槽的状态照旧：
/// 掉盘之后根环里读不到的槽不止 N+1 那一个。
#[test]
fn group_a_device_missing() {
    for last in [14u64, 15, 16, 17] {
        let built = through(last);
        for lost in [0u32, 1] {
            let mut degraded = built.pool.clone();
            degraded.devices.remove(&DeviceIdentity(lost));
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let verdicts = check_pool_image(&degraded);
                verdicts
                    .iter()
                    .filter(|(name, _)| *name == "I-7.18" || *name == "I-7.19" || *name == "I-3.1")
                    .map(|(name, verdict)| format!("{name}={}", short(verdict)))
                    .collect::<Vec<_>>()
                    .join(" ")
            })) {
                Ok(summary) => println!("ROWDEGRADED N={last} 掉盘 {lost}: {summary}"),
                Err(_) => println!("ROWDEGRADED N={last} 掉盘 {lost}: 池级 checker 判不了（panic）"),
            }
        }
    }
}

/// 第七组（第二圈以后的错 t 镜像）：N = 26 的满环（N+1 = 27，落点位置 27 mod 12 = 3，那一槽在环里住着 txg 15），N+1 的根槽写成全零；
/// 最新根 txg 26 的 t 扫 0..=18，p 取 255、3（位置 3：钉的就是这个落点）、4。第一圈里钉值就是 N+1−3S，第二圈以后钉值（槽位置）小于 N+1−3S，
/// 看「p 写成 255 而其实钉着」那一形在第二圈以后甲是不是还放过。
#[test]
fn group_second_revolution_wrong_t_and_p_mirrors() {
    let built = through(26);
    let (writes, root_index, _salt) = next_publish(&built);
    let before = after(&built.pool, &writes[..root_index]);
    let mut target = before.clone();
    overwrite_slot(&mut target, &writes[root_index], vec![0u8; SLOT_BYTES]);
    row("second_revolution_honest", "N=26 真写者的 t", &target);
    for t in 0..=18u64 {
        for p in [255u8, 3, 4] {
            row(
                "second_revolution_sweep",
                &format!("t={t} p={p}"),
                &with_t_and_p(&target, 26, t, p),
            );
        }
    }
}
