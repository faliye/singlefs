//! 正推腿探针（T6）：`segs0` 的口径。判决 G1 字段表定义句：「签发那次发布写下的记账第 11 项里 R 所在盘的值」；
//! 会话（`mounted_session::advance_compaction_by_one_publish`）写进意图的是签发之前那一版的分配记录按记录口径数出来的（首盘）全空聚簇段数。
//! 两者在签发那次发布自己开了一个新聚簇段时差一：这里把开放段填到只剩两槽，让签发那次发布开新段，比两个数。
mod common;

use common::{build_pool, parameters};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::UnitFootprint;
use singlefs_core::compaction_plan::{empty_cluster_segments_by_the_records, UnitAreaOfOneDevice};
use singlefs_core::logged_operations::{CompactionIntent, CompactionRegion};
use singlefs_core::records::STATISTIC_EMPTY_CLUSTER_SEGMENTS;
use singlefs_core::transaction::{publish_compaction_step, CompactionIntentStep, PoolWriter};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

fn issue_and_compare(fill_the_open_segment_to_two_free_slots: bool) -> (u64, u64, bool) {
    let mut pool = build_pool("probe-forward-segs0");
    let previous = pool.output.clone();
    let open_segment_before = pool.allocator.open_segment();
    if fill_the_open_segment_to_two_free_slots {
        let open = open_segment_before.expect("第一个文件那次发布之后有开放段").0;
        loop {
            let used: u64 = pool
                .allocator
                .records()
                .iter()
                .filter(|record| record.device == DeviceIdentity(0) && record.slot.0 >= open && record.slot.0 < open + 64)
                .map(|record| u64::from(record.span_slots))
                .sum();
            if used >= 62 {
                break;
            }
            pool.allocator
                .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(previous.root.checkpoint_txg.0))
                .expect("填开放段");
        }
    }
    let device_map = &pool.allocator.devices[0];
    let unit_area = UnitAreaOfOneDevice {
        device: device_map.device,
        start: device_map.unit_area_start(),
        slots: device_map.unit_area_slots(),
    };
    let segs0 = empty_cluster_segments_by_the_records(&previous.allocation_records, unit_area);
    let intent = CompactionIntent {
        region: CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(50240)).expect("64 的倍数"),
        allocation_generation_at_issue: previous.root.checkpoint_txg,
        empty_cluster_segments_at_issue: segs0,
    };
    pool.allocator.compaction_mut().issue(intent);
    let publish_parameters = parameters();
    let issued = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_compaction_step(&mut writer, &mut pool.allocator, &previous, &[], CompactionIntentStep::Issue(intent), INSTANCE)
            .expect("签发")
    };
    let row_11 = issued
        .accounting_entries
        .iter()
        .find(|entry| {
            entry.statistic == STATISTIC_EMPTY_CLUSTER_SEGMENTS
                && entry.device == DeviceIdentity(0)
                && entry.generation == issued.root.checkpoint_txg
        })
        .expect("签发那次发布写了第 11 项")
        .value;
    let opened_a_new_segment = pool.allocator.open_segment() != open_segment_before;
    (segs0, row_11, opened_a_new_segment)
}

#[test]
fn probe_forward_segs0_against_the_accounting_row_eleven_written_by_the_issue_publish() {
    let (segs0_plain, row_plain, opened_plain) = issue_and_compare(false);
    println!("name=probe_forward_segs0 case=issue_inside_the_open_segment segs0_written_in_the_intent={segs0_plain} accounting_row_11_written_by_the_issue_publish={row_plain} issue_publish_opened_a_new_segment={opened_plain}");
    let (segs0_full, row_full, opened_full) = issue_and_compare(true);
    println!("name=probe_forward_segs0 case=issue_publish_opens_a_new_segment segs0_written_in_the_intent={segs0_full} accounting_row_11_written_by_the_issue_publish={row_full} issue_publish_opened_a_new_segment={opened_full}");
}
