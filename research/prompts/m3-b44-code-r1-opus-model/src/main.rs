//! m3-b44 代码轮攻方原型（K1：`one_pointer_has_both_copies_overwritten_by_a_later_instance`，W1 的实现原样，不装候选开关）。
//! 骨架照 C834 第三轮攻方原型（`research/prompts/c834-r3-opus-model/src/main.rs`）：R 照入库搭建 `common_unwitnessed_publish` 录
//! （384 槽小盘、第一个文件、覆盖写 k 次或前缀动作，R 是一次覆盖写或顺序写三个数据单元），崩在 R 的根槽之后、系统配置轮换之前；
//! M1 是入库 `urJ` 那一形的可写挂载（R 的根槽、单元、记录读回全零，M1 退回 P、取号、写行），M1 的写流按崩溃点重放的枚举域
//! （`crash_state_enumeration_tables` + `extract_flow_facts`，每一段 `full_expansion`，原地覆写取三态）逐个崩溃状态展开；
//! 每个状态上做被判的那次挂载 M2（今天的产品代码），两种交盘次序、几组读故障各一次。副本里只多一处记录（`patch/instrument_k1.py`）。
//!
//! 读故障按「R 的哪一份」点名：tt0 / tt1（R 的树表在第一、第二条位置条目上那一份），mr0 / mr1（映射根同上），rec（R 的全部记录）；`none` 不藏。
//!
//! 用法：m3-b44-k1-attack-model <overwrite|seq3> <k 或前缀> survey <读故障组,…>
//!       m3-b44-k1-attack-model <overwrite|seq3> <k 或前缀> count
//!       m3-b44-k1-attack-model <overwrite|seq3> <k 或前缀> run <起点序号> <终点序号> <读故障组,…>
//! run 只逐行打出走到那一判的（状态，读故障，次序），没走到的按（读故障，次序，结局）汇成计数行。
#[path = "../../tree/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs"]
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
use singlefs_core::address::DeviceIdentity;
use singlefs_core::mount::{k1_take_judge_lines, mount_writable};
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use std::collections::{BTreeMap, BTreeSet};

fn every_record_of_r(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    (1..=recorded.record_count()).flat_map(|ordinal| recorded.record_writes_of_the_record(ordinal)).collect()
}

fn copy_of_r_at<'recorded>(recorded: &'recorded RecordedPublish, pointer: &singlefs_core::pointer::NodePointer, location_index: usize) -> Vec<&'recorded RetainedWrite> {
    let device = pointer.locations[location_index].device;
    recorded.unit_writes_at(pointer).into_iter().filter(|write| write.device == device).collect()
}

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

fn red_invariants(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| name)
        .collect()
}

fn joined(red: &[&str]) -> String {
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

fn image_with(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let chosen: Vec<RetainedWrite> = writes.iter().zip(persisted).filter(|(_, kept)| **kept).map(|(write, _)| write.clone()).collect();
    let mut image = base.clone();
    image.apply_writes(&chosen);
    image
}

/// 被判的那次挂载（今天的产品代码）：交回（那一判记下的行，结局串）。
fn judged_mount(recorded: &RecordedPublish, image: &MemoryPool, order: &str, faults: &RangesReadingBackAllZero) -> (Vec<String>, String) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, faults, &stream);
    if order == "reversed" {
        devices.reverse();
    }
    let _ = k1_take_judge_lines();
    let outcome = mount_writable(&recorded.pool.parameters, &mut devices);
    let judge = k1_take_judge_lines();
    let text = match outcome {
        Err(refusal) => {
            let name = format!("{refusal:?}");
            let short: String = name.chars().take_while(|c| c.is_alphanumeric()).collect();
            let inner = name.split(|c: char| !c.is_alphanumeric()).filter(|piece| !piece.is_empty()).nth(1).unwrap_or("");
            format!("refused:{short}:{inner}:writes={}", stream.operation_count())
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
            let r_in_ring = root_is_in_the_ring(&before_row_root, recorded.key);
            let red = if newly > 0 { red_invariants(&before_row_root) } else { Vec::new() };
            format!(
                "mounted:eff=({},{}):inst={}:new={newly}:total={}:gone={gone}:r_in_ring={r_in_ring}:red={}",
                mounted.output.effective_root.instance.0,
                mounted.output.effective_root.checkpoint_txg.0,
                mounted.output.instance.0,
                total.len(),
                joined(&red)
            )
        }
    };
    (judge, text)
}

/// 那一判记下的行压成一个字段：not_reached，或每一次的 pass/refuse 与四个落点读出的实例代号。
fn judge_field(lines: &[String]) -> String {
    if lines.is_empty() {
        return "not_reached".to_string();
    }
    lines
        .iter()
        .map(|line| {
            let pass = line.split_whitespace().find_map(|piece| piece.strip_prefix("pass=")).unwrap_or("?");
            let slots = line.split_whitespace().find_map(|piece| piece.strip_prefix("slots=")).unwrap_or("?");
            format!("{}[{slots}]", if pass == "true" { "pass" } else { "refuse" })
        })
        .collect::<Vec<_>>()
        .join("+")
}

fn first_mount(recorded: &RecordedPublish, crash_image: &MemoryPool) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, String) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(crash_image, &ranges(recorded, "root+units+rec"), &stream);
    let first = mount_writable(&recorded.pool.parameters, &mut devices).expect("M1 做得成");
    let summary = format!(
        "m1_instance={} m1_effective=({},{})",
        first.output.instance.0, first.output.effective_root.instance.0, first.output.effective_root.checkpoint_txg.0
    );
    let (writes, segments, _indexes) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
    (writes, segments, summary)
}

/// 盘面上某一槽读出什么（诊断，口径同产品那一判：先 16 KiB 解索引节点，再 32 KiB 解数据单元、打包记录单元）。
fn describe_slot(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> String {
    let node = image.devices[&device].read(singlefs_core::address::DeviceOffsetInBytes(offset), 16384);
    if let Ok(header) = singlefs_core::unit::parse_index_node(&node) {
        return format!("idx:{}", header.instance.0);
    }
    let unit = image.devices[&device].read(singlefs_core::address::DeviceOffsetInBytes(offset), 32768);
    if let Ok(header) = singlefs_core::unit::parse_data_unit(&unit) {
        return format!("data:{}", header.write_order.instance.0);
    }
    if let Ok(header) = singlefs_core::unit::parse_packed_unit(&unit) {
        return format!("packed:{}", header.write_order.instance.0);
    }
    if node.iter().all(|byte| *byte == 0) { "zero".to_string() } else { "other".to_string() }
}

fn pointer_slots(recorded: &RecordedPublish, image: &MemoryPool) -> String {
    let root = recorded.root();
    let mut out = Vec::new();
    for (name, pointer) in [("tt", &root.tree_table), ("mr", &root.mapping_root)] {
        for (index, location) in pointer.locations.iter().enumerate() {
            out.push(format!("{name}{index}:{}", describe_slot(image, location.device, location.slot.to_device_offset().0)));
        }
    }
    out.join(",")
}

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
    let row_root = writes.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("M1 写行根");
    println!(
        "name=k1_history history={history} r=({},{}) p=({},{}) r_records={} r_unit_writes={} {summary} writes={} segments={} row_root_write={row_root} state_count={state_count}",
        recorded.key.0 .0, recorded.key.1 .0, recorded.previous_key.0 .0, recorded.previous_key.1 .0,
        recorded.record_count(), recorded.unit_writes().len(), writes.len(), segments.len()
    );
    if mode == "count" {
        return;
    }
    let fault_argument_index = if mode == "survey" { 4 } else { 6 };
    let fault_sets: Vec<String> = arguments.get(fault_argument_index).map_or_else(|| vec!["none".to_string()], |value| value.split(',').map(str::to_string).collect());
    let fault_ranges: Vec<RangesReadingBackAllZero> = fault_sets.iter().map(|name| ranges(&recorded, name)).collect();
    if mode == "survey" {
        // M1 写行根之前的写全落（入库 `urJ` 用例那个盘面换几何）。
        let mut landed = crash_image.clone();
        landed.apply_writes(&writes[..row_root]);
        let state_red = red_invariants(&landed);
        let gone = units_of_r_gone_on(&recorded, &landed);
        for (fault_name, faults) in fault_sets.iter().zip(&fault_ranges) {
            for order in ["forward", "reversed"] {
                let (judge, outcome) = judged_mount(&recorded, &landed, order, faults);
                println!(
                    "name=k1_survey history={history} faults={fault_name} order={order} r_units_gone={gone} on_disk={} state_red={} judge={} outcome={outcome}",
                    pointer_slots(&recorded, &landed), joined(&state_red), judge_field(&judge)
                );
            }
        }
        return;
    }
    assert!(state_count <= 1_000_000, "一次全量不超过约 10⁶ 个状态");
    let tables = crash_state_enumeration_tables(&crash_image, &writes, &segments, &full_expansion);
    let facts = extract_flow_facts(&crash_image, &writes, &segments, &full_expansion);
    assert_eq!(tables.state_count, state_count);
    let start: u64 = arguments.get(4).map_or(0, |value| value.parse().expect("起点序号"));
    let end: u64 = arguments.get(5).map_or(state_count, |value| value.parse().expect("终点序号")).min(state_count);
    let mut not_reached: BTreeMap<(String, String, String), u64> = BTreeMap::new();
    for ordinal in start..end {
        let persisted = facts.persisted_writes_of_state(ordinal);
        let image = image_with(&crash_image, &tables.enumerated_writes, &persisted);
        let segment = facts.segment_of_state(ordinal);
        let state_red = red_invariants(&image);
        let in_state = units_of_r_overwritten_on(&recorded, &image).len();
        let gone = units_of_r_gone_on(&recorded, &image);
        let r_in_ring = root_is_in_the_ring(&image, recorded.key);
        for (fault_name, faults) in fault_sets.iter().zip(&fault_ranges) {
            for order in ["forward", "reversed"] {
                let (judge, outcome) = judged_mount(&recorded, &image, order, faults);
                if judge.is_empty() {
                    let short: String = outcome.split(':').take(2).collect::<Vec<_>>().join(":");
                    *not_reached.entry((fault_name.clone(), order.to_string(), short)).or_default() += 1;
                    continue;
                }
                println!(
                    "name=k1_state history={history} faults={fault_name} ordinal={ordinal} segment={segment} r_in_ring={r_in_ring} r_copies_overwritten={in_state} r_units_gone={gone} on_disk={} state_red={} order={order} judge={} outcome={outcome}",
                    pointer_slots(&recorded, &image), joined(&state_red), judge_field(&judge)
                );
            }
        }
    }
    for ((fault_name, order, outcome), count) in not_reached {
        println!("name=k1_not_reached history={history} faults={fault_name} order={order} start={start} end={end} outcome={outcome} states={count}");
    }
}
