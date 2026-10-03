//! C651 攻方原型。候选由环境变量 C651_CANDIDATE 选（副本里 core 与 checker 同一个开关，patch_copy.py 装的）。
//! 每段历史：前缀（旧池、mkfs、用户动作）直接在内存稀疏盘上做；尾巴（一次可写挂载）录成写流，
//! 按层 0 的枚举域（每段都展开、原地覆写三态）逐个状态判：恢复（实现的读法）、池级 checker（checker 的读法）、oracle。
use std::collections::BTreeMap;

use singlefs_checker::image::{system_configuration_slot_readings, SystemConfigurationSlotReading};
use singlefs_checker_tier::crash::{
    judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites, Layer0SegmentExpansion,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, MemoryPool, PublishedVersion, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 单元区 384 槽那一档小盘（`crates/singlefs-harness/tests/common_admission_pool/mod.rs`）：环 6 MiB，盘 = (1408 + 384) 槽。
const JOURNAL_RING_BYTES: u64 = 6 << 20;
const DEVICE_BYTES: u64 = (1408 + 384) * 16384;
const NEW_FSID: [u8; 16] = *b"c651-new-pool-B!";
const OLD_FSID: [u8; 16] = *b"c651-old-pool-A!";
const WRITE_TIME: u64 = 1_788_000_000;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn geometry(io_min: u32) -> SystemImmutableSizes {
    SystemImmutableSizes {
        physical_block_size: 512,
        minimum_input_output_bytes: io_min,
        fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(io_min),
        journal_ring_bytes: JOURNAL_RING_BYTES,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    }
}

fn parameters(fsid: [u8; 16], io_min: u32) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: fsid,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: geometry(io_min),
    }
}

fn fresh_device() -> SparseBlockDevice {
    SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512))
}

fn fresh_devices() -> Devices {
    vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())]
}

fn clone_device(device: &SparseBlockDevice) -> SparseBlockDevice {
    let mut copy = fresh_device();
    copy.image = device.image.clone();
    copy
}

fn image_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

fn content(seed: u64) -> Vec<u8> {
    (0..3000u64).map(|i| u8::try_from((i * 137 + seed * 11 + 3) % 251).expect("<256")).collect()
}

/// 一个池在 mkfs 之后、同一个进程里做的事（用户动作）：
/// 0 = 只 mkfs；1 = 取号 + 暖机（槽 1 世代 3、槽 0 世代 4）；2 + k = 再发第一个文件与 k 次覆盖写（每次发布轮换一次）。
fn make_and_use(devices: &mut Devices, parameters: &MakeFilesystemParameters, usage: u32, seed: u64) -> Vec<PublishedVersion> {
    let genesis = make_filesystem(parameters, devices).expect("mkfs");
    let mut versions = Vec::new();
    if usage == 0 {
        return versions;
    }
    let mut allocator = allocator_after_make_filesystem(parameters, devices, &genesis);
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    if usage == 1 {
        return versions;
    }
    let first_content = content(seed);
    let mut previous = publish_first_file(
        &mut writer,
        &mut allocator,
        warmed.roots.last().expect("两代根"),
        FirstFile { content: &first_content, write_time_seconds: WRITE_TIME },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("第一个文件");
    versions.push(PublishedVersion { instance, checkpoint_txg: previous.root.checkpoint_txg, content: first_content });
    for k in 0..usage - 2 {
        let c = content(seed + 1 + u64::from(k));
        previous = publish_overwrite(
            &mut writer,
            &mut allocator,
            &previous,
            FirstFile { content: &c, write_time_seconds: WRITE_TIME + 1 + u64::from(k) },
            instance,
        )
        .expect("覆盖写");
        versions.push(PublishedVersion { instance, checkpoint_txg: previous.root.checkpoint_txg, content: c });
    }
    versions
}

struct History {
    name: String,
    base: MemoryPool,
    writes: Vec<singlefs_harness::memory_pool::RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// 尾巴：进程重开、可写挂载一次（只做过 mkfs 的池：取号 + 暖机两次；有文件的池：取号），录成写流。
fn record_tail(devices: &Devices, parameters: &MakeFilesystemParameters, name: String, versions: Vec<PublishedVersion>) -> History {
    let base = image_of(devices);
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = devices
        .iter()
        .map(|(identity, device)| (*identity, RecordingBlockDevice::with_shared_stream(*identity, clone_device(device), stream.clone())))
        .collect();
    if versions.is_empty() && !name.ends_with(":n=1") {
        // 只做过 mkfs 的池：尾巴是第一次可写挂载（取号 + 暖机两次）。
        mount_writable(parameters, &mut recorded).expect("尾巴：可写挂载");
    } else {
        // 用过的池：尾巴只录下一次可写挂载的第一次写——取号（两块盘各写一个系统配置槽）。挂载的其余写不进流（缩历史长度，见报告）。
        let mut writer = PoolWriter::new(parameters, recorded.as_mut_slice());
        acquire_instance(&mut writer).expect("尾巴：取号");
    }
    let operations = stream.retained_operations();
    let fixed = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let (writes, segments) = writes_and_segments(&operations, &fixed);
    let judged_root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
    History { name, base, writes, segments, judged_root_index, versions }
}

fn histories() -> Vec<History> {
    let mut out = Vec::new();
    // C 组：新盘、纯崩溃。y = mkfs 时的 io_min；n = mkfs 之后同一个进程里的用户动作（见 make_and_use）。
    for y in [512u32, 8192, 65536] {
        for n in [0u32, 2, 3, 4] {
            let p = parameters(NEW_FSID, y);
            let mut d = fresh_devices();
            let versions = make_and_use(&mut d, &p, n, 100);
            out.push(record_tail(&d, &p, format!("C:y={y}:n={n}"), versions));
        }
    }
    // S 组：盘上有一个更早的池（io_min x、用户动作 u），换了 io_min y 重做 mkfs（两块盘都是旧盘 / 只有盘 0 是旧盘；fsid 不同 / 相同）。
    for (x, y) in [(8192u32, 512u32), (65536, 512), (512, 8192), (512, 65536), (65536, 8192)] {
        for u in [0u32, 1, 5] {
            for reuse in ["both", "disk0"] {
                for same_fsid in [false, true] {
                    for n in [0u32, 2, 4] {
                        let old_parameters = parameters(if same_fsid { NEW_FSID } else { OLD_FSID }, x);
                        let mut old = fresh_devices();
                        make_and_use(&mut old, &old_parameters, u, 7);
                        let mut d: Devices = if reuse == "both" {
                            old
                        } else {
                            vec![(DeviceIdentity(0), clone_device(&old[0].1)), (DeviceIdentity(1), fresh_device())]
                        };
                        let p = parameters(NEW_FSID, y);
                        let versions = make_and_use(&mut d, &p, n, 100);
                        let fsid = if same_fsid { "same" } else { "diff" };
                        out.push(record_tail(&d, &p, format!("S:x={x}:y={y}:u={u}:reuse={reuse}:fsid={fsid}:n={n}"), versions));
                    }
                }
            }
        }
    }
    out
}

fn fsid_name(fsid: &[u8; 16]) -> &'static str {
    if *fsid == NEW_FSID { "new" } else if *fsid == OLD_FSID { "old" } else { "other" }
}

fn variant(debug: String) -> String {
    debug.split(|c: char| c == '{' || c == '(' || c == ' ').next().unwrap_or("").to_string()
}

fn core_choice(reader: &dyn PoolReader) -> String {
    match choose_system_configuration(reader) {
        Ok(c) => format!(
            "{}@{}g{}d{}",
            fsid_name(&c.immutable.filesystem_identifier),
            c.immutable.sizes.fixed_structure_slot_spacing,
            c.quantities.slot_generation,
            c.immutable.this_device.0
        ),
        Err(e) => format!("Err:{}", variant(format!("{e:?}"))),
    }
}

fn checker_slot_one(reader: &dyn singlefs_checker::image::ImageReader) -> (bool, String) {
    let readings = system_configuration_slot_readings(reader);
    let all_slot_zero_unusable = readings.iter().all(|(_, r)| !matches!(r[0], SystemConfigurationSlotReading::Usable(..)));
    let slot_one: Vec<String> = readings
        .iter()
        .map(|(_, r)| match &r[1] {
            SystemConfigurationSlotReading::Usable(view, geometry) => {
                format!("{}@{}g{}", fsid_name(&view.filesystem_identifier), geometry.slot_spacing, view.slot_generation)
            }
            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { offset_in_bytes, .. } => format!("refused@{offset_in_bytes}"),
            SystemConfigurationSlotReading::NotSelfVerified => "notverified".to_string(),
            SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => "notchoosable".to_string(),
        })
        .collect();
    (all_slot_zero_unusable, slot_one.join("|"))
}

fn outcome_name(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { root } => format!("NoFile(i{},t{})", root.0 .0, root.1 .0),
        RecoveryOutcome::FileRead { root, .. } => format!("FileRead(i{},t{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { failure, .. } => format!("Failed:{}", variant(format!("{failure:?}"))),
    }
}

fn red_invariants(verdicts: &[(&'static str, singlefs_checker::image::InvariantVerdict)]) -> Vec<&'static str> {
    verdicts
        .iter()
        .filter(|(_, v)| matches!(v, singlefs_checker::image::InvariantVerdict::Violated(_)))
        .map(|(i, _)| *i)
        .collect()
}

/// 介质损坏：整条尾巴持久之后，两块盘各把 [0, length) 抹成 0（length = 4096：只坏槽 0；17408：GPT 那 34 个扇区）。
fn media_cell(candidate: &str, history: &History, length: u64) {
    let mut image = history.base.clone();
    image.apply_writes(&history.writes);
    for device in image.devices.values_mut() {
        device.zero_fill(DeviceOffsetInBytes(0), length);
    }
    let (all_slot_zero_unusable, checker) = checker_slot_one(&image);
    let core = core_choice(&image);
    let outcome = outcome_name(&recover(&image, JournalPolicy::Consult).outcome);
    let red = red_invariants(&singlefs_checker::walk::check_pool_image(&image));
    println!(
        "name=media candidate={candidate} h={} zeroed={length} all_slot0_unusable={all_slot_zero_unusable} core={core} outcome={outcome} checker_slot1={checker} checker_red={red:?}",
        history.name
    );
}

fn main() {
    let candidate = std::env::var("C651_CANDIDATE").unwrap_or_else(|_| "today".to_string());
    let filter = std::env::var("C651_FILTER").unwrap_or_default();
    let started = std::time::Instant::now();
    let mut total_states = 0u64;
    let mut history_count = 0u64;
    for history in histories().into_iter().filter(|h| h.name.contains(&filter)) {
        history_count += 1;
        let expansion = |_: usize, _: &[usize]| Layer0SegmentExpansion::EveryProperSubset;
        let two_state = closed_form_state_count(&history.segments);
        let states = layer0_state_count_with_torn_in_place_overwrites(&history.base, &history.writes, &history.segments, &expansion);
        assert!(states <= 1_000_000, "{}：{states} 态超过一次 10^6 的上限", history.name);
        let mut signatures: BTreeMap<String, u64> = BTreeMap::new();
        let mut all_slot_zero_unusable_states = 0u64;
        let mut oracle_red = 0u64;
        let mut checker_red = 0u64;
        let judged = judge_layer0_state_range(
            &history.base,
            &history.writes,
            &history.segments,
            history.judged_root_index,
            &history.versions,
            &expansion,
            0..states,
            &mut |_ordinal, image, judgement| {
                let (all_slot_zero_unusable, checker) = checker_slot_one(image);
                let oracle = judgement.consulted_oracle.as_ref().map_or("-".to_string(), |v| format!("{:?}", v.kind));
                let red = red_invariants(&judgement.pool_checker);
                if oracle != "-" {
                    oracle_red += 1;
                }
                if !red.is_empty() {
                    checker_red += 1;
                }
                if all_slot_zero_unusable {
                    all_slot_zero_unusable_states += 1;
                }
                if all_slot_zero_unusable || oracle != "-" || !red.is_empty() {
                    let key = format!(
                        "all_slot0_unusable={all_slot_zero_unusable} core={} outcome={} oracle={oracle} checker_slot1={checker} checker_red={red:?}",
                        core_choice(image),
                        outcome_name(&judgement.consulted.outcome)
                    );
                    *signatures.entry(key).or_insert(0) += 1;
                }
            },
        );
        assert_eq!(judged, states);
        total_states += states;
        let shape: Vec<usize> = history.segments.iter().map(Vec::len).collect();
        println!(
            "name=history candidate={candidate} h={} segments={shape:?} states_two_state={two_state} states={states} all_slot0_unusable={all_slot_zero_unusable_states} oracle_red={oracle_red} checker_red={checker_red}",
            history.name
        );
        for (key, count) in &signatures {
            println!("name=sig candidate={candidate} h={} count={count} {key}", history.name);
        }
        media_cell(&candidate, &history, 4096);
        media_cell(&candidate, &history, 17408);
    }
    println!(
        "name=done candidate={candidate} histories={history_count} states={total_states} elapsed_seconds={:.1}",
        started.elapsed().as_secs_f64()
    );
}
