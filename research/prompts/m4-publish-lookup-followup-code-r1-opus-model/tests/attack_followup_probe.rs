//! 攻方腿（m4-publish-lookup-followup-code-r1）探针用例：只在打了探针补丁（tools/probe.patch）的副本里编，量几处没改的线性查找的比较次数随池大小怎么涨。
//! 池按第一个文件之外 F 个八单元的文件造（F 取 ATTACK_PROBE_FILES，逗号隔开），在三件事上各读一次探针：一批 8 项的整理发批、重挂（恢复）、
//! 一次普通写；另在一个快照下整份重写一个 N 单元的文件（N 取 ATTACK_PROBE_REWRITE_UNITS）。
mod common;

use std::collections::BTreeSet;
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{DeviceIdentity, InodeNumber, TreeIdentifier};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{CompactionBatchSize, CompactionBatchUnits, CompactionSwitches};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable_with_compaction_switches, raise_rollback_floor_to_the_admission_ceiling,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter,
    RelocatedUnitHolder, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::SparseBlockDevice;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn sparse_device() -> SparseBlockDevice {
    SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512))
}

fn switches_with_the_batch_size(batch_units: usize) -> CompactionSwitches {
    let mut switches = CompactionSwitches::PRODUCT;
    switches.batch_size = CompactionBatchSize::ForcedForTests(
        CompactionBatchUnits::new(batch_units).expect("批大小至少 1"),
    );
    switches
}

fn formatted_and_mounted(switches: CompactionSwitches) -> (Devices, MountedSession) {
    let parameters = common::parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, sparse_device()))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file = common::file_content();
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile { content: &first_file, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable_with_compaction_switches(&parameters, &mut devices, switches)
        .expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (devices, session)
}

fn digest_of_the_images(devices: &Devices) -> String {
    let mut written = Vec::new();
    for (identity, device) in devices {
        written.extend_from_slice(&identity.0.to_le_bytes());
        for (sector, bytes) in device.image.written_sectors() {
            written.extend_from_slice(&sector.to_le_bytes());
            written.extend_from_slice(bytes);
        }
    }
    singlefs_harness::sha256::sha256_digest(&written)
        .iter()
        .take(12)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

fn content(data_units: usize, seed: u64, short_by: usize) -> Vec<u8> {
    let mut random = Random(seed | 1);
    (0..(data_units * data_unit_payload_capacity()).saturating_sub(short_by).max(1))
        .map(|_| u8::try_from(random.next() & 0xff).expect("低 8 位"))
        .collect()
}

fn version_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树")
}

fn environment_list(name: &str, default: &str) -> Vec<u64> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .map(|text| text.trim().parse().expect("逗号隔开的正整数"))
        .collect()
}

/// 建 `files` 个 inode、各整份写 `units` 个数据单元（经会话，一个一次发布）：交回文件的号。
fn write_files(session: &mut MountedSession, devices: &mut Devices, files: u64, units: usize) -> Vec<InodeNumber> {
    let first = version_of(session).inode_number_watermark();
    let mut write_time = common::FIXED_WRITE_TIME_SECONDS + 600;
    session
        .publish_user_change(devices, UserChange::NewInodes { count: files, write_time_seconds: write_time })
        .expect("建 inode");
    let inodes: Vec<InodeNumber> = (first..first + files).map(InodeNumber).collect();
    for inode in &inodes {
        write_time += 1;
        let bytes = content(units, inode.0, 5);
        session
            .publish_user_change(devices, UserChange::SequentialWriteOfAnInode { inode: *inode, content: &bytes, write_time_seconds: write_time })
            .expect("写");
    }
    inodes
}

fn counts_line(counts: [u64; singlefs_core::attack_probe::SITES]) -> String {
    format!(
        "mapping_positions_find={} mount_placements_find={} kept_contains={} allocation_node_find={} first_file_rewritten_contains={} mapping_positions_calls={} record_roles_contains={}",
        counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6]
    )
}

/// 一档池（`files` 个八单元的文件）上的四件事，各读一次本线程的探针：交回要打的几行。
fn pool_scenario(files: u64) -> Vec<String> {
    use singlefs_core::address::DataUnitIndexInFile;
    use singlefs_core::attack_probe;
    use singlefs_core::compaction_plan::PlannedRelocation;
    use singlefs_core::transaction::{
        central_mapping_locations_in_the_version, publish_relocation_batch, MappingLookup, TransactionUnit,
    };
    let parameters = common::parameters();
    let mut lines = Vec::new();
    {
        let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
        let inodes = write_files(&mut session, &mut devices, files, 8);
        let units = version_of(&session).units.len();
        let mapped = version_of(&session).mapped_units.len();
        // 一批 8 项：第一个文件之外第二个文件的 8 个数据单元（在 units 里靠前），再一批取最后写的那个文件的 8 个（在 units 里靠后）。
        for (which, source_inode) in [("from_the_second_file", inodes[1]), ("from_the_last_file", inodes[inodes.len() - 1])] {
            let previous = version_of(&session).clone();
            let planned: Vec<PlannedRelocation> = (0..8)
                .map(|unit| {
                    let role = TransactionUnit::DataOfAFileOtherThanTheFirst { inode: source_inode, unit: DataUnitIndexInFile(unit) };
                    let (_, key) = previous.mapped_units.iter().find(|(mapped, _)| *mapped == role).expect("进映射");
                    let MappingLookup::Found(locations) = central_mapping_locations_in_the_version(&previous, key) else {
                        panic!("查得到")
                    };
                    PlannedRelocation { mapping_key: key.clone(), source: Placement { slot: locations[0].slot, span: role.span_slots() } }
                })
                .collect();
            let _ = attack_probe::take();
            let relocated = {
                let mut writer = PoolWriter::new(&parameters, &mut devices);
                publish_relocation_batch(&mut writer, &mut session.allocator, &previous, &planned, session.instance)
                    .expect("搬 8 项的一批")
            };
            lines.push(format!("name=probe files={files} units={units} mapped={mapped} op=relocation_batch_of_8_{which} {}", counts_line(attack_probe::take())));
            session.current = PoolVersion::of_a_version_of_trees(relocated);
        }
        // 一次普通写：整份重写第三个文件（8 个单元）。
        let bytes = content(8, 99, 5);
        let _ = attack_probe::take();
        session
            .publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode: inodes[2], content: &bytes, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 90_000 })
            .expect("整份重写");
        lines.push(format!("name=probe files={files} units={units} mapped={mapped} op=whole_rewrite_of_8 {}", counts_line(attack_probe::take())));
        // 抬 F：先整份重写四分之一的文件（换下的落点留在释放代里），再抬 F 到准入上限（一串空发布，回收那些落点）。
        for (position, inode) in inodes.iter().enumerate().skip(3).take(usize::try_from(files / 4).expect("个数")) {
            let bytes = content(8, 5_000 + inode.0, 5);
            session
                .publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode: *inode, content: &bytes, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 100_000 + u64::try_from(position).expect("序号") })
                .expect("整份重写");
        }
        let mut current = version_of(&session).clone();
        let _ = attack_probe::take();
        let raised = raise_rollback_floor_to_the_admission_ceiling(&parameters, &mut devices, &mut session.allocator, &mut current, session.shadow_ledger);
        lines.push(format!("name=probe files={files} units={units} mapped={mapped} op=raise_floor raised={} {}", raised.is_ok(), counts_line(attack_probe::take())));
        session.current = PoolVersion::of_a_version_of_trees(current);
        // 重挂：恢复重建这一版。
        drop(session);
        let _ = attack_probe::take();
        let mounted = mount_writable_with_compaction_switches(&parameters, &mut devices, CompactionSwitches::PRODUCT).expect("重挂");
        lines.push(format!("name=probe files={files} units={units} mapped={mapped} op=remount {}", counts_line(attack_probe::take())));
        drop(mounted);
    }
    lines
}

/// 快照下整份重写一个 N 单元的文件：释放两处 kept_for_the_snapshots.contains、记录点名那一处随 N 怎么涨。
fn rewrite_scenario(rewrite_units: u64) -> Vec<String> {
    use singlefs_core::attack_probe;
    {
        let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
        let inodes = write_files(&mut session, &mut devices, 4, usize::try_from(rewrite_units).expect("单元数"));
        session.publish_user_change(&mut devices, UserChange::CreateSnapshot).expect("建快照");
        let bytes = content(usize::try_from(rewrite_units).expect("单元数"), 1234, 5);
        let _ = attack_probe::take();
        session
            .publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode: inodes[1], content: &bytes, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 90_000 })
            .expect("快照下整份重写");
        vec![format!("name=probe rewrite_units={rewrite_units} op=whole_rewrite_under_a_snapshot {}", counts_line(attack_probe::take()))]
    }
}

/// 六个场景互不依赖、各在一条线程上跑（探针按线程记）；场景只有六个，线程数取场景数。输出按场景的次序合并再打。
#[test]
fn attack_followup_probe_of_the_unchanged_linear_lookups() {
    let pools = environment_list("ATTACK_PROBE_FILES", "16,64,256");
    let rewrites = environment_list("ATTACK_PROBE_REWRITE_UNITS", "8,32,128");
    let outputs: Vec<Vec<String>> = std::thread::scope(|scope| {
        let workers: Vec<_> = pools
            .iter()
            .map(|files| {
                let files = *files;
                scope.spawn(move || pool_scenario(files))
            })
            .chain(rewrites.iter().map(|units| {
                let units = *units;
                scope.spawn(move || rewrite_scenario(units))
            }))
            .collect();
        workers.into_iter().map(|worker| worker.join().expect("场景没 panic")).collect()
    });
    for line in outputs.into_iter().flatten() {
        println!("{line}");
    }
}
