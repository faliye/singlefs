//! harness 测试文件 `crates/singlefs-harness/tests/instance_table_chain_read.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{memory_pool_of_sparse_devices, parameters, IMAGE_BYTES};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_checker::image::INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::ByteWriter;
use singlefs_core::instance_table::{InstanceRow, InstanceTablePageIndex};
use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord};
use singlefs_core::make_filesystem::{
    instance_table_chain_record, location_entries, make_filesystem, INSTANCE_TABLE_SLOT,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::pointer::{BirthSequence, NodePointer, PointerHead};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, instance_table_of_root, readable_roots, PoolReader,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::unit::{build_packed_unit, parse_packed_unit, unit_filesystem_identifier};
use singlefs_format::{DATA_UNIT_BYTES, INSTANCE_ROW_BYTES, JOURNAL_RECORD_BYTES};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 手搭的第二片落在这个槽上：单元区里、32768 对齐（偶数槽），离开放段（50240 起）远，这几次挂载写不到它。
pub const SECOND_PAGE_SLOT: SlotNumber = SlotNumber(60_000);

/// mkfs 之后连着可写挂载几次：最新那一版是实例 4，它那张表里有实例 1、2、3 各一行。
pub const WRITABLE_MOUNTS: u32 = 4;

pub type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 两块 4 GiB 内存盘上 mkfs，之后连着可写挂载 `WRITABLE_MOUNTS` 次、不写文件。
pub fn pool_without_file_after_writable_mounts() -> (Devices, SharedStream) {
    let stream = SharedStream::new();
    let mut devices: Devices = DISKS
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    for mount_number in 1..=WRITABLE_MOUNTS {
        let mounted = mount_writable(&parameters(), &mut devices)
            .unwrap_or_else(|error| panic!("第 {mount_number} 次可写挂载要成立：{error:?}"));
        assert_eq!(mounted.output.instance, InstanceGeneration(mount_number));
    }
    (devices, stream)
}

pub fn newest_root(devices: &Devices) -> RootRecord {
    let system_configuration = choose_system_configuration(devices).expect("系统配置");
    choose_root(devices, &system_configuration).expect("最新根")
}

pub fn read_unit(devices: &Devices, slot: SlotNumber) -> Vec<u8> {
    PoolReader::read(
        devices,
        DISKS[0],
        slot.to_device_offset(),
        usize::try_from(DATA_UNIT_BYTES).expect("32768"),
    )
    .expect("读得到")
}

/// 两块盘同槽写一个单元（不经录制器：手搭镜像不算挂载发的写）。
pub fn write_unit(devices: &mut Devices, slot: SlotNumber, unit: &[u8]) {
    for (_, device) in devices.iter_mut() {
        device
            .wrapped_device_mut()
            .image
            .write(slot.to_device_offset(), unit);
    }
}

/// 一条链指针记录：`kind 1 | 有无下一片 1 | 位置指针 86`（D18（块里携带什么信息） 已定项 11）。
pub fn chain_record(has_next_page: u8, pointer: &NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
    writer.put_u8(1);
    writer.put_u8(has_next_page);
    pointer.write_to(&mut writer);
    writer.assert_position(INSTANCE_ROW_BYTES, "链指针记录");
    writer.into_bytes()
}

/// 「有无下一片」= 1、位置指针指着下一片的链指针记录（读者与 checker 按字段名读的「有」）。
pub fn chain_record_to(next_page: &NodePointer) -> Vec<u8> {
    chain_record(1, next_page)
}

/// 指着某个槽上那个单元的指针：位置条目两盘同槽、带那个单元的整单元校验和，头与尾段照 `like` 的（实例表单元的指针同型，
/// D19（块指针的结构与宽度预算） 已定项 7 / 8）。
pub fn pointer_to_unit_at(like: &NodePointer, slot: SlotNumber, unit: &[u8]) -> NodePointer {
    NodePointer {
        head: like.head,
        locations: location_entries(&DISKS, slot, unit),
        instance: like.instance,
        birth_sequence: like.birth_sequence,
    }
}

/// 手搭好的两片链。
pub struct TwoPageChain {
    pub first_page_slot: SlotNumber,
    /// 第 0 片的链指针记录原本该指着的第二片（写在 `SECOND_PAGE_SLOT`）。
    pub second_page_pointer: NodePointer,
}

/// 把最新那一版的实例表换成两片：第 0 片装 `first_page_rows` 加 `first_page_chain_record(第二片的指针)`，原地重写；
/// 第二片装 `second_page_rows` 加「无下一片」，身份 (0, 4, 1, 0)，写在 `SECOND_PAGE_SLOT`。两片的诞生代号与写序照原来那一片；
/// 第二片的出生序号取第 0 片的加一（值没有条款定；读者不看它，checker 只判它与指着它的链指针记的相等，I-1.2 的遍历方向）。指着原来那一片的每一条根（写行那次发布与之后的暖机空发布）
/// 换上第 0 片的新整单元校验和、重写根槽。
pub fn write_two_page_chain(
    devices: &mut Devices,
    first_page_rows: &[InstanceRow],
    first_page_chain_record: impl Fn(&NodePointer) -> Vec<u8>,
    second_page_rows: &[InstanceRow],
) -> TwoPageChain {
    let system_configuration = choose_system_configuration(&*devices).expect("系统配置");
    let newest = choose_root(&*devices, &system_configuration).expect("最新根");
    let original_first_page_pointer = newest.instance_table;
    let first_page_slot = original_first_page_pointer.locations[0].slot;
    let original = parse_packed_unit(&read_unit(devices, first_page_slot)).expect("第 0 片解得开");
    let filesystem_identifier = parameters().filesystem_identifier;
    let record_width = u16::try_from(INSTANCE_ROW_BYTES).expect("88");

    let second_page_birth_sequence = BirthSequence(original.birth_sequence.0 + 1);
    let second_page_records: Vec<Vec<u8>> = second_page_rows
        .iter()
        .map(InstanceRow::to_bytes)
        .chain([instance_table_chain_record()])
        .collect();
    let second_page = build_packed_unit(
        InstanceTablePageIndex(1).packed_identity(),
        record_width,
        &second_page_records,
        original.birth_txg,
        &filesystem_identifier,
        original.write_order,
        second_page_birth_sequence,
    );
    write_unit(devices, SECOND_PAGE_SLOT, &second_page);
    let second_page_pointer = NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(0),
            birth_txg: original.birth_txg,
        },
        locations: location_entries(&DISKS, SECOND_PAGE_SLOT, &second_page),
        instance: original.write_order.instance,
        birth_sequence: second_page_birth_sequence,
    };

    let first_page_records: Vec<Vec<u8>> = first_page_rows
        .iter()
        .map(InstanceRow::to_bytes)
        .chain([first_page_chain_record(&second_page_pointer)])
        .collect();
    let first_page = build_packed_unit(
        InstanceTablePageIndex::FIRST.packed_identity(),
        record_width,
        &first_page_records,
        original.birth_txg,
        &filesystem_identifier,
        original.write_order,
        original.birth_sequence,
    );
    write_unit(devices, first_page_slot, &first_page);
    let first_page_pointer =
        pointer_to_unit_at(&original_first_page_pointer, first_page_slot, &first_page);

    let slots_per_region = system_configuration
        .immutable
        .sizes
        .root_ring_slots_per_region;
    let spacing = parameters().geometry.fixed_structure_slot_spacing;
    let root_slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("512");
    let roots = readable_roots(
        &*devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let mut rewritten_roots = 0;
    for root in roots
        .iter()
        .filter(|root| root.instance_table == original_first_page_pointer)
    {
        let rewritten = RootRecord {
            filesystem_identifier: root.filesystem_identifier,
            unmount_marker: root.unmount_marker,
            instance: root.instance,
            checkpoint_txg: root.checkpoint_txg,
            tree_table: root.tree_table,
            tree_identifier_watermark: root.tree_identifier_watermark,
            rollback_floor: root.rollback_floor,
            instance_table: first_page_pointer,
            mapping_root: root.mapping_root,
            allocation_record_tree_root: root.allocation_record_tree_root,
            reclaim_threshold: root.reclaim_threshold,
            pinned_ring_slot: root.pinned_ring_slot,
        };
        let target = target_for_publish(root.checkpoint_txg, slots_per_region);
        let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
        let offset: DeviceOffsetInBytes = slot_offset(target, spacing);
        devices
            .iter_mut()
            .find(|(identity, _)| *identity == device)
            .expect("区域的盘在池里")
            .1
            .wrapped_device_mut()
            .image
            .write(offset, &rewritten.to_slot(root_slot_bytes));
        rewritten_roots += 1;
    }
    assert!(rewritten_roots >= 1, "至少最新那条根指着原来那一片");
    let rewritten_versions: Vec<(InstanceGeneration, CheckpointTxg)> = roots
        .iter()
        .filter(|root| root.instance_table == original_first_page_pointer)
        .map(|root| (root.instance, root.checkpoint_txg))
        .collect();
    point_the_records_of_these_versions_at_the_first_page(
        devices,
        &rewritten_versions,
        &first_page_pointer,
        system_configuration.immutable.sizes.journal_ring_bytes,
    );
    TwoPageChain {
        first_page_slot,
        second_page_pointer,
    }
}

/// 根环里换了实例表指针的那几版，同 (实例, txg) 的 journal 记录新根段里的实例表指针也换成 `first_page_pointer`（I-8.10 从第 8 批起比七段，
/// 有它；盘上留下「写者从一开始就写出了这样的根」的样子），之后同实例的记录反向链按前一条改后的记录头重算（I-8.6 只链实例内逻辑前一条），
/// `JournalRecord::to_bytes` 重封，两盘各一份。环里从计数器 1 起读到第一格解不出就停：迭代上界是环的槽数。
pub fn point_the_records_of_these_versions_at_the_first_page(
    devices: &mut Devices,
    versions: &[(InstanceGeneration, CheckpointTxg)],
    first_page_pointer: &NodePointer,
    ring_bytes: u64,
) {
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in DISKS {
        // 跨轮携带：计数器小 1 的那一条若被改过，记它的实例与改后的字节（下一条的反向链按它重算）。
        let mut rewritten_previous_record: Option<(InstanceGeneration, Vec<u8>)> = None;
        for counter in 1..=ring_bytes / JOURNAL_RECORD_BYTES {
            let offset = record_offset(counter, ring_bytes);
            let Some(bytes) = PoolReader::read(&*devices, device, offset, record_bytes) else {
                break;
            };
            let Some(record) = JournalRecord::parse(&bytes, filesystem_identifier) else {
                break;
            };
            let mut rewritten = record.clone();
            if versions.contains(&(record.instance, record.checkpoint_txg)) {
                rewritten.new_instance_table = *first_page_pointer;
            }
            if let Some((previous_instance, previous_bytes)) = &rewritten_previous_record {
                if *previous_instance == record.instance {
                    rewritten.back_chain = back_chain_of(previous_bytes);
                }
            }
            rewritten_previous_record = if rewritten == record {
                None
            } else {
                let rewritten_bytes = rewritten.to_bytes();
                devices
                    .iter_mut()
                    .find(|(identity, _)| *identity == device)
                    .expect("盘在池里")
                    .1
                    .wrapped_device_mut()
                    .image
                    .write(offset, &rewritten_bytes);
                Some((record.instance, rewritten_bytes))
            };
        }
    }
}

/// 最新那一版原来那张表（一片）里的行：实例 1、2、3 各一行。
pub fn rows_of_the_newest_table(devices: &Devices) -> Vec<InstanceRow> {
    let rows = instance_table_of_root(devices, &newest_root(devices))
        .expect("实例表读得出")
        .rows;
    assert_eq!(
        rows.iter().map(|row| row.instance).collect::<Vec<_>>(),
        vec![
            InstanceGeneration(1),
            InstanceGeneration(2),
            InstanceGeneration(3)
        ],
        "连着可写挂载 4 次之后最新那张表里是实例 1、2、3 各一行"
    );
    rows
}

/// 合法的两片链：前两行在第 0 片，第三行在第二片。
pub fn pool_with_a_valid_two_page_chain() -> (Devices, SharedStream, Vec<InstanceRow>, TwoPageChain)
{
    let (mut devices, stream) = pool_without_file_after_writable_mounts();
    let rows = rows_of_the_newest_table(&devices);
    let chain = write_two_page_chain(&mut devices, &rows[..2], chain_record_to, &rows[2..]);
    (devices, stream, rows, chain)
}

/// 合法的两片链上池级 checker 一条都不红：第二片的身份、校验和、位置条目次序、记录形态与链指针记录都判过（I-1.1、I-2.1、I-2.5、I-3.14
/// 判成成立，不是「不适用」），两片的行接起来判 I-3.8。
pub fn the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds(
    images: &mut PoolCheckerImages,
) {
    let (devices, _stream, _rows, _chain) = pool_with_a_valid_two_page_chain();
    // 这几条在两片链上判成立
    images.hand_out(
        "两片合法的链",
        || memory_pool_of_sparse_devices(&devices),
        PoolCheckerExpectation::no_invariant_violated()
            .and_holds("I-1.1")
            .and_holds("I-2.1")
            .and_holds("I-2.5")
            .and_holds("I-3.8")
            .and_holds("I-1.7")
            .and_holds(INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED),
    );
}

/// 第二片的链指针指错（坏镜像）：第 0 片的链指针指到 mkfs 写的那一片实例表（槽 50176，身份 (0, 4, 0, 0)），
/// 位置条目带的就是那一片的整单元校验和——I-2.1 挡不住，链上第 1 片的身份该是 (0, 4, 1, 0)，I-1.1 红；
/// 链指针的诞生代号、实例、尾段照第二片的指针抄，与 mkfs 那一片头里的 (0, 0, 0) 不等，I-1.2 红（那一行「实例表单元按例外只免谓词那一半，
/// 链上每一片的指针与头相等仍挂遍历方向」）；
/// 链从这里断了，最新根走不完（I-7.2），它引用的单元对不上（I-4.8、I-7.4 按最新根那一格红）；
/// mkfs 那一片在写行那次发布被换下、记着已释放（释放代 3），候选集里每一条根又经这条链指针引用它，I-3.9 红
/// （那一行「若每一条有效根都还引用它，它不许带已释放标志」；只跳过比最后引用它的那条根晚而走不全的根，这里最后引用它的就是最新根）。
pub fn chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant(
    images: &mut PoolCheckerImages,
) {
    let (mut devices, _stream) = pool_without_file_after_writable_mounts();
    let rows = rows_of_the_newest_table(&devices);
    let make_filesystem_page = read_unit(&devices, INSTANCE_TABLE_SLOT);
    let chain = write_two_page_chain(
        &mut devices,
        &rows[..2],
        |second_page_pointer| {
            chain_record_to(&pointer_to_unit_at(
                second_page_pointer,
                INSTANCE_TABLE_SLOT,
                &make_filesystem_page,
            ))
        },
        &rows[2..],
    );
    assert_ne!(chain.first_page_slot, INSTANCE_TABLE_SLOT);
    // I-3.5：mkfs 种的那一片（诞生代号 0）被最新那一版的链再次引用，前一版不引用它。
    // I-1.2 红在链上第 1 片的出生身份；I-3.9 红在 mkfs 那一片（已释放、又被链指针引用）；
    // 位置条目带的是那一片的真校验和：I-2.1 不先挡，判成立。
    images.hand_out(
        "第二片的链指针指到 mkfs 那一片",
        || memory_pool_of_sparse_devices(&devices),
        PoolCheckerExpectation::exactly_these_violated(&[
            "I-1.1", "I-1.2", "I-3.9", "I-4.8", "I-7.2", "I-7.4", "I-3.5",
        ])
        .and_violated_with_a_detail_containing("I-1.2", "实例表第 1 片")
        .and_violated_with_a_detail_containing("I-3.9", &format!("槽 {}", INSTANCE_TABLE_SLOT.0))
        .and_holds("I-2.1"),
    );
    assert_eq!(
        instance_table_of_root(&devices, &newest_root(&devices)),
        None,
        "读者同样认出链上第 1 片不是它：整张表不可读"
    );
}

/// 第二片的链指针校验和不对（坏镜像）：I-2.1 红，两份都读不出，链从这里断了（I-7.2、I-4.8、I-7.4）。
pub fn chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant(
    images: &mut PoolCheckerImages,
) {
    let (mut devices, _stream) = pool_without_file_after_writable_mounts();
    let rows = rows_of_the_newest_table(&devices);
    write_two_page_chain(
        &mut devices,
        &rows[..2],
        |second_page_pointer| {
            let mut wrong = *second_page_pointer;
            for location in &mut wrong.locations {
                location.unit_checksum ^= 1;
            }
            chain_record_to(&wrong)
        },
        &rows[2..],
    );
    images.hand_out(
        "第二片的链指针校验和不对",
        || memory_pool_of_sparse_devices(&devices),
        PoolCheckerExpectation::exactly_these_violated(&["I-2.1", "I-4.8", "I-7.2", "I-7.4"]),
    );
}

/// 两片上各有一行实例 2（坏镜像）：行按实例代号唯一是对整张表说的，I-3.8 红，别的不红。
pub fn the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant(
    images: &mut PoolCheckerImages,
) {
    let (mut devices, _stream) = pool_without_file_after_writable_mounts();
    let rows = rows_of_the_newest_table(&devices);
    write_two_page_chain(
        &mut devices,
        &rows[..2],
        chain_record_to,
        &[rows[1], rows[2]],
    );
    images.hand_out(
        "两片上各有一行实例 2",
        || memory_pool_of_sparse_devices(&devices),
        PoolCheckerExpectation::exactly_these_violated(&["I-3.8"]),
    );
}

/// 链指针记录自相矛盾（坏镜像）：「有无下一片」写 0 而位置指针没清零（条款：无下一片时清零占位），
/// 与「有无下一片」写 2（字段表里只有有、无两个值）。两样都是 I-3.14（实例表每片记录形态合法） 那条链指针记录的判定红；
/// 链在第 0 片就停了，第二片没人引用，别的不红。
pub fn contradictory_chain_record_reddens_only_the_instance_table_invariant(
    images: &mut PoolCheckerImages,
) {
    for (has_next_page, what) in [(0u8, "无下一片而指针不全零"), (2u8, "有无下一片写 2")]
    {
        let (mut devices, _stream) = pool_without_file_after_writable_mounts();
        let rows = rows_of_the_newest_table(&devices);
        write_two_page_chain(
            &mut devices,
            &rows[..2],
            |second_page_pointer| chain_record(has_next_page, second_page_pointer),
            &rows[2..],
        );
        images.hand_out(
            what,
            || memory_pool_of_sparse_devices(&devices),
            PoolCheckerExpectation::exactly_these_violated(&[
                INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
            ]),
        );
        assert_eq!(
            instance_table_of_root(&devices, &newest_root(&devices)),
            None,
            "{what}：读者同样拒收这一片"
        );
    }
}
