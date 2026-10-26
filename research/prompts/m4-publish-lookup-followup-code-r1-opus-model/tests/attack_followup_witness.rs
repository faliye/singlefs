//! 攻方腿（m4-publish-lookup-followup-code-r1）变异见证用例：只在改后副本里编（用到七·6 后续批新加的计数点），打几处计数给变异前后比，不判。
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

fn version_of(session: &MountedSession) -> Option<&TransactionOutput> {
    session.current.version_of_trees()
}

/// 这一版的摘要：留存条目数、留存条目里重复的 key 数、第一个文件之外的文件数、其中重复 inode 号的个数、活快照数。
fn summary_of(session: &MountedSession) -> String {
    let Some(version) = version_of(session) else {
        return "no-trees".to_string();
    };
    let kept = &version.mapping_entries_kept_for_the_snapshots;
    let distinct_kept: BTreeSet<&[u8]> = kept.iter().map(|(key, _)| key.as_slice()).collect();
    let files = &version.files_other_than_the_first;
    let distinct_files: BTreeSet<u64> = files.iter().map(|file| file.inode.0).collect();
    format!(
        "kept={} dupkept={} files={} dupfiles={} units={} snapshots={}",
        kept.len(),
        kept.len() - distinct_kept.len(),
        files.len(),
        files.len() - distinct_files.len(),
        version.units.len(),
        live_snapshots(version).len()
    )
}

fn live_snapshots(version: &TransactionOutput) -> Vec<TreeIdentifier> {
    version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| entry.tree)
        .filter(|tree| !version.snapshots_being_destroyed.contains(tree))
        .collect()
}

/// 变异的见证：几处在快档用例里不判建索引次数的计数，打一行给变异前后比（不判，只打）。
/// 一个八文件、每个四单元的池建一个快照，快照下整份重写一个文件、再删快照；另一个 48 文件、每个八单元的池上整份重写一个文件。
#[test]
fn attack_followup_witness_of_the_counted_sites() {
    use singlefs_core::transaction::lookup_work::{lookup_work_while, LookupSite};
    let sites = [
        LookupSite::SnapshotJudgement,
        LookupSite::FilesOfTheReplacedRolesInThePreviousVersion,
        LookupSite::KeptMappingEntriesReleasedByTheSnapshotDestruction,
        LookupSite::CarriedExtentTreeNodes,
        LookupSite::CarriedNodesOfTheMultiLevelTrees,
    ];
    for (files, units, with_a_snapshot) in [(8u64, 4usize, true), (48, 8, false)] {
        let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
        let first = version_of(&session).expect("有树").inode_number_watermark();
        let mut write_time = common::FIXED_WRITE_TIME_SECONDS + 600;
        session
            .publish_user_change(&mut devices, UserChange::NewInodes { count: files, write_time_seconds: write_time })
            .expect("建 inode");
        for inode in first..first + files {
            write_time += 1;
            let bytes = content(units, inode, 5);
            session
                .publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode: InodeNumber(inode), content: &bytes, write_time_seconds: write_time })
                .expect("写");
        }
        if with_a_snapshot {
            session.publish_user_change(&mut devices, UserChange::CreateSnapshot).expect("建快照");
        }
        write_time += 1;
        let bytes = content(units, 77, 5);
        let rewritten = InodeNumber(first + 2);
        let (result, rewrite_work) = lookup_work_while(|| {
            session.publish_user_change(&mut devices, UserChange::SequentialWriteOfAnInode { inode: rewritten, content: &bytes, write_time_seconds: write_time })
        });
        result.expect("整份重写");
        for site in sites {
            println!("witness files={files} rewrite site={site:?} {:?} none_site={:?}", rewrite_work.of_the_site(site), rewrite_work.work_of_each_site.get(&None));
        }
        if with_a_snapshot {
            let snapshot = live_snapshots(version_of(&session).expect("有树"))[0];
            let (result, destruction_work) = lookup_work_while(|| {
                session.publish_user_change(&mut devices, UserChange::DestroySnapshot { snapshot })
            });
            result.expect("删快照");
            println!(
                "witness files={files} destruction site=KeptMappingEntriesReleasedByTheSnapshotDestruction {:?}",
                destruction_work.of_the_site(LookupSite::KeptMappingEntriesReleasedByTheSnapshotDestruction)
            );
        }
    }
}
