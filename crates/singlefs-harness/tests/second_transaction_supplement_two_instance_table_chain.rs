//! 里程碑「第二个事务」增补 2 收口表第 38 行（实例表第二片）的读路径那一半：一张多于一片的实例表，读者沿链读、
//! 准入按片数算、checker 沿链判（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录
//! `kind 1 | 有无下一片 1 | 位置指针 86` 指着第 k + 1 片，身份四元组 (0, 4, 片序号, 0)）。
//!
//! **写路径今天只写一片**：第二片在提交内生块的 bump 次序里排第几（D3（空间分配） 已定项 10 ⑤ 只写「实例表单元最前」，
//! 排序规则管的是「其余」，它同时定第二片的出生序号，D19（块指针的结构与宽度预算） 已定项 9）、行怎么分到各片，两处没有条款，
//! 可写挂载与回退在取号之前就拒（`MountError::InstanceTableChainLongerThanOnePageUndecided`）。所以这里的两片链是用例手搭的：
//! mkfs 之后连着可写挂载几次、不写文件（树表 0 条的那一版上每次写行），把最新那一版的实例表拆成两片——第 0 片原地重写、
//! 第二片写进一个远离开放段的空槽，指着第 0 片的每一条根补上它的新整单元校验和。第二片的出生序号随手取第 0 片的加一：
//! 各片怎么发出生序号正是没定的那一格，读者与 checker 都不看它（实例表单元按 I-1.2（块头写序已发布） 那一行的例外不进出生身份那两格）。
//! 树表 0 条的那一版没有记账树，I-3.1（已分配统计对得上） 不适用，手搭的第二片不进分配记录也不会让它红。

mod common;

use common::{disk_snapshot, memory_pool_of_sparse_devices, parameters, DiskSnapshot, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::ByteWriter;
use singlefs_core::instance_table::{InstanceRow, InstanceTablePageIndex, InstanceTableRecords};
use singlefs_core::make_filesystem::{
    instance_table_chain_record, location_entries, make_filesystem, INSTANCE_TABLE_SLOT,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, MountError, Mounted, RollbackCandidateExclusion,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::pointer::{BirthSequence, NodePointer, PointerHead};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, instance_table_chain_of_root, instance_table_of_root,
    readable_roots, recover, verified_system_configuration_slots, JournalPolicy, PoolReader,
    RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::unit::{build_packed_unit, parse_packed_unit};
use singlefs_format::{DATA_UNIT_BYTES, INSTANCE_ROW_BYTES};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
/// 手搭的第二片落在这个槽上：单元区里、32768 对齐（偶数槽），离开放段（50240 起）远，这几次挂载写不到它。
const SECOND_PAGE_SLOT: SlotNumber = SlotNumber(60_000);
/// mkfs 之后连着可写挂载几次：最新那一版是实例 4，它那张表里有实例 1、2、3 各一行。
const WRITABLE_MOUNTS: u32 = 4;

type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 两块 4 GiB 内存盘上 mkfs，之后连着可写挂载 `WRITABLE_MOUNTS` 次、不写文件。
fn pool_without_file_after_writable_mounts() -> (Devices, SharedStream) {
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

fn newest_root(devices: &Devices) -> RootRecord {
    let system_configuration = choose_system_configuration(devices).expect("系统配置");
    choose_root(devices, &system_configuration).expect("最新根")
}

fn read_unit(devices: &Devices, slot: SlotNumber) -> Vec<u8> {
    PoolReader::read(
        devices,
        DISKS[0],
        slot.to_device_offset(),
        usize::try_from(DATA_UNIT_BYTES).expect("32768"),
    )
    .expect("读得到")
}

/// 两块盘同槽写一个单元（不经录制器：手搭镜像不算挂载发的写）。
fn write_unit(devices: &mut Devices, slot: SlotNumber, unit: &[u8]) {
    for (_, device) in devices.iter_mut() {
        device
            .inner_mut()
            .image
            .write(slot.to_device_offset(), unit);
    }
}

/// 一条链指针记录：`kind 1 | 有无下一片 1 | 位置指针 86`（D18（块里携带什么信息） 已定项 11）。
fn chain_record(has_next_page: u8, pointer: &NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
    writer.put_u8(1);
    writer.put_u8(has_next_page);
    pointer.write_to(&mut writer);
    writer.assert_position(INSTANCE_ROW_BYTES, "链指针记录");
    writer.into_bytes()
}

/// 「有无下一片」= 1、位置指针指着下一片的链指针记录（读者与 checker 按字段名读的「有」）。
fn chain_record_to(next_page: &NodePointer) -> Vec<u8> {
    chain_record(1, next_page)
}

/// 指着某个槽上那个单元的指针：位置条目两盘同槽、带那个单元的整单元校验和，头与尾段照 `like` 的（实例表单元的指针同型，
/// D19（块指针的结构与宽度预算） 已定项 7 / 8）。
fn pointer_to_unit_at(like: &NodePointer, slot: SlotNumber, unit: &[u8]) -> NodePointer {
    NodePointer {
        head: like.head,
        locations: location_entries(&DISKS, slot, unit),
        instance: like.instance,
        birth_sequence: like.birth_sequence,
    }
}

/// 手搭好的两片链。
struct TwoPageChain {
    first_page_slot: SlotNumber,
    /// 第 0 片的链指针记录原本该指着的第二片（写在 `SECOND_PAGE_SLOT`）。
    second_page_pointer: NodePointer,
}

/// 把最新那一版的实例表换成两片：第 0 片装 `first_page_rows` 加 `first_page_chain_record(第二片的指针)`，原地重写；
/// 第二片装 `second_page_rows` 加「无下一片」，身份 (0, 4, 1, 0)，写在 `SECOND_PAGE_SLOT`。两片的诞生代号与写序照原来那一片；
/// 第二片的出生序号取第 0 片的加一（没有条款的那一格，读者与 checker 不看它）。指着原来那一片的每一条根（写行那次发布与之后的暖机空发布）
/// 换上第 0 片的新整单元校验和、重写根槽。
fn write_two_page_chain(
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
            instance: root.instance,
            checkpoint_txg: root.checkpoint_txg,
            tree_table: root.tree_table,
            tree_identifier_watermark: root.tree_identifier_watermark,
            rollback_floor: root.rollback_floor,
            instance_table: first_page_pointer,
            mapping_root: root.mapping_root,
            allocation_record_tree_root: root.allocation_record_tree_root,
        };
        let target = target_for_publish(root.checkpoint_txg, slots_per_region);
        let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
        let offset: DeviceOffsetInBytes = slot_offset(target, spacing);
        devices
            .iter_mut()
            .find(|(identity, _)| *identity == device)
            .expect("区域的盘在池里")
            .1
            .inner_mut()
            .image
            .write(offset, &rewritten.to_slot(root_slot_bytes));
        rewritten_roots += 1;
    }
    assert!(rewritten_roots >= 1, "至少最新那条根指着原来那一片");
    TwoPageChain {
        first_page_slot,
        second_page_pointer,
    }
}

/// 最新那一版原来那张表（一片）里的行：实例 1、2、3 各一行。
fn rows_of_the_newest_table(devices: &Devices) -> Vec<InstanceRow> {
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
fn pool_with_a_valid_two_page_chain() -> (Devices, SharedStream, Vec<InstanceRow>, TwoPageChain) {
    let (mut devices, stream) = pool_without_file_after_writable_mounts();
    let rows = rows_of_the_newest_table(&devices);
    let chain = write_two_page_chain(&mut devices, &rows[..2], chain_record_to, &rows[2..]);
    (devices, stream, rows, chain)
}

fn verdicts(devices: &Devices) -> Vec<(&'static str, InvariantVerdict)> {
    check_pool_image(&memory_pool_of_sparse_devices(devices))
}

fn violated(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<&'static str> {
    verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect()
}

fn verdict_of(verdicts: &[(&'static str, InvariantVerdict)], invariant: &str) -> InvariantVerdict {
    verdicts
        .iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
        .clone()
}

/// 读者沿链读：整张表是两片的行按链上的次序接起来，指针两条（第 0 片是根记录里那一条、第 1 片是第 0 片链指针记录里那一条）；
/// 只拿第 0 片的字节解整张表要交 `None`（交回前半张会被当成整张表判）。
#[test]
fn the_reader_follows_the_chain_into_the_second_page_and_a_single_page_parse_refuses_a_chained_first_page(
) {
    let (devices, _stream, rows, chain) = pool_with_a_valid_two_page_chain();
    let newest = newest_root(&devices);
    let read = instance_table_chain_of_root(&devices, &newest).expect("两片都读得出");
    assert_eq!(read.records.rows, rows, "三行都在，按链上的次序");
    assert_eq!(
        read.page_pointers,
        vec![newest.instance_table, chain.second_page_pointer],
        "第 0 片是根记录里那一条，第 1 片是第 0 片链指针记录里那一条"
    );
    assert_eq!(
        instance_table_of_root(&devices, &newest)
            .expect("整张表读得出")
            .rows,
        rows
    );
    assert_eq!(
        InstanceTableRecords::parse(&read_unit(&devices, chain.first_page_slot)),
        None,
        "第 0 片的链指针说还有下一片：只拿这一片的字节解不出整张表"
    );
}

/// 冷启动（只读那一路，`recovery::recover`）沿链读：两片都读得出时照常走完（树表 0 条 ⇒ 没有文件）；
/// 第二片两份都坏了，整张表就不可读，走读停在第二片（与第 0 片读不出同一个结局）。
#[test]
fn cold_start_walks_every_page_of_the_instance_table_and_an_unreadable_second_page_stops_it() {
    let (mut devices, _stream, _rows, _chain) = pool_with_a_valid_two_page_chain();
    let newest = newest_root(&devices);
    let report_with_both_pages_readable = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report_with_both_pages_readable.outcome,
        RecoveryOutcome::NoFile {
            root: (newest.instance, newest.checkpoint_txg)
        },
        "两片都读得出：照常走完"
    );

    let mut second_page = read_unit(&devices, SECOND_PAGE_SLOT);
    second_page[20_000] ^= 0xff;
    write_unit(&mut devices, SECOND_PAGE_SLOT, &second_page);
    let report_with_the_second_page_unreadable = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report_with_the_second_page_unreadable.outcome,
        RecoveryOutcome::Failed {
            root: Some((newest.instance, newest.checkpoint_txg)),
            failure: RecoveryFailure::UnitUnreadable {
                slot: SECOND_PAGE_SLOT
            },
        },
        "第二片两份的校验和都对不上：表不可读"
    );
}

/// 两块盘四个系统配置槽里自证过的那些槽写着的实例代号，按盘排。
fn system_configuration_instances(
    devices: &Devices,
) -> Vec<(DeviceIdentity, Vec<InstanceGeneration>)> {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    DISKS
        .iter()
        .map(|device| {
            let mut instances: Vec<InstanceGeneration> = verified_system_configuration_slots(
                devices,
                *device,
                spacing,
                &parameters().filesystem_identifier,
            )
            .iter()
            .map(|system_configuration| system_configuration.quantities.journal_instance)
            .collect();
            instances.sort();
            (*device, instances)
        })
        .collect()
}

fn snapshot(devices: &Devices, stream: &SharedStream) -> DiskSnapshot {
    disk_snapshot(&memory_pool_of_sparse_devices(devices), stream)
}

/// 可写挂载的准入按片数算：这一版的表是两片（行数按两片接起来数，是 3 不是 2），第一版不重写多于一片的链
/// （第二片怎么写没有条款）——在取号之前拒，盘上逐字节不变、系统配置里的实例代号不变。
#[test]
fn writable_mount_counts_the_rows_of_every_page_and_refuses_a_chain_longer_than_one_page_before_acquisition(
) {
    let (mut devices, stream, _rows, _chain) = pool_with_a_valid_two_page_chain();
    let before = snapshot(&devices, &stream);
    let instances_before = system_configuration_instances(&devices);
    let refused: Result<Mounted, MountError> = mount_writable(&parameters(), &mut devices);
    match refused {
        Err(MountError::InstanceTableChainLongerThanOnePageUndecided {
            instance_to_acquire,
            rows_in_version,
            pages_in_version,
            rows_to_write,
            pages_after_this_publish,
        }) => {
            assert_eq!(
                (
                    instance_to_acquire,
                    rows_in_version,
                    pages_in_version,
                    rows_to_write,
                    pages_after_this_publish
                ),
                (InstanceGeneration(5), 3, 2, 1, 1),
                "要取的号、这一版两片共几行、几片、这次要写几行、这次之后要几片"
            );
        }
        other => panic!(
            "这一版的表是两片：要在取号之前拒绝：{:?}",
            other.map(|_| "挂上了")
        ),
    }
    assert_eq!(
        snapshot(&devices, &stream),
        before,
        "盘上逐字节不变：系统配置槽、根环里的根、录制流步数（一个写、一道屏障都没发）"
    );
    assert_eq!(
        system_configuration_instances(&devices),
        instances_before,
        "系统配置里的实例代号不变：号没烧"
    );
}

/// 回退候选集读的是整张表：实例 3 那一行只在第二片上、它的 T 比实例 3 最后那条根的 txg 小一，
/// 那条根就在被抛弃的时间线上，回退到它在任何写之前被拒（只读第 0 片就看不到这一行，回退会照做）。
#[test]
fn rollback_candidate_set_reads_the_row_that_lives_on_the_second_page() {
    let (mut devices, stream) = pool_without_file_after_writable_mounts();
    let rows = rows_of_the_newest_table(&devices);
    let system_configuration = choose_system_configuration(&devices).expect("系统配置");
    let last_root_of_instance_three = readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| root.instance == InstanceGeneration(3))
    .max_by_key(|root| root.checkpoint_txg)
    .expect("实例 3 发布过根");
    assert_eq!(
        rows[2].selected_root_txg, last_root_of_instance_three.checkpoint_txg,
        "写行时实例 3 那一行的 T 就是它最后那条根的 txg"
    );
    let row_that_abandons_the_last_root = InstanceRow {
        instance: InstanceGeneration(3),
        selected_root_txg: CheckpointTxg(last_root_of_instance_three.checkpoint_txg.0 - 1),
        applied_transaction_high_water: 0,
        is_rollback: false,
    };
    write_two_page_chain(
        &mut devices,
        &rows[..2],
        chain_record_to,
        &[row_that_abandons_the_last_root],
    );
    let before = snapshot(&devices, &stream);
    let target = RollbackTarget {
        instance: last_root_of_instance_three.instance,
        checkpoint_txg: last_root_of_instance_three.checkpoint_txg,
    };
    match mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On) {
        Err(MountError::RollbackTargetNotACandidate {
            target: refused_target,
            exclusion,
        }) => {
            assert_eq!(refused_target, target);
            assert_eq!(
                exclusion,
                RollbackCandidateExclusion::OnAbandonedTimeline,
                "第二片上那一行把实例 3 最后那条根判进被抛弃的时间线"
            );
        }
        other => panic!(
            "第二片上那一行把目标判出候选集：要拒绝：{:?}",
            other.map(|_| "回退做了")
        ),
    }
    assert_eq!(snapshot(&devices, &stream), before, "在任何写之前拒绝");
}

/// 合法的两片链上池级 checker 一条都不红：第二片的身份、校验和、位置条目次序、链指针记录都判过（I-1.1、I-2.1、I-2.5、I-3.8
/// 判成成立，不是「不适用」），两片的行接起来判 I-3.8。
#[test]
fn the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds() {
    let (devices, _stream, _rows, _chain) = pool_with_a_valid_two_page_chain();
    let verdicts = verdicts(&devices);
    assert_eq!(violated(&verdicts), Vec::<&str>::new(), "{verdicts:?}");
    for invariant in ["I-1.1", "I-2.1", "I-2.5", "I-3.8", "I-1.7"] {
        assert_eq!(
            verdict_of(&verdicts, invariant),
            InvariantVerdict::Holds,
            "{invariant} 在两片链上判成立"
        );
    }
}

/// 第二片的链指针指错（坏镜像）：第 0 片的链指针指到 mkfs 写的那一片实例表（槽 50176，身份 (0, 4, 0, 0)），
/// 位置条目带的就是那一片的整单元校验和——I-2.1 挡不住，链上第 1 片的身份该是 (0, 4, 1, 0)，I-1.1 红；
/// 链从这里断了，最新根走不完（I-7.2），它引用的单元对不上（I-4.8、I-7.4 按最新根那一格红）。
#[test]
fn chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant() {
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
    let verdicts = verdicts(&devices);
    assert_eq!(
        violated(&verdicts),
        vec!["I-1.1", "I-4.8", "I-7.2", "I-7.4"],
        "{verdicts:?}"
    );
    assert_eq!(
        verdict_of(&verdicts, "I-2.1"),
        InvariantVerdict::Holds,
        "位置条目带的是那一片的真校验和：I-2.1 不先挡"
    );
    assert_eq!(
        instance_table_of_root(&devices, &newest_root(&devices)),
        None,
        "读者同样认出链上第 1 片不是它：整张表不可读"
    );
}

/// 第二片的链指针校验和不对（坏镜像）：I-2.1 红，两份都读不出，链从这里断了（I-7.2、I-4.8、I-7.4）。
#[test]
fn chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant() {
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
    let verdicts = verdicts(&devices);
    assert_eq!(
        violated(&verdicts),
        vec!["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        "{verdicts:?}"
    );
}

/// 两片上各有一行实例 2（坏镜像）：行按实例代号唯一是对整张表说的，I-3.8 红，别的不红。
#[test]
fn the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant() {
    let (mut devices, _stream) = pool_without_file_after_writable_mounts();
    let rows = rows_of_the_newest_table(&devices);
    write_two_page_chain(
        &mut devices,
        &rows[..2],
        chain_record_to,
        &[rows[1], rows[2]],
    );
    let verdicts = verdicts(&devices);
    assert_eq!(violated(&verdicts), vec!["I-3.8"], "{verdicts:?}");
}

/// 链指针记录自相矛盾（坏镜像）：「有无下一片」写 0 而位置指针没清零（条款：无下一片时清零占位），
/// 与「有无下一片」写 2（字段表里只有有、无两个值）。两样都是 I-3.8 那条链指针记录的判定红；
/// 链在第 0 片就停了，第二片没人引用，别的不红。
#[test]
fn contradictory_chain_record_reddens_only_the_instance_table_invariant() {
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
        let verdicts = verdicts(&devices);
        assert_eq!(violated(&verdicts), vec!["I-3.8"], "{what}：{verdicts:?}");
        assert_eq!(
            instance_table_of_root(&devices, &newest_root(&devices)),
            None,
            "{what}：读者同样拒收这一片"
        );
    }
}

/// 影子账隔离被抛弃根引用的整条链（D23（journal 的角色与格式） 已定项 14「被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配」）：
/// 回退到实例 3 最后那条根，实例 4 的根全被抛弃；它们指着的两片实例表都只被被抛弃根引用，两片都隔离——每块盘 2 + 2 槽，
/// 再加实例 4 写行那一版自己那片分配记录树节点 1 槽（根指针住根记录那一项，同样只被被抛弃根引用）。
/// 只认根记录里那一条实例表指针的话第二片那 2 槽不隔离，回退之后当空闲槽发得出去。
#[test]
fn rollback_isolates_every_page_of_the_instance_table_chain_that_only_abandoned_roots_reference() {
    let (mut devices, _stream, _rows, _chain) = pool_with_a_valid_two_page_chain();
    let system_configuration = choose_system_configuration(&devices).expect("系统配置");
    let last_root_of_instance_three = readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| root.instance == InstanceGeneration(3))
    .max_by_key(|root| root.checkpoint_txg)
    .expect("实例 3 发布过根");
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: last_root_of_instance_three.instance,
            checkpoint_txg: last_root_of_instance_three.checkpoint_txg,
        },
        ShadowLedger::On,
    )
    .expect("实例 3 最后那条根在候选集里（第二片上实例 3 那一行的 T 就是它的 txg），回退那一版的表只有一片");
    assert_eq!(
        rolled_back.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 5), (DeviceIdentity(1), 5)],
        "实例 4 那张表的第 0 片 2 槽、第二片 2 槽、实例 4 那片分配记录树节点 1 槽，逐盘"
    );
    assert_eq!(rolled_back.output.abandoned_roots_unreadable, 0);
}
