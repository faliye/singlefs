//! 攻方原型（m3-unlink-code-r1，opus 腿）：E7 准入开着（产品式子，小盘照随机历史那几档的补足）一次顺序写把文件写到式子放得下的最大，
//! 再 unlink（批大小 K）。走空一步一步在进程里发（与会话同一条发布路径），直到某一步被空间拒（会话此时会推抬 F）——
//! 在这一刻「崩」（推抬 F 之前），重开可写挂载（同一个 K），看挂不挂得上、再挂一次、只读挂载读什么。只在草稿副本上跑。

mod common_admission;

use common_admission::{checker_violations_on, plain_devices_on, start_plain, PoolUnderTest};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{mount_writable_with_test_only_switches_and_deletion_switches, MountError, ShadowLedger};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{publish_unlink_intent, publish_unlink_step, PoolWriter, PublishError, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

fn switches(batch: u64) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: if batch >= 144 { UnlinkBatchSize::OneExtentLowerLeaf } else { UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(batch).unwrap()) },
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

fn mount_k(parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters, image: &MemoryPool, batch: u64) -> (Result<singlefs_core::mount::Mounted, MountError>, MemoryPool) {
    let mut devices = plain_devices_on(image);
    let r = mount_writable_with_test_only_switches_and_deletion_switches(parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches(batch));
    let after = MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: image.device_size_in_bytes };
    (r, after)
}

fn short(e: &impl std::fmt::Debug) -> String { format!("{e:?}").chars().take(220).collect() }

/// 一次顺序写写到式子放得下的最大单元数（二分），交回单元数。
fn write_the_largest_file(pool: &mut PoolUnderTest<SparseBlockDevice>, upper: usize) -> usize {
    // 先在拷贝上二分：每次从同一份起点试。
    let image = pool.image();
    let parameters = pool.parameters.clone();
    let (mut lo, mut hi) = (1usize, upper);
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let mut probe = start_plain_copy(pool, &image, &parameters);
        let ok = probe.sequential_write((mid - 1) * data_unit_payload_capacity() + 100).is_ok();
        if ok { lo = mid } else { hi = mid - 1 }
    }
    pool.sequential_write((lo - 1) * data_unit_payload_capacity() + 100).expect("最大那一次写得成");
    lo
}

fn start_plain_copy(pool: &PoolUnderTest<SparseBlockDevice>, image: &MemoryPool, _p: &singlefs_core::make_filesystem::MakeFilesystemParameters) -> PoolUnderTest<SparseBlockDevice> {
    let session = pool.session.as_ref().unwrap();
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: plain_devices_on(image),
        session: Some(MountedSession {
            allocator: session.allocator.clone(),
            current: session.current.clone(),
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            parameters_and_device_table: session.parameters_and_device_table.clone(),
        }),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

fn case(width: HistoryDeviceWidth, batch: u64, remount_first: bool, overwrites: usize) {
    let mut pool = start_plain(width);
    if remount_first {
        let (r, after) = mount_k(&pool.parameters.clone(), &pool.image(), batch);
        let mounted = r.expect("第一个文件之后重挂");
        pool.devices = plain_devices_on(&after);
        let (_o, session) = MountedSession::of_the_mount(mounted);
        pool.session = Some(session);
    } else {
        pool.session.as_mut().unwrap().allocator.set_deletion_switches(switches(batch));
    }
    for _ in 0..overwrites { let _ = pool.overwrite(2999); }
    let units = write_the_largest_file(&mut pool, 2000);
    // 走空一步一步在进程里发：意图，然后批，直到被空间拒或走空。
    let mut steps = 0usize;
    let mut refusal: Option<String> = None;
    {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().unwrap();
        let mut current = session.current.version_of_trees().unwrap().clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        match publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), session.instance) {
            Ok(v) => { current = v; steps += 1; }
            Err(e) => refusal = Some(format!("intent: {}", short(&e))),
        }
        while refusal.is_none() && !current.deleted_inodes.is_empty() {
            match publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), switches(batch).batch_size, session.instance) {
                Ok(v) => { current = v; steps += 1; }
                Err(e @ (PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. })) => refusal = Some(short(&e)),
                Err(e) => { refusal = Some(format!("other: {}", short(&e))); }
            }
        }
    }
    let label = format!("width={width:?} K={batch} remount_first={remount_first} P={overwrites} units={units} steps_published={steps}");
    let Some(refusal) = refusal else {
        println!("E7 {label} walk finished without a space refusal");
        return;
    };
    // 「崩」在会话推抬 F 之前：会话丢掉，盘停在最后一步之后。
    pool.session = None;
    let image = pool.image();
    let checker = checker_violations_on(&image);
    let (first, after_first) = mount_k(&pool.parameters, &image, batch);
    let first_text = match &first { Ok(m) => format!("Mounted(queue={}, cont_steps={}, cont_raises={}, mount_space={:?})", m.current.version_of_trees().map_or(0, |v| v.deleted_inodes.len()), m.output.deletion_queue_continuation.len(), m.output.floor_raises_pushed_by_the_continuation.len(), short(&m.output.space_admission)), Err(e) => short(e) };
    let (second, _) = mount_k(&pool.parameters, &after_first, batch);
    let second_text = match &second { Ok(_) => "Mounted".to_string(), Err(e) => short(e) };
    let ro = mount_read_only(&after_first).map(|m| (m.effective_root.instance.0, m.effective_root.checkpoint_txg.0)).map_err(|e| short(&e));
    println!("E7 {label} refused_at_step={} refusal={refusal}\nE7    checker_at_crash={} first_remount={first_text}\nE7    second_remount={second_text} read_only={ro:?}", steps + 1, checker.len());
    // 对照：同一刻会话照常推抬 F 接着走（不崩）。
}

#[test]
fn e7_walk_to_the_landing_wall_then_crash() {
    for width in [HistoryDeviceWidth::UnitAreaOf384Slots, HistoryDeviceWidth::UnitAreaOf256Slots, HistoryDeviceWidth::UnitAreaOf240Slots] {
        for batch in [1u64, 2, 3] {
            for remount_first in [false, true] {
                case(width, batch, remount_first, 0);
            }
        }
    }
}

#[test]
fn e7_session_unlink_of_the_largest_file() {
    for width in [HistoryDeviceWidth::UnitAreaOf384Slots, HistoryDeviceWidth::UnitAreaOf240Slots] {
        for batch in [1u64, 2] {
            let mut pool = start_plain(width);
            pool.session.as_mut().unwrap().allocator.set_deletion_switches(switches(batch));
            let units = write_the_largest_file(&mut pool, 2000);
            let r = pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER) }, None);
            println!("E7S width={width:?} K={batch} units={units} unlink={}", match &r { Ok(p) => format!("Ok(versions={}, raises={}, refusals={:?})", p.versions_published.len(), p.floor_raises.len(), p.refusals_that_pushed_the_floor_raises.iter().map(|e| short(e).chars().take(30).collect::<String>()).collect::<Vec<_>>()), Err(e) => short(e) });
        }
    }
}
