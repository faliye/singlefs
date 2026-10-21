//! 攻方模型（m3-close-rootring-r3，云端攻方腿，攻补丁前的 `crates/`）：C773 合成形态的 I-7.18「checker 取根自己带的 F 现算门槛、要求等于 t」
//! 在零单元发布（写行之后的暖机、卸载）上怎么判。先量今天：环转满之后（环洞池 S = 4，实例 2 覆盖写到 txg 19，环里 txg 8–19）实例 3 一次可写挂载、
//! 再卸载（卸载入口记成录制流的一段），崩在这串里每一条根的根槽之后（之后的写全丢），池级 checker 在每张镜像上判 I-3.1；每条根打出：这次发布写没写单元、根带的 F、
//! 环里最旧的根、checker 现算的门槛 max(F, 环里最旧的有效根)，与这条根之前最近一条带单元的发布那一刻的同一个数（写者记账用的门槛，
//! 零单元发布照抄它那一版的记账）。
//! 两组：`clean`（挂载时没有读故障）；`pinned`（挂载那一刻根环一槽读回全零，写者把门槛钉在那一槽可能住过的最旧 txg）。
mod common;
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
mod common_unwitnessed_publish;
#[path = "common_images_for_the_checker_tier/instance_table_sweep_at_the_end_of_a_writable_mount.rs"]
mod scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount;

use common_root_ring_holes::{fixed_geometry_of_the_ring_hole_pools, parameters_of_the_ring_hole_pools};
use common_unwitnessed_publish::{recorded_devices_reading_back_all_zero_on, RangesReadingBackAllZero};
use scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::pool_with_a_dead_instance;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_test_only_switches, unmount, ShadowLedger};
use singlefs_harness::RecordedPublishEntry;
use singlefs_core::recovery::{choose_system_configuration, readable_roots};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes_and_entries, MemoryPool};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn verdict_text(verdict: &InvariantVerdict) -> String {
    match verdict {
        InvariantVerdict::Holds => "Holds".to_string(),
        InvariantVerdict::NotApplicable(why) => format!("NotApplicable({why})"),
        InvariantVerdict::PartiallyJudged(what) => format!("PartiallyJudged({})", what.chars().take(200).collect::<String>()),
        InvariantVerdict::Violated(what) => format!("Violated({})", what.chars().take(260).collect::<String>()),
    }
}

/// 镜像上环里自证过的根的最小 txg（这几张镜像上没有被抛弃的根：实例 1 的根早已轮出环）。
fn oldest_root_in_the_ring(image: &MemoryPool) -> u64 {
    let configuration = choose_system_configuration(image).expect("系统配置");
    readable_roots(
        image,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    )
    .iter()
    .map(|root| root.checkpoint_txg.0)
    .min()
    .expect("环里有根")
}

fn run(group: &str, unreadable_at_the_mount: &RangesReadingBackAllZero) -> (u32, u32) {
    let parameters = parameters_of_the_ring_hole_pools();
    let fsid = parameters.filesystem_identifier;
    let base = pool_with_a_dead_instance(None).image;
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(&base, unreadable_at_the_mount, &stream);
    let mut mounted = mount_writable_with_test_only_switches(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
    )
    .expect("实例 3 的可写挂载");
    stream.record_entry(RecordedPublishEntry::Unmount, || {
        unmount(&parameters, &mut devices, &mut mounted.allocator, &mut mounted.current, ShadowLedger::On)
    })
    .expect("卸载");
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &stream.retained_operations(),
        &stream.entry_spans(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let mut red = 0u32;
    let mut roots = 0u32;
    let mut previous_root_index = None;
    let mut threshold_of_the_last_publish_with_units = None;
    for (index, write) in writes.iter().enumerate() {
        if write.kind != StepKind::RootRecordFua {
            continue;
        }
        let units_written = writes[previous_root_index.map_or(0, |previous| previous + 1)..index]
            .iter()
            .filter(|each| each.kind == StepKind::UnitWrite)
            .count();
        previous_root_index = Some(index);
        let root = RootRecord::parse_slot(write.bytes().expect("根槽写带字节"), &fsid).expect("根自证");
        let mut image = base.clone();
        image.apply_writes(&writes[..=index]);
        let oldest = oldest_root_in_the_ring(&image);
        let floor = root.rollback_floor.0;
        let computed_threshold = oldest.max(floor);
        if units_written > 0 {
            threshold_of_the_last_publish_with_units = Some(computed_threshold);
        }
        let i31 = check_pool_image(&image)
            .into_iter()
            .find(|(name, _)| *name == "I-3.1")
            .map(|(_, verdict)| verdict)
            .expect("判了 I-3.1");
        red += u32::from(matches!(i31, InvariantVerdict::Violated(_)));
        roots += 1;
        println!(
            "R3ZERO group={group} root=({},{}) units_written={units_written} F={floor} oldest_root_in_the_ring={oldest} checker_threshold={computed_threshold} threshold_at_the_last_publish_with_units={} I-3.1={}",
            root.instance.0,
            root.checkpoint_txg.0,
            threshold_of_the_last_publish_with_units.map_or("-".to_string(), |value: u64| value.to_string()),
            verdict_text(&i31)
        );
    }
    println!("R3ZERO_SUMMARY group={group} roots={roots} i31_red={red}");
    (roots, red)
}

#[test]
fn zero_unit_roots_after_the_ring_turned_full_on_todays_checker() {
    run("clean", &RangesReadingBackAllZero::default());
    // 根环区域 0 的槽 2：落在它上面的 txg 是 6、18（区域 txg mod 3 = 0，槽 (txg div 3) mod 4 = 2），挂载那一刻住着 txg 18，钉在 6。
    let slot = RootRingSlot { region: 0, slot: 2 };
    let parameters = parameters_of_the_ring_hole_pools();
    let device = parameters.region_devices[0];
    let offset = slot_offset(slot, parameters.geometry.fixed_structure_slot_spacing).0;
    run("pinned", &RangesReadingBackAllZero(vec![(device, offset, 512)]));
}
