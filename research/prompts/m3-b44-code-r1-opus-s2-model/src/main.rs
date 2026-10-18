//! m3-b44 代码轮攻方第二次抽样，格 K1：C834 新判法 `one_pointer_has_both_copies_overwritten_by_a_later_instance`（冻结副本原样，不打补丁）。
//!
//! 历史：R 照入库搭建 `common_unwitnessed_publish` 录（384 槽小盘、第一个文件、前缀之后录 R：一次覆盖写或顺序写三个数据单元），
//! 崩在 R 的根槽之后、系统配置轮换之前。M1 是入库 `urJ` 那一形的可写挂载：R 的根槽、R 写出的全部单元、R 的全部记录读回全零，
//! M1 看不见 R、退回 P、取号、写行。被判的那次可写挂载 M2 在 M1 留下的盘面上做，读故障按 R 的每一份分开点
//! （tt0 / tt1 / mr0 / mr1：R 的树表、映射根在第一、第二条位置条目上那一份所在的字节；rec：R 的全部记录；none：不藏），两种交盘次序。
//!
//! 模式：
//!   survey <overwrite|seq3> <k|前缀>：M1 写行根之前全落的那一个盘面，读四个指针槽，每组读故障、两序各挂一次 M2。
//!   count  <overwrite|seq3> <k|前缀>：只打 M1 写流的崩溃状态数。
//!   run    <overwrite|seq3> <k|前缀> <起点> <终点> <读故障组,…>：M1 写流的崩溃状态逐个展开（崩溃点重放同一个枚举域：
//!          `crash_state_enumeration_tables` + `extract_flow_facts`，每段 `full_expansion`，原地覆写取三态），每个状态先跑池级 checker，
//!          再挂 M2；M2 做成的，在 M2 写行根落盘之前那一刻与 M2 全部写落盘之后各跑一次池级 checker。
#[path = "../../repo/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;
mod prefix;

use common_unwitnessed_publish::{
    fixed_geometry_of_the_384_slot_small_devices, record_one_publish_after_overwrites,
    recorded_devices_reading_back_all_zero_on, root_is_in_the_ring, units_of_r_overwritten_on,
    PublishRecordedAsR, RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, crash_state_enumeration_tables, full_expansion,
};
use singlefs_checker_tier::crash_facts::extract_flow_facts;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::mount::mount_writable;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::PoolReader;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use std::collections::BTreeSet;

fn every_record_of_r(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    (1..=recorded.record_count()).flat_map(|ordinal| recorded.record_writes_of_the_record(ordinal)).collect()
}

fn copy_of_r_at<'recorded>(recorded: &'recorded RecordedPublish, pointer: &NodePointer, location_index: usize) -> Vec<&'recorded RetainedWrite> {
    let device = pointer.locations[location_index].device;
    recorded.unit_writes_at(pointer).into_iter().filter(|write| write.device == device).collect()
}

/// 读故障组：`none`，或 tt0、tt1、mr0、mr1、rec 用 `+` 连。
fn ranges(recorded: &RecordedPublish, name: &str) -> RangesReadingBackAllZero {
    let root_of_r = recorded.root();
    let mut ranges = RangesReadingBackAllZero::default();
    if name == "none" {
        return ranges;
    }
    for token in name.split('+') {
        ranges = match token {
            "tt0" => ranges.with_the_writes(&copy_of_r_at(recorded, &root_of_r.tree_table, 0)),
            "tt1" => ranges.with_the_writes(&copy_of_r_at(recorded, &root_of_r.tree_table, 1)),
            "mr0" => ranges.with_the_writes(&copy_of_r_at(recorded, &root_of_r.mapping_root, 0)),
            "mr1" => ranges.with_the_writes(&copy_of_r_at(recorded, &root_of_r.mapping_root, 1)),
            "rec" => ranges.with_the_writes(&every_record_of_r(recorded)),
            "root" => ranges.with_the_writes(&[recorded.root_slot_write()]),
            "units" => ranges.with_the_writes(&recorded.unit_writes()),
            other => panic!("认不出读故障 {other}"),
        };
    }
    ranges
}

fn red_invariants(image: &MemoryPool) -> String {
    let red: Vec<&str> = check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| name)
        .collect();
    if red.is_empty() { "-".to_string() } else { red.join("+") }
}

fn units_of_r_gone_on(recorded: &RecordedPublish, image: &MemoryPool) -> usize {
    let overwritten: BTreeSet<(DeviceIdentity, u64)> = units_of_r_overwritten_on(recorded, image).into_iter().collect();
    recorded
        .unit_writes_grouped_by_unit()
        .iter()
        .filter(|group| group.iter().all(|copy| overwritten.contains(&(copy.device, copy.offset.0))))
        .count()
}

/// 一槽读出什么：先按 16 KiB 解索引节点，再按 32 KiB 解数据单元与打包记录单元；都不是时看全零、前一槽起那个 32 KiB 单元的后半截。
fn describe_slot(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> String {
    let slot = singlefs_format::SLOT_BYTES;
    let node = image.devices[&device].read(DeviceOffsetInBytes(offset), usize::try_from(slot).expect("16384"));
    if let Ok(header) = singlefs_core::unit::parse_index_node(&node) {
        return format!("idx:{}", header.instance.0);
    }
    let unit = image.devices[&device].read(DeviceOffsetInBytes(offset), 32768);
    if let Ok(header) = singlefs_core::unit::parse_data_unit(&unit) {
        return format!("data:{}", header.write_order.instance.0);
    }
    if let Ok(header) = singlefs_core::unit::parse_packed_unit(&unit) {
        return format!("packed:{}", header.write_order.instance.0);
    }
    if node.iter().all(|byte| *byte == 0) {
        return "zero".to_string();
    }
    let previous = image.devices[&device].read(DeviceOffsetInBytes(offset - slot), 32768);
    if let Ok(header) = singlefs_core::unit::parse_data_unit(&previous) {
        return format!("half_data:{}", header.write_order.instance.0);
    }
    if let Ok(header) = singlefs_core::unit::parse_packed_unit(&previous) {
        return format!("half_packed:{}", header.write_order.instance.0);
    }
    "other".to_string()
}

/// 判法的凭据，照条款另写一份（不调 core 的私有函数）：经 M2 那一套读故障读每个槽，16 KiB 解索引节点，不成再 32 KiB 解数据单元、打包记录单元。
fn later_instance_at<Reader: PoolReader + ?Sized>(reader: &Reader, device: DeviceIdentity, offset: DeviceOffsetInBytes) -> Option<u32> {
    if let Some(node) = reader.read(device, offset, 16384) {
        if let Ok(header) = singlefs_core::unit::parse_index_node(&node) {
            return Some(header.instance.0);
        }
    }
    let unit = reader.read(device, offset, 32768)?;
    singlefs_core::unit::parse_data_unit(&unit)
        .map(|header| header.write_order.instance.0)
        .or_else(|_| singlefs_core::unit::parse_packed_unit(&unit).map(|header| header.write_order.instance.0))
        .ok()
}

fn clause_credential(recorded: &RecordedPublish, image: &MemoryPool, faults: &RangesReadingBackAllZero) -> bool {
    let stream = SharedStream::retaining_contents();
    let devices = recorded_devices_reading_back_all_zero_on(image, faults, &stream);
    let root = recorded.root();
    [root.tree_table, root.mapping_root]
        .into_iter()
        .filter(|pointer| *pointer != NodePointer::empty_root())
        .any(|pointer| {
            pointer.locations.iter().all(|location| {
                later_instance_at(devices.as_slice(), location.device, location.slot.to_device_offset())
                    .is_some_and(|instance| instance > root.instance.0)
            })
        })
}

/// M2 一次：结局串。做成的另交 M2 写行根之前那一刻与全部写落盘之后的盘面（跑 checker 用）。
fn judged_mount(recorded: &RecordedPublish, image: &MemoryPool, order: &str, faults: &RangesReadingBackAllZero, with_checker: bool) -> String {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, faults, &stream);
    if order == "reversed" {
        devices.reverse();
    }
    match mount_writable(&recorded.pool.parameters, &mut devices) {
        Err(refusal) => {
            let name = format!("{refusal:?}");
            let pieces: Vec<&str> = name.split(|c: char| !c.is_alphanumeric()).filter(|piece| !piece.is_empty()).take(2).collect();
            format!("refused:{}:writes={}", pieces.join(":"), stream.operation_count())
        }
        Ok(mounted) => {
            let (writes, _segments, _indexes) =
                writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
            let row_root = writes.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("写行那次发布有根槽写");
            let mut before_row_root = image.clone();
            before_row_root.apply_writes(&writes[..row_root]);
            let already: BTreeSet<(DeviceIdentity, u64)> = units_of_r_overwritten_on(recorded, image).into_iter().collect();
            let total: BTreeSet<(DeviceIdentity, u64)> = units_of_r_overwritten_on(recorded, &before_row_root).into_iter().collect();
            let newly = total.difference(&already).count();
            let gone = units_of_r_gone_on(recorded, &before_row_root);
            let (red_before_row_root, red_after) = if with_checker {
                let mut after = image.clone();
                after.apply_writes(&writes);
                (red_invariants(&before_row_root), red_invariants(&after))
            } else {
                ("not_run".to_string(), "not_run".to_string())
            };
            format!(
                "mounted:eff=({},{}):inst={}:new={newly}:gone={gone}:red_before_row_root={red_before_row_root}:red_after={red_after}",
                mounted.output.effective_root.instance.0,
                mounted.output.effective_root.checkpoint_txg.0,
                mounted.output.instance.0
            )
        }
    }
}

fn first_mount(recorded: &RecordedPublish, crash_image: &MemoryPool) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, String) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(crash_image, &ranges(recorded, "root+units+rec"), &stream);
    let first = mount_writable(&recorded.pool.parameters, &mut devices).expect("M1 做得成");
    let summary = format!("m1_instance={}", first.output.instance.0);
    let (writes, segments, _indexes) =
        writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
    (writes, segments, summary)
}

fn pointer_slots(recorded: &RecordedPublish) -> Vec<(&'static str, DeviceIdentity, u64)> {
    let root = recorded.root();
    vec![
        ("tt0", root.tree_table.locations[0].device, root.tree_table.locations[0].slot.to_device_offset().0),
        ("tt1", root.tree_table.locations[1].device, root.tree_table.locations[1].slot.to_device_offset().0),
        ("mr0", root.mapping_root.locations[0].device, root.mapping_root.locations[0].slot.to_device_offset().0),
        ("mr1", root.mapping_root.locations[1].device, root.mapping_root.locations[1].slot.to_device_offset().0),
    ]
}

/// survey 每个盘面都挂的读故障组：入库三格（none、只藏映射根两槽、只藏树表两槽）与它们各加记录。
const SURVEY_FAULTS: [&str; 6] = ["none", "mr0+mr1", "tt0+tt1", "rec", "mr0+mr1+rec", "tt0+tt1+rec"];

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let kind = arguments.get(1).cloned().unwrap_or_else(|| "overwrite".to_string());
    let prefix_argument = arguments.get(2).cloned().unwrap_or_else(|| "5".to_string());
    let mode = arguments.get(3).cloned().unwrap_or_else(|| "count".to_string());
    let publish = match kind.as_str() {
        "overwrite" => PublishRecordedAsR::Overwrite,
        "seq3" => PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
        other => panic!("认不出 R 的种类 {other}"),
    };
    let (recorded, history) = match prefix_argument.parse::<u64>() {
        Ok(overwrites_before) => (record_one_publish_after_overwrites(overwrites_before, publish), format!("urj_{kind}_k{overwrites_before}")),
        Err(_) => match prefix::record_after_prefix(&prefix_argument, publish) {
            Ok(recorded) => (recorded, format!("urj_{kind}_p{prefix_argument}")),
            Err(refusal) => {
                println!("name=k1_history history=urj_{kind}_p{prefix_argument} refused={refusal}");
                return;
            }
        },
    };
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let (writes, segments, summary) = first_mount(&recorded, &crash_image);
    let state_count = crash_state_count_with_torn_in_place_overwrites(&crash_image, &writes, &segments, &full_expansion);
    println!(
        "name=k1_history history={history} r=({},{}) p=({},{}) r_records={} {summary} writes={} segments={} state_count={state_count}",
        recorded.key.0 .0, recorded.key.1 .0, recorded.previous_key.0 .0, recorded.previous_key.1 .0,
        recorded.record_count(), writes.len(), segments.len()
    );
    if mode == "count" {
        return;
    }
    let slots = pointer_slots(&recorded);
    if mode == "survey" {
        let row_root = writes.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("M1 写行根");
        let mut landed = crash_image.clone();
        landed.apply_writes(&writes[..row_root]);
        let on_disk: Vec<String> = slots.iter().map(|(role, device, offset)| format!("{role}={}", describe_slot(&landed, *device, *offset))).collect();
        println!(
            "name=k1_geometry history={history} r_copies_covered_by_m1={} r_units_gone={} r_in_ring={} {} landed_red={}",
            units_of_r_overwritten_on(&recorded, &landed).len(),
            units_of_r_gone_on(&recorded, &landed),
            root_is_in_the_ring(&landed, recorded.key),
            on_disk.join(" "),
            red_invariants(&landed)
        );
        for fault_name in SURVEY_FAULTS {
            let faults = ranges(&recorded, fault_name);
            let credential = clause_credential(&recorded, &landed, &faults);
            for order in ["forward", "reversed"] {
                println!(
                    "name=k1_landed history={history} faults={fault_name} order={order} clause_credential={credential} outcome={}",
                    judged_mount(&recorded, &landed, order, &faults, true)
                );
            }
        }
        return;
    }
    assert_eq!(mode, "run");
    assert!(state_count <= 1_000_000, "一次全量不超过约 10⁶ 个状态");
    let tables = crash_state_enumeration_tables(&crash_image, &writes, &segments, &full_expansion);
    let facts = extract_flow_facts(&crash_image, &writes, &segments, &full_expansion);
    assert_eq!(tables.state_count, state_count);
    let start: u64 = arguments.get(4).map_or(0, |value| value.parse().expect("起点序号"));
    let end: u64 = arguments.get(5).map_or(state_count, |value| value.parse().expect("终点序号")).min(state_count);
    let fault_sets: Vec<String> = arguments.get(6).map_or_else(|| vec!["none".to_string()], |value| value.split(',').map(str::to_string).collect());
    let fault_ranges: Vec<RangesReadingBackAllZero> = fault_sets.iter().map(|name| ranges(&recorded, name)).collect();
    for ordinal in start..end {
        let persisted = facts.persisted_writes_of_state(ordinal);
        let chosen: Vec<RetainedWrite> = tables
            .enumerated_writes
            .iter()
            .zip(&persisted)
            .filter(|(_, is_persisted)| **is_persisted)
            .map(|(write, _)| write.clone())
            .collect();
        let mut image = crash_image.clone();
        image.apply_writes(&chosen);
        let segment = facts.segment_of_state(ordinal);
        let state_red = red_invariants(&image);
        let overwritten = units_of_r_overwritten_on(&recorded, &image).len();
        let gone = units_of_r_gone_on(&recorded, &image);
        let r_in_ring = root_is_in_the_ring(&image, recorded.key);
        let on_disk: Vec<String> = slots.iter().map(|(role, device, offset)| format!("{role}:{}", describe_slot(&image, *device, *offset))).collect();
        for (fault_name, faults) in fault_sets.iter().zip(&fault_ranges) {
            let credential = clause_credential(&recorded, &image, faults);
            for order in ["forward", "reversed"] {
                let outcome = judged_mount(&recorded, &image, order, faults, true);
                let mounted = outcome.starts_with("mounted:");
                let newly = mounted && !outcome.contains(":new=0:");
                let wrong = mounted && r_in_ring && gone == 0 && newly;
                let refused_with_r_gone = !mounted && gone > 0;
                println!(
                    "name=k1_state history={history} faults={fault_name} ordinal={ordinal} segment={segment} r_in_ring={r_in_ring} r_copies_overwritten={overwritten} r_units_gone={gone} on_disk={} state_red={state_red} order={order} clause_credential={credential} wrong={wrong} refused_with_r_gone={refused_with_r_gone} outcome={outcome}",
                    on_disk.join(",")
                );
            }
        }
    }
}
