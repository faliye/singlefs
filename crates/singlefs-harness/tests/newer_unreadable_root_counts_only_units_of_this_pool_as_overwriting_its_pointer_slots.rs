//! C834 那一判只认本池的单元头：可写挂载的预隔离（C）走到「较新可读根 R 那次发布的记录一条都读不出、账读不出、中央映射也现算不成」那一判时，
//! R 的树表或中央映射树根两个槽里读出的自证过、实例代号比 R 大的单元头，头里的 fsid 与本池系统配置的 fsid 相同才算「被更新单元盖掉」
//! （用户定「比 fsid」，`records/2026-10-13-代码欠账批.md` 里「用户弹窗定 C834 那一判加比 fsid」那一条，零轮；判法本身见
//! `research/prompts/c834-r3-main-verification.md`「四、用户定案」与 `mount.rs` 的 `one_pointer_has_both_copies_overwritten_by_a_later_instance`）。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次（最后那一版叫 P），再录一次覆盖写 R，崩在 R 的根槽落盘之后、
//! 系统配置轮换之前。看不见 R 的那次可写挂载 M1（入库 `urJ` 那一形：R 的根槽、单元、记录读回全零）只落了取号那两次系统配置槽写；
//! 再把 M1 写行那次发布写在 R 的树表与映射根四个槽上的单元（自证过、实例代号是 M1 取的号）摆进这四个槽。被判的那次可写挂载 M2 时
//! R 的记录读回全零。两条用例只差摆进去的单元头里那 8 字节 fsid：
//! - 换成别的池的 fsid（重封头校验和）：四个槽都不算本池更新实例的单元，照「记录不齐」那一支办，两种交盘次序都在任何写之前拒、两块盘逐字节不变；
//! - fsid 不动（阳性对照）：两个指针都是两份本池更新实例的单元，放过、做成、落到 P。
//!
//! 搭建里 M1 的写、R 的记录、两种交盘次序那几样与 `newer_unreadable_root_passes_only_when_both_copies_of_one_pointer_are_newer.rs` 同一个做法
//! （那几个函数住在那个测试文件里，不在共用模块）。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    record_one_publish_after_overwrites, recorded_devices_reading_back_all_zero_on,
    units_of_r_overwritten_on, PublishRecordedAsR, RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{
    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::unit::{
    parse_data_unit, parse_index_node, parse_packed_unit, seal_header_checksum,
    unit_filesystem_identifier, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use singlefs_format::{
    DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, NODE_BYTES, PACKED_UNIT_HEADER_BYTES,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 覆盖写几次之后录 R：与入库 Y2、`urJ` 用例同（k = 5）。
const OVERWRITES_BEFORE_THE_RECORDED_PUBLISH: u64 = 5;
/// 单元头第 6 个字节是单元类（共同前缀：magic 4、格式版本 2 之后）。
const UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER: usize = 6;
/// 认单元类时先读的字节数：一个物理块。
const BYTES_READ_TO_TELL_THE_UNIT_CLASS: usize = 512;
/// 码 2 索引节点（`singlefs_core::unit` 里码 2 那张偏移表）：key 宽住偏移 51，key 区间是两个 key；fsid 住 60 + 2k，头校验和罩 [0, 86 + 2k)。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
const INDEX_NODE_KEYS_IN_THE_KEY_RANGE: usize = 2;
const INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_WITHOUT_THE_KEY_RANGE: usize = 60;
const INDEX_NODE_PLAIN_HEADER_BYTES_WITHOUT_THE_KEY_RANGE: usize = 86;
/// 码 1 数据单元的 fsid 住偏移 83（`build_data_unit` 的偏移表）；码 3 打包记录单元的住 81（诞生代号 73 之后、载荷校验和 89 之前）。
const DATA_UNIT_FILESYSTEM_IDENTIFIER_OFFSET: usize = 83;
const PACKED_UNIT_FILESYSTEM_IDENTIFIER_OFFSET: usize = 81;
/// 单元头里的 fsid 是 8 字节：系统配置 fsid 的低 8 字节（`unit_filesystem_identifier`）。
const FILESYSTEM_IDENTIFIER_IN_THE_UNIT_HEADER_BYTES: usize = 8;

/// M2 交盘的次序。
#[derive(Clone, Copy, Debug)]
enum DeviceOrder {
    AsRecorded,
    Reversed,
}

/// 摆进 R 的四个指针槽的单元头里记哪个池的 fsid。
#[derive(Clone, Copy, Debug)]
enum FilesystemIdentifierInThePlantedHeaders {
    /// 本池的（M1 写下的原样）。
    OfThisPool,
    /// 别的池的：本池那 8 字节逐位取反，与本池的必不相同。
    OfAnotherPool,
}

/// R 的每一条记录（两块盘各一份）。
fn every_record_of_the_recorded_publish(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    (1..=recorded.record_count())
        .flat_map(|ordinal| recorded.record_writes_of_the_record(ordinal))
        .collect()
}

/// 入库 `urJ` 那一形的 M1：R 的根槽、R 写出的全部单元、R 的记录读回全零，M1 看不见 R、退回 P、取号、写行。交回 M1 按落盘次序的全部写。
fn writes_of_the_mount_that_could_not_see_the_recorded_publish(
    recorded: &RecordedPublish,
) -> Vec<RetainedWrite> {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&every_record_of_the_recorded_publish(recorded));
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(&crash_image, &ranges, &stream);
    mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("看不见 R 的那次挂载做得成（入库 urJ 用例的前提）");
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    writes
}

/// `image` 上从 `location` 那一槽起的一个单元的整单元字节：按头里的单元类取宽（码 2 一槽，码 1、码 3 两槽）。
fn unit_starting_at(image: &MemoryPool, location: LocationEntry) -> Vec<u8> {
    let device = &image.devices[&location.device];
    let offset = location.slot.to_device_offset();
    let first_block = device.read(offset, BYTES_READ_TO_TELL_THE_UNIT_CLASS);
    let unit_bytes = match first_block[UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER] {
        UNIT_CLASS_INDEX_NODE => NODE_BYTES,
        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => DATA_UNIT_BYTES,
        unknown_class => panic!("{location:?} 起不是一个单元的开头：单元类 {unknown_class}"),
    };
    device.read(
        offset,
        usize::try_from(unit_bytes).expect("单元宽装得进 usize"),
    )
}

/// 一个单元头里 fsid 住的偏移，与头校验和罩到的末尾（码 2 按偏移 51 的 key 宽算）。
fn filesystem_identifier_offset_and_header_end(unit: &[u8]) -> (usize, usize) {
    match unit[UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER] {
        UNIT_CLASS_INDEX_NODE => {
            let key_range_bytes =
                INDEX_NODE_KEYS_IN_THE_KEY_RANGE * usize::from(unit[INDEX_NODE_KEY_WIDTH_OFFSET]);
            (
                INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_WITHOUT_THE_KEY_RANGE + key_range_bytes,
                INDEX_NODE_PLAIN_HEADER_BYTES_WITHOUT_THE_KEY_RANGE + key_range_bytes,
            )
        }
        UNIT_CLASS_DATA => (
            DATA_UNIT_FILESYSTEM_IDENTIFIER_OFFSET,
            usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105"),
        ),
        UNIT_CLASS_PACKED => (
            PACKED_UNIT_FILESYSTEM_IDENTIFIER_OFFSET,
            usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107"),
        ),
        unknown_class => panic!("只认三类单元，读到类 {unknown_class}"),
    }
}

/// `unit` 头里的 fsid（本池的 `filesystem_identifier_of_this_pool`）换成 `planted`、重封头校验和。fsid 在头校验和罩的那一段里、
/// 不在载荷校验和罩的那一段里，载荷校验和不动。
fn unit_with_the_filesystem_identifier_in_its_header_set_to(
    unit: &[u8],
    filesystem_identifier_of_this_pool: u64,
    planted: u64,
) -> Vec<u8> {
    let (filesystem_identifier_offset, header_end) =
        filesystem_identifier_offset_and_header_end(unit);
    let filesystem_identifier_end =
        filesystem_identifier_offset + FILESYSTEM_IDENTIFIER_IN_THE_UNIT_HEADER_BYTES;
    let field = filesystem_identifier_offset..filesystem_identifier_end;
    assert_eq!(
        unit[field.clone()],
        filesystem_identifier_of_this_pool.to_le_bytes(),
        "偏移表对得上：M1 写下的单元头里那 8 字节是本池的 fsid"
    );
    let mut planted_unit = unit.to_vec();
    planted_unit[field].copy_from_slice(&planted.to_le_bytes());
    seal_header_checksum(&mut planted_unit, header_end);
    planted_unit
}

/// `image` 上 `location` 那一槽起的单元自证过（头校验和与载荷校验和都过）时，头里的 fsid 与写序里的实例代号；自证不过是 None。
fn filesystem_identifier_and_instance_of_the_self_verified_unit_at(
    image: &MemoryPool,
    location: LocationEntry,
) -> Option<(u64, InstanceGeneration)> {
    let unit = unit_starting_at(image, location);
    match unit[UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER] {
        UNIT_CLASS_INDEX_NODE => parse_index_node(&unit)
            .ok()
            .map(|header| (header.filesystem_identifier, header.instance)),
        UNIT_CLASS_DATA => parse_data_unit(&unit)
            .ok()
            .map(|header| (header.filesystem_identifier, header.write_order.instance)),
        UNIT_CLASS_PACKED => parse_packed_unit(&unit)
            .ok()
            .map(|header| (header.filesystem_identifier, header.write_order.instance)),
        unknown_class => panic!("{location:?} 起不是一个单元的开头：单元类 {unknown_class}"),
    }
}

/// 摆好的盘面：崩在 R 的根槽落盘之后的盘面上落了 M1 取号那两次系统配置槽写，R 的树表与映射根四个槽里摆上 M1 写行那次发布写在那里的单元，
/// 头里的 fsid 照 `planted`。M1 别的写（记录、别的单元、根）一样不落：这四个单元不被盘上任何别的东西引用，两条用例的盘面只差这四个单元头里的 fsid
/// 与跟着变的头校验和。
fn image_with_later_units_planted_in_the_pointer_slots_of_the_recorded_root(
    recorded: &RecordedPublish,
    planted: FilesystemIdentifierInThePlantedHeaders,
) -> MemoryPool {
    let writes_of_the_mount_that_could_not_see_it =
        writes_of_the_mount_that_could_not_see_the_recorded_publish(recorded);
    let row_root = writes_of_the_mount_that_could_not_see_it
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("M1 写行那次发布有根槽写");
    let acquisition_writes: Vec<usize> = writes_of_the_mount_that_could_not_see_it
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::SystemConfigurationSlot)
        .map(|(index, _)| index)
        .take(2)
        .collect();
    assert_eq!(
        acquisition_writes.len(),
        2,
        "前提：M1 取号每块盘写一个系统配置槽"
    );
    assert!(
        acquisition_writes[1] < row_root,
        "前提：M1 取号在写行那次发布的根槽写之前"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut image_of_the_urj_form = crash_image.clone();
    image_of_the_urj_form.apply_writes(&writes_of_the_mount_that_could_not_see_it[..row_root]);
    let mut image = crash_image;
    image.apply_writes(&writes_of_the_mount_that_could_not_see_it[..=acquisition_writes[1]]);
    let filesystem_identifier_of_this_pool =
        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier);
    let filesystem_identifier_in_the_planted_headers = match planted {
        FilesystemIdentifierInThePlantedHeaders::OfThisPool => filesystem_identifier_of_this_pool,
        FilesystemIdentifierInThePlantedHeaders::OfAnotherPool => {
            !filesystem_identifier_of_this_pool
        }
    };
    let root_of_the_recorded_publish = recorded.root();
    let pointer_slots_of_the_recorded_publish: Vec<LocationEntry> = [
        root_of_the_recorded_publish.tree_table,
        root_of_the_recorded_publish.mapping_root,
    ]
    .into_iter()
    .flat_map(|pointer| pointer.locations)
    .collect();
    for location in &pointer_slots_of_the_recorded_publish {
        let planted_unit = unit_with_the_filesystem_identifier_in_its_header_set_to(
            &unit_starting_at(&image_of_the_urj_form, *location),
            filesystem_identifier_of_this_pool,
            filesystem_identifier_in_the_planted_headers,
        );
        image
            .devices
            .get_mut(&location.device)
            .expect("R 的指针槽所在的盘在池里")
            .write(location.slot.to_device_offset(), &planted_unit);
    }
    let overwritten = units_of_r_overwritten_on(recorded, &image);
    for location in &pointer_slots_of_the_recorded_publish {
        let (filesystem_identifier_in_the_header, instance) =
            filesystem_identifier_and_instance_of_the_self_verified_unit_at(&image, *location)
                .unwrap_or_else(|| panic!("前提：{location:?} 摆进去的单元自证得过"));
        assert!(
            instance > recorded.key.0,
            "前提：{location:?} 摆进去的单元实例代号 {instance:?} 比 R 的 {:?} 大",
            recorded.key.0
        );
        assert_eq!(
            filesystem_identifier_in_the_header, filesystem_identifier_in_the_planted_headers,
            "前提：{location:?} 摆进去的单元头里的 fsid 照 {planted:?}"
        );
        assert!(
            overwritten.contains(&(location.device, location.slot.to_device_offset().0)),
            "前提：R 在 {location:?} 的那一份已被盖掉"
        );
    }
    image
}

/// 被判的那次可写挂载 M2：R 的记录读回全零，盘按 `order` 交。交回结局，与挂载之后两块盘是不是都与 `image` 逐字节相同。
fn judged_mount(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    order: DeviceOrder,
) -> (
    Result<(InstanceGeneration, CheckpointTxg), MountError>,
    bool,
) {
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&every_record_of_the_recorded_publish(recorded));
    let mut devices = devices_reading_back_all_zero_on(image, &ranges);
    match order {
        DeviceOrder::AsRecorded => {}
        DeviceOrder::Reversed => devices.reverse(),
    }
    let outcome = mount_writable(&recorded.pool.parameters, &mut devices).map(|mounted| {
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg,
        )
    });
    let every_device_is_unchanged = devices
        .iter()
        .all(|(identity, device)| device.inner.image == image.devices[identity]);
    (outcome, every_device_is_unchanged)
}

/// R 的树表与映射根四个槽里都是自证过、实例代号比 R 大、头里的 fsid 是别的池的单元头。两个指针都不算被本池更新实例的单元盖掉，
/// 照「记录不齐」那一支办（重读一次仍读不出、没有 C694 的 B 凭据）：两种交盘次序都在任何写之前拒，拒因是 C 的映射那一支的成员、点名 R，
/// 两块盘逐字节不变。判别力：不比 fsid（变异「C834 比 fsid：较新可读根指针槽里别的池的单元头也算被更新单元盖掉」）时放过、做成，这一条红。
#[test]
fn newer_root_whose_pointer_slots_hold_self_verified_units_of_another_pool_refuses_the_writable_mount_before_any_write_in_both_device_orders(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let image = image_with_later_units_planted_in_the_pointer_slots_of_the_recorded_root(
        &recorded,
        FilesystemIdentifierInThePlantedHeaders::OfAnotherPool,
    );
    for order in [DeviceOrder::AsRecorded, DeviceOrder::Reversed] {
        let (outcome, every_device_is_unchanged) = judged_mount(&recorded, &image, order);
        let refusal = outcome.expect_err(&format!(
            "盘序 {order:?}：R 的指针槽里只有别的池的单元头，不算被本池更新实例的单元盖掉，可写挂载在取号之前拒"
        ));
        assert!(
            matches!(
                &refusal,
                MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                    if matches!(
                        **still_unreadable,
                        StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                            unapplied_publish: RollbackTarget { instance, checkpoint_txg }
                        } if (instance, checkpoint_txg) == recorded.key
                    )
            ),
            "盘序 {order:?}：拒因是 C 的映射那一支的成员、点名 R：{refusal:?}"
        );
        assert!(
            every_device_is_unchanged,
            "盘序 {order:?}：拒在任何写之前，两块盘逐字节不变（{refusal:?}）"
        );
    }
}

/// 阳性对照：同一个摆法，头里的 fsid 不动（本池的）。两个指针都是两份本池更新实例的单元，放过：两种交盘次序都做成、落到 P。
/// 它钉住摆法本身（取 M1 的单元、过一遍改 fsid 与重封）造得出那一判认的凭据：别的池那一条拒，只因为单元头里的 fsid。
#[test]
fn newer_root_whose_pointer_slots_hold_the_same_planted_units_of_this_pool_is_passed_in_both_device_orders(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let image = image_with_later_units_planted_in_the_pointer_slots_of_the_recorded_root(
        &recorded,
        FilesystemIdentifierInThePlantedHeaders::OfThisPool,
    );
    for order in [DeviceOrder::AsRecorded, DeviceOrder::Reversed] {
        let (outcome, _every_device_is_unchanged) = judged_mount(&recorded, &image, order);
        let effective = outcome.unwrap_or_else(|refusal| {
            panic!("盘序 {order:?}：R 的两个指针都是两份本池更新实例的单元，放过、可写挂载做成：{refusal:?}")
        });
        assert_eq!(
            effective, recorded.previous_key,
            "盘序 {order:?}：R 走读失败、系统配置没见证它，落到 P"
        );
    }
}
