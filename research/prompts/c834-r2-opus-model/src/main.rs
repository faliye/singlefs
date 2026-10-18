//! C834 第二轮攻方原型（只攻 K6 与指针槽读法）。几段历史各录一次「取号之后写」的挂载写流，按崩溃点重放的枚举域
//! （`crash_state_enumeration_tables` + `extract_flow_facts`，每一段 `full_expansion`，原地覆写取三态）逐个崩溃状态：
//! 跑池级 checker；再在这个状态上做下一次可写挂载（下称「被判的那次挂载」），两种交盘次序各一次。
//! 被判的那次挂载先照 today 跑一遍：没走到那一判，结局对每个候选都一样；走到了，那一判记下全部凭据，按凭据算出每个候选判拒还是放过，
//! 再强制走另一支跑一遍（那一判之前的路径对每个候选逐字相同，之后只看拒 / 放过）；那一判走到不止一次的，逐个候选各跑一遍。
//! 每个（状态，次序）一行，`cands=` 里逐个候选写（判，结局）。
//!
//! 历史：
//!   clean / tree_table_and_record / root_units_and_record / acquisition_rolled_back  第一轮攻方的甲至戊（同一造法）
//!   three_mount  M0 无读故障、择 R、取号 2、写行发布的单元与记录落了、根没落（第一轮正推 later_mount 那一形）；
//!                M2 在这张盘面上、R 的记录、树表、映射根读回全零，那一判放过（today、K3、K6 都放过），取号 3、写行……；
//!                枚举 M2 写流的崩溃状态，被判的那次挂载是 M3
//!   three_mount_tail  同上，缩短：M2 取号与写行发布的单元写都已落（并进基镜像），从 M2 的第一条记录写起往后枚举
//! 被判的那次挂载的读故障（第 5 个参数）：y2、tree_and_mapping、none、record_only，以及只给 three_mount 一族的 y2_m0_record
//! （y2 再加 M0 写行发布的那条记录）。
//! 模式 `nine`：第一轮正推的九段历史（同一造法），每段两种次序、每个候选各挂一次。
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
use singlefs_core::recovery::{choose_system_configuration, scan_journal, verified_system_configuration_slots};
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use std::collections::BTreeSet;

const OVERWRITES_BEFORE_R: u64 = 5;
const CANDIDATES: [&str; 6] = ["today", "k3", "k6", "pointer_r1", "pointer_per_kind", "always_refuse"];

fn every_record_of_r(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    (1..=recorded.record_count())
        .flat_map(|ordinal| recorded.record_writes_of_the_record(ordinal))
        .collect()
}

/// 读回全零的段：`y2` R 的树表、映射根、全部记录；`tree_and_mapping` 只藏树表与映射根；`record_only` 只藏 R 的记录；`none` 不藏；
/// `root_units_and_record` R 的根槽、R 写出的全部单元、R 的第 1 条记录（入库 `urJ` 那一形的 M1）；`tree_table_and_record` R 的树表与 R 的记录。
fn ranges(recorded: &RecordedPublish, name: &str) -> RangesReadingBackAllZero {
    let root_of_r = recorded.root();
    match name {
        "none" => RangesReadingBackAllZero::default(),
        "y2" => RangesReadingBackAllZero::default()
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.tree_table))
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.mapping_root))
            .with_the_writes(&every_record_of_r(recorded)),
        "tree_and_mapping" => RangesReadingBackAllZero::default()
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.tree_table))
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.mapping_root)),
        "record_only" => RangesReadingBackAllZero::default().with_the_writes(&every_record_of_r(recorded)),
        "tree_table_and_record" => RangesReadingBackAllZero::default()
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.tree_table))
            .with_the_writes(&recorded.record_writes_of_the_record(1)),
        "root_units_and_record" => RangesReadingBackAllZero::default()
            .with_the_writes(&[recorded.root_slot_write()])
            .with_the_writes(&recorded.unit_writes())
            .with_the_writes(&recorded.record_writes_of_the_record(1)),
        other => panic!("认不出读故障 {other}"),
    }
}

fn instances_of_each_device(image: &MemoryPool) -> Vec<(u32, Vec<u32>)> {
    let configuration = choose_system_configuration(image).expect("择得到系统配置");
    let spacing = u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    image
        .devices
        .keys()
        .map(|device| {
            let slots = verified_system_configuration_slots(
                image,
                *device,
                spacing,
                &configuration.immutable.filesystem_identifier,
            )
            .iter()
            .map(|configuration| configuration.quantities.journal_instance.0)
            .collect();
            (device.0, slots)
        })
        .collect()
}

fn later_records(image: &MemoryPool, r_instance: InstanceGeneration) -> Vec<u32> {
    let configuration = choose_system_configuration(image).expect("择得到系统配置");
    let mut instances: Vec<u32> = scan_journal(image, &configuration)
        .values()
        .filter(|record| record.instance > r_instance)
        .map(|record| record.instance.0)
        .collect();
    instances.sort_unstable();
    instances
}

fn red_invariants(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| name)
        .collect()
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

/// 一次可写挂载：交回（那一判记下的行，结局串）。结局串：refused:<拒因>:writes=N，或
/// mounted:eff=(i,t):inst=N:new=<这次挂载写行根落盘之前新写掉的 R 单元份数>:total=<那一刻一共被写掉的>:r_in_ring=<bool>:red=<那一刻 checker 红的>。
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
            let r_in_ring = root_is_in_the_ring(&before_row_root, recorded.key);
            let red = if newly > 0 { red_invariants(&before_row_root) } else { Vec::new() };
            format!(
                "mounted:eff=({},{}):inst={}:new={newly}:total={}:r_in_ring={r_in_ring}:red={}",
                mounted.output.effective_root.instance.0,
                mounted.output.effective_root.checkpoint_txg.0,
                mounted.output.instance.0,
                total.len(),
                if red.is_empty() { "-".to_string() } else { red.join("+") }
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
    let later = field(line, "later_root_or_record") == "true";
    let mapping = field(line, "mapping_root_newer") == "true";
    match candidate {
        "today" => chosen <= r_instance,
        "k3" => !later,
        "k6" => !(later || mapping),
        "pointer_r1" => field(line, "pointer_r1") != "true",
        "pointer_per_kind" => field(line, "pointer_per_kind") != "true",
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
        ["chosen", "later_root_or_record", "mapping_root_newer", "pointer_r1", "pointer_per_kind", "slots"]
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
    // 那一判走到不止一次：逐个候选各跑一遍。
    for candidate in CANDIDATES {
        c834_set_candidate(candidate);
        let (lines, outcome) = judged_mount(recorded, image, order, faults);
        let verdicts: Vec<&str> = lines.iter().map(|line| field(line, "verdict")).collect();
        cells.push(format!("{candidate}:{}:{outcome}", verdicts.join("+")));
    }
    c834_set_candidate("today");
    (format!("many{};{creds}", judge.len()), cells.join(";"))
}

/// 取新实例的那次挂载：`image` 上 `faults` 读回全零，`force` 给那一判；交回它的写（`Err` 是拒因）。
fn writes_of_a_mount(recorded: &RecordedPublish, image: &MemoryPool, faults: &RangesReadingBackAllZero, force: Option<bool>) -> Result<(Vec<RetainedWrite>, Vec<Vec<usize>>, Vec<String>, String), String> {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, faults, &stream);
    c834_force(force);
    let _ = c834_take_judge_lines();
    let outcome = mount_writable(&recorded.pool.parameters, &mut devices);
    let judge = c834_take_judge_lines();
    c834_force(None);
    let mounted = outcome.map_err(|refusal| format!("{refusal:?}").chars().take(200).collect::<String>())?;
    let (writes, segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let summary = format!(
        "instance={} effective=({},{})",
        mounted.output.instance.0,
        mounted.output.effective_root.instance.0,
        mounted.output.effective_root.checkpoint_txg.0
    );
    Ok((writes, segments, judge, summary))
}

mod first_round_histories;
mod nine;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let history = arguments.get(1).cloned().unwrap_or_else(|| "clean".to_string());
    let mode = arguments.get(2).cloned().unwrap_or_else(|| "count".to_string());
    let recorded = record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    if history == "nine" {
        nine::run(&recorded);
        return;
    }
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut m0_record_writes: Vec<RetainedWrite> = Vec::new();
    let (base, writes, segments, summary) = if history == "three_mount" || history == "three_mount_tail" {
        let (m0, _segments, _judge, m0_summary) =
            writes_of_a_mount(&recorded, &crash_image, &ranges(&recorded, "none"), None).expect("M0 做得成");
        let row_root = m0.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("M0 写行根");
        let mut image_a = crash_image.clone();
        image_a.apply_writes(&m0[..row_root]);
        m0_record_writes = m0[..row_root].iter().filter(|write| write.kind == StepKind::JournalRecord).cloned().collect();
        let (m2, m2_segments, m2_judge, m2_summary) =
            writes_of_a_mount(&recorded, &image_a, &ranges(&recorded, "y2"), Some(false)).expect("M2 放过之后做得成");
        let a_red = red_invariants(&image_a);
        let summary = format!(
            "m0=({m0_summary}) m0_writes_before_row_root={row_root} image_a_red={} image_a_r_units_overwritten={} m2=({m2_summary}) m2_judge_calls={} m2_judge={}",
            if a_red.is_empty() { "-".to_string() } else { a_red.join("+") },
            units_of_r_overwritten_on(&recorded, &image_a).len(),
            m2_judge.len(),
            m2_judge.first().map_or("-".to_string(), |line| line.replace(' ', "|"))
        );
        if history == "three_mount" {
            (image_a, m2, m2_segments, summary)
        } else {
            // 缩短的历史：M2 取号与写行发布的单元写都已落（并进基镜像），从 M2 的第一条记录写起往后枚举。
            let first_record = m2.iter().position(|write| write.kind == StepKind::JournalRecord).expect("M2 有记录写");
            let starting_segment = m2_segments
                .iter()
                .position(|segment| segment.first() == Some(&first_record))
                .expect("M2 的第一条记录写开一段");
            let mut tail_base = image_a.clone();
            tail_base.apply_writes(&m2[..first_record]);
            let tail_segments: Vec<Vec<usize>> = m2_segments[starting_segment..]
                .iter()
                .map(|segment| segment.iter().map(|index| index - first_record).collect())
                .collect();
            let summary = format!("{summary} tail_from_write={first_record} tail_from_segment={starting_segment}");
            (tail_base, m2[first_record..].to_vec(), tail_segments, summary)
        }
    } else {
        let (writes, segments, summary) = first_round_histories::first_mount(&recorded, &crash_image, &history);
        (crash_image.clone(), writes, segments, summary)
    };
    let state_count = crash_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    println!(
        "name=c834_history history={history} r=({},{}) p=({},{}) {summary} writes={} segments={} state_count={state_count}",
        recorded.key.0 .0,
        recorded.key.1 .0,
        recorded.previous_key.0 .0,
        recorded.previous_key.1 .0,
        writes.len(),
        segments.len()
    );
    for (index, segment) in segments.iter().enumerate() {
        let kinds: Vec<String> = segment
            .iter()
            .map(|write_index| format!("{:?}@d{}", writes[*write_index].kind, writes[*write_index].device.0))
            .collect();
        println!("name=c834_segment history={history} segment={index} writes={}", kinds.join(","));
    }
    if mode == "count" {
        return;
    }
    assert!(state_count <= 1_000_000, "一次全量不超过约 10⁶ 个状态");
    let tables = crash_state_enumeration_tables(&base, &writes, &segments, &full_expansion);
    let facts = extract_flow_facts(&base, &writes, &segments, &full_expansion);
    assert_eq!(tables.state_count, state_count);
    let start: u64 = arguments.get(3).map_or(0, |value| value.parse().expect("起点序号"));
    let end: u64 = arguments.get(4).map_or(state_count, |value| value.parse().expect("终点序号")).min(state_count);
    let fault_name = arguments.get(5).cloned().unwrap_or_else(|| "y2".to_string());
    let faults = match fault_name.as_str() {
        // R 的树表、映射根、全部记录（同 y2），再加 M0 那次写行发布的记录（每块盘一份）。
        "y2_m0_record" => {
            let m0_records: Vec<&RetainedWrite> = m0_record_writes.iter().collect();
            assert!(!m0_records.is_empty(), "y2_m0_record 只给 three_mount 一族用");
            ranges(&recorded, "y2").with_the_writes(&m0_records)
        }
        other => ranges(&recorded, other),
    };
    for ordinal in start..end {
        let persisted = facts.persisted_writes_of_state(ordinal);
        let image = image_with(&base, &tables.enumerated_writes, &persisted);
        let segment = facts.segment_of_state(ordinal);
        let state_red = red_invariants(&image);
        let in_state = units_of_r_overwritten_on(&recorded, &image).len();
        let per_device = instances_of_each_device(&image);
        let later = later_records(&image, recorded.key.0);
        let r_in_ring = root_is_in_the_ring(&image, recorded.key);
        for order in ["forward", "reversed"] {
            let (creds, cells) = every_candidate(&recorded, &image, order, &faults);
            let per_device_text: String = per_device
                .iter()
                .map(|(device, slots)| format!("{device}:{}", slots.iter().map(u32::to_string).collect::<Vec<_>>().join("/")))
                .collect::<Vec<_>>()
                .join(",");
            let later_text: String = later.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
            println!(
                "name=c834_state history={history} faults={fault_name} ordinal={ordinal} segment={segment} per_device={per_device_text} later_records=[{later_text}] r_in_ring={r_in_ring} r_units_overwritten_in_state={in_state} state_red={} order={order} creds={creds} cands={cells}",
                if state_red.is_empty() { "-".to_string() } else { state_red.join("+") }
            );
        }
    }
}
