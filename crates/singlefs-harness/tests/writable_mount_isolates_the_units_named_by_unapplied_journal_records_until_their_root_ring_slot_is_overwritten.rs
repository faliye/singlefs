//! C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条，用户 2026-10-07 定「C 与 B 都做」）：可写挂载读阶段、
//! 重建分配器之后，把水位之上、没被施加的全部 journal 记录点名的落点隔离（选法 `all`），只在这个进程写掉那条记录的 txg 对应的根环槽、
//! 那次发布取完落点之后解开（`ring`），进程内重读读回全零的那一槽（D16（发布语义） 已定项 1）读出什么都不解开它；第九项（D28（挂载期承诺量）
//! 已定项 1）随它扩成分配器实际隔离的集合（判决第三节第 3 条）。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次发布 R，崩在 R 的根槽落盘之后、
//! 系统配置轮换之前；取新实例的那次可写挂载 M1 里 R 的根槽与单元读回全零（第一轮攻方 K1-2 那一形，`ur`），所选那一版落到 R 之前那一版 P，
//! R 的记录读得出、点名的单元验不过、不施加。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    image_of_devices_reading_back_all_zero, pool_after_the_mount,
    record_one_publish_after_overwrites, recorded_devices_reading_back_all_zero_on,
    units_of_r_overwritten_on, PublishRecordedAsR, RangesReadingBackAllZero, RecordedPublish,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::admission::{admission_reading_before_a_publish, BytesOnOneDevice};
use singlefs_core::journal::JournalRecord;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 覆盖写几次之后录 R：第一轮攻方 K1-2 与第二轮攻方第一、二节都取 k = 5（384 槽这一档里 R 那次发布新开了一段，P 的账里那一段是空的，
/// 取新实例那次挂载的提交内生块开到同一段上）。
const OVERWRITES_BEFORE_R: u64 = 5;

/// M1 里读回全零的：R 的根槽与 R 写出的每个单元（`ur`）。
fn root_slot_and_units_of_r(recorded: &RecordedPublish) -> RangesReadingBackAllZero {
    RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes())
}

/// 取新实例那次挂载写行那次发布的根落盘之前断电（第一轮攻方 K1-2）：今天那次挂载取的提交内生块落在 R 的单元上（R 的单元在 P 的账里是空闲槽），
/// 断电之后恢复择到 R（盘上 txg 最大的根）、走读失败；C 隔离了 R 的记录点名的落点，那次挂载一个都不碰，恢复落到 R 上读回 R 写的内容。
/// 判别力：C 不隔离（变异「C 一个落点都不隔离」）时 R 的单元被写掉、走读失败，F2 乙再退回 P——落到的不是 R、内容是 P 的，两条断言都红。
#[test]
fn a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance_still_reads_back_the_root_whose_units_read_back_zero_at_that_mount(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(
        &crash_image,
        &root_slot_and_units_of_r(&recorded),
        &stream,
    );
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("系统配置没见证 R：取新实例的那次可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽读回全零、点名的单元验不过：所选那一版是 P"
    );
    let (mount_writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let row_root = mount_writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("那次挂载写行那次发布有根槽写");
    let mut image_before_the_row_root = crash_image.clone();
    image_before_the_row_root.apply_writes(&mount_writes[..row_root]);
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &image_before_the_row_root),
        Vec::<(DeviceIdentity, u64)>::new(),
        "写行那次发布的根落盘之前，那次挂载写下的一切都没碰 R 的单元（C 隔离了 R 的记录点名的落点）"
    );
    let report = recover(&image_before_the_row_root, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some(recorded.key),
        "写行那次的根没落：恢复择到盘上 txg 最大的根 R"
    );
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: recorded.key,
            content: recorded.content.clone(),
        },
        "落到 R 上读回 R 写的内容（今天走读失败：UnitUnreadable）"
    );
}

/// 根环线 K4-1（判决 `research/prompts/m3-batch10y-code-r1-main-verification.md` 第二节，删除线判决 `research/prompts/m3-batch33t-code-r1-main-verification.md`
/// 第二节「K1 / K2 T1 C 的候选豁免」那一行）：取新实例的那次可写挂载里只有 R 的中央映射树根那个单元读回全零（`common_unwitnessed_publish` 那块盘
/// 每次只让 R 的一个单元读回全零，根环线正推的造法），R 的根槽与账都读得出、走读失败，所选那一版退到 P——R 按实例表有效、txg ≥ F，是回退候选根。
/// 写行那次发布的根落盘之前断电：那次挂载写下的一切都没碰 R 的单元，恢复择到 R、读回 R 写的内容。
/// 判别力：C 那一圈照旧豁免候选根引用的槽（变异「C 豁免候选根引用的槽」）时 R 的单元正是候选根 R 自己引用的，一个都不隔离，
/// 那次挂载在根落盘之前写掉它们（根环线正推量到 14 份），两条断言都红。
#[test]
fn a_power_loss_before_the_row_root_of_the_mount_still_reads_back_a_candidate_root_whose_mapping_root_read_back_zero_at_that_mount(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let root_of_r = RootRecord::parse_slot(
        recorded.root_slot_write().bytes().expect("根槽写带字节"),
        &recorded.pool.parameters.filesystem_identifier,
    )
    .expect("R 的根自证");
    let mapping_root_writes_of_r: Vec<&RetainedWrite> = recorded
        .unit_writes()
        .into_iter()
        .filter(|write| {
            root_of_r.mapping_root.locations.iter().any(|location| {
                location.device == write.device && location.slot.to_device_offset() == write.offset
            })
        })
        .collect();
    assert_eq!(
        mapping_root_writes_of_r.len(),
        2,
        "R 写了自己的中央映射树根，两块盘各一份"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&mapping_root_writes_of_r),
        &stream,
    );
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("R 的映射根读回全零、系统配置没见证 R：可写挂载退到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的映射根读回全零、走读失败，所选那一版退到见证到的 P"
    );
    let (mount_writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let row_root = mount_writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("那次挂载写行那次发布有根槽写");
    let mut image_before_the_row_root = crash_image.clone();
    image_before_the_row_root.apply_writes(&mount_writes[..row_root]);
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &image_before_the_row_root),
        Vec::<(DeviceIdentity, u64)>::new(),
        "写行那次发布的根落盘之前，那次挂载写下的一切都没碰 R 的单元（C 不豁免候选根 R 自己引用的槽）"
    );
    let report = recover(&image_before_the_row_root, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: recorded.key,
            content: recorded.content.clone(),
        },
        "写行那次的根没落：恢复择到 R、读回 R 写的内容"
    );
}

/// 第九项（D28（挂载期承诺量） 已定项 1）随 C 扩成分配器实际隔离的集合（判决第三节第 3 条）：M1 之后每块盘上分配器隔离的槽数，
/// 就是 R 写出的单元在那块盘上占的槽数（R 的根槽那一次读回全零，环里没有读得出的被抛弃根，影子账那一部分是 0，隔离的全是 C 的），
/// 准入读数的第九项逐盘等于它。判别力：C 不隔离时两样都是 0，「隔离数等于 R 的单元槽数」那一条红。
#[test]
fn the_ninth_admission_term_equals_the_slots_isolated_for_the_records_above_the_water_mark_on_every_device(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices =
        devices_reading_back_all_zero_on(&crash_image, &root_slot_and_units_of_r(&recorded));
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("系统配置没见证 R：可写挂载做得成");
    let reading =
        admission_reading_before_a_publish(&mounted.allocator, mounted.current.file_version());
    for device_map in &mounted.allocator.devices {
        let slots_of_r_on_this_device: u64 = recorded
            .unit_writes()
            .into_iter()
            .filter(|write| write.device == device_map.device)
            .map(|write| write.length_in_bytes() / SLOT_BYTES)
            .sum();
        assert_eq!(
            device_map.isolated_slots(),
            slots_of_r_on_this_device,
            "盘 {:?}：分配器隔离的槽数 = R 写出的单元在这块盘上占的槽数（C 那一部分）",
            device_map.device
        );
        let terms = reading
            .per_device()
            .iter()
            .find(|terms| terms.device == device_map.device)
            .expect("读数里每块盘一项");
        assert_eq!(
            terms.isolated_for_abandoned_roots_and_unapplied_journal_records,
            BytesOnOneDevice::of_slots(device_map.isolated_slots()),
            "盘 {:?}：第九项 = 分配器隔离的槽数（含 C 那一部分）",
            device_map.device
        );
    }
}

/// 第二轮攻方第一节那一形：M1 里 R 的根槽读回全零（钉在回收门槛里），之后照常覆盖写；R 的根槽成了环里最小的那个时，进程内记根先重读那一槽、
/// 读出 R、按实例表判被抛弃（D16 已定项 1 那一行，今天把它当没有根）。C 隔离的 R 的落点不随这一次重读解开，只在这个进程写掉 R 那一槽、
/// 那次发布取完落点之后解开：R 的根槽被盖掉之前（含盖掉它的那一次发布）R 的单元一个都不被取到。
/// 每次覆盖写之后核两样：R 的单元盘上没被写掉；R 那一槽还没被盖掉时，R 写出的每一槽在分配器上不空闲（落点政策取不取到它们看运气，
/// 不空闲不看运气）。判别力：解开退成随重读解开（变异「C 随 D16 那一行的重读解开」）时，重读出 R 的那一次发布之后 R 的槽就空闲了，「不空闲」那一条红。
#[test]
fn the_units_named_by_the_record_of_a_root_reread_as_abandoned_stay_isolated_until_this_process_overwrites_its_root_ring_slot(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices =
        devices_reading_back_all_zero_on(&crash_image, &root_slot_and_units_of_r(&recorded));
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("系统配置没见证 R：可写挂载做得成");
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
    let slots_per_region = recorded.pool.parameters.geometry.root_ring_slots_per_region;
    let ring_slot_of_r = target_for_publish(recorded.key.1, slots_per_region);
    let overwrites_limit = 4 * slots_per_region.count();
    let mut overwrites = 0;
    // 迭代上界 overwrites_limit（根环三区、每区 S 槽，一圈 3S 次发布之内必写到 R 那一槽）；每一轮发一次覆盖写、核一次。
    loop {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上这一串覆盖写都落得下");
        overwrites += 1;
        let current_txg = pool.session().current.root().checkpoint_txg;
        assert_eq!(
            units_of_r_overwritten_on(&recorded, &pool.image()),
            Vec::<(DeviceIdentity, u64)>::new(),
            "第 {overwrites} 次覆盖写（txg {current_txg:?}）之后，R 的根槽还没被这个进程盖掉或刚被盖掉：R 的单元一个都没被取到"
        );
        if current_txg > recorded.key.1
            && target_for_publish(current_txg, slots_per_region) == ring_slot_of_r
        {
            break;
        }
        // R 那一槽还没被这个进程盖掉：R 写出的每个落点在分配器上都不空闲（隔离着，或被候选根与现行账引用着）。
        let session = pool.session();
        for write in recorded.unit_writes() {
            let device_map = session
                .allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == write.device)
                .expect("单元写的盘在池里");
            assert!(
                !device_map.is_free(SlotNumber(write.offset.0 / SLOT_BYTES)),
                "第 {overwrites} 次覆盖写（txg {current_txg:?}）之后 R 那一槽还没被盖掉：R 写在盘 {:?} 偏移 {} 的单元那一槽不空闲",
                write.device,
                write.offset.0
            );
        }
        assert!(
            overwrites < overwrites_limit,
            "{overwrites_limit} 次覆盖写之内这个进程写到 R 那一槽"
        );
    }
    assert!(
        overwrites > slots_per_region.count(),
        "走到 R 那一槽被盖掉的那一次发布要转过大半圈根环（R 的根槽在那之前成了环里最小的那个，进程内记根重读过它）"
    );
}

/// 选法 `all`（判决第三节第 1 条）：R 是一次三条记录的发布（顺序写三个数据单元），M1 里 R 的根槽与末条记录读回全零、单元照读
/// （末条读不出、前两条读得出、点名的单元验得过：F1 乙不拒，那是正常崩溃在写记录中途的样子）。前两条记录点名的每个落点 C 都隔离——
/// M1 之后分配器上它们不空闲。判别力：选法退成只认带末条标志的记录（变异「C 只认末条」）时这次发布一个落点都不隔离，「不空闲」那一条红。
#[test]
fn the_units_named_by_the_readable_records_of_a_publish_whose_last_record_reads_back_zero_are_isolated(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let last_record_writes = recorded.record_writes_of_the_record(3);
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&last_record_writes);
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("末条读不出、读得出的那两条点名的单元验得过：F1 乙不拒，可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽读回全零、末条读不出：R 整次不施加，所选那一版是 P"
    );
    let filesystem_identifier =
        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier);
    for ordinal in [1, 2] {
        let record_bytes = recorded.record_writes_of_the_record(ordinal)[0]
            .bytes()
            .expect("记录写带字节");
        let record =
            JournalRecord::parse(record_bytes, filesystem_identifier).expect("R 的记录解得开");
        assert!(!record.named.is_empty(), "第 {ordinal} 条记录点名了单元");
        for named in &record.named {
            for location in &named.locations {
                let device_map = mounted
                    .allocator
                    .devices
                    .iter()
                    .find(|device_map| device_map.device == location.device)
                    .expect("点名项的盘在池里");
                assert!(
                    !device_map.is_free(location.slot),
                    "R 第 {ordinal} 条记录点名的落点（盘 {:?} 槽 {:?}）M1 之后不空闲（C 隔离了它）",
                    location.device,
                    location.slot
                );
            }
        }
    }
}
