//! 整理每读一版只建一份映射条目的按身份索引（用户 2026-10-26 定「每读一版建一次」；七·3 CPU 侧快写法交回时留下的「索引在哪建」那一问）：
//! `singlefs_core::mounted_session` 推整理的一步按这一版的映射条目建一次 `singlefs_core::region_scan::CentralMappingIdentityIndex`，
//! 这一版里扫几个 R（选段时逐段先扫、续做意图时扫 R）都经 `scan_region_over_the_identity_index` 共用它，不再每扫一个 R 现建一份。
//! 数法照 `region_scan.rs` 只供测试读的查找计数：特性 `lookup-work-counters` 下本线程上建了几份索引、扫了几个 R
//! （`singlefs_core::region_scan::identity_index_builds_and_region_scans_while`）。
//! 池按 `common::build_pool` 起；另一次挂载里顺序写第一个文件 40 个数据单元，重挂之后第一块盘上段 50176、50240 只装着数据单元
//! （草稿探针量过：29 个、11 个）。重挂之后把第一块盘上每个未释放单元的头改坏（只写盘，不动这一版），区域扫描把它们都判成不一致、
//! 一段里一个能搬的都没有：不点名选段时选段一段段往下扫，扫完每一段都没得选。显式入口一趟的预算压到 1：这一趟只推一步、只读一版。
mod common;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::compaction_plan::{
    CompactionBatchSize, CompactionBatchUnits, CompactionRunForcing, CompactionRunStop,
    CompactionSwitches,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession, UserChange};
use singlefs_core::region_scan::identity_index_builds_and_region_scans_while;
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::data_unit_payload_capacity;

/// mkfs 写的每区根槽数 S（`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`）。
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;
/// 显式入口一趟只推一步：「已经过去的发布数」取 `3S − 5`，`B_gc` = `3S − 4 − 已经过去的发布数` = 1。
const PUBLISHES_PASSED_LEAVING_A_BUDGET_OF_ONE: u64 = 3 * ROOT_RING_SLOTS_PER_REGION - 4 - 1;
/// 另一次挂载里顺序写第一个文件的数据单元数。
const DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT: usize = 40;
/// 改坏单元头写的字节数与字节：从槽起点写一个物理块，单元头（类型标签在里面）整个落在这一块里；0xA5 不是任何一类单元的类型标签。
const CORRUPTED_HEADER_BYTES: usize = 4096;
const CORRUPTED_HEADER_BYTE: u8 = 0xA5;

/// 重开、可写挂载（产品默认 `cpu`）、开会话。
fn mounted_session_of(pool: &mut BuiltPool) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (session, devices)
}

#[test]
fn choosing_a_region_over_several_segments_without_a_movable_unit_builds_the_mapping_identity_index_once_for_the_version(
) {
    let mut pool = build_pool("compaction-builds-the-mapping-identity-index-once-per-version");
    let (mut session_of_the_write, mut devices_of_the_write) = mounted_session_of(&mut pool);
    let content: Vec<u8> = (0..DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT
        * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 241).expect("小于 256"))
        .collect();
    session_of_the_write
        .publish_user_change(
            &mut devices_of_the_write,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
        )
        .expect("顺序写第一个文件");
    drop(session_of_the_write);
    pool.devices = Some(devices_of_the_write);
    let (mut session, mut devices) = mounted_session_of(&mut pool);
    let unreleased_slots_on_the_first_device: Vec<_> = session
        .current
        .version_of_trees()
        .expect("有树的一版")
        .allocation_records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && !record.is_released)
        .map(|record| record.slot)
        .collect();
    let (_, first_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == DeviceIdentity(0))
        .expect("池里有第一块盘");
    for slot in &unreleased_slots_on_the_first_device {
        first_device
            .write_at(
                slot.to_device_offset(),
                &[CORRUPTED_HEADER_BYTE; CORRUPTED_HEADER_BYTES],
                WriteDurability::Plain,
            )
            .expect("改坏单元头：槽起点按物理块对齐、落在盘里");
    }
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(1).expect("至少 1"),
            ),
            forced_region: None,
            run_forcing: CompactionRunForcing::FollowTheWatermarks,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
    session.compaction.publishes_since_the_last_user_change =
        PUBLISHES_PASSED_LEAVING_A_BUDGET_OF_ONE;
    let (run, counted) = identity_index_builds_and_region_scans_while(|| {
        session.run_compaction(&mut devices).expect("整理一趟")
    });
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoEligibleSegment)
        ) && run.versions_published.is_empty(),
        "每一段都没有能搬的单元：一个都没发、停在没有合格段（这一趟只推了一步、只读了一版）：{:?}",
        run.outcome
    );
    assert!(
        counted.regions_scanned >= 2,
        "前提：不点名选段，这一版里选段一段段扫了不止一个 R（{counted:?}）"
    );
    assert_eq!(
        counted.identity_indexes_built, 1,
        "这一版扫了 {} 个 R，映射条目的按身份索引只建了一份、各次共用",
        counted.regions_scanned
    );
}
