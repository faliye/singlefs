//! C834 第一轮攻方原型：录一次发布 R（崩在 R 的根槽之后、轮换之前），再录一次取新实例的可写挂载 M1 的写流，
//! 按崩溃点重放的枚举域（`crash_state_enumeration_tables` + `extract_flow_facts`，每一段 `full_expansion`，原地覆写取三态）
//! 逐个崩溃状态：跑池级 checker；再在这个状态上做下一次可写挂载 M2（R 的树表、中央映射树根与 R 的记录读回全零），
//! 两种交盘次序各一次，按命令行给的候选（`C834_CANDIDATE`）判，记那一判的结局、M2 写行那次发布的根落盘之前写掉了 R 的几份单元、
//! 那一刻的池级 checker 判红的不变量。一行一个（状态，次序）。
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
use singlefs_core::mount::{c834_take_judge_lines, mount_writable};
use singlefs_core::recovery::{scan_journal, verified_system_configuration_slots, choose_system_configuration};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const OVERWRITES_BEFORE_R: u64 = 5;

/// M1 那次挂载里读回全零的段（前缀）：`clean` 一段都不藏；`tree_table_and_record` 藏 R 的树表与 R 的记录（映射现算得出，M1 隔离 R 的单元、
/// 把写行记录写进 R 的记录槽）。
fn ranges_of_the_first_mount(recorded: &RecordedPublish, prefix: &str) -> RangesReadingBackAllZero {
    let root_of_r = recorded.root();
    match prefix {
        "clean" => RangesReadingBackAllZero::default(),
        "tree_table_and_record" => RangesReadingBackAllZero::default()
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.tree_table))
            .with_the_writes(&recorded.record_writes_of_the_record(1)),
        // 入库 `urJ` 那一形的 M1：R 的根槽、R 写出的全部单元与 R 的记录读回全零，M1 看不见 R、写掉它的单元与记录槽。
        "root_units_and_record" => RangesReadingBackAllZero::default()
            .with_the_writes(&[recorded.root_slot_write()])
            .with_the_writes(&recorded.unit_writes())
            .with_the_writes(&recorded.record_writes_of_the_record(1)),
        other => panic!("认不出前缀 {other}"),
    }
}

/// M2 那次挂载里读回全零的段：`y2` 是 R 的树表、中央映射树根与 R 的记录（攻方 Y2 那一形的三样）；`tree_and_mapping` 只藏树表与映射根
/// （R 的记录槽已被 M1 的记录占了的那几个状态用：藏记录槽就把 M1 的记录也藏了）；`none` 一段都不藏（盘上原样）。
fn ranges_of_the_second_mount(recorded: &RecordedPublish, faults: &str) -> RangesReadingBackAllZero {
    let root_of_r = recorded.root();
    match faults {
        "y2" => RangesReadingBackAllZero::default()
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.tree_table))
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.mapping_root))
            .with_the_writes(&recorded.record_writes_of_the_record(1)),
        "tree_and_mapping" => RangesReadingBackAllZero::default()
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.tree_table))
            .with_the_writes(&recorded.unit_writes_at(&root_of_r.mapping_root)),
        "none" => RangesReadingBackAllZero::default(),
        other => panic!("认不出 M2 的读故障 {other}"),
    }
}

fn instances_of_each_device(image: &MemoryPool) -> Vec<(u32, u32)> {
    let configuration = choose_system_configuration(image).expect("择得到系统配置");
    let spacing = u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    image
        .devices
        .keys()
        .map(|device| {
            let instance = verified_system_configuration_slots(
                image,
                *device,
                spacing,
                &configuration.immutable.filesystem_identifier,
            )
            .iter()
            .map(|configuration| configuration.quantities.journal_instance.0)
            .max()
            .unwrap_or(0);
            (device.0, instance)
        })
        .collect()
}

/// 这张镜像上 journal 环里实例代号大于 R 的记录有几条。
fn later_records(image: &MemoryPool, r_instance: InstanceGeneration) -> usize {
    let configuration = choose_system_configuration(image).expect("择得到系统配置");
    scan_journal(image, &configuration)
        .values()
        .filter(|record| record.instance > r_instance)
        .count()
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

/// M2：`image` 上 R 的树表、映射根与记录读回全零，按 `order`（`forward` 盘表按身份升序、`reversed` 倒过来）交盘做一次可写挂载。
fn second_mount(recorded: &RecordedPublish, image: &MemoryPool, order: &str, faults: &str) -> String {
    let stream = SharedStream::retaining_contents();
    let mut devices =
        recorded_devices_reading_back_all_zero_on(image, &ranges_of_the_second_mount(recorded, faults), &stream);
    if order == "reversed" {
        devices.reverse();
    }
    let _ = c834_take_judge_lines();
    let outcome = mount_writable(&recorded.pool.parameters, &mut devices);
    let judge: Vec<String> = c834_take_judge_lines()
        .into_iter()
        .map(|line| {
            line.split_whitespace()
                .filter(|field| ["verdict=", "chosen=", "k6=", "pointer_", "mapping_newer", "diag="].iter().any(|prefix| field.starts_with(prefix)))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect();
    let judge = if judge.is_empty() { "not_reached".to_string() } else { judge.join(";") };
    match outcome {
        Err(refusal) => {
            let name = format!("{refusal:?}");
            let short: String = name.chars().take_while(|c| c.is_alphanumeric()).collect();
            let inner = name
                .split(|c: char| !c.is_alphanumeric())
                .filter(|piece| !piece.is_empty())
                .nth(1)
                .unwrap_or("");
            format!(
                "judge={judge} m2=refused:{short}:{inner} writes={} r_units_overwritten_before_row_root=0 red_before_row_root=-",
                stream.operation_count()
            )
        }
        Ok(mounted) => {
            let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
                &stream.retained_operations(),
                &fixed_geometry_of_the_384_slot_small_devices(),
            );
            let row_root = writes
                .iter()
                .position(|write| write.kind == StepKind::RootRecordFua)
                .expect("M2 写行那次发布有根槽写");
            let mut before_row_root = image.clone();
            before_row_root.apply_writes(&writes[..row_root]);
            let overwritten = units_of_r_overwritten_on(recorded, &before_row_root).len();
            let r_in_ring = root_is_in_the_ring(&before_row_root, recorded.key);
            let red = red_invariants(&before_row_root);
            format!(
                "judge={judge} m2=mounted:instance={}:effective=({},{}) writes={} r_units_overwritten_before_row_root={overwritten} r_root_in_ring_before_row_root={r_in_ring} red_before_row_root={}",
                mounted.output.instance.0,
                mounted.output.effective_root.instance.0,
                mounted.output.effective_root.checkpoint_txg.0,
                writes.len(),
                if red.is_empty() { "-".to_string() } else { red.join("+") }
            )
        }
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let prefix = arguments.get(1).cloned().unwrap_or_else(|| "clean".to_string());
    let mode = arguments.get(2).cloned().unwrap_or_else(|| "count".to_string());
    let candidate = std::env::var("C834_CANDIDATE").unwrap_or_else(|_| "today".to_string());
    let recorded = record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    // M1：取新实例的那次可写挂载，录它的写流。
    let stream = SharedStream::retaining_contents();
    let first_summary = if prefix == "acquisition_rolled_back" {
        // M1 的取号写之后那道屏障在盘 0 上报错（整池第 3 次设备屏障调用）：取号回卷成旧代号，M1 报失败。
        let calls = std::rc::Rc::new(std::cell::Cell::new(0u32));
        let mut devices: Vec<(DeviceIdentity, singlefs_harness::RecordingBlockDevice<FailingTheNthBarrier>)> =
            common_unwitnessed_publish::plain_devices_on(&crash_image)
                .into_iter()
                .map(|(identity, inner)| {
                    (
                        identity,
                        singlefs_harness::RecordingBlockDevice::with_shared_stream(
                            identity,
                            FailingTheNthBarrier { inner, calls: calls.clone(), fail_at: 3 },
                            stream.clone(),
                        ),
                    )
                })
                .collect();
        let failure = mount_writable(&recorded.pool.parameters, &mut devices).expect_err("M1 的取号回卷");
        let text = format!("{failure:?}");
        format!("m1=failed:{}", text.chars().filter(|c| !c.is_whitespace()).take(160).collect::<String>())
    } else {
        let mut devices =
            recorded_devices_reading_back_all_zero_on(&crash_image, &ranges_of_the_first_mount(&recorded, &prefix), &stream);
        let first = mount_writable(&recorded.pool.parameters, &mut devices).expect("M1 做得成");
        format!(
            "m1_instance={} m1_effective=({},{})",
            first.output.instance.0,
            first.output.effective_root.instance.0,
            first.output.effective_root.checkpoint_txg.0
        )
    };
    let (writes, segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let state_count = crash_state_count_with_torn_in_place_overwrites(
        &crash_image,
        &writes,
        &segments,
        &full_expansion,
    );
    println!(
        "name=c834_history prefix={prefix} r=({},{}) p=({},{}) {first_summary} m1_writes={} m1_segments={} state_count={state_count} candidate={candidate}",
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
        println!("name=c834_segment prefix={prefix} segment={index} writes={}", kinds.join(","));
    }
    if mode == "count" {
        return;
    }
    assert!(state_count <= 1_000_000, "一次全量不超过约 10⁶ 个状态");
    let tables = crash_state_enumeration_tables(&crash_image, &writes, &segments, &full_expansion);
    let facts = extract_flow_facts(&crash_image, &writes, &segments, &full_expansion);
    assert_eq!(tables.state_count, state_count);
    let r_instance = recorded.key.0;
    let start: u64 = arguments.get(3).map_or(0, |value| value.parse().expect("起点序号"));
    let end: u64 = arguments.get(4).map_or(state_count, |value| value.parse().expect("终点序号")).min(state_count);
    let step: u64 = arguments.get(5).map_or(1, |value| value.parse().expect("步长"));
    let faults = arguments.get(6).cloned().unwrap_or_else(|| "y2".to_string());
    for ordinal in (start..end).step_by(usize::try_from(step).expect("步长")) {
        let persisted = facts.persisted_writes_of_state(ordinal);
        let image = image_with(&crash_image, &tables.enumerated_writes, &persisted);
        let segment = facts.segment_of_state(ordinal);
        let state_red = red_invariants(&image);
        let r_units_overwritten_in_state = units_of_r_overwritten_on(&recorded, &image).len();
        let per_device = instances_of_each_device(&image);
        let later = later_records(&image, r_instance);
        for order in ["forward", "reversed"] {
            let result = second_mount(&recorded, &image, order, &faults);
            println!(
                "name=c834_state prefix={prefix} m2_faults={faults} candidate={candidate} ordinal={ordinal} segment={segment} per_device={per_device:?} later_records={later} r_units_overwritten_in_state={r_units_overwritten_in_state} state_red={} order={order} {result}",
                if state_red.is_empty() { "-".to_string() } else { state_red.join("+") }
            );
        }
    }
    let _ = DeviceIdentity(0);
}

/// 整池第 `fail_at` 次设备屏障调用报错，别的照转给稀疏盘（几块盘共用一个计数）。
struct FailingTheNthBarrier {
    inner: singlefs_harness::memory_pool::SparseBlockDevice,
    calls: std::rc::Rc<std::cell::Cell<u32>>,
    fail_at: u32,
}

impl singlefs_core::block_device::BlockDevice for FailingTheNthBarrier {
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let call = self.calls.get() + 1;
        self.calls.set(call);
        if call == self.fail_at {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                std::io::Error::other("注入的屏障错"),
            ));
        }
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}
