//! 第 19 批代码轮第一轮云端攻方原型（流在这里自己造，不从任何层 0 用例拷）：
//! mkfs（环长 `B19_RING_MIB`，单元区 `B19_SLOTS` 槽）→ 可写挂载 `B19_MOUNTS` 次（只写行、暖机）→ 最后一次挂载上写第一个文件 V。
//! 录下 V 那一次发布的写流，按屏障与 FUA 切段，先用 `closed_form_state_count` 算全量状态数（超过 10⁶ 不跑），
//! 再调 `enumerate_layer0`（每一段都展开、段内整写子集逐个枚举）把每个崩溃状态恢复之后跑 checker，报每条不变量红了几个状态。
//! 坏实现 H2 由工作区副本里 `allocator.rs` 的补丁读 `B19_GATE_SLACK` 进来（0 = 原实现）。

use singlefs_checker_tier::crash::{enumerate_layer0, enumerate_layer0_selecting};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, MemoryPool, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type MemoryDevice = RecordingBlockDevice<SparseBlockDevice>;

const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const WRITE_TIME_SECONDS: u64 = 1_788_000_060;
const STATE_CAP: u64 = 1_000_000;

fn knob(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .map_or(default, |text| text.trim().parse().expect("数"))
}

fn parameters(ring_bytes: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn file_content() -> Vec<u8> {
    (0..3000usize)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

fn pool_of(devices: &[(DeviceIdentity, MemoryDevice)], device_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

fn main() {
    let ring_bytes = knob("B19_RING_MIB", 1) << 20;
    let unit_area_slots = knob("B19_SLOTS", 2000);
    let mounts = knob("B19_MOUNTS", 9);
    let slack = knob("B19_GATE_SLACK", 0);
    let parameters = parameters(ring_bytes);
    let unit_area_start = UnitAreaStart::following_the_journal_ring(ring_bytes)
        .expect("环长让单元区起点落在段边界上")
        .slot()
        .0;
    let device_bytes = (unit_area_start + unit_area_slots) * SLOT_BYTES;
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, MemoryDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(number),
                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut mounted = None;
    for _ in 0..mounts {
        mounted = Some(mount_writable(&parameters, &mut devices).expect("可写挂载"));
    }
    let mounted = mounted.expect("至少挂载一次");
    let PoolVersion::WithoutFile(version) = &mounted.current else {
        panic!("只暖机过的池上现行那一版树表 0 条")
    };
    let base = pool_of(&devices, device_bytes);
    let operations_before = stream.operation_count();
    let mut allocator = mounted.allocator.clone();
    let first_file = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &file_content(),
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            mounted.output.instance,
            &version.record_bytes,
        )
        .expect("第一个文件")
    };
    let data_slot = first_file.data_pointers[0].locations[0].slot.0;
    let operations = stream.retained_operations()[operations_before..].to_vec();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: ring_bytes,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    };
    let (writes, segments) = writes_and_segments(&operations, &geometry);
    let states = closed_form_state_count(&segments);
    let segment_lengths: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!(
        "name=history ring_mib={} unit_area_slots={unit_area_slots} mounts={mounts} gate_slack={slack} txg={} data_rel_U={} writes={} segments={segment_lengths:?} closed_form_states={states}",
        ring_bytes >> 20,
        first_file.root.checkpoint_txg.0,
        data_slot - unit_area_start,
        writes.len(),
    );
    let root_index = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("V 那一次发布写了根槽");
    // `B19_EXPAND_FROM` = k > 0：只展开下标 ≥ k 的段（估时与定点取样用，不是层 0 全量；报告里分开写）。缺省 0 = 每一段都展开。
    let expand_from = usize::try_from(knob("B19_EXPAND_FROM", 0)).expect("小");
    let started = std::time::Instant::now();
    let (mode, tally) = if expand_from == 0 {
        if states > STATE_CAP {
            println!("name=skipped reason=closed_form_states_over_cap cap={STATE_CAP}");
            return;
        }
        (
            "full".to_string(),
            enumerate_layer0(&base, &writes, &segments, root_index, &file_content()),
        )
    } else {
        (
            format!("expand_from_segment_{expand_from}"),
            enumerate_layer0_selecting(
                &base,
                &writes,
                &segments,
                root_index,
                &file_content(),
                &|segment_index, _segment| segment_index >= expand_from,
            ),
        )
    };
    println!(
        "name=layer0 mode={mode} gate_slack={slack} states={} oracle_violations={} root_persisted_states={} failed_states={} seconds={:.1}",
        tally.states,
        tally.violations,
        tally.root_persisted_states,
        tally.failed_states,
        started.elapsed().as_secs_f64()
    );
    for (invariant, count) in &tally.checker_violated_states {
        if *count > 0 {
            println!(
                "name=checker-red gate_slack={slack} invariant={invariant} red_states={count} first={}",
                tally
                    .checker_first_violation
                    .get(invariant)
                    .map_or(String::new(), |text| text.chars().take(200).collect())
            );
        }
    }
    for invariant in ["I-5.5"] {
        println!(
            "name=i55 gate_slack={slack} evaluated={} red={} not_applicable={}",
            tally.checker_evaluated_states.get(invariant).copied().unwrap_or(0),
            tally.checker_violated_states.get(invariant).copied().unwrap_or(0),
            tally.checker_not_applicable_states.get(invariant).copied().unwrap_or(0),
        );
    }
}
