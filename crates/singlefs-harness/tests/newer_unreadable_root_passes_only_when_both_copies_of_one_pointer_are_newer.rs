//! C834（择到的系统配置实例代号随盘序变）的新判法（用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`
//! 「四、用户定案」，被攻过零轮）：可写挂载的预隔离（C）走到「较新可读根 R 那次发布的记录一条都读不出、账读不出、中央映射也现算不成」那一判时，
//! R 的根记录里树表与中央映射树根两个指针中有一个，它两条位置条目指着的两个槽都读出自证过、写序里的实例代号比 R 大的单元头，就放过；
//! 否则照「记录不齐」那一支办（重读一次、看 C694 的 B 凭据，仍不成就在取号之前拒），不读系统配置里取过号的实例代号（那个号随盘序变）。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R（一条记录），崩在 R 的根槽落盘之后、
//! 系统配置轮换之前；之后取新实例的那次可写挂载 M1 写到一半断电，被判的那次可写挂载 M2 里几段读回全零。三轮里的历史各一条用例：
//! - 第三轮攻方（`research/prompts/c834-r3-opus-output.md` 第二节 T2）：M1 是入库 `urJ` 那一形（R 的根槽、单元、记录读回全零），崩在写行那次发布的
//!   单元写那一段里、只落了盘 0 上盖住 R 的树表（与映射根）那几份，R 每个单元还剩一份、走得完；M2 时 R 剩下的那几份与 R 的记录读回全零。期望拒。
//! - 入库 `urJ` 那一形：M1 写行那次发布的根落盘之前全部落了，R 的四个指针槽都已是 M1 的单元。期望放过、退到 P。
//! - 第一轮正推（`research/prompts/c834-r1-sonnet-model/tests/c834_order_of_devices_decides_the_newer_root_judgement.rs`）：取号只落了一块盘、
//!   取号两块盘落完一个字节没写；M2 时 R 的记录、映射根、树表读回全零。期望拒。
//! - 第一、二轮攻方的丙（`research/prompts/c834-r1-opus-output.md` 第二节）：M1 时 R 的树表与记录读回全零，M1 从映射现算、护住 R 的单元，
//!   把写行记录写进 R 的记录槽，写行根没落；M2 时 R 的树表与映射根读回全零。期望拒。
//!
//! 每格两种交盘次序各挂一次，结局相同：拒的拒在任何写之前、拒因是 C 的映射那一支的成员、点名 R；放过的做成、落到 P。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    fixed_geometry_of_the_384_slot_small_devices, record_one_publish_after_overwrites,
    recorded_devices_reading_back_all_zero_on, units_of_r_overwritten_on, PublishRecordedAsR,
    RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{
    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{choose_system_configuration, scan_journal};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 覆盖写几次之后录 R：第三轮攻方的 k = 5，与入库 Y2、`urJ` 用例同。
const OVERWRITES_BEFORE_THE_RECORDED_PUBLISH: u64 = 5;

/// M2 交盘的次序。
#[derive(Clone, Copy, Debug)]
enum DeviceOrder {
    AsRecorded,
    Reversed,
}

/// R 的每一条记录（两块盘各一份）。
fn every_record_of_the_recorded_publish(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    (1..=recorded.record_count())
        .flat_map(|ordinal| recorded.record_writes_of_the_record(ordinal))
        .collect()
}

/// R 自己写的那个单元（`pointer` 指着的）在第 `location_index` 条位置条目那块盘上的那一份。
fn copy_of_the_recorded_publish_at<'recorded>(
    recorded: &'recorded RecordedPublish,
    pointer: &NodePointer,
    location_index: usize,
) -> &'recorded RetainedWrite {
    let device = pointer.locations[location_index].device;
    let copies: Vec<&RetainedWrite> = recorded
        .unit_writes_at(pointer)
        .into_iter()
        .filter(|write| write.device == device)
        .collect();
    assert_eq!(
        copies.len(),
        1,
        "R 自己写了这个单元，第 {location_index} 条位置条目那块盘上一份"
    );
    copies[0]
}

/// R 的两份都还在的单元有几个没了（两份都被盖掉）。
fn units_of_the_recorded_publish_with_both_copies_gone(
    recorded: &RecordedPublish,
    image: &MemoryPool,
) -> usize {
    let overwritten = units_of_r_overwritten_on(recorded, image);
    recorded
        .unit_writes_grouped_by_unit()
        .iter()
        .filter(|copies| {
            copies
                .iter()
                .all(|copy| overwritten.contains(&(copy.device, copy.offset.0)))
        })
        .count()
}

/// 一次可写挂载的写：按落盘次序的写与按屏障切的段（每段是写的下标）。
struct WritesOfAMount {
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
}

/// 取新实例的那次可写挂载（`image` 上 `ranges` 读回全零）做成，交回它的全部写。
fn writes_of_the_mount_that_takes_a_new_instance(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> WritesOfAMount {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, ranges, &stream);
    mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("取新实例的那次挂载做得成（这几形的前提）");
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    WritesOfAMount { writes, segments }
}

/// 入库 `urJ` 那一形的 M1：R 的根槽、R 写出的全部单元、R 的记录读回全零，M1 看不见 R、退回 P、取号、写行。
fn writes_of_the_mount_that_could_not_see_the_recorded_publish(
    recorded: &RecordedPublish,
) -> WritesOfAMount {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&every_record_of_the_recorded_publish(recorded));
    writes_of_the_mount_that_takes_a_new_instance(recorded, &crash_image, &ranges)
}

/// 被判的那次可写挂载 M2（`image` 上 `ranges` 读回全零），盘按 `order` 交：交回结局与这次挂载的写一共几次。
fn judged_mount(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    order: DeviceOrder,
) -> (
    Result<(InstanceGeneration, CheckpointTxg), MountError>,
    usize,
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, ranges, &stream);
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
    (outcome, stream.operation_count())
}

/// 两种交盘次序都在任何写之前拒，拒因是 C 的映射那一支的成员（`AccountOfAnUnappliedPublishWithIncompleteReadableRecords`）、点名 R。
fn assert_refused_before_any_write_in_both_device_orders(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    cell: &str,
) {
    for order in [DeviceOrder::AsRecorded, DeviceOrder::Reversed] {
        let (outcome, write_count) = judged_mount(recorded, image, ranges, order);
        let refusal = outcome.expect_err(&format!(
            "{cell}（盘序 {order:?}）：R 的单元还在、两个指针都没有两份更新的单元头，可写挂载在取号之前拒"
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
            "{cell}（盘序 {order:?}）：拒因是 C 的映射那一支的成员、点名 R：{refusal:?}"
        );
        assert_eq!(write_count, 0, "{cell}（盘序 {order:?}）：拒在任何写之前");
    }
}

/// 两种交盘次序都做成、落到 P。
fn assert_mounted_on_the_previous_version_in_both_device_orders(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    cell: &str,
) {
    for order in [DeviceOrder::AsRecorded, DeviceOrder::Reversed] {
        let (outcome, _write_count) = judged_mount(recorded, image, ranges, order);
        let effective = outcome.unwrap_or_else(|refusal| {
            panic!("{cell}（盘序 {order:?}）：R 有一个指针两份都已是更新实例的单元，放过、可写挂载做成：{refusal:?}")
        });
        assert_eq!(
            effective, recorded.previous_key,
            "{cell}（盘序 {order:?}）：R 走读失败、系统配置没见证它，落到 P"
        );
    }
}

/// k = 5 那一段：M1（入库 `urJ` 那一形）崩在写行那次发布的单元写那一段里，那一段只落了 `pointers_whose_copy_on_the_first_device_landed`
/// 各自第 0 条位置条目那块盘上盖住 R 那一份的那几次写（之前的段全落了）。交回那一刻的盘面。
fn image_with_the_mount_taking_a_new_instance_crashed_inside_its_unit_writes(
    recorded: &RecordedPublish,
    pointers_whose_copy_on_the_first_device_landed: &[NodePointer],
) -> MemoryPool {
    let mount_taking_a_new_instance =
        writes_of_the_mount_that_could_not_see_the_recorded_publish(recorded);
    let unit_write_segment = mount_taking_a_new_instance
        .segments
        .iter()
        .position(|segment| {
            segment
                .iter()
                .any(|index| mount_taking_a_new_instance.writes[*index].kind == StepKind::UnitWrite)
        })
        .expect("M1 写行那次发布有单元写");
    let mut persisted: Vec<RetainedWrite> = mount_taking_a_new_instance.segments
        [..unit_write_segment]
        .iter()
        .flatten()
        .map(|index| mount_taking_a_new_instance.writes[*index].clone())
        .collect();
    for pointer in pointers_whose_copy_on_the_first_device_landed {
        let location = pointer.locations[0];
        let landed: Vec<&RetainedWrite> = mount_taking_a_new_instance.segments[unit_write_segment]
            .iter()
            .map(|index| &mount_taking_a_new_instance.writes[*index])
            .filter(|write| {
                write.kind == StepKind::UnitWrite
                    && write.device == location.device
                    && write.offset == location.slot.to_device_offset()
            })
            .collect();
        assert_eq!(
            landed.len(),
            1,
            "前提：M1 的单元写那一段里有一次写正盖在 R 的这一份上（{location:?}）"
        );
        persisted.push(landed[0].clone());
    }
    let mut image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    image.apply_writes(&persisted);
    image
}

/// 第三轮攻方序号 1032（`research/prompts/c834-r3-opus-model/out/excerpts.txt`）：M1 只落了盘 0 上盖住 R 的树表那一写；M2 时 R 在另一块盘上的树表、
/// R 的映射根两份、R 的记录读回全零（`tt1+mr0+mr1+rec`）。树表一份是 M1 的单元、一份读回全零，映射根两份读回全零：没有一个指针两份都更新，拒。
/// 判别力：「有一份更新就放过」（变异「C834 同一指针只要一份更新就放过」）时放过、写行那次发布的根落盘之前写掉 R 剩下的那一份，这一条红。
#[test]
fn newer_root_whose_tree_table_has_one_copy_overwritten_and_the_rest_reading_back_zero_refuses_the_writable_mount_in_both_device_orders(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let root_of_the_recorded_publish = recorded.root();
    let image = image_with_the_mount_taking_a_new_instance_crashed_inside_its_unit_writes(
        &recorded,
        std::slice::from_ref(&root_of_the_recorded_publish.tree_table),
    );
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &image).len(),
        1,
        "前提：R 写出的单元只被盖掉一份（树表那一份）"
    );
    assert_eq!(
        units_of_the_recorded_publish_with_both_copies_gone(&recorded, &image),
        0,
        "前提：R 每个单元还剩一份，R 走得完"
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[copy_of_the_recorded_publish_at(
            &recorded,
            &root_of_the_recorded_publish.tree_table,
            1,
        )])
        .with_the_writes(&recorded.unit_writes_at(&root_of_the_recorded_publish.mapping_root))
        .with_the_writes(&every_record_of_the_recorded_publish(&recorded));
    assert_refused_before_any_write_in_both_device_orders(
        &recorded,
        &image,
        &ranges,
        "M1 只盖了树表一份、其余读回全零",
    );
}

/// 第三轮攻方序号 1288：M1 只落了盘 0 上盖住 R 的映射根与树表那两写；M2 时 R 在另一块盘上的树表与映射根、R 的记录读回全零（`tt1+mr1+rec`）。
/// 两个指针都是一份 M1 的单元、一份读回全零：拒。判别力同上一条（「有一份更新就放过」时放过、写掉 R 剩下的那一份）。
#[test]
fn newer_root_whose_tree_table_and_mapping_root_each_have_one_copy_overwritten_and_the_other_reading_back_zero_refuses_the_writable_mount_in_both_device_orders(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let root_of_the_recorded_publish = recorded.root();
    let image = image_with_the_mount_taking_a_new_instance_crashed_inside_its_unit_writes(
        &recorded,
        &[
            root_of_the_recorded_publish.tree_table,
            root_of_the_recorded_publish.mapping_root,
        ],
    );
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &image).len(),
        2,
        "前提：R 写出的单元只被盖掉两份（树表、映射根各一份）"
    );
    assert_eq!(
        units_of_the_recorded_publish_with_both_copies_gone(&recorded, &image),
        0,
        "前提：R 每个单元还剩一份，R 走得完"
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[
            copy_of_the_recorded_publish_at(&recorded, &root_of_the_recorded_publish.tree_table, 1),
            copy_of_the_recorded_publish_at(
                &recorded,
                &root_of_the_recorded_publish.mapping_root,
                1,
            ),
        ])
        .with_the_writes(&every_record_of_the_recorded_publish(&recorded));
    assert_refused_before_any_write_in_both_device_orders(
        &recorded,
        &image,
        &ranges,
        "M1 只盖了树表与映射根各一份、另一份读回全零",
    );
}

/// 入库 `urJ` 那一形：M1 写行那次发布的根落盘之前全部落了，R 的树表与映射根四个槽都已是 M1 的单元、R 的记录槽是 M1 的记录。
/// M2 没有读故障，或只藏 R 的映射根两槽（凭树表那一对放过），或只藏 R 的树表两槽（凭映射根那一对放过）：都放过、做成、落到 P。
/// 判别力：一律拒（变异「C834 那一判一律拒」）时这一条红。
fn assert_the_mount_after_the_urj_form_is_mounted_on_the_previous_version(
    hidden_pointer: Option<fn(&singlefs_core::root_record::RootRecord) -> NodePointer>,
    cell: &str,
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let mount_taking_a_new_instance =
        writes_of_the_mount_that_could_not_see_the_recorded_publish(&recorded);
    let row_root = mount_taking_a_new_instance
        .writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("M1 写行那次发布有根槽写");
    let mut image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    image.apply_writes(&mount_taking_a_new_instance.writes[..row_root]);
    let root_of_the_recorded_publish = recorded.root();
    for pointer in [
        root_of_the_recorded_publish.tree_table,
        root_of_the_recorded_publish.mapping_root,
    ] {
        assert_eq!(
            units_of_r_overwritten_on(&recorded, &image)
                .iter()
                .filter(|(device, offset)| pointer
                    .locations
                    .iter()
                    .any(|location| location.device == *device
                        && location.slot.to_device_offset().0 == *offset))
                .count(),
            2,
            "前提：R 的这个指针两槽都已被 M1 盖掉"
        );
    }
    let ranges = match hidden_pointer {
        None => RangesReadingBackAllZero::default(),
        Some(pointer_of) => RangesReadingBackAllZero::default()
            .with_the_writes(&recorded.unit_writes_at(&pointer_of(&root_of_the_recorded_publish))),
    };
    assert_mounted_on_the_previous_version_in_both_device_orders(&recorded, &image, &ranges, cell);
}

#[test]
fn newer_root_whose_pointer_slots_all_hold_units_of_the_mount_that_could_not_see_it_is_passed_in_both_device_orders(
) {
    assert_the_mount_after_the_urj_form_is_mounted_on_the_previous_version(
        None,
        "urJ 那一形、没有读故障",
    );
}

#[test]
fn newer_root_whose_tree_table_slots_hold_units_of_a_later_instance_is_passed_with_its_mapping_root_slots_reading_back_zero(
) {
    assert_the_mount_after_the_urj_form_is_mounted_on_the_previous_version(
        Some(|root| root.mapping_root),
        "urJ 那一形、映射根两槽读回全零",
    );
}

#[test]
fn newer_root_whose_mapping_root_slots_hold_units_of_a_later_instance_is_passed_with_its_tree_table_slots_reading_back_zero(
) {
    assert_the_mount_after_the_urj_form_is_mounted_on_the_previous_version(
        Some(|root| root.tree_table),
        "urJ 那一形、树表两槽读回全零",
    );
}

/// 入库 Y2 那几段：R 的记录、映射根两份、树表两份读回全零。
fn ranges_hiding_the_records_mapping_root_and_tree_table_of_the_recorded_publish(
    recorded: &RecordedPublish,
) -> RangesReadingBackAllZero {
    let root_of_the_recorded_publish = recorded.root();
    RangesReadingBackAllZero::default()
        .with_the_writes(&every_record_of_the_recorded_publish(recorded))
        .with_the_writes(&recorded.unit_writes_at(&root_of_the_recorded_publish.mapping_root))
        .with_the_writes(&recorded.unit_writes_at(&root_of_the_recorded_publish.tree_table))
}

/// 第一轮正推那两段：M0 没有读故障（择到 R、建在 R 上）取号，取号那两次系统配置槽写落了 `acquisition_writes_landed` 次就断电（之后一个字节没写）；
/// M2 时 Y2 那几段读回全零。R 的单元一份不少，两个指针四个槽读回全零，没有凭据：拒。今天的判法读择到那一份系统配置的号，
/// 取号只落一块盘时随盘序变、两块盘都落了时放过、写掉 R 的单元。
fn assert_a_mount_after_an_acquisition_that_wrote_nothing_else_is_refused(
    acquisition_writes_landed: usize,
    cell: &str,
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mount_choosing_the_recorded_publish = writes_of_the_mount_that_takes_a_new_instance(
        &recorded,
        &crash_image,
        &RangesReadingBackAllZero::default(),
    );
    let acquisition_writes: Vec<usize> = mount_choosing_the_recorded_publish
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::SystemConfigurationSlot)
        .map(|(index, _)| index)
        .take(2)
        .collect();
    assert_eq!(
        acquisition_writes.len(),
        2,
        "前提：M0 取号每块盘写一个系统配置槽"
    );
    let mut image = crash_image;
    image.apply_writes(
        &mount_choosing_the_recorded_publish.writes
            [..=acquisition_writes[acquisition_writes_landed - 1]],
    );
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &image),
        Vec::<(DeviceIdentity, u64)>::new(),
        "前提：R 的单元一份不少"
    );
    assert_refused_before_any_write_in_both_device_orders(
        &recorded,
        &image,
        &ranges_hiding_the_records_mapping_root_and_tree_table_of_the_recorded_publish(&recorded),
        cell,
    );
}

/// 取号只落了一块盘（两块盘的系统配置一新一旧）。
#[test]
fn newer_root_after_an_acquisition_that_landed_on_one_device_only_refuses_the_writable_mount_in_both_device_orders(
) {
    assert_a_mount_after_an_acquisition_that_wrote_nothing_else_is_refused(1, "取号只落一块盘");
}

/// 取号两块盘都落了、之后一个字节没写。
#[test]
fn newer_root_after_an_acquisition_that_landed_on_both_devices_and_wrote_nothing_else_refuses_the_writable_mount_in_both_device_orders(
) {
    assert_a_mount_after_an_acquisition_that_wrote_nothing_else_is_refused(
        2,
        "取号两块盘落完、没写",
    );
}

/// 丙：M1 时 R 的树表两份与 R 的记录读回全零，M1 退回 P、从映射现算护住 R 的单元、取号，把写行记录写进 R 的记录槽，写行根落盘之前断电；
/// M2 时 R 的树表与映射根读回全零。带更大号的记录与系统配置都在，R 的单元却一份不少；两个指针四个槽读回全零，没有凭据：拒。
#[test]
fn newer_root_whose_units_a_later_mount_kept_while_taking_its_record_slot_refuses_the_writable_mount_in_both_device_orders(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let root_of_the_recorded_publish = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mount_taking_a_new_instance = writes_of_the_mount_that_takes_a_new_instance(
        &recorded,
        &crash_image,
        &RangesReadingBackAllZero::default()
            .with_the_writes(&recorded.unit_writes_at(&root_of_the_recorded_publish.tree_table))
            .with_the_writes(&every_record_of_the_recorded_publish(&recorded)),
    );
    let row_root = mount_taking_a_new_instance
        .writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("M1 写行那次发布有根槽写");
    let mut image = crash_image;
    image.apply_writes(&mount_taking_a_new_instance.writes[..row_root]);
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &image),
        Vec::<(DeviceIdentity, u64)>::new(),
        "前提：M1 护住了 R 的单元，一份不少"
    );
    let configuration = choose_system_configuration(&image).expect("镜像上择得到系统配置");
    assert!(
        scan_journal(&image, &configuration)
            .values()
            .all(|record| (record.instance, record.checkpoint_txg) != recorded.key),
        "前提：R 的记录槽已被 M1 的记录占了，R 的记录一条都读不出"
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded.unit_writes_at(&root_of_the_recorded_publish.tree_table))
        .with_the_writes(&recorded.unit_writes_at(&root_of_the_recorded_publish.mapping_root));
    assert_refused_before_any_write_in_both_device_orders(
        &recorded,
        &image,
        &ranges,
        "丙：M1 护住 R 的单元、占了 R 的记录槽",
    );
}
