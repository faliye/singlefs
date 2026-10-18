//! C834 第三轮攻方原型（只攻指针槽读法与映射根变体，正文「三、格」T1、T2）。由第二轮攻方原型改来。
//!
//! 历史：R 照入库搭建 `common_unwitnessed_publish` 录（384 槽小盘、第一个文件、覆盖写 k 次，R 是一次覆盖写或顺序写三个数据单元），
//! 崩在 R 的根槽之后、系统配置轮换之前。M1 是入库 `urJ` 那一形的可写挂载：R 的根槽、R 写出的全部单元、R 的全部记录读回全零，
//! M1 看不见 R、退回 P、取号、写行。M1 的写流按崩溃点重放的枚举域（`crash_state_enumeration_tables` + `extract_flow_facts`，
//! 每一段 `full_expansion`，原地覆写取三态）逐个崩溃状态展开；每个状态上做被判的那次挂载 M2，两种交盘次序、几种读故障各一次。
//! 被判的那次挂载先照 today 跑：没走到那一判，结局对每个候选相同；走到了，按记下的凭据算每个候选拒还是放过，再强制走另一支跑一遍。
//!
//! M2 的读故障按「R 的哪一份」点名（每块盘一份分开点）：tt0 / tt1（R 的树表在盘表第一、第二条位置条目上那一份），
//! mr0 / mr1（R 的映射根同上），rec（R 的全部记录，两块盘）；`none` 不藏。几样用 `+` 连。
//!
//! 用法：c834-r3-attack-model <overwrite|seq3> <k> survey
//!       c834-r3-attack-model <overwrite|seq3> <k> count
//!       c834-r3-attack-model <overwrite|seq3> <k> run <起点序号> <终点序号> <读故障组,读故障组,…>
#[path = "../../tree-candidates/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    fixed_geometry_of_the_384_slot_small_devices, record_one_publish_after_overwrites,
    recorded_devices_reading_back_all_zero_on, root_is_in_the_ring, units_of_r_overwritten_on,
    PublishRecordedAsR, RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, crash_state_enumeration_tables,
    full_expansion,
};
use singlefs_checker_tier::crash_facts::extract_flow_facts;
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{c834_force, c834_set_candidate, c834_take_judge_lines, mount_writable};
use singlefs_core::recovery::{choose_system_configuration, scan_journal};
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use std::collections::BTreeSet;

pub const CANDIDATES: [&str; 6] = ["today", "pointer_per_kind", "mapping_root_only", "pointer_pair", "mapping_pair", "always_refuse"];

pub fn every_record_of_r(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    (1..=recorded.record_count())
        .flat_map(|ordinal| recorded.record_writes_of_the_record(ordinal))
        .collect()
}

/// R 在某个指针某一条位置条目上那一份单元写（R 自己写了那个单元才有）。
fn copy_of_r_at<'recorded>(recorded: &'recorded RecordedPublish, pointer: &singlefs_core::pointer::NodePointer, location_index: usize) -> Vec<&'recorded RetainedWrite> {
    let device = pointer.locations[location_index].device;
    recorded
        .unit_writes_at(pointer)
        .into_iter()
        .filter(|write| write.device == device)
        .collect()
}

/// 读故障组：`none`，或 tt0、tt1、mr0、mr1、rec、root、units 用 `+` 连。
pub fn ranges(recorded: &RecordedPublish, name: &str) -> RangesReadingBackAllZero {
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

pub fn red_invariants(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| name)
        .collect()
}

fn joined(red: &[&str]) -> String {
    if red.is_empty() { "-".to_string() } else { red.join("+") }
}

/// R 写出的单元里两份都已不是 R 写的那几个（单元整个没了）。
pub fn units_of_r_gone_on(recorded: &RecordedPublish, image: &MemoryPool) -> usize {
    let overwritten: BTreeSet<(DeviceIdentity, u64)> = units_of_r_overwritten_on(recorded, image).into_iter().collect();
    recorded
        .unit_writes_grouped_by_unit()
        .iter()
        .filter(|group| group.iter().all(|copy| overwritten.contains(&(copy.device, copy.offset.0))))
        .count()
}

fn image_with(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let chosen: Vec<RetainedWrite> = writes
        .iter()
        .zip(persisted)
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|(write, _)| write.clone())
        .collect();
    let mut image = base.clone();
    image.apply_writes(&chosen);
    image
}

/// 盘面上某一槽读出什么（与补丁里那一读法同口径：先按 16 KiB 解索引节点，再按 32 KiB 解数据单元与打包记录单元；都不是时看是不是前一槽起那个 32 KiB 单元的后半截）。
pub fn describe_slot(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> String {
    let slot = singlefs_format::SLOT_BYTES;
    let node = image.devices[&device].read(singlefs_core::address::DeviceOffsetInBytes(offset), usize::try_from(slot).expect("16384"));
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
    if node.iter().all(|byte| *byte == 0) {
        return "zero".to_string();
    }
    let previous = image.devices[&device].read(singlefs_core::address::DeviceOffsetInBytes(offset - slot), 32768);
    if let Ok(header) = singlefs_core::unit::parse_data_unit(&previous) {
        return format!("second_half_of_data:{}", header.write_order.instance.0);
    }
    if let Ok(header) = singlefs_core::unit::parse_packed_unit(&previous) {
        return format!("second_half_of_packed:{}", header.write_order.instance.0);
    }
    "other".to_string()
}

/// 一次可写挂载：交回（那一判记下的行，结局串）。结局串：refused:<拒因>:writes=N，或
/// mounted:eff=(i,t):inst=N:new=<写行根落盘之前新写掉的 R 单元份数>:total=<那一刻一共被写掉的份数>:gone=<那一刻两份都没了的 R 单元数>:r_in_ring=<bool>:red=<那一刻 checker 红的>。
fn judged_mount(recorded: &RecordedPublish, image: &MemoryPool, order: &str, faults: &RangesReadingBackAllZero) -> (Vec<String>, String) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, faults, &stream);
    if order == "reversed" {
        devices.reverse();
    }
    let _ = c834_take_judge_lines();
    let outcome = mount_writable(&recorded.pool.parameters, &mut devices);
    let judge = c834_take_judge_lines();
    let text = match outcome {
        Err(refusal) => {
            let name = format!("{refusal:?}");
            let short: String = name.chars().take_while(|c| c.is_alphanumeric()).collect();
            let inner = name
                .split(|c: char| !c.is_alphanumeric())
                .filter(|piece| !piece.is_empty())
                .nth(1)
                .unwrap_or("");
            format!("refused:{short}:{inner}:writes={}", stream.operation_count())
        }
        Ok(mounted) => {
            let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
                &stream.retained_operations(),
                &fixed_geometry_of_the_384_slot_small_devices(),
            );
            let row_root = writes
                .iter()
                .position(|write| write.kind == StepKind::RootRecordFua)
                .expect("写行那次发布有根槽写");
            let mut before_row_root = image.clone();
            before_row_root.apply_writes(&writes[..row_root]);
            let already: BTreeSet<(DeviceIdentity, u64)> = units_of_r_overwritten_on(recorded, image).into_iter().collect();
            let total: BTreeSet<(DeviceIdentity, u64)> =
                units_of_r_overwritten_on(recorded, &before_row_root).into_iter().collect();
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

fn field<'line>(line: &'line str, name: &str) -> &'line str {
    line.split_whitespace()
        .find_map(|piece| piece.strip_prefix(&format!("{name}=")))
        .unwrap_or("?")
}

/// 照那一判记下的凭据算每个候选判拒（true）还是放过。
fn refuses(candidate: &str, line: &str, r_instance: u32) -> bool {
    let chosen: u32 = field(line, "chosen").parse().expect("chosen");
    match candidate {
        "today" => chosen <= r_instance,
        "pointer_per_kind" => field(line, "pointer_per_kind") != "true",
        "mapping_root_only" => field(line, "mapping_root_only") != "true",
        "pointer_pair" => field(line, "pointer_pair") != "true",
        "mapping_pair" => field(line, "mapping_pair") != "true",
        "always_refuse" => true,
        other => panic!("{other}"),
    }
}

/// 被判的那次挂载对每个候选的（判，结局）；`creds` 是那一判记下的凭据（没走到是 not_reached）。
fn every_candidate(recorded: &RecordedPublish, image: &MemoryPool, order: &str, faults: &RangesReadingBackAllZero) -> (String, String) {
    let r_instance = recorded.key.0 .0;
    c834_force(None);
    c834_set_candidate("today");
    let (judge, today) = judged_mount(recorded, image, order, faults);
    if judge.is_empty() {
        return ("not_reached".to_string(), format!("all:-:{today}"));
    }
    let creds: String = {
        let line = &judge[0];
        ["chosen", "pointer_per_kind", "mapping_root_only", "pointer_pair", "mapping_pair", "slots"]
            .iter()
            .map(|name| format!("{name}:{}", field(line, name)))
            .collect::<Vec<_>>()
            .join(",")
    };
    let mut cells = Vec::new();
    if judge.len() == 1 {
        let today_refuses = field(&judge[0], "verdict") == "refuse";
        c834_force(Some(!today_refuses));
        let (other_judge, other) = judged_mount(recorded, image, order, faults);
        c834_force(None);
        if other_judge.len() == 1 {
            for candidate in CANDIDATES {
                let refuse = refuses(candidate, &judge[0], r_instance);
                let outcome = if refuse == today_refuses { &today } else { &other };
                cells.push(format!("{candidate}:{}:{outcome}", if refuse { "refuse" } else { "pass" }));
            }
            return (format!("once;{creds}"), cells.join(";"));
        }
    }
    for candidate in CANDIDATES {
        c834_set_candidate(candidate);
        let (lines, outcome) = judged_mount(recorded, image, order, faults);
        let verdicts: Vec<&str> = lines.iter().map(|line| field(line, "verdict")).collect();
        cells.push(format!("{candidate}:{}:{outcome}", verdicts.join("+")));
    }
    c834_set_candidate("today");
    (format!("many{};{creds}", judge.len()), cells.join(";"))
}

/// M1：`crash_image` 上 R 的根槽、R 写出的全部单元、R 的全部记录读回全零（入库 `urJ` 那一形的 M1）；交回 M1 的写、段与一行摘要。
fn first_mount(recorded: &RecordedPublish, crash_image: &MemoryPool) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, String) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(crash_image, &ranges(recorded, "root+units+rec"), &stream);
    let first = mount_writable(&recorded.pool.parameters, &mut devices).expect("M1 做得成");
    let summary = format!(
        "m1_instance={} m1_effective=({},{})",
        first.output.instance.0,
        first.output.effective_root.instance.0,
        first.output.effective_root.checkpoint_txg.0
    );
    let (writes, segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    (writes, segments, summary)
}

/// R 写出的每个单元：（角色，盘，偏移）；角色 tt / mr 是树表与映射根，别的写 unit。
fn roles_of_r_units(recorded: &RecordedPublish) -> Vec<(String, DeviceIdentity, u64)> {
    let root = recorded.root();
    recorded
        .unit_writes()
        .into_iter()
        .map(|write| {
            let at = |pointer: &singlefs_core::pointer::NodePointer| {
                pointer.locations.iter().position(|location| location.device == write.device && location.slot.to_device_offset() == write.offset)
            };
            let role = match (at(&root.tree_table), at(&root.mapping_root)) {
                (Some(index), _) => format!("tt{index}"),
                (_, Some(index)) => format!("mr{index}"),
                _ => "unit".to_string(),
            };
            (role, write.device, write.offset.0)
        })
        .collect()
}

mod prefix;

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
    // 第二个参数是数字：k 次覆盖写（入库搭建原样）；是字母串：前缀动作（`prefix.rs`）。
    let (recorded, history) = match prefix_argument.parse::<u64>() {
        Ok(overwrites_before) => (record_one_publish_after_overwrites(overwrites_before, publish), format!("urj_{kind}_k{overwrites_before}")),
        Err(_) => match prefix::record_after_prefix(&prefix_argument, publish) {
            Ok(recorded) => (recorded, format!("urj_{kind}_p{prefix_argument}")),
            Err(refusal) => {
                println!("name=c834_history history=urj_{kind}_p{prefix_argument} refused={refusal}");
                return;
            }
        },
    };
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let (writes, segments, summary) = first_mount(&recorded, &crash_image);
    let state_count = crash_state_count_with_torn_in_place_overwrites(&crash_image, &writes, &segments, &full_expansion);
    println!(
        "name=c834_history history={history} r=({},{}) p=({},{}) r_records={} r_unit_writes={} {summary} writes={} segments={} state_count={state_count}",
        recorded.key.0 .0,
        recorded.key.1 .0,
        recorded.previous_key.0 .0,
        recorded.previous_key.1 .0,
        recorded.record_count(),
        recorded.unit_writes().len(),
        writes.len(),
        segments.len()
    );
    for (index, segment) in segments.iter().enumerate() {
        let kinds: Vec<String> = segment
            .iter()
            .map(|write_index| format!("{:?}@d{}@{}", writes[*write_index].kind, writes[*write_index].device.0, writes[*write_index].offset.0 / singlefs_format::SLOT_BYTES))
            .collect();
        println!("name=c834_segment history={history} segment={index} writes={}", kinds.join(","));
    }
    if mode == "survey" {
        let row_root = writes.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("M1 写行根");
        let mut landed = crash_image.clone();
        landed.apply_writes(&writes[..row_root]);
        for (role, device, offset) in roles_of_r_units(&recorded) {
            println!(
                "name=c834_r_unit history={history} role={role} device={} slot={} r_wrote={} after_m1={}",
                device.0,
                offset / singlefs_format::SLOT_BYTES,
                describe_slot(&crash_image, device, offset),
                describe_slot(&landed, device, offset)
            );
        }
        return;
    }
    if mode == "count" {
        return;
    }
    assert!(state_count <= 1_000_000, "一次全量不超过约 10⁶ 个状态");
    let tables = crash_state_enumeration_tables(&crash_image, &writes, &segments, &full_expansion);
    let facts = extract_flow_facts(&crash_image, &writes, &segments, &full_expansion);
    assert_eq!(tables.state_count, state_count);
    let start: u64 = arguments.get(4).map_or(0, |value| value.parse().expect("起点序号"));
    let end: u64 = arguments.get(5).map_or(state_count, |value| value.parse().expect("终点序号")).min(state_count);
    let fault_sets: Vec<String> = arguments.get(6).map_or_else(|| vec!["rec".to_string()], |value| value.split(',').map(str::to_string).collect());
    let fault_ranges: Vec<RangesReadingBackAllZero> = fault_sets.iter().map(|name| ranges(&recorded, name)).collect();
    let roles = roles_of_r_units(&recorded);
    for ordinal in start..end {
        let persisted = facts.persisted_writes_of_state(ordinal);
        let image = image_with(&crash_image, &tables.enumerated_writes, &persisted);
        let segment = facts.segment_of_state(ordinal);
        let state_red = red_invariants(&image);
        let in_state = units_of_r_overwritten_on(&recorded, &image).len();
        let gone = units_of_r_gone_on(&recorded, &image);
        let r_in_ring = root_is_in_the_ring(&image, recorded.key);
        let configuration = choose_system_configuration(&image).expect("择得到系统配置");
        let mut later: Vec<u32> = scan_journal(&image, &configuration)
            .values()
            .filter(|record| record.instance > InstanceGeneration(recorded.key.0 .0))
            .map(|record| record.instance.0)
            .collect();
        later.sort_unstable();
        let on_disk: Vec<String> = roles
            .iter()
            .filter(|(role, _, _)| role != "unit")
            .map(|(role, device, offset)| format!("{role}:{}", describe_slot(&image, *device, *offset)))
            .collect();
        for (fault_name, faults) in fault_sets.iter().zip(&fault_ranges) {
            for order in ["forward", "reversed"] {
                let (creds, cells) = every_candidate(&recorded, &image, order, faults);
                println!(
                    "name=c834_state history={history} faults={fault_name} ordinal={ordinal} segment={segment} later_records=[{}] r_in_ring={r_in_ring} r_copies_overwritten={in_state} r_units_gone={gone} on_disk={} state_red={} order={order} creds={creds} cands={cells}",
                    later.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
                    on_disk.join(","),
                    joined(&state_red)
                );
            }
        }
    }
}
