//! m4-close-fixes-code-r3 攻方腿模型（副本用）：穷举表里「不接」的两处读与没有变异行的水位那一读，各给一段能分辨的历史；
//! 另造一张两把 key 的映射位置对换、key 集合相等、封得对的盘。盘是两块内存稀疏盘。断言钉今天的行为，目标绿 = 报告里写的几形在这棵树上成立。

mod common;

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::allocation_record_tree::{AllocationRecordTreeNode, AllocationRecordTreeNodePosition};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_session::{
    MountedSession, RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing, UserChange,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::{build_mapping_entry, parse_mapping_entry};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file,
    publish_sequential_write, warm_up, FirstFile, MappingLookup, PartialWriteBytes, PoolVersion,
    PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
use singlefs_verification::memory_pool::SparseBlockDevice;

const BYTES_OF_A_NODE: usize = 16384;
const BYTES_OF_A_DATA_UNIT: usize = 32768;
const FLIPPED_BYTE: usize = 4000;
const FIRST_FILE: InodeNumber = InodeNumber(1);

/// 照转给内存盘，数点名那一份（盘, 偏移）上发了几次读。
struct DeviceCountingReadsOfOneCopy {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    watched: Arc<Mutex<Option<(DeviceIdentity, u64)>>>,
    reads_of_the_watched_copy: Arc<AtomicU64>,
}

impl BlockDevice for DeviceCountingReadsOfOneCopy {
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if *self.watched.lock().expect("用例单线程改") == Some((self.identity, offset.0)) {
            self.reads_of_the_watched_copy.fetch_add(1, Ordering::SeqCst);
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn discard_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.inner.discard_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
    fn journal_record_offsets_hint(
        &self,
        ring_start: singlefs_core::address::DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<singlefs_core::address::DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

type Devices = Vec<(DeviceIdentity, DeviceCountingReadsOfOneCopy)>;

struct Pool {
    devices: Devices,
    version: TransactionOutput,
    watched: Arc<Mutex<Option<(DeviceIdentity, u64)>>>,
    reads_of_the_watched_copy: Arc<AtomicU64>,
}

fn content_of_units(units: u64) -> Vec<u8> {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let length = usize::try_from(units * payload - 1000).expect("长度");
    (0..length)
        .map(|index| u8::try_from((index * 37 + 11) % 253).expect("小于 256"))
        .collect()
}

fn pool_holding(content: &[u8]) -> Pool {
    let parameters = parameters();
    let mut plain: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut plain).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &plain, &genesis);
    let first_content = file_content();
    let version = {
        let mut writer = PoolWriter::new(&parameters, plain.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
        publish_sequential_write(
            &mut writer,
            &mut allocator,
            &first,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            instance,
        )
        .expect("顺序写")
    };
    let watched = Arc::new(Mutex::new(None));
    let reads_of_the_watched_copy = Arc::new(AtomicU64::new(0));
    let devices = plain
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceCountingReadsOfOneCopy {
                    identity,
                    inner,
                    watched: Arc::clone(&watched),
                    reads_of_the_watched_copy: Arc::clone(&reads_of_the_watched_copy),
                },
            )
        })
        .collect();
    Pool {
        devices,
        version,
        watched,
        reads_of_the_watched_copy,
    }
}

fn inner_of(devices: &mut Devices, device: DeviceIdentity) -> &mut SparseBlockDevice {
    &mut devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘")
        .1
        .inner
}

fn damage_the_copy_at(devices: &mut Devices, location: &LocationEntry, unit_bytes: usize) {
    let device = inner_of(devices, location.device);
    let mut unit = vec![0u8; unit_bytes];
    device
        .read_at(location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    unit[FLIPPED_BYTE] ^= 0xFF;
    device
        .write_at(location.slot.to_device_offset(), &unit, WriteDurability::Plain)
        .expect("写回改过的那一份");
    assert_ne!(crc32_castagnoli(&unit), location.unit_checksum, "前提：弄坏了");
}

fn copy_self_verifies(devices: &mut Devices, location: &LocationEntry, unit_bytes: usize) -> bool {
    let mut unit = vec![0u8; unit_bytes];
    inner_of(devices, location.device)
        .read_at(location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    crc32_castagnoli(&unit) == location.unit_checksum
}

fn session_of_a_writable_mount(devices: &mut Devices) -> MountedSession {
    let mounted = mount_writable(&parameters(), devices).expect("可写挂载");
    MountedSession::of_the_mount(mounted).1
}

fn current_of(session: &MountedSession) -> TransactionOutput {
    session.current.file_version().expect("带文件").clone()
}

fn new_inode_change(offset: u64) -> UserChange<'static> {
    UserChange::NewInodes {
        count: 1,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + offset,
    }
}

fn mapped_locations(version: &TransactionOutput, key: &[u8]) -> [LocationEntry; 2] {
    match central_mapping_locations_in_the_version(version, key) {
        MappingLookup::Found(locations) => locations,
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => panic!("这把 key 在映射里"),
    }
}

fn key_of_role(version: &TransactionOutput, role: TransactionUnit) -> Vec<u8> {
    version
        .mapped_units
        .iter()
        .find(|(identity, _)| *identity == role)
        .expect("这个角色进映射")
        .1
        .clone()
}

fn watch(pool: &Pool, location: &LocationEntry) {
    *pool.watched.lock().expect("单线程") = Some((location.device, location.slot.to_device_offset().0));
    pool.reads_of_the_watched_copy.store(0, Ordering::SeqCst);
}

/// 写入口轮换系统配置算 F 生效值读最新那条根的实例表（`transaction.rs` 的 `write_system_configuration_slot`）：每次发布都读，不进账。
/// 实例表一片弄坏一份，发一次用户改动：盘层数到读（点名的那一份上发过读），别的读那个数 0，另发那一批 NotNeeded，那一份还坏着。
#[test]
fn the_rotation_read_of_the_instance_table_reads_the_damaged_copy_and_nothing_counts_it() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table = current_of(&session).root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table[0], BYTES_OF_A_DATA_UNIT);
    watch(&pool, &table[0]);
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(10))
        .expect("建一个空 inode");
    let reads = pool.reads_of_the_watched_copy.load(Ordering::SeqCst);
    let other = session.other_unit_reads_with_one_copy_failing_in_this_mount();
    let outcome = format!(
        "{:?}",
        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
    );
    let still_damaged = !copy_self_verifies(&mut pool.devices, &table[0], BYTES_OF_A_DATA_UNIT);
    println!("attack-fixes-r3: rotation_read reads_of_the_damaged_copy={reads} other={other} outcome={outcome} still_damaged={still_damaged}");
    assert!(reads > 0, "前提：这次改动读过那一份");
    assert_eq!(other, 0);
    assert!(matches!(
        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing,
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded
    ));
    assert!(still_damaged);
}

/// 建 `changes` 次改动、抬 F 到上限，`damaged` 给了就在抬 F 之前弄坏那个分配记录树节点的头一份。交回抬之前后的节点指针、别的读那个数、读到的次数。
struct FloorRaiseRun {
    pool: Pool,
    session: MountedSession,
    before: TransactionOutput,
    after: TransactionOutput,
    reads: u64,
    other: u64,
    raise: String,
}

fn run_with_a_floor_raise(damaged: Option<AllocationRecordTreeNode>) -> FloorRaiseRun {
    let mut pool = pool_holding(&content_of_units(400));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    for change in 0..8 {
        session
            .publish_user_change(&mut pool.devices, new_inode_change(10 + change))
            .expect("建一个空 inode");
    }
    let before = current_of(&session);
    if let Some(node) = damaged {
        let location = before.allocation_record_tree.pointer_of(node).expect("节点在树里").locations[0];
        damage_the_copy_at(&mut pool.devices, &location, BYTES_OF_A_NODE);
        watch(&pool, &location);
    }
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("现行那一版带文件");
    };
    let raise = raise_rollback_floor_to_the_admission_ceiling(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        Arc::make_mut(current),
        ShadowLedger::On,
    );
    let raise = format!("{raise:?}");
    let reads = pool.reads_of_the_watched_copy.load(Ordering::SeqCst);
    let other = session.other_unit_reads_with_one_copy_failing_in_this_mount();
    let after = current_of(&session);
    FloorRaiseRun {
        pool,
        session,
        before,
        after,
        reads,
        other,
        raise,
    }
}

/// 可写挂载期间抬 F 重算影子账读每条候选根的账（`mount.rs` 的 `isolate_slots_referenced_only_by_abandoned_roots` 里 `placements_referenced_by_root`）：
/// 现行那一版也在候选里，读的是现行那一版的分配记录树。这一读不进账，穷举表写它「挂载这一串」「读的是别的版本」。
/// 取一片抬 F 那一串没动的叶（现行那一版引用它），弄坏头一份：抬 F 读过那一份，别的读那个数 0，下一次用户改动不换它，那一份还坏着。
#[test]
fn the_floor_raise_reading_the_account_of_the_current_version_reads_a_damaged_leaf_and_nothing_counts_it() {
    let dry = run_with_a_floor_raise(None);
    let unchanged: BTreeSet<AllocationRecordTreeNode> = dry
        .before
        .allocation_record_tree
        .nodes
        .iter()
        .filter(|(node, pointer)| dry.after.allocation_record_tree.pointer_of(*node) == Some(*pointer))
        .map(|(node, _)| *node)
        .filter(|node| {
            matches!(
                node,
                AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition { level: 0, .. })
            )
        })
        .collect();
    println!(
        "attack-fixes-r3: floor_raise_dry raise={} nodes={} unchanged_leaves={}",
        dry.raise,
        dry.before.allocation_record_tree.nodes.len(),
        unchanged.len()
    );
    let node = *unchanged.iter().next().expect("抬 F 那一串有没动的叶");
    let mut run = run_with_a_floor_raise(Some(node));
    let location = run.before.allocation_record_tree.pointer_of(node).expect("在树里").locations[0];
    let published = run
        .session
        .publish_user_change(&mut run.pool.devices, new_inode_change(40))
        .expect("再建一个空 inode");
    let outcome = format!(
        "{:?}",
        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
    );
    let other_after_the_change = run.session.other_unit_reads_with_one_copy_failing_in_this_mount();
    let pointer_after_the_change = current_of(&run.session).allocation_record_tree.pointer_of(node);
    let still_damaged = !copy_self_verifies(&mut run.pool.devices, &location, BYTES_OF_A_NODE);
    println!(
        "attack-fixes-r3: floor_raise node={node:?} raise={} reads_of_the_damaged_copy={} other={} other_after_the_change={other_after_the_change} outcome={outcome} leaf_moved={} still_damaged={still_damaged}",
        run.raise,
        run.reads,
        run.other,
        pointer_after_the_change != run.before.allocation_record_tree.pointer_of(node),
    );
    assert!(run.reads > 0, "前提：抬 F 读过那一份（读现行那一版的账）");
    assert_eq!(run.other, 0, "这一读不进账");
}

/// 回退取环里各根的 inode 号水位（`mount.rs` 的 `highest_inode_number_watermark_in_the_ring`）：环里一条既不是现行也不是目标的根引用的单元，只有这一读读到。
/// 所以「接了线而没有变异行」的这一处不是等价：只有中间那一版引用的数据单元弄坏一份，回退照样做成，别的读那个数是 1，只能是水位这一读记的。
#[test]
fn only_the_watermark_read_of_a_rollback_sees_a_unit_of_a_root_between_the_target_and_the_current_version() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let target = current_of(&session);
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    let patch_of_the_middle = vec![0x5Au8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            UserChange::WriteAtOffset {
                inode: FIRST_FILE,
                offset: FileOffsetInBytes(payload + 100),
                bytes: PartialWriteBytes::new(&patch_of_the_middle).expect("不空"),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 4,
            },
        )
        .expect("部分写");
    let middle = current_of(&session);
    let middle_locations = mapped_locations(
        &middle,
        &key_of_role(&middle, TransactionUnit::Data(DataUnitIndexInFile(1))),
    );
    let patch_of_the_current = vec![0xA5u8; 64];
    session
        .publish_user_change(
            &mut pool.devices,
            UserChange::WriteAtOffset {
                inode: FIRST_FILE,
                offset: FileOffsetInBytes(payload + 100),
                bytes: PartialWriteBytes::new(&patch_of_the_current).expect("不空"),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 5,
            },
        )
        .expect("再部分写");
    let mut current = current_of(&session);
    damage_the_copy_at(&mut pool.devices, &middle_locations[0], BYTES_OF_A_DATA_UNIT);
    watch(&pool, &middle_locations[0]);
    let rolled = roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: target.root.instance,
            checkpoint_txg: target.root.checkpoint_txg,
        },
    );
    let reads = pool.reads_of_the_watched_copy.load(Ordering::SeqCst);
    let other = session.other_unit_reads_with_one_copy_failing_in_this_mount();
    println!("attack-fixes-r3: watermark_read rolled_back={} reads_of_the_damaged_copy={reads} other={other}", rolled.is_ok());
    assert!(rolled.is_ok(), "回退照样做成：{:?}", rolled.as_ref().err());
    assert_eq!(other, 1, "只有水位那一读读到中间那一版的单元");
}

/// 两个数据单元的映射条目位置对换（key 集合相等、条数相等、两道校验和都对、根重封）：key 集合那一判过不了它要判的东西，可写挂载照收。
/// 只读观测：挂不挂得上、挂上之后经会话读整个文件读出什么。与「映射 key 被换」同一类造出来的盘，这一形被攻过零轮。
#[test]
fn a_disk_with_the_positions_of_two_keys_swapped_passes_the_key_set_judgement() {
    let content = content_of_units(3);
    let mut pool = pool_holding(&content);
    let version = pool.version.clone();
    let key_one = key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(1)));
    let key_two = key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(2)));
    let locations_one = mapped_locations(&version, &key_one);
    let locations_two = mapped_locations(&version, &key_two);
    let original = parse_index_node(
        &version
            .units
            .iter()
            .find(|unit| unit.identity == TransactionUnit::MappingTree)
            .expect("三个单元的池映射树根兼叶")
            .bytes,
    )
    .expect("映射树根解得开");
    let entries: Vec<Vec<u8>> = original
        .entries
        .iter()
        .map(|entry| {
            let (entry_key, locations) = parse_mapping_entry(entry).expect("映射条目切得开");
            let locations = if entry_key == key_one {
                locations_two
            } else if entry_key == key_two {
                locations_one
            } else {
                locations
            };
            build_mapping_entry(&entry_key, locations)
        })
        .collect();
    let forged = build_index_node(
        original.tree,
        original.level,
        original.key_width,
        &original.smallest_key,
        &original.largest_key,
        original.birth_txg,
        &version.root.filesystem_identifier,
        original.instance,
        original.birth_sequence,
        u16::try_from(original.entry_width).expect("条目宽"),
        &entries,
    );
    let forged_checksum = crc32_castagnoli(&forged);
    let mut root = version.root;
    for location in &mut root.mapping_root.locations {
        inner_of(&mut pool.devices, location.device)
            .write_at(location.slot.to_device_offset(), &forged, WriteDurability::Plain)
            .expect("写回映射树根");
        location.unit_checksum = forged_checksum;
    }
    let pool_parameters = parameters();
    let ring_slot = target_for_publish(
        root.checkpoint_txg,
        pool_parameters.geometry.root_ring_slots_per_region,
    );
    let device_of_the_ring_slot =
        pool_parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    inner_of(&mut pool.devices, device_of_the_ring_slot)
        .write_at(
            slot_offset(ring_slot, pool_parameters.geometry.fixed_structure_slot_spacing),
            &root.to_slot(usize::try_from(pool_parameters.geometry.physical_block_size).expect("根槽宽")),
            WriteDurability::Plain,
        )
        .expect("写回根槽");
    match mount_writable(&parameters(), &mut pool.devices) {
        Err(refusal) => println!("attack-fixes-r3: swapped_positions mount=refused cause={refusal:?}"),
        Ok(mounted) => {
            let session = MountedSession::of_the_mount(mounted).1;
            let size = current_of(&session).inode_record.expect("第一个文件那条记录").size;
            let read = session.read_the_file(&pool.devices, FIRST_FILE, FileOffsetInBytes(0), size);
            let mapped_one = central_mapping_locations_in_the_version(&current_of(&session), &key_one);
            println!(
                "attack-fixes-r3: swapped_positions mount=accepted read={} same_as_the_file={} mapping_of_key_one_is_the_slot_of_unit_two={}",
                match &read { Ok(_) => "ok".to_string(), Err(refused) => format!("{refused:?}") },
                match &read { Ok(output) => output.bytes == content, Err(_) => false },
                matches!(mapped_one, MappingLookup::Found(found) if found == locations_two),
            );
        }
    }
}
