//! C751（只读挂载在实例表读不出时越过已确认的版本） 甲（用户 2026-10-20 定「共用实例表指针才收」）的清扫换表那一形：
//! 实例表被同一个实例里的清扫发布重写（`instance_table_sweep.rs` 用同一实例重写实例表），于是同实例的根不都共用一张实例表——
//! 根环三方第三轮两腿各自打中「见证同实例就收」那一收窄形态的就是这一形（判决 `research/prompts/m3-close-rootring-r3-main-verification.md`
//! 第二节 C751 收窄形态那一格）。盘面照辩方模型 `research/prompts/m3-close-rootring-r3-sonnet-model/tests/rootring_r3_defense_sweep_shape.rs`：
//! `pool_with_a_dead_instance`（实例 1 死了、实例 2 覆盖写到 txg 19）→ 实例 3 可写挂载，末尾清扫发布（删实例 1 的行，重写实例表）→
//! 同一次挂载里再覆盖写一版 R，崩在 R 的根槽之后、轮换之前；读的那一刻 R 的实例表两份与每块盘较新的那个系统配置槽读回全零。
//! 见证到的那一版是清扫发布之前那一条（与 R 不共用实例表指针），清扫发布那条与 R 共用那张读不出的新表。
//! 判别力：`crates/mutations.tsv`「根环一簇 R2：所选根的实例表读不出时一律另收同实例的根」那一行（退到清扫发布那条，恢复读不通那张表、留在 R 失败），这一条红。
mod common;
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
mod common_unwitnessed_publish;
#[path = "common_images_for_the_checker_tier/instance_table_sweep_at_the_end_of_a_writable_mount.rs"]
mod scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount;

use common_root_ring_holes::{
    content_of_the_version, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools,
};
use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    RangesReadingBackAllZero,
};
use scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::{
    pool_with_a_dead_instance, publish_an_overwrite, sweeping_at_the_end_of_the_mount_on_any_row,
    sweeping_mount_on,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::instance_table_sweep::{
    InstanceTableSweepAtTheEndOfTheMount, InstanceTableSweepOutcome,
};
use singlefs_core::mount::{
    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, instance_table_chain_of_root, recover, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool};
use singlefs_harness::segments::StepKind;

/// 清扫发布之后那次覆盖写的内容种子。
const SALT_OF_THE_OVERWRITE_AFTER_THE_SWEEP: u64 = 400;
/// 实例表单元的宽（两槽，32 KiB）。
const INSTANCE_TABLE_UNIT_BYTES: u64 = 32768;

fn key_of(root: &RootRecord) -> (InstanceGeneration, CheckpointTxg) {
    (root.instance, root.checkpoint_txg)
}

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

/// 场景：见上（覆盖写 R 崩在根槽之后、轮换之前；R 的实例表与每块盘较新的系统配置槽读回全零）。
/// 预期：见证到的是清扫发布之前那一条，它与所选根 R 的实例表指针不同：不另收同实例的根，只认恰好等于它的那条——恢复与只读挂载都落在它上面
/// （改法之前只读挂载留在 R）；不退到清扫发布那条（它与 R 共用那张读不出的表）。可写挂载照它自己那一拒：判抛弃用的最新那条根 R 的实例表
/// 读不出，在取号之前拒、盘上逐字节不变（成败各看各，C755）。
#[test]
fn a_sweep_publish_that_rewrote_the_instance_table_in_the_same_instance_is_not_taken_as_kept() {
    let parameters = parameters_of_the_ring_hole_pools();
    let pool = pool_with_a_dead_instance(None);
    let (mut mounted, mut devices, stream) =
        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
    assert!(
        matches!(
            mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::RowsDeleted { .. }
            )
        ),
        "前提：这次挂载末尾清扫发布删了行：{:?}",
        mounted.output.instance_table_sweep
    );
    let current = mounted
        .current
        .file_version()
        .expect("这一版带文件")
        .clone();
    publish_an_overwrite(
        &mut devices,
        &mut mounted.allocator,
        &current,
        &content_of_the_version(SALT_OF_THE_OVERWRITE_AFTER_THE_SWEEP),
        mounted.output.instance,
    );
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let root_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let parse = |index: usize| {
        RootRecord::parse_slot(
            writes[index].bytes().expect("根槽写带字节"),
            &parameters.filesystem_identifier,
        )
        .expect("这次挂载写的根自证")
    };
    let last = *root_writes.last().expect("有根槽写");
    let root_r = parse(last);
    let sweep_root = parse(root_writes[root_writes.len() - 2]);
    let root_before_the_sweep = parse(root_writes[root_writes.len() - 3]);
    assert!(
        root_r.instance_table == sweep_root.instance_table
            && root_before_the_sweep.instance_table != sweep_root.instance_table
            && root_before_the_sweep.instance == root_r.instance
            && sweep_root.instance == root_r.instance,
        "前提：三条根同实例；清扫发布重写了实例表，R 照抄清扫发布那张，清扫之前那一条指着旧表"
    );
    let mut image = pool.image.clone();
    image.apply_writes(&writes[..=last]);
    let mut ranges: Vec<(DeviceIdentity, u64, u64)> = root_r
        .instance_table
        .locations
        .iter()
        .map(|location| {
            (
                location.device,
                location.slot.to_device_offset().0,
                INSTANCE_TABLE_UNIT_BYTES,
            )
        })
        .collect();
    ranges.extend(newer_system_configuration_slot_of_each_device(&image));
    let ranges = RangesReadingBackAllZero(ranges);
    let devices_of_the_reads = devices_reading_back_all_zero_on(&image, &ranges);
    let configuration = choose_system_configuration(&devices_of_the_reads).expect("择得到系统配置");
    let chosen = choose_root(&devices_of_the_reads, &configuration).expect("择得到根");
    assert_eq!(key_of(&chosen), key_of(&root_r), "前提：所选根是 R");
    assert!(
        instance_table_chain_of_root(&devices_of_the_reads, &chosen).is_err(),
        "前提：R 的实例表读不出"
    );

    let witnessed = key_of(&root_before_the_sweep);
    let recovered = recover(&devices_of_the_reads, JournalPolicy::Consult);
    assert!(
        matches!(recovered.outcome, RecoveryOutcome::FileRead { root, .. } if root == witnessed),
        "恢复落在清扫之前那一条 {witnessed:?}，不退到清扫发布那条 {:?}：{:?}",
        key_of(&sweep_root),
        recovered.outcome
    );
    let read_only = mount_read_only(&devices_of_the_reads).expect("只读挂载落在清扫之前那一条");
    assert_eq!(
        key_of(&read_only.effective_root),
        witnessed,
        "只读挂载：三处同落一版（判退看并集）"
    );
    // 可写挂载判退同样落在清扫之前那一条，可成不成另看它自己的读者：重建分配器判抛弃用的是最新那条根 R 的实例表，读不出、重读一次仍读不出，
    // 在取号之前拒（D23（journal 的角色与格式） 已定项 14 拒可写的单子那一拒；与入库用例
    // `a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version` 同一拒）。
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(&image, &ranges);
    let refusal = mount_writable(&parameters, &mut devices_of_the_mount)
        .expect_err("判抛弃用的最新那条根 R 的实例表读不出：可写挂载拒");
    assert!(
        matches!(
            &refusal,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if **still_unreadable
                    == StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                        newest_root_on_the_reread: Some(RollbackTarget {
                            instance: root_r.instance,
                            checkpoint_txg: root_r.checkpoint_txg,
                        }),
                    }
        ),
        "拒因是最新那条根 R 的实例表重读一次仍读不出：{refusal:?}"
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices_of_the_mount, image.device_size_in_bytes)
            == image,
        "可写挂载在任何写之前拒：盘上逐字节不变"
    );
}
