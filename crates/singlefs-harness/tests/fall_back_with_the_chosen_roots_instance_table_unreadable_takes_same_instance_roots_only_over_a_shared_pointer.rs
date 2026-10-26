//! 所选根的实例表读不出时 F2 退到哪一条（C751（只读挂载在实例表读不出时越过已确认的版本），用户 2026-10-20 定「甲：共用实例表指针才收」，
//! D23（journal 的角色与格式） 已定项 14 F2 段）：只认与见证到的那一版恰好相等、读得出的那条根；见证到的那一版与所选根的实例表指针相同时
//! 另收区间里与所选根同实例、读得出的根（两个指针都在根记录里，不读表）。判退看三样读者的并集、退不成留在所选那一版（C755），这几形里三处的结局照它。
//! C751 那一行「仍欠」列的五形，这里三形：登记形、`cross`、「见证读不出而区间有同实例根」；清扫换表那一形在
//! `fall_back_past_a_same_instance_sweep_publish_that_rewrote_the_instance_table.rs`；`:656` 那一形就是入库用例
//! `recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` 的
//! `a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version`（照旧绿）。
//! 盘面造法照根环三方第三轮辩方的模型（`research/prompts/m3-close-rootring-r3-sonnet-model/tests/rootring_r3_defense_narrow_on_the_r2_shapes.rs`），
//! 在 384 槽小盘上第一个文件之后覆盖写 5 次、再录一次覆盖写 R（`tests/common_unwitnessed_publish/mod.rs`）。
//! 判别力：`crates/mutations.tsv` 里以「根环一簇 R2」开头的几行（不另收、一律另收、见证读不出也另收），各让这里点名的用例红。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    image_of_devices_reading_back_all_zero, record_one_publish_after_overwrites,
    recorded_devices_reading_back_all_zero_on, PublishRecordedAsR, RangesReadingBackAllZero,
    RecordedPublish,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches, MountError, RollbackTarget,
    ShadowLedger, StillUnreadableAfterOneReread,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, instance_table_chain_of_root, recover, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const OVERWRITES_BEFORE_R: u64 = 5;
/// 实例表单元的宽（两槽，32 KiB）：读回全零那一段按它算。
const INSTANCE_TABLE_UNIT_BYTES: u64 = 32768;

/// 每块盘较新的那个系统配置槽（世代号大的那一槽）那一段。
fn newer_system_configuration_slot_of_each_device(
    image: &MemoryPool,
) -> Vec<(DeviceIdentity, u64, u64)> {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let slot_spacing_in_bytes =
        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
    image
        .devices
        .iter()
        .map(|(device, sparse_device)| {
            let newer_slot_offset = [0, slot_spacing_in_bytes]
                .into_iter()
                .filter_map(|offset| {
                    SystemConfiguration::parse_slot(
                        &sparse_device.read(DeviceOffsetInBytes(offset), slot_bytes),
                    )
                    .ok()
                    .map(|slot| (slot.quantities.slot_generation, offset))
                })
                .max()
                .map(|(_, offset)| offset)
                .expect("每块盘至少一槽自证过");
            (*device, newer_slot_offset, SYSTEM_CONFIGURATION_SLOT_BYTES)
        })
        .collect()
}

/// 一条根指着的实例表那两份。
fn instance_table_ranges_of(root: &RootRecord) -> Vec<(DeviceIdentity, u64, u64)> {
    root.instance_table
        .locations
        .iter()
        .map(|location| {
            (
                location.device,
                location.slot.to_device_offset().0,
                INSTANCE_TABLE_UNIT_BYTES,
            )
        })
        .collect()
}

/// 三处各落到哪一版：恢复（读回内容的那一版，失败时 `None`）、只读挂载、可写挂载（各自做成时那一版，做不成时 `None`）。
#[derive(Debug, PartialEq, Eq)]
struct LandingOfTheThreePlaces {
    recovery: Option<(InstanceGeneration, CheckpointTxg)>,
    read_only_mount: Option<(InstanceGeneration, CheckpointTxg)>,
    writable_mount: Option<(InstanceGeneration, CheckpointTxg)>,
}

fn landing_of_the_three_places(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> LandingOfTheThreePlaces {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let recovery = match recover(&devices, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, content } => {
            if let Some(content_of_the_version) = recorded.pool.content_of_each_root.get(&root) {
                assert_eq!(
                    &content, content_of_the_version,
                    "恢复读回的是它落到的那一版的内容"
                );
            }
            Some(root)
        }
        RecoveryOutcome::NoFile { root } => Some(root),
        RecoveryOutcome::Failed { .. } => None,
    };
    let read_only_mount = mount_read_only(&devices).ok().map(|mounted| {
        (
            mounted.effective_root.instance,
            mounted.effective_root.checkpoint_txg,
        )
    });
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
    let writable_mount = mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount)
        .ok()
        .map(|mounted| {
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg,
            )
        });
    LandingOfTheThreePlaces {
        recovery,
        read_only_mount,
        writable_mount,
    }
}

/// `chosen` 那条根的实例表在 `ranges` 下读不出（这几形的前提）。
fn assert_the_instance_table_of_the_chosen_root_is_unreadable(
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    chosen: (InstanceGeneration, CheckpointTxg),
) {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let configuration = choose_system_configuration(&devices).expect("择得到系统配置");
    let chosen_root = choose_root(&devices, &configuration).expect("择得到根");
    assert_eq!(
        (chosen_root.instance, chosen_root.checkpoint_txg),
        chosen,
        "前提：所选根是 {chosen:?}"
    );
    assert!(
        instance_table_chain_of_root(&devices, &chosen_root).is_err(),
        "前提：所选根的实例表读不出"
    );
}

/// 登记形的盘面：崩在 R 的根槽之后、轮换之前（P 已确认）；读的那一刻 R 的单元、每块盘较新的那个系统配置槽（P 那次轮换写的）、R 的根指着的实例表
/// （R 没重写实例表，那一片也是 P、Pm 的）读回全零。见证到的那一版随较新的系统配置槽读回全零落到 Pm。交回盘面与读回全零的几段。
fn registered_shape(recorded: &RecordedPublish) -> (MemoryPool, RangesReadingBackAllZero) {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut ranges = newer_system_configuration_slot_of_each_device(&crash_image);
    ranges.extend(instance_table_ranges_of(&recorded.root()));
    let ranges = RangesReadingBackAllZero(ranges).with_the_writes(&recorded.unit_writes());
    (crash_image, ranges)
}

/// P 之前那一版 Pm 的 (实例, txg)：覆盖写一次一次发布，txg 逐个加一。
fn version_before_the_previous(recorded: &RecordedPublish) -> (InstanceGeneration, CheckpointTxg) {
    let (instance, previous_txg) = recorded.previous_key;
    (instance, CheckpointTxg(previous_txg.0 - 1))
}

/// 场景：C751 登记那一形（覆盖写、k = 5）。R、P、Pm 同实例、共用一张实例表，见证到的是 Pm。
/// 预期：见证到的 Pm 与所选根 R 的实例表指针相同，另收区间里同实例、读得出的 P：只读挂载打开 P（今天打开 Pm，越过已确认的 P）；
/// 恢复与可写挂载在 P 上也读实例表、读不通，留在所选的 R、照报失败。
/// 判别力：「根环一簇 R2：所选根的实例表读不出时不另收同实例的根」那一行（只读挂载落回 Pm），这一条红。
#[test]
fn the_registered_shape_falls_back_to_the_confirmed_version_sharing_the_instance_table_pointer_of_the_chosen_root(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (image, ranges) = registered_shape(&recorded);
    assert_the_instance_table_of_the_chosen_root_is_unreadable(&image, &ranges, recorded.key);
    assert_eq!(
        landing_of_the_three_places(&recorded, &image, &ranges),
        LandingOfTheThreePlaces {
            recovery: None,
            read_only_mount: Some(recorded.previous_key),
            writable_mount: None,
        },
        "只读挂载落在已确认的 P（Pm 是 {:?}），恢复与可写挂载在 P 上读不通实例表、留在 R 照报失败",
        version_before_the_previous(&recorded)
    );
}

/// 场景：登记那一形再加 Pm 的根槽读回全零：见证到的那一版的根读不出，比不了它与所选根的实例表指针。
/// 预期：不另收同实例的根；与 Pm 恰好相等的那条也读不出，区间里没有可退的根，三处都留在 R、照报失败（只读挂载打不开 R：R 的单元读回全零）。
/// 判别力：「根环一簇 R2：见证到的那一版的根读不出时也当共用指针」那一行（只读挂载落到 P），这一条红。
#[test]
fn with_the_witnessed_root_unreadable_the_same_instance_roots_in_the_range_are_not_taken() {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (image, ranges) = registered_shape(&recorded);
    let ranges = ranges.with_the_root_slot_of(&image, version_before_the_previous(&recorded));
    assert_the_instance_table_of_the_chosen_root_is_unreadable(&image, &ranges, recorded.key);
    assert_eq!(
        landing_of_the_three_places(&recorded, &image, &ranges),
        LandingOfTheThreePlaces {
            recovery: None,
            read_only_mount: None,
            writable_mount: None,
        },
        "见证到的 Pm 的根读不出：不退，三处都照报 R 读不出"
    );
}

/// 在 `image` 上做一次可写挂载（`ranges` 那几段读回全零），录下这次挂载的全部写（按落盘次序）。
fn writes_of_a_writable_mount(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> Vec<RetainedWrite> {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, ranges, &stream);
    mount_writable_with_test_only_switches(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
    )
    .expect("这一次可写挂载做得成");
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    writes
}

/// 场景：`cross` 那一形（覆盖写、k = 5）。R 整次落盘（已确认，实例 1），之后一次干净的可写挂载取实例 2（写行一次、暖机），崩在这次挂载第二条根
/// （暖机那一条）的根槽之后、它那次轮换之前；读的那一刻那条根的实例表与每块盘较新的那个系统配置槽读回全零。见证到的是 R（实例 1），
/// 与所选根（实例 2）指着的实例表不是同一张。
/// 预期：不另收同实例的根，只认恰好等于 R 的那条：恢复与只读挂载落在 R（改法之前只读挂载留在所选根）；可写挂载照它自己那一拒——
/// 判抛弃用的最新那条根（所选根）的实例表读不出，在取号之前拒、盘上逐字节不变（成败各看各，C755）。
/// 判别力：「根环一簇 R2：所选根的实例表读不出时一律另收同实例的根」那一行（退到实例 2 写行那条根，恢复读不通实例表、留在所选根失败），这一条红。
#[test]
fn the_cross_instance_shape_falls_back_only_to_the_witnessed_version_of_the_older_instance() {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let fsid = recorded.pool.parameters.filesystem_identifier;
    let confirmed = recorded.image_with_the_whole_publish_persisted();
    let writes =
        writes_of_a_writable_mount(&recorded, &confirmed, &RangesReadingBackAllZero::default());
    let root_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let second_root_write = root_writes[1];
    let mut image = confirmed;
    image.apply_writes(&writes[..=second_root_write]);
    let root_of_the_crash = RootRecord::parse_slot(
        writes[second_root_write].bytes().expect("根槽写带字节"),
        &fsid,
    )
    .expect("这次挂载写的根自证");
    let row_root =
        RootRecord::parse_slot(writes[root_writes[0]].bytes().expect("根槽写带字节"), &fsid)
            .expect("写行那条根自证");
    assert_eq!(
        (row_root.instance, root_of_the_crash.instance),
        (InstanceGeneration(2), InstanceGeneration(2)),
        "前提：这次挂载取实例 2，写行与暖机两条根都是实例 2 的"
    );
    let mut ranges = instance_table_ranges_of(&root_of_the_crash);
    ranges.extend(newer_system_configuration_slot_of_each_device(&image));
    let ranges = RangesReadingBackAllZero(ranges);
    assert_the_instance_table_of_the_chosen_root_is_unreadable(
        &image,
        &ranges,
        (root_of_the_crash.instance, root_of_the_crash.checkpoint_txg),
    );
    assert_eq!(
        landing_of_the_three_places(&recorded, &image, &ranges),
        LandingOfTheThreePlaces {
            recovery: Some(recorded.key),
            read_only_mount: Some(recorded.key),
            writable_mount: None,
        },
        "见证到的是实例 1 的 R、指针与所选根不同：只认恰好等于 R 的那条，恢复与只读挂载落 R（写行那条根 {:?} 不收）；\
         可写挂载判抛弃用的最新那条根（所选根）的实例表读不出，拒",
        (row_root.instance, row_root.checkpoint_txg)
    );
    // 可写挂载判退同样落 R，成不成另看它自己的读者：重建分配器判抛弃用的最新那条根就是所选根，它的实例表读不出、重读一次仍读不出，
    // 在取号之前拒（D23（journal 的角色与格式） 已定项 14 拒可写的单子那一拒；与入库用例
    // `a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version` 同一拒）。
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(&image, &ranges);
    let refusal = mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount)
        .expect_err("判抛弃用的最新那条根的实例表读不出：可写挂载拒");
    assert!(
        matches!(
            &refusal,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if **still_unreadable
                    == StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                        newest_root_on_the_reread: Some(RollbackTarget {
                            instance: root_of_the_crash.instance,
                            checkpoint_txg: root_of_the_crash.checkpoint_txg,
                        }),
                    }
        ),
        "拒因是最新那条根（所选根）的实例表重读一次仍读不出：{refusal:?}"
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices_of_the_mount, recorded.pool.device_bytes)
            == image,
        "可写挂载在任何写之前拒：两块盘逐字节不变"
    );
}
