//! 不带一批的删意图（意图达成那一支 `CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords`、计划是空的）落盘之前那一核：取落点放开了 R，
//! 分配代是这次发布的记录不算 R 不空，上一版就在 R 里、没释放的记录照算（D26（后台整理与放置回收） 已定项 2；停机谓词「删意图那一刻 R 按记录全空」）。
//! R 里还装着上一版的记录时这一次拒成 `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted`：任何写之前交回，盘上逐字节不变，
//! 分配器这一侧意图与 R 排他照旧。
//! 造法：`common::build_pool` 起（第一个文件那次发布之后），在第一个文件那次发布的提交内生块那一段上经发布入口直接签一条意图（同
//! `compaction_intent_issue_writes_the_empty_segments_counted_before_the_issue.rs`），再在签发那一版上不带一批地删它。
mod common;

use common::{build_pool, disk_snapshot, parameters, BuiltPool};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::compaction_plan::{
    empty_cluster_segments_by_the_records, region_is_empty_by_the_records, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::{
    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
};
use singlefs_core::transaction::{
    publish_compaction_step, CompactionIntentStep, PoolWriter, RelocationBatchRefused,
    TransactionOutput,
};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);
/// 第一个文件那次发布的提交内生块那一段（`build_pool` 之后 0 号盘上的候选段，同
/// `compaction_intent_issue_writes_the_empty_segments_counted_before_the_issue.rs`）。
const REGION_START_SLOT: u64 = 50_240;

/// 一块盘的单元区（按记录数全空段用）。
fn unit_area_of_the_first_device(pool: &BuiltPool) -> UnitAreaOfOneDevice {
    let device_map = &pool.allocator.devices[0];
    UnitAreaOfOneDevice {
        device: device_map.device,
        start: device_map.unit_area_start(),
        slots: device_map.unit_area_slots(),
    }
}

/// 在 R 上签一条意图：`segs0` 照会话的取法交签发之前那一版按记录数出的全空段数，分配器先排他 R（同会话），交回签发那一版与 R。
fn issue_on_the_segment_of_the_first_file(
    pool: &mut BuiltPool,
) -> (TransactionOutput, CompactionRegion) {
    let previous = pool.output.clone();
    let region =
        CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(REGION_START_SLOT))
            .expect("段起点是 64 的倍数");
    let intent = CompactionIntent {
        region,
        allocation_generation_at_issue: previous.root.checkpoint_txg,
        empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
            &previous.allocation_records,
            unit_area_of_the_first_device(pool),
        ),
    };
    pool.allocator.compaction_mut().issue(intent);
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let issued = publish_compaction_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        &[],
        CompactionIntentStep::Issue(intent),
        INSTANCE,
    )
    .expect("签发");
    (issued, region)
}

/// R 里还装着上一版就在的、没释放的记录：不带一批的删意图那一核把它们算成 R 不空，拒成 `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted`，
/// 盘上逐字节不变，签发那一版的意图与分配器这一侧的 R 排他都留着。
/// 判别力：`crates/mutations.tsv`「不带一批的删意图那一核滤掉上一版的记录」那一行。
#[test]
fn an_intent_deletion_without_a_batch_is_refused_while_the_region_keeps_records_of_earlier_versions(
) {
    let mut pool = build_pool("compaction-intent-deletion-without-a-batch-region-not-empty");
    let (issued, region) = issue_on_the_segment_of_the_first_file(&mut pool);
    assert!(
        live_compaction_intent_of_the_entries(&issued.logged_operations).is_some()
            && !region_is_empty_by_the_records(&issued.allocation_records, region)
            && issued.allocation_records.iter().any(|record| {
                !record.is_released
                    && record.generation != issued.root.checkpoint_txg
                    && !region_is_empty_by_the_records(std::slice::from_ref(record), region)
            }),
        "前提：签发那一版有活意图，R 里装着签发之前就在、没释放的记录"
    );
    let before_the_deletion = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let publish_parameters = parameters();
    let deleting = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_compaction_step(
            &mut writer,
            &mut pool.allocator,
            &issued,
            &[],
            CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
            INSTANCE,
        )
    };
    match deleting {
        Err(RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
            region: refused_region,
        }) => assert_eq!(refused_region, region, "拒的是这条意图的 R"),
        Err(other) => panic!("应当拒成 R 按记录不空，实际拒成 {other:?}"),
        Ok(published) => panic!(
            "R 里还有上一版的记录，不带一批的删意图应当被拒，实际发成了 txg {}",
            published.root.checkpoint_txg.0
        ),
    }
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_deletion,
        "被拒在任何写之前：系统配置槽、根环、录制流的步数逐字节不变"
    );
    assert!(
        pool.allocator
            .compaction()
            .live_intent()
            .is_some_and(|live| live.intent.region == region)
            && pool.allocator.region_excluded_from_placement() == Some(region),
        "被拒之后分配器这一侧的活意图照旧，取落点放开的 R 排他放回"
    );
}
