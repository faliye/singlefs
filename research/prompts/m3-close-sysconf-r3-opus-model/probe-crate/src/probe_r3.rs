//! m3-close-sysconf-r3 攻方原型的新模式：counts（新池新建文件流在两块、三块 4 GiB 盘上的崩溃状态闭式数，今天与取号写两轮两条臂各跑一次）、
//! snapshot（最新那次发布单元写到一半时取下的快照盘换回池里，落后支各臂下可写挂载的结局）。
use crate::common_admission_pool::{fixed_geometry_of_the_384_slot_small_devices, plain_devices_on, PoolUnderTest, OVERWRITE_BYTES};
use crate::probe_body::{arm_name, sysconf_signature, variant_name};
use singlefs_checker_tier::crash::{crash_state_count_with_torn_in_place_overwrites, CrashPointReplaySegmentExpansion};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::{e142_parameters, run_new_pool_file_creation};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const FOUR_GIBIBYTES: u64 = 4 << 30;

/// 新池新建文件流（mkfs → 取号 → 暖机 → 新池新建文件）录下 mkfs 之后的写，数闭式与带撕裂态的状态数。盘数 2 或 3。
fn counts_of_the_new_pool_file_creation_stream(device_count: u32) {
    let mut parameters = e142_parameters(512, 512);
    if device_count == 3 {
        parameters.region_devices = [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(2)];
    }
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..device_count)
        .map(|number| {
            let identity = DeviceIdentity(number);
            (identity, RecordingBlockDevice::with_shared_stream(identity, SparseBlockDevice::new(FOUR_GIBIBYTES, PhysicalBlockSizeInBytes(512)), stream.clone()))
        })
        .collect();
    let run = run_new_pool_file_creation(&parameters, &mut devices, &stream, |_, _| {}).expect("新池新建文件");
    let operations = stream.retained_operations();
    let identities: Vec<DeviceIdentity> = (0..device_count).map(DeviceIdentity).collect();
    let mut base = MemoryPool::with_devices(&identities, FOUR_GIBIBYTES);
    base.apply(&operations[..run.mkfs_operation_count]);
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let (writes, segments) = writes_and_segments(&operations[run.mkfs_operation_count..], &geometry);
    let expansion = |_: usize, _: &[usize]| CrashPointReplaySegmentExpansion::EveryProperSubset;
    let torn = crash_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &expansion);
    let closed = closed_form_state_count(&segments);
    let sizes: Vec<String> = segments.iter().map(|segment| segment.len().to_string()).collect();
    println!(
        "probe=counts stream=new_pool_file_creation devices={device_count} {} writes={} segments={} closed_form_states={closed} states_with_torn={torn} segment_sizes=[{}]",
        arm_name(), writes.len(), segments.len(), sizes.join(",")
    );
}

/// 最新那次发布 P（一次会话覆盖写）的单元写到一半时取下盘 1 的快照（落了 `landed` 个单元写，记录、根、系统配置都没落），池在两块盘上做完 P，
/// 再把盘 1 换成那份快照，可写挂载一次。臂由 SINGLEFS_PROBE_BEHIND 选（tail 是今天）。
fn snapshot_taken_in_the_middle_of_the_newest_publish(overwrites_before: usize) {
    let parameters = crate::pool_parameters();
    let stream = SharedStream::retaining_contents();
    let wrap = |identity: DeviceIdentity, sparse: SparseBlockDevice| RecordingBlockDevice::with_shared_stream(identity, sparse, stream.clone());
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(parameters.clone(), crate::common_admission_pool::device_bytes_of_the_384_slot_small_devices(), wrap);
    for _ in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES).expect("前面的覆盖写");
    }
    let before = pool.image();
    let mark = stream.retained_operations().len();
    pool.overwrite(OVERWRITE_BYTES).expect("P");
    let after = pool.image();
    let p_root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let (writes, _segments) = writes_and_segments(&operations[mark..], &geometry);
    let unit_writes_on_device_one: Vec<_> = writes
        .iter()
        .filter(|write| write.device == DeviceIdentity(1) && !matches!(write.kind, StepKind::SystemConfigurationSlot | StepKind::JournalRecord | StepKind::RootRecordFua))
        .cloned()
        .collect();
    let kinds: Vec<String> = writes.iter().map(|write| format!("d{}:{:?}", write.device.0, write.kind)).collect();
    println!("probe=snapshot {} overwrites_before={overwrites_before} p_txg={} p_instance={} p_writes={} unit_writes_on_device_one={} kinds={}", arm_name(), p_root.checkpoint_txg.0, p_root.instance.0, writes.len(), unit_writes_on_device_one.len(), kinds.join(" "));
    for landed in [unit_writes_on_device_one.len().saturating_sub(1), unit_writes_on_device_one.len()] {
        let mut snapshot = before.clone();
        snapshot.apply_writes(&unit_writes_on_device_one[..landed]);
        let mut swapped = after.clone();
        swapped.devices.insert(DeviceIdentity(1), snapshot.devices[&DeviceIdentity(1)].clone());
        let mut devices = plain_devices_on(&swapped);
        let outcome = match mount_writable(&parameters, &mut devices) {
            Ok(mounted) => format!("mounted instance={} effective_txg={}", mounted.output.instance.0, mounted.output.effective_root.checkpoint_txg.0),
            Err(error) => format!("refused {}", variant_name(&format!("{error:?}"))),
        };
        println!(
            "probe=snapshot {} overwrites_before={overwrites_before} landed_unit_writes={landed}_of_{} sysconf={} outcome={outcome}",
            arm_name(), unit_writes_on_device_one.len(), sysconf_signature(&swapped)
        );
    }
}

pub fn run(args: &[String]) {
    for name in args {
        match name.as_str() {
            "counts2" => counts_of_the_new_pool_file_creation_stream(2),
            "counts3" => counts_of_the_new_pool_file_creation_stream(3),
            "snapshot0" => snapshot_taken_in_the_middle_of_the_newest_publish(0),
            "snapshot1" => snapshot_taken_in_the_middle_of_the_newest_publish(1),
            other => panic!("认不出 {other}"),
        }
    }
}
