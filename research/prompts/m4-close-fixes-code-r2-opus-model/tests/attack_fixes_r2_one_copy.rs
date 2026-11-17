//! m4-close-fixes-code-r2 攻方模型：打第一轮修补（别的读计数、豁免三类主动修复、查不到 key 照计）。
//! 带断言的用例钉今天的行为；观测行以 `attack-fixes-r2:` 起头。盘是内存稀疏盘，池在进程里造，不枚举崩溃状态。
#![allow(dead_code, unused_imports)]

mod common;

use std::num::NonZeroU64;
use std::time::Instant;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::code_two_tree::{
    CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodeContents,
};
use singlefs_core::compaction_plan::{
    RelocationKeptForTheSnapshotsRefusedForSpace, RelocationKeptForTheSnapshotsRefusedNotForSpace,
};
use singlefs_core::instance_table_sweep::{
    InstanceTableSweepConfiguration, InstanceTableSweepSwitch,
};
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration, unmount, ShadowLedger,
};
use singlefs_core::mounted_session::{
    MountedSession, RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing, UserChange,
    UserChangePublished,
};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::records::{build_mapping_entry, parse_mapping_entry};
use singlefs_core::recovery::{
    instance_table_chain_of_root, PoolReaderCountingUnitReadsWithOneCopyFailing,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file,
    publish_sequential_write, warm_up, FirstFile, MappingLookup, PartialWriteBytes, PoolWriter,
    RelocationBatchRefused, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;
type Outcome = RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing;

const FIRST_FILE: InodeNumber = InodeNumber(1);
const FLIPPED_BYTE_IN_THE_UNIT: usize = 4000;
const BYTES_OF_A_NODE: usize = 16384;
const BYTES_OF_A_DATA_UNIT: usize = 32768;
const BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY: usize = 6;
const UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE: u64 = 400;
/// 实例表一片写满 369 行再开下一片（`instance_table.rs` 文件头）：多挂几次，让链长到两片。
const MOUNTS_MAKING_THE_INSTANCE_TABLE_TWO_PAGES: usize = 375;

fn payload_capacity() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

fn content_of_units(units: u64) -> Vec<u8> {
    let length = usize::try_from(units * payload_capacity() - 1000).expect("装得进 usize");
    (0..length)
        .map(|index| u8::try_from((index * 37 + 11) % 253).expect("小于 256"))
        .collect()
}

struct Pool {
    devices: Devices,
    version: TransactionOutput,
}

fn pool_holding(content: &[u8]) -> Pool {
    let parameters = parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let first_content = file_content();
    let version = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
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
    Pool { devices, version }
}

fn memory_pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

fn device_named(devices: &mut Devices, device: DeviceIdentity) -> &mut SparseBlockDevice {
    &mut devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘")
        .1
}

fn damage_the_copy_at(devices: &mut Devices, location: &LocationEntry, unit_bytes: usize) {
    let device = device_named(devices, location.device);
    let mut unit = vec![0u8; unit_bytes];
    device
        .read_at(location.slot.to_device_offset(), &mut unit)
        .expect("读得出那一份");
    unit[FLIPPED_BYTE_IN_THE_UNIT] ^= 0xFF;
    device
        .write_at(
            location.slot.to_device_offset(),
            &unit,
            WriteDurability::Plain,
        )
        .expect("写回改过的那一份");
    assert_ne!(crc32_castagnoli(&unit), location.unit_checksum);
}

fn session_of_a_writable_mount(devices: &mut Devices) -> MountedSession {
    let mounted = mount_writable(&parameters(), devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    session
}

fn current_version_of(session: &MountedSession) -> TransactionOutput {
    session
        .current
        .file_version()
        .expect("现行那一版带文件")
        .clone()
}

fn new_inode_change(write_time_offset: u64) -> UserChange<'static> {
    UserChange::NewInodes {
        count: 1,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + write_time_offset,
    }
}

fn mapping_key_of_role(version: &TransactionOutput, role: TransactionUnit) -> Vec<u8> {
    version
        .mapped_units
        .iter()
        .find(|(identity, _)| *identity == role)
        .expect("这个角色进映射")
        .1
        .clone()
}

fn mapped_locations(version: &TransactionOutput, key: &[u8]) -> Option<[LocationEntry; 2]> {
    match central_mapping_locations_in_the_version(version, key) {
        MappingLookup::Found(locations) => Some(locations),
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
    }
}

fn read_the_first_file_through_the_session(session: &MountedSession, devices: &Devices) {
    let size = current_version_of(session)
        .inode_record
        .expect("第一个文件那条记录")
        .size;
    session
        .read_the_file(devices, FIRST_FILE, FileOffsetInBytes(0), size)
        .expect("经会话读得出");
}

fn outcome_of(published: &UserChangePublished) -> &Outcome {
    &published.relocation_after_the_change_of_the_units_read_with_one_copy_failing
}

fn name_of_the_outcome(outcome: &Outcome) -> String {
    match outcome {
        Outcome::NotNeeded => "NotNeeded".to_string(),
        Outcome::Relocated {
            mapping_keys,
            units_exempt_from_the_mapping_replaced,
            floor_raises,
        } => format!(
            "Relocated keys={} exempt={units_exempt_from_the_mapping_replaced:?} floor_raises={}",
            mapping_keys.len(),
            floor_raises.floor_raises.len()
        ),
        Outcome::NoSpaceAfterRaisingTheFloor { stop, .. } => {
            format!("NoSpaceAfterRaisingTheFloor stop={stop:?}")
        }
        Outcome::RefusedNotForSpace { refusal, .. } => {
            format!("RefusedNotForSpace refusal={refusal:?}")
        }
        Outcome::FloorRaiseFailedWhilePushingForSpace { cause, .. } => {
            format!("FloorRaiseFailedWhilePushingForSpace cause={cause:?}")
        }
    }
}

fn leftmost_mapping_tree_leaf_of(version: &TransactionOutput) -> NodePointer {
    let mapping_tree = &version.central_mapping_tree;
    assert!(mapping_tree.shape.nodes().len() > 1, "前提：映射树多层");
    mapping_tree
        .shape
        .nodes()
        .iter()
        .zip(&mapping_tree.pointers)
        .find(|(node, _)| matches!(node.contents, CodeTwoTreeNodeContents::Leaf { .. }))
        .map(|(_, pointer)| *pointer)
        .expect("多层的树有叶")
}

fn violated_invariants(devices: &Devices) -> Vec<(&'static str, String)> {
    check_pool_image(&memory_pool_of(devices))
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn names_of_the_violated_invariants(devices: &Devices) -> Vec<&'static str> {
    violated_invariants(devices)
        .into_iter()
        .map(|(invariant, _)| invariant)
        .collect()
}

/// 这一读（一个数只坏一份的读者沿链读一遍）读到几份只坏一份：对照，证那条链上确有一份坏的、谁数谁数得到。
fn instance_table_reads_with_one_copy_failing(
    devices: &Devices,
    version: &TransactionOutput,
) -> u64 {
    let counting = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(devices);
    instance_table_chain_of_root(&counting, &version.root).expect("另一份读得出");
    counting.unit_reads_with_one_copy_failing()
}

fn force_one_space_refusal_of_the_batch(session: &mut MountedSession) {
    session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts: NonZeroU64::new(1).expect("拒一次：1 不是 0"),
            },
        );
}

/// `mount.rs` 的 `admission_ceilings_on_caller_inputs_agreeing_with_the_disk` 在推抬 F 那一路沿链读实例表，读者是盘表，不计。
/// 抬得动时紧接着的 `raise_the_floor_through` 再读一遍、计数；抬不动（F 已在上限）就停，这一读是唯一的一读。
/// 用户动作放开扫：弄坏之前多发几次改动，决定 F 抬不抬得动。只有抬不动那几格钉断言。
#[test]
fn the_chain_read_for_the_admission_ceiling_counts_nothing_when_the_floor_cannot_be_raised() {
    let base = pool_holding(&content_of_units(3));
    for changes_before_the_damage in [0u64, 1, 2, 3] {
        let mut pool = copy_of_the_pool(&base);
        let mut session = session_of_a_writable_mount(&mut pool.devices);
        for offset in 0..changes_before_the_damage {
            session
                .publish_user_change(&mut pool.devices, new_inode_change(3 + offset))
                .expect("建一个空 inode，留下一条非空有效根");
        }
        let version = current_version_of(&session);
        let key = mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(1)));
        let data_locations = mapped_locations(&version, &key).expect("在映射里");
        let table_locations = version.root.instance_table.locations;
        damage_the_copy_at(&mut pool.devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
        read_the_first_file_through_the_session(&session, &pool.devices);
        assert_eq!(
            session.other_unit_reads_with_one_copy_failing_in_this_mount(),
            1
        );
        damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
        assert_eq!(
            instance_table_reads_with_one_copy_failing(&pool.devices, &version),
            1,
            "对照：谁数谁数得到那条链上那一份坏的"
        );
        force_one_space_refusal_of_the_batch(&mut session);
        let published = session
            .publish_user_change(&mut pool.devices, new_inode_change(20))
            .expect("建一个空 inode");
        let other_after_the_change = session.other_unit_reads_with_one_copy_failing_in_this_mount();
        let next = session
            .publish_user_change(&mut pool.devices, new_inode_change(21))
            .expect("再建一个空 inode");
        let other_after_the_next = session.other_unit_reads_with_one_copy_failing_in_this_mount();
        let table_after_the_next = current_version_of(&session).root.instance_table.locations;
        println!(
            "attack-fixes-r2: ceiling changes_before_the_damage={changes_before_the_damage} outcome={} other_after_the_change={other_after_the_change} next_outcome={} other_after_the_next={other_after_the_next} table_moved={}",
            name_of_the_outcome(outcome_of(&published)),
            name_of_the_outcome(outcome_of(&next)),
            table_after_the_next != table_locations,
        );
        if matches!(
            outcome_of(&published),
            Outcome::NoSpaceAfterRaisingTheFloor { .. }
        ) {
            assert_eq!(
                other_after_the_change, 1,
                "推抬 F 那一路读到坏的那一份，不计"
            );
            assert!(matches!(outcome_of(&next), Outcome::NotNeeded));
            assert_eq!(table_after_the_next, table_locations, "实例表没换");
        }
    }
}

/// 另发那一批认位置时自己沿链读实例表（`mounted_session.rs` 的 `repair_of_the_exempt_units_at_these_locations(&*devices, …)`），读者是盘表，不计。
/// 账上只有一组换下来的数据单元落点时走这一读：链上那一份坏的这一读读到了，不进账、不换。
#[test]
fn the_chain_read_of_the_repair_itself_counts_nothing_and_leaves_the_damaged_page_in_place() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(1)));
    let data_locations = mapped_locations(&version, &key).expect("在映射里");
    let table_locations = version.root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let new_content: Vec<u8> = content_of_units(3).iter().map(|byte| byte ^ 0x5A).collect();
    let published = session
        .publish_user_change(
            &mut pool.devices,
            UserChange::SequentialWrite(FirstFile {
                content: &new_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            }),
        )
        .expect("整份顺序写");
    let other_after_the_change = session.other_unit_reads_with_one_copy_failing_in_this_mount();
    let after = current_version_of(&session);
    let counted_by_a_counting_reader =
        instance_table_reads_with_one_copy_failing(&pool.devices, &after);
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("建一个空 inode");
    println!(
        "attack-fixes-r2: repair_chain_read outcome={} other_after_the_change={other_after_the_change} counted_by_a_counting_reader={counted_by_a_counting_reader} next_outcome={} other_after_the_next={} table_moved={}",
        name_of_the_outcome(outcome_of(&published)),
        name_of_the_outcome(outcome_of(&next)),
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        current_version_of(&session).root.instance_table.locations != table_locations,
    );
    assert!(matches!(outcome_of(&published), Outcome::NotNeeded));
    assert_eq!(other_after_the_change, 1, "只有释放之前读盘核那一读进数");
    assert_eq!(counted_by_a_counting_reader, 1, "对照：链上确有一份坏的");
    assert!(matches!(outcome_of(&next), Outcome::NotNeeded));
    assert_eq!(
        current_version_of(&session).root.instance_table.locations,
        table_locations,
        "实例表没换"
    );
}

/// 同一份坏副本挂载读过之后：数据单元的那一组挂载交回之前从账上去掉（认成 key），之后再读、或另发那一批释放之前读盘核，别的读那个数加一；
/// 映射树叶的那一组留在账上（认不出 key），之后再读、再换，别的读那个数不加。两边都是挂载读到一次。
#[test]
fn the_other_reads_count_differs_between_a_data_unit_and_a_mapping_tree_leaf_read_while_mounting() {
    let mut data_pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(
        &data_pool.version,
        TransactionUnit::Data(DataUnitIndexInFile(1)),
    );
    let data_locations = mapped_locations(&data_pool.version, &key).expect("在映射里");
    damage_the_copy_at(
        &mut data_pool.devices,
        &data_locations[0],
        BYTES_OF_A_DATA_UNIT,
    );
    let mut read_again = data_pool_copy(&data_pool);
    let session = session_of_a_writable_mount(&mut read_again);
    let mount_count_of_the_data_unit = session.unit_reads_with_one_copy_failing_while_mounting();
    read_the_first_file_through_the_session(&session, &read_again);
    let data_unit_read_again = session.other_unit_reads_with_one_copy_failing_in_this_mount();
    let mut not_read_again = data_pool_copy(&data_pool);
    let mut session_not_reading = session_of_a_writable_mount(&mut not_read_again);
    let published = session_not_reading
        .publish_user_change(&mut not_read_again, new_inode_change(3))
        .expect("建一个空 inode");
    let data_unit_relocated_without_another_read =
        session_not_reading.other_unit_reads_with_one_copy_failing_in_this_mount();
    assert!(matches!(outcome_of(&published), Outcome::Relocated { .. }));
    let mut leaf_pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let leaf = leftmost_mapping_tree_leaf_of(&leaf_pool.version);
    damage_the_copy_at(&mut leaf_pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut leaf_session = session_of_a_writable_mount(&mut leaf_pool.devices);
    let mount_count_of_the_leaf = leaf_session.unit_reads_with_one_copy_failing_while_mounting();
    read_the_first_file_through_the_session(&leaf_session, &leaf_pool.devices);
    let leaf_read_again = leaf_session.other_unit_reads_with_one_copy_failing_in_this_mount();
    let leaf_published = leaf_session
        .publish_user_change(&mut leaf_pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let leaf_replaced = leaf_session.other_unit_reads_with_one_copy_failing_in_this_mount();
    assert!(matches!(
        outcome_of(&leaf_published),
        Outcome::Relocated { .. }
    ));
    println!(
        "attack-fixes-r2: other_reads_after_the_mount data_mount={mount_count_of_the_data_unit} data_read_again={data_unit_read_again} data_relocated_without_another_read={data_unit_relocated_without_another_read} leaf_mount={mount_count_of_the_leaf} leaf_read_again={leaf_read_again} leaf_replaced={leaf_replaced}"
    );
    assert_eq!(
        (mount_count_of_the_data_unit, mount_count_of_the_leaf),
        (1, 1)
    );
    assert_eq!(
        data_unit_read_again, 1,
        "数据单元挂载读过、再读：别的读加一"
    );
    assert_eq!(
        data_unit_relocated_without_another_read, 1,
        "数据单元挂载读过、没人再读：那一批释放之前读盘核自己加一"
    );
    assert_eq!(leaf_read_again, 0, "映射树叶挂载读过、再读：别的读不加");
    assert_eq!(leaf_replaced, 0, "映射树叶挂载读过、换下：别的读不加");
}

fn copy_of_the_pool(pool: &Pool) -> Pool {
    Pool {
        devices: data_pool_copy(pool),
        version: pool.version.clone(),
    }
}

fn data_pool_copy(pool: &Pool) -> Devices {
    pool.devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

fn mapping_key_with_the_transaction_number_raised_by_one(key: &[u8]) -> Vec<u8> {
    let mut transaction_number_bytes = [0u8; 8];
    transaction_number_bytes[..BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY]
        .copy_from_slice(&key[key.len() - BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY..]);
    let raised = u64::from_le_bytes(transaction_number_bytes) + 1;
    let mut raised_key = key.to_vec();
    let tail_starts_at = raised_key.len() - BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY;
    raised_key[tail_starts_at..]
        .copy_from_slice(&raised.to_le_bytes()[..BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY]);
    raised_key
}

/// 同被判用例那一份造法：`version` 的映射树根兼叶上 `key` 那一条换成 `replacement`，位置条目不动，封得对、仍升序，根槽重封。
fn replace_the_key_in_the_mapping_root_on_disk(
    devices: &mut Devices,
    version: &TransactionOutput,
    key: &[u8],
    replacement: &[u8],
) -> RootRecord {
    let mut root = version.root;
    let original = parse_index_node(
        &version
            .units
            .iter()
            .find(|unit| unit.identity == TransactionUnit::MappingTree)
            .expect("三个单元的池映射树根兼叶")
            .bytes,
    )
    .expect("映射树根解得开");
    let entries: Vec<(Vec<u8>, [LocationEntry; 2])> = original
        .entries
        .iter()
        .map(|entry| {
            let (entry_key, locations) = parse_mapping_entry(entry).expect("映射条目切得开");
            if entry_key == key {
                (replacement.to_vec(), locations)
            } else {
                (entry_key, locations)
            }
        })
        .collect();
    assert!(
        entries.windows(2).all(|pair| {
            CodeTwoTreeKey::new(&pair[0].0, CodeTwoKeyFieldWidths::CENTRAL_MAPPING)
                < CodeTwoTreeKey::new(&pair[1].0, CodeTwoKeyFieldWidths::CENTRAL_MAPPING)
        }),
        "前提：换了 key 之后条目仍严格升序"
    );
    let forged = build_index_node(
        original.tree,
        original.level,
        original.key_width,
        &original.smallest_key,
        &original.largest_key,
        original.birth_txg,
        &root.filesystem_identifier,
        original.instance,
        original.birth_sequence,
        u16::try_from(original.entry_width).expect("条目宽"),
        &entries
            .iter()
            .map(|(entry_key, locations)| build_mapping_entry(entry_key, *locations))
            .collect::<Vec<Vec<u8>>>(),
    );
    let forged_checksum = crc32_castagnoli(&forged);
    for location in &mut root.mapping_root.locations {
        device_named(devices, location.device)
            .write_at(
                location.slot.to_device_offset(),
                &forged,
                WriteDurability::Plain,
            )
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
    device_named(devices, device_of_the_ring_slot)
        .write_at(
            slot_offset(
                ring_slot,
                pool_parameters.geometry.fixed_structure_slot_spacing,
            ),
            &root.to_slot(
                usize::try_from(pool_parameters.geometry.physical_block_size).expect("根槽宽"),
            ),
            WriteDurability::Plain,
        )
        .expect("写回根槽");
    root
}

/// 搬迁之后提示是过期的（这批修补让它在产品路径上走得到）：映射 key 被换、封得对的盘，挂载照「查不到就留提示」收下，第一次发布按指针重算，
/// 把这把 key 盖回去时指的是提示那一组（隔离着的旧落点），映射里那组真落点（搬到的新槽）丢了。被判用例 `a_disk_with_a_key_replaced_…` 只罩提示没过期那一形。
#[test]
fn a_key_replaced_after_a_relocation_points_the_mapping_back_at_the_quarantined_copies() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(2)));
    let old_locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &old_locations[0], BYTES_OF_A_DATA_UNIT);
    read_the_first_file_through_the_session(&session, &pool.devices);
    let relocated = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert!(matches!(outcome_of(&relocated), Outcome::Relocated { .. }));
    let after = current_version_of(&session);
    let new_locations = mapped_locations(&after, &key).expect("搬之后映射里查得到");
    assert_ne!(
        new_locations[0].slot, old_locations[0].slot,
        "前提：搬到了新槽"
    );
    drop(session);
    let replacement = mapping_key_with_the_transaction_number_raised_by_one(&key);
    replace_the_key_in_the_mapping_root_on_disk(&mut pool.devices, &after, &key, &replacement);
    let mut mounted_over_the_replaced_key = session_of_a_writable_mount(&mut pool.devices);
    let version_after_the_mount = current_version_of(&mounted_over_the_replaced_key);
    let mapped_after_the_mount = mapped_locations(&version_after_the_mount, &key);
    println!(
        "attack-fixes-r2: key_replaced_after_relocation mapped_is_old={} mapped_is_new={} mount_count={} violated_after_the_mount={:?}",
        mapped_after_the_mount == Some(old_locations),
        mapped_after_the_mount == Some(new_locations),
        mounted_over_the_replaced_key.unit_reads_with_one_copy_failing_while_mounting(),
        names_of_the_violated_invariants(&pool.devices),
    );
    let next = mounted_over_the_replaced_key
        .publish_user_change(&mut pool.devices, new_inode_change(4))
        .expect("建一个空 inode");
    println!(
        "attack-fixes-r2: key_replaced_after_relocation next_outcome={} violated_after_the_change={:?}",
        name_of_the_outcome(outcome_of(&next)),
        names_of_the_violated_invariants(&pool.devices),
    );
    assert_eq!(
        mapped_after_the_mount,
        Some(old_locations),
        "挂载把这把 key 盖回提示那一组，不是搬到的新槽"
    );
}

/// 放回字节之外的判法：修好之后卸载（F 抬到现行那一版，改动之前的根离开候选集），不放回，直接判真盘。
/// 这几条只打观测行、不钉断言：绿就说明「放回」不是必需、这一形可以替代它；红就是放回之前看不到的东西。头一条是对照：没修的坏副本照样红，这一判有牙。
#[test]
fn the_pool_checker_on_the_disk_with_the_damage_still_on_it_after_the_repair_and_the_unmount() {
    let base = pool_holding(&content_of_units(3));
    let unmount_and_judge = |name: &str, session: &mut MountedSession, devices: &mut Devices| {
        unmount(
            &parameters(),
            devices,
            &mut session.allocator,
            &mut session.current,
            session.shadow_ledger,
        )
        .expect("卸载");
        println!(
            "attack-fixes-r2: unrestored {name} violated={:?}",
            violated_invariants(devices)
        );
    };
    let mut pool = copy_of_the_pool(&base);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(1)));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    unmount_and_judge(
        "control_data_unit_not_repaired",
        &mut session,
        &mut pool.devices,
    );

    let mut pool = copy_of_the_pool(&base);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(1)));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    read_the_first_file_through_the_session(&session, &pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    unmount_and_judge("data_unit_relocated", &mut session, &mut pool.devices);

    let mut pool = copy_of_the_pool(&base);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table = current_version_of(&session).root.tree_table.locations;
    damage_the_copy_at(&mut pool.devices, &table[0], BYTES_OF_A_NODE);
    read_the_first_file_through_the_session(&session, &pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    unmount_and_judge("tree_table_replaced", &mut session, &mut pool.devices);

    let mut pool = copy_of_the_pool(&base);
    let instance_table = pool.version.root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &instance_table[0], BYTES_OF_A_DATA_UNIT);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    unmount_and_judge(
        "instance_table_replaced_by_the_mount",
        &mut session,
        &mut pool.devices,
    );

    let mut pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let leaf = leftmost_mapping_tree_leaf_of(&pool.version);
    damage_the_copy_at(&mut pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    unmount_and_judge(
        "mapping_tree_leaf_rewritten",
        &mut session,
        &mut pool.devices,
    );
}

fn partial_write_into_the_second_unit<'content>(
    patch: &'content [u8],
    write_time_offset: u64,
) -> UserChange<'content> {
    UserChange::WriteAtOffset {
        inode: FIRST_FILE,
        offset: FileOffsetInBytes(payload_capacity() + 100),
        bytes: PartialWriteBytes::new(patch).expect("写的字节不是空的"),
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + write_time_offset,
    }
}

/// 设计问题第 7 条：账上有一组已经换下来的落点（与实例表无关）、实例表两份都读不出时，整批拒，连带这一批要搬的好 key 一起丢、只试这一次。
/// 那一组是不是实例表片答不出，可实例表既然读不出也写不了 `InstanceTablePlan::Rewrite`；两份都读不出时跳过实例表、照搬 key 同样是「任何写之前」能判的。
#[test]
fn a_stale_location_and_an_unreadable_instance_table_drop_the_relocation_of_an_unrelated_key() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key_of_the_unit_kept =
        mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(0)));
    let key_of_the_unit_rewritten =
        mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(2)));
    let locations_of_the_unit_kept =
        mapped_locations(&version, &key_of_the_unit_kept).expect("在映射里");
    let locations_of_the_unit_rewritten =
        mapped_locations(&version, &key_of_the_unit_rewritten).expect("在映射里");
    damage_the_copy_at(
        &mut pool.devices,
        &locations_of_the_unit_kept[0],
        BYTES_OF_A_DATA_UNIT,
    );
    damage_the_copy_at(
        &mut pool.devices,
        &locations_of_the_unit_rewritten[0],
        BYTES_OF_A_DATA_UNIT,
    );
    read_the_first_file_through_the_session(&session, &pool.devices);
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        2
    );
    for location in &version.root.instance_table.locations {
        damage_the_copy_at(&mut pool.devices, location, BYTES_OF_A_DATA_UNIT);
    }
    let patch = vec![0x5Au8; 64];
    let published = session
        .publish_user_change(
            &mut pool.devices,
            partial_write_into_the_second_unit(&patch, 3),
        )
        .expect("部分写");
    let after = current_version_of(&session);
    println!(
        "attack-fixes-r2: unreadable_instance_table outcome={} unit_kept_still_at_the_damaged_location={}",
        name_of_the_outcome(outcome_of(&published)),
        mapped_locations(&after, &key_of_the_unit_kept) == Some(locations_of_the_unit_kept),
    );
    assert!(
        matches!(
            outcome_of(&published),
            Outcome::RefusedNotForSpace { refusal, .. }
                if matches!(
                    **refusal,
                    RelocationBatchRefused::InstanceTableOfTheVersionUnreadable(_)
                )
        ),
        "{:?}",
        outcome_of(&published)
    );
    assert_eq!(
        mapped_locations(&after, &key_of_the_unit_kept),
        Some(locations_of_the_unit_kept),
        "好 key 没搬"
    );
}

/// 「只试一次」的射程：那一批被拒（不是空间不够）之后账上清掉，可同一组再读到就再记、下一次改动再试。
/// 被判用例 `a_mapping_tree_leaf_rewrite_refused_not_for_space_is_not_tried_again_by_the_next_change` 两次改动之间没有读，罩不到这一形。
#[test]
fn a_batch_refused_not_for_space_is_tried_again_once_the_unit_is_read_again() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(1)));
    let locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    session
        .compaction
        .force_the_relocation_kept_for_the_snapshots_refused_not_for_space_for_tests(
            RelocationKeptForTheSnapshotsRefusedNotForSpace::ForcedOnEveryAttemptForTests,
        );
    let mut outcomes = Vec::new();
    for attempt in 0..3u64 {
        read_the_first_file_through_the_session(&session, &pool.devices);
        let published = session
            .publish_user_change(&mut pool.devices, new_inode_change(3 + attempt))
            .expect("建一个空 inode");
        outcomes.push(name_of_the_outcome(outcome_of(&published)));
        assert!(
            matches!(outcome_of(&published), Outcome::RefusedNotForSpace { .. }),
            "每一次读过之后的那次改动都再试：{:?}",
            outcome_of(&published)
        );
    }
    println!(
        "attack-fixes-r2: tried_again other={} outcomes={outcomes:?}",
        session.other_unit_reads_with_one_copy_failing_in_this_mount()
    );
    assert_eq!(
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        3
    );
}

/// 多片实例表链：实现员报告说没有用例。挂载时不扫，挂到第二片；第二片（不是根记录直接指着的那一片）一份坏，推抬 F 读到，下一次改动整条链重写。
/// 钉：两片都报在结局里、行不变、新链不引用旧槽；坏的那一片两份隔离、好的那一片不隔离。观测行带耗时。
#[test]
fn an_instance_table_chain_of_two_pages_with_the_second_page_damaged_is_rewritten_whole() {
    let started = Instant::now();
    let configuration = InstanceTableSweepConfiguration {
        switch: InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted,
        ..InstanceTableSweepConfiguration::PRODUCT
    };
    let mut pool = pool_holding(&content_of_units(3));
    let mount_keeping_the_rows = |devices: &mut Devices| {
        mount_writable_with_instance_table_sweep_configuration(
            &parameters(),
            devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            configuration,
        )
        .expect("可写挂载")
    };
    for _ in 0..MOUNTS_MAKING_THE_INSTANCE_TABLE_TWO_PAGES {
        mount_keeping_the_rows(&mut pool.devices);
    }
    let (_, mut session) = MountedSession::of_the_mount(mount_keeping_the_rows(&mut pool.devices));
    for offset in 0..2u64 {
        session
            .publish_user_change(&mut pool.devices, new_inode_change(3 + offset))
            .expect("建一个空 inode，留下一条非空有效根");
    }
    let version = current_version_of(&session);
    let chain = instance_table_chain_of_root(&pool.devices, &version.root).expect("实例表读得出");
    assert_eq!(chain.page_pointers.len(), 2, "前提：链长到两片");
    let key = mapping_key_of_role(&version, TransactionUnit::Data(DataUnitIndexInFile(1)));
    let data_locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &data_locations[0], BYTES_OF_A_DATA_UNIT);
    read_the_first_file_through_the_session(&session, &pool.devices);
    let second_page = chain.page_pointers[1].locations;
    damage_the_copy_at(&mut pool.devices, &second_page[0], BYTES_OF_A_DATA_UNIT);
    force_one_space_refusal_of_the_batch(&mut session);
    let published = session
        .publish_user_change(&mut pool.devices, new_inode_change(20))
        .expect("建一个空 inode");
    let next = session
        .publish_user_change(&mut pool.devices, new_inode_change(21))
        .expect("再建一个空 inode");
    let after = current_version_of(&session);
    let chain_after = instance_table_chain_of_root(&pool.devices, &after.root).expect("读得回");
    let quarantined: Vec<(TransactionUnit, u64)> = after
        .quarantined_after_release_checksum_mismatch
        .iter()
        .map(|copy| (copy.unit, copy.placement.slot.0))
        .collect();
    println!(
        "attack-fixes-r2: two_pages outcome={} next_outcome={} other={} quarantined={quarantined:?} elapsed_seconds={}",
        name_of_the_outcome(outcome_of(&published)),
        name_of_the_outcome(outcome_of(&next)),
        session.other_unit_reads_with_one_copy_failing_in_this_mount(),
        started.elapsed().as_secs(),
    );
    assert_eq!(chain_after.records.rows, chain.records.rows, "行不变");
    for pointer in &chain_after.page_pointers {
        for old in chain.page_pointers.iter().flat_map(|page| page.locations) {
            assert_ne!(pointer.locations[0].slot, old.slot, "新链不引用旧槽");
        }
    }
}
