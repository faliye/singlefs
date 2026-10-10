//! 攻方 m3-formal-code-b20-r1-opus 的副本探针（harness 档，只放进攻方自己拷出的副本里跑）：
//! 发布臂 nlink = 0 那一支在什么盘面上走得到，走到时直接建 inode 与「会话先走空」各交回什么、之后池级 checker 判什么。
//! 盘面：窄盘上第一个文件长到几个单元、发完 unlink 意图（与之后几批）就崩（照 admission_raises_the_floor_before_refusing.rs 的造法），
//! 再用崩溃注入第二截那一种调法（mount_writable_with_space_admission，产品删除开关）可写挂载。
//! 臂：A = 20-1 的发布臂（活着覆盖写 / nlink = 0 或不在树里建 inode / 删完建 inode / 树表 0 条第一个文件）；
//! C = 20-1 之前的发布臂（带文件一律覆盖写）；S-new = 会话建一个 inode；S-ow = 会话覆盖写；S-un = 会话 unlink inode 1。
//! 每个形状、每条臂各从同一份崩溃镜像现挂一次（挂载是确定的），臂之后的盘面跑池级 checker。

mod common_admission;

use common_admission::{
    checker_violations_on, plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_space_admission, DeletionQueueContinuationOutcome, Mounted,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::transaction::{
    publish_first_file, publish_new_inodes, publish_overwrite, publish_unlink_intent,
    publish_unlink_step, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::model_comparison::refusal_reason_of_publish_error;
use std::panic::{catch_unwind, AssertUnwindSafe};

const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_100;

#[derive(Clone, Copy, Debug)]
struct Shape {
    unit_area_slots: u64,
    overwrites: usize,
    data_units: usize,
    batches_after_the_intent: usize,
}

/// 照 admission_raises_the_floor_before_refusing.rs 的 pool_crashed_between_deletion_batches 造盘（不经会话、不走空，批大小 1），
/// 交回崩溃那一刻的镜像；形状造不出来（盘太窄）交回 None。
fn crash_image_of(shape: Shape) -> Option<MemoryPool> {
    let built = catch_unwind(AssertUnwindSafe(|| {
        let mut pool =
            start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                HistoryDeviceWidth::UnitAreaOf384Slots,
                shape.unit_area_slots,
            );
        for _ in 0..shape.overwrites {
            pool.overwrite(OVERWRITE_BYTES).ok()?;
        }
        if shape.data_units > 1 {
            pool.sequential_write((shape.data_units - 1) * data_unit_payload_capacity() + 100)
                .ok()?;
        }
        let parameters = pool.parameters.clone();
        let write_time_seconds = pool.write_time_seconds;
        let session = pool.session.as_mut()?;
        let mut current = session.current.version_of_trees()?.clone();
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        current = publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            write_time_seconds,
            session.instance,
        )
        .ok()?;
        for _ in 0..shape.batches_after_the_intent {
            current = publish_unlink_step(
                &mut writer,
                &mut session.allocator,
                &current,
                InodeNumber(FIRST_INODE_NUMBER),
                UnlinkBatchSize::CappedForTests(
                    UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在范围里"),
                ),
                session.instance,
            )
            .ok()?;
        }
        drop(writer);
        pool.session = None;
        Some(pool.image())
    }));
    built.ok().flatten()
}

fn mount_on(image: &MemoryPool) -> (Vec<(DeviceIdentity, SparseBlockDevice)>, Result<Mounted, String>) {
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let mut devices = plain_devices_on(image);
    let mounted = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .map_err(|error| short(&format!("{error:?}")));
    (devices, mounted)
}

fn short(text: &str) -> String {
    text.chars().take(90).collect()
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)], device_size_in_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {
    version
        .inode_record
        .as_ref()
        .is_some_and(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    TwentyOne,
    BeforeTwentyOne,
    SessionNewInode,
    SessionOverwrite,
    SessionUnlink,
}

const ARMS: [Arm; 5] = [
    Arm::TwentyOne,
    Arm::BeforeTwentyOne,
    Arm::SessionNewInode,
    Arm::SessionOverwrite,
    Arm::SessionUnlink,
];

/// 从同一份崩溃镜像现挂一次、跑一条臂：交回臂的结局与之后盘面的池级 checker 违例。
fn run_arm(image: &MemoryPool, arm: Arm) -> (String, Vec<String>) {
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let (mut devices, mounted) = mount_on(image);
    let Ok(mounted) = mounted else {
        return ("挂载被拒".to_string(), Vec::new());
    };
    let content = vec![0x5a_u8; 120];
    let file = FirstFile {
        content: &content,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS,
    };
    let outcome = match arm {
        Arm::TwentyOne | Arm::BeforeTwentyOne => {
            let instance = mounted.output.instance;
            let mut allocator = mounted.allocator;
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let published = match (&mounted.current, arm) {
                (PoolVersion::WithFile(current), Arm::BeforeTwentyOne) => {
                    publish_overwrite(&mut writer, &mut allocator, current, file, instance).map(|_| ())
                }
                (PoolVersion::WithFile(current), _) if first_file_is_a_live_record(current) => {
                    publish_overwrite(&mut writer, &mut allocator, current, file, instance).map(|_| ())
                }
                (PoolVersion::WithFile(current) | PoolVersion::WithTreesAndNoFile(current), _) => {
                    publish_new_inodes(&mut writer, &mut allocator, current, 1, FIXED_WRITE_TIME_SECONDS, instance)
                        .map(|_| ())
                }
                (PoolVersion::WithoutFile(current), _) => publish_first_file(
                    &mut writer,
                    &mut allocator,
                    &current.root,
                    file,
                    instance,
                    &current.record_bytes,
                )
                .map(|_| ()),
            };
            match published {
                Ok(()) => "发成".to_string(),
                Err(error) => format!(
                    "拒 {:?} / {}",
                    refusal_reason_of_publish_error(&error),
                    short(&format!("{error:?}"))
                ),
            }
        }
        Arm::SessionNewInode | Arm::SessionOverwrite | Arm::SessionUnlink => {
            let before_the_change = image_of(&devices, image.device_size_in_bytes);
            let (_output, mut session) = MountedSession::of_the_mount(mounted);
            let change = match arm {
                Arm::SessionNewInode => UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
                Arm::SessionOverwrite => UserChange::Overwrite(file),
                _ => UserChange::Unlink {
                    inode: InodeNumber(FIRST_INODE_NUMBER),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
            };
            let answer = match session.publish_user_change(&mut devices, change) {
                Ok(published) => format!(
                    "发成 {} 版、推 {} 串抬 F",
                    published.versions_published.len(),
                    published.floor_raises.len()
                ),
                Err(refusal) => format!("拒 {}", short(&format!("{refusal:?}"))),
            };
            let disk_changed = image_of(&devices, image.device_size_in_bytes) != before_the_change;
            format!("{answer}；盘动没动 {disk_changed}")
        }
    };
    let after = image_of(&devices, image.device_size_in_bytes);
    (outcome, checker_violations_on(&after))
}

#[test]
fn b20_opus_k2_publish_arm_on_a_short_deletion_continuation() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut shapes_built = 0;
    let mut mounts_short = 0;
    let mut mounts_walked = 0;
    let mut mounts_refused = 0;
    // 取样范围由环境变量切片（几片并行跑）：B20_SLOTS_FROM、B20_SLOTS_TO、B20_SLOTS_STEP，缺省 200..=320 步 8。
    let variable = |name: &str, default: u64| {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(default)
    };
    let slots_from = variable("B20_SLOTS_FROM", 200);
    let slots_to = variable("B20_SLOTS_TO", 320);
    let slots_step = usize::try_from(variable("B20_SLOTS_STEP", 8)).expect("步长");
    for unit_area_slots in (slots_from..=slots_to).step_by(slots_step) {
        for overwrites in [0, 1, 2, 3, 4, 6, 8] {
            for data_units in [1, 2, 6, 14, 30, 60] {
                for batches_after_the_intent in [0, 1] {
                    if batches_after_the_intent >= data_units {
                        continue;
                    }
                    let shape = Shape {
                        unit_area_slots,
                        overwrites,
                        data_units,
                        batches_after_the_intent,
                    };
                    let Some(image) = crash_image_of(shape) else {
                        continue;
                    };
                    shapes_built += 1;
                    let (_devices, mounted) = mount_on(&image);
                    let mounted = match mounted {
                        Err(error) => {
                            mounts_refused += 1;
                            println!("SHAPE {shape:?} MOUNT refused {error}");
                            continue;
                        }
                        Ok(mounted) => mounted,
                    };
                    let current = mounted.current.version_of_trees().cloned();
                    let short = matches!(
                        mounted.output.deletion_queue_continuation_outcome,
                        DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }
                    );
                    drop(mounted);
                    if !short {
                        mounts_walked += 1;
                        continue;
                    }
                    mounts_short += 1;
                    let current = current.expect("续做走到的那一版有树");
                    println!(
                        "SHAPE {shape:?} MOUNT short queue={:?} inode1_nlink={:?} live={} checker_after_mount={:?}",
                        current.deleted_inodes,
                        current.inode_record.map(|record| record.nlink),
                        first_file_is_a_live_record(&current),
                        {
                            let (devices, _) = mount_on(&image);
                            checker_violations_on(&image_of(&devices, image.device_size_in_bytes))
                        }
                    );
                    for arm in ARMS {
                        let (outcome, violations) = run_arm(&image, arm);
                        println!("  ARM {arm:?}: {outcome} | checker {violations:?}");
                    }
                }
            }
        }
    }
    println!(
        "SUMMARY shapes_built={shapes_built} mounts_refused={mounts_refused} mounts_walked_to_empty={mounts_walked} mounts_short={mounts_short}"
    );
}
