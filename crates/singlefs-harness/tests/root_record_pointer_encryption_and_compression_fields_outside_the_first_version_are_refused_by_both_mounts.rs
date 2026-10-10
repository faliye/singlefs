//! 根记录里树表、实例表、中央映射树、分配记录树四条指针的头部里，加密与压缩那几段（MAC 16 / nonce 12、算法类型 1、
//! 压缩算法码 1、压后长度 2）第一版恒 0，读者遇到不是第一版的取值判这条根记录损坏，与 journal 新根段那四条指针同一判法
//! （I-2.4（头校验和覆盖范围）「加密关着时恒 0、读者遇到非 0 一律判该结构损坏」，D19（块指针的结构与宽度预算） 已定项 3，
//! D9（加密） 已定项 14；判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2「F2 与 core」那一格）。
//! 判不过的根与 extent 偏移非 0 的根同一个处置：自证不过、择根时不可择（`RootRecord::parse_slot`）。
//!
//! 坏镜像照攻方 f1–f3 那一份造（`research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-c）：最新那条根的根槽里改一条指针头部的
//! 一个字段、整槽自证校验和重封。journal 里那次发布的记录不照字节抄：抄过去的话 journal 读者（新根段四条指针早已判全）也拒那条记录，
//! 两个读者的判混在一个结局里，分不出根记录读者判没判；这里只改根槽，记录照旧完好，可择的根退到更早那条、记录把它施加回同一版。
//! 四条用例四条指针各改一个字段，四类字段（算法类型、压缩算法码、压后长度、MAC / nonce）各出现一次。
//! 改之前根记录读者对这几段只判 extent 偏移：这条根照样自证过、被择，记录一条不施加（攻方 f1–f3 的 `chosen_txg=4 records_applied=0`）。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::ByteReader;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover,
    BadRootRingSlotReading, JournalPolicy, RecoveryOutcome, RootRingSlotReading,
};
use singlefs_core::root_record::{RootRecord, ROOT_CHECKSUM_OFFSET};
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, NODE_POINTER_BYTES};
use singlefs_harness::memory_pool::SparseBlockDevice;

const DEVICE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-m3-rrph";
const WRITE_TIME_SECONDS: u64 = 1_788_300_000;
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512，根槽宽也是 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
/// 第一个文件那次发布写的根的 txg（mkfs 第 0 代、暖机 txg 1、2 之后）。
const FIRST_FILE_TXG: CheckpointTxg = CheckpointTxg(3);
/// 暖机最后那条根的 txg：第一个文件那条根不可择时退到它。
const LAST_WARM_UP_TXG: CheckpointTxg = CheckpointTxg(2);

/// 指针头部里各字段的偏移（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11 的字段表）：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2。
const MAC_IN_THE_POINTER_HEAD: usize = 0;
const ALGORITHM_TYPE_IN_THE_POINTER_HEAD: usize = 16 + 12;
const COMPRESSION_ALGORITHM_CODE_IN_THE_POINTER_HEAD: usize = 16 + 12 + 1;
const COMPRESSED_LENGTH_IN_THE_POINTER_HEAD: usize = 16 + 12 + 1 + 1;

/// 根槽里四条指针的偏移（`RootRecord::to_slot` 的写法；用例里拿解出来的根核一遍）：树表指针紧跟 magic 4、fsid 16、flags 4、
/// 实例代号 4、checkpoint_txg 8；它之后是树 ID 水位 8、回退下界 F 8、自证校验和 32，再接实例表、映射根、分配记录树根三条。
const TREE_TABLE_POINTER_IN_THE_ROOT_SLOT: usize = 4 + 16 + 4 + 4 + 8;
const INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT: usize = ROOT_CHECKSUM_OFFSET + 32;
const MAPPING_ROOT_POINTER_IN_THE_ROOT_SLOT: usize =
    INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT + NODE_POINTER_BYTES_AS_SLOT_OFFSET;
const ALLOCATION_RECORD_TREE_ROOT_POINTER_IN_THE_ROOT_SLOT: usize =
    MAPPING_ROOT_POINTER_IN_THE_ROOT_SLOT + NODE_POINTER_BYTES_AS_SLOT_OFFSET;
/// 一条节点指针的宽度，当根槽里的偏移用：与 `singlefs_format::NODE_POINTER_BYTES` 同一个数（常量里用不了 `try_from`；改槽之前核一遍）。
const NODE_POINTER_BYTES_AS_SLOT_OFFSET: usize = 86;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

/// 根记录里的哪一条指针。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug)]
enum RootRecordPointer {
    TreeTable,
    InstanceTable,
    MappingRoot,
    AllocationRecordTreeRoot,
}

impl RootRecordPointer {
    fn offset_in_the_root_slot(self) -> usize {
        match self {
            RootRecordPointer::TreeTable => TREE_TABLE_POINTER_IN_THE_ROOT_SLOT,
            RootRecordPointer::InstanceTable => INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT,
            RootRecordPointer::MappingRoot => MAPPING_ROOT_POINTER_IN_THE_ROOT_SLOT,
            RootRecordPointer::AllocationRecordTreeRoot => {
                ALLOCATION_RECORD_TREE_ROOT_POINTER_IN_THE_ROOT_SLOT
            }
        }
    }

    fn as_parsed_from(self, root: &RootRecord) -> NodePointer {
        match self {
            RootRecordPointer::TreeTable => root.tree_table,
            RootRecordPointer::InstanceTable => root.instance_table,
            RootRecordPointer::MappingRoot => root.mapping_root,
            RootRecordPointer::AllocationRecordTreeRoot => root.allocation_record_tree_root,
        }
    }
}

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn first_file_content() -> Vec<u8> {
    (0..3000u32)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

/// 两块内存盘上 mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）。
fn pool_after_the_first_file() -> Devices {
    let parameters = parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
    let mut allocator: PoolAllocator =
        allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = first_file_content();
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号 1");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    publish_first_file(
        &mut writer,
        &mut allocator,
        warmed.roots.last().expect("暖机写了两条根"),
        FirstFile {
            content: &content,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("第一个文件");
    devices
}

fn root_ring_slot_reading(devices: &Devices, ring_slot: RootRingSlot) -> RootRingSlotReading {
    let system_configuration = choose_system_configuration(devices).expect("系统配置择得到");
    read_root_ring_slot(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        ring_slot,
    )
}

/// 根环里自证过、txg 是 `checkpoint_txg` 的每一条根，连同它住的槽。
fn self_verified_roots_of(
    devices: &Devices,
    checkpoint_txg: CheckpointTxg,
) -> Vec<(RootRingSlot, RootRecord)> {
    let system_configuration = choose_system_configuration(devices).expect("系统配置择得到");
    every_root_ring_slot(&system_configuration.immutable.sizes)
        .into_iter()
        .filter_map(
            |ring_slot| match root_ring_slot_reading(devices, ring_slot) {
                RootRingSlotReading::SelfVerified(root)
                    if root.checkpoint_txg == checkpoint_txg =>
                {
                    Some((ring_slot, root))
                }
                RootRingSlotReading::SelfVerified(_) | RootRingSlotReading::Bad(_) => None,
            },
        )
        .collect()
}

/// 第一个文件那条根（txg 3）住的每一个根槽：`pointer` 那条指针头部偏移 `field_offset_in_the_pointer_head` 那一字节写成 `value`、
/// 整槽自证校验和重封（攻方 f1–f3 的改法；journal 记录不动，理由在文件头）。交回改过的槽。
fn rewrite_a_pointer_head_byte_in_the_first_file_root(
    devices: &mut Devices,
    pointer: RootRecordPointer,
    field_offset_in_the_pointer_head: usize,
    value: u8,
) -> Vec<RootRingSlot> {
    let parameters = parameters();
    let root_slot_bytes =
        usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽装得进 usize");
    let roots = self_verified_roots_of(devices, FIRST_FILE_TXG);
    for (ring_slot, root) in &roots {
        let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let offset = slot_offset(*ring_slot, parameters.geometry.fixed_structure_slot_spacing);
        let device_under_change = &mut devices
            .iter_mut()
            .find(|(identity, _)| *identity == device)
            .expect("池里有这块盘")
            .1;
        let mut slot = device_under_change.image.read(offset, root_slot_bytes);
        assert_eq!(
            u64::try_from(NODE_POINTER_BYTES_AS_SLOT_OFFSET).expect("86"),
            NODE_POINTER_BYTES,
            "根槽偏移按节点指针宽度 {NODE_POINTER_BYTES} 累加"
        );
        let pointer_offset = pointer.offset_in_the_root_slot();
        assert_eq!(
            NodePointer::read_from(&mut ByteReader::at(&slot, pointer_offset)),
            pointer.as_parsed_from(root),
            "根槽偏移 {pointer_offset} 起的 {NODE_POINTER_BYTES} 字节就是 {pointer:?} 那条指针"
        );
        let byte_at = pointer_offset + field_offset_in_the_pointer_head;
        assert_eq!(slot[byte_at], 0, "第一版写者在指针头部这一段写的是 0");
        slot[byte_at] = value;
        let digest = wide_checksum_with_field_zeroed(&slot, root_slot_bytes, ROOT_CHECKSUM_OFFSET);
        slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        device_under_change.image.write(offset, &slot);
    }
    roots.into_iter().map(|(ring_slot, _)| ring_slot).collect()
}

/// 改过之后两个入口的结局：改过的每个槽读成自证不过；只读恢复择暖机最后那条根（txg 2）、施加第一个文件那条记录、读出文件；
/// 可写挂载同样择 txg 2、施加之后走 txg 3。改之前这几样依次是：自证过、择 txg 3 一条记录不施加。
fn assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
    mut devices: Devices,
    rewritten_slots: &[RootRingSlot],
    what_was_rewritten: &str,
) {
    assert!(
        !rewritten_slots.is_empty(),
        "{what_was_rewritten}：根环里第一个文件那条根至少一份"
    );
    for ring_slot in rewritten_slots {
        assert_eq!(
            root_ring_slot_reading(&devices, *ring_slot),
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
            "{what_was_rewritten}：槽 {ring_slot:?} 这条根判损坏、与自证不过同一个处置"
        );
    }
    let recovered = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        recovered.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), LAST_WARM_UP_TXG),
            content: first_file_content(),
        },
        "{what_was_rewritten}：只读恢复择暖机最后那条根（txg 2），施加记录之后照样读出第一个文件"
    );
    assert_eq!(
        (recovered.effective_root, recovered.journal.prefix_applied),
        (Some((InstanceGeneration(1), FIRST_FILE_TXG)), 1),
        "{what_was_rewritten}：只读恢复施加第一个文件那一条记录、回到 txg 3 那一版"
    );
    let mounted = mount_writable(&parameters(), &mut devices).unwrap_or_else(|error| {
        panic!("{what_was_rewritten}：可写挂载退到更早的根照样挂上，实际 {error:?}")
    });
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied,
        ),
        (LAST_WARM_UP_TXG, FIRST_FILE_TXG, 1),
        "{what_was_rewritten}：可写挂载择 txg 2、施加一条记录、走 txg 3 那一版"
    );
}

/// 场景：第一个文件那条根的树表指针算法类型写 1（登记过的加密算法，第一版不加密；攻方 f1）。
/// 预期：这条根判损坏，两个入口都择暖机最后那条根、由记录施加回同一版。
#[test]
fn a_root_whose_tree_table_pointer_names_an_encryption_algorithm_is_not_chosen_by_either_mount() {
    let mut devices = pool_after_the_first_file();
    let rewritten_slots = rewrite_a_pointer_head_byte_in_the_first_file_root(
        &mut devices,
        RootRecordPointer::TreeTable,
        ALGORITHM_TYPE_IN_THE_POINTER_HEAD,
        1,
    );
    assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
        devices,
        &rewritten_slots,
        "树表指针算法类型 1",
    );
}

/// 场景：第一个文件那条根的实例表指针压缩算法码写 1（登记表第一版只有码 0）。
/// 预期：这条根判损坏，两个入口都择暖机最后那条根、由记录施加回同一版。
#[test]
fn a_root_whose_instance_table_pointer_names_an_unregistered_compression_algorithm_is_not_chosen_by_either_mount(
) {
    let mut devices = pool_after_the_first_file();
    let rewritten_slots = rewrite_a_pointer_head_byte_in_the_first_file_root(
        &mut devices,
        RootRecordPointer::InstanceTable,
        COMPRESSION_ALGORITHM_CODE_IN_THE_POINTER_HEAD,
        1,
    );
    assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
        devices,
        &rewritten_slots,
        "实例表指针压缩算法码 1",
    );
}

/// 场景：第一个文件那条根的映射根指针压缩算法码是 0、压后长度写 1。
/// 预期：这条根判损坏，两个入口都择暖机最后那条根、由记录施加回同一版。
#[test]
fn a_root_whose_mapping_root_pointer_carries_a_compressed_length_without_compression_is_not_chosen_by_either_mount(
) {
    let mut devices = pool_after_the_first_file();
    let rewritten_slots = rewrite_a_pointer_head_byte_in_the_first_file_root(
        &mut devices,
        RootRecordPointer::MappingRoot,
        COMPRESSED_LENGTH_IN_THE_POINTER_HEAD,
        1,
    );
    assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
        devices,
        &rewritten_slots,
        "映射根指针压后长度 1、压缩算法码 0",
    );
}

/// 场景：第一个文件那条根的分配记录树根指针（带文件的一版这一项整条全零）MAC 第一个字节写 1。
/// 预期：这条根判损坏（全零指针的豁免只管 I-2.5 的升序，不管头部这几段），两个入口都择暖机最后那条根、由记录施加回同一版。
#[test]
fn a_root_whose_allocation_record_tree_root_pointer_carries_a_non_zero_mac_is_not_chosen_by_either_mount(
) {
    let mut devices = pool_after_the_first_file();
    let rewritten_slots = rewrite_a_pointer_head_byte_in_the_first_file_root(
        &mut devices,
        RootRecordPointer::AllocationRecordTreeRoot,
        MAC_IN_THE_POINTER_HEAD,
        1,
    );
    assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
        devices,
        &rewritten_slots,
        "分配记录树根指针 MAC 非 0",
    );
}
