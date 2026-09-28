//! checker 档模块：crash
//! E164（崩溃放量新设计的小范围功能测试）第三段的 checker 档探针：H-长的崩溃状态逐个判定（跑前登记 5.9、修订 R10）。
//! 装置把这份文本放进仓副本的 `crates/singlefs-checker-tier/tests/e164_crash_states.rs`，带 `SINGLEFS_HEAVY_TESTS=user-request`
//! 在副本里起 `cargo test --release -p singlefs-checker-tier --test e164_crash_states`；主仓里没有这份文件。
//! H-长的录制与第二段探针（`probe_recording.rs` 的 `records_h_long`）是同一段做法、同一组常量；两份录出的每一步写数与写表摘要由装置逐步比对。
//! 每个状态一行 `E7RESULT probe=crash_state …`：所在步、状态键（持久了的写按次序叠出的扇区终值摘要）、红没红、
//! 池级 checker 违反的不变量、记录核对器、两种 journal 政策的 oracle、恢复结局摘要、违例正文摘要。

use std::collections::BTreeMap;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    check_records, classified_oracle_violation_for_versions, enumerate_layer0_selecting_versions_observing_each_state,
};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_sequential_write, warm_up, FirstFile, PoolWriter,
};
use singlefs_harness::memory_pool::{
    newest_persisted_root, writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite, SparseBlockDevice, WrittenContents,
};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::sha256::sha256_hexadecimal;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 以下常量与第二段探针逐字相同。
const DEVICE_BYTES: u64 = 4 << 30;
const SECTOR_BYTES: u64 = 512;
const FIRST_OVERWRITE_BYTES: usize = 2000;
const SEQUENTIAL_WRITE_BYTES: usize = 70000;
const SECOND_OVERWRITE_BYTES: usize = 1500;
const FIRST_OVERWRITE_SECONDS_AFTER: u64 = 60;
const SEQUENTIAL_WRITE_SECONDS_AFTER: u64 = 120;
const SECOND_OVERWRITE_SECONDS_AFTER: u64 = 180;
/// 跑前登记 5.9 的小域：段内写数不超过 15（两态 2¹⁵ 个状态）的段全展开，别的只取全持久那一个。
const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 15;

type RecordedSparse = RecordingBlockDevice<SparseBlockDevice>;

fn first_overwrite_content() -> Vec<u8> {
    (0..FIRST_OVERWRITE_BYTES).map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256")).collect()
}

fn sequential_write_content() -> Vec<u8> {
    (0..SEQUENTIAL_WRITE_BYTES).map(|index| u8::try_from((index * 11 + 1) % 239).expect("小于 256")).collect()
}

fn second_overwrite_content() -> Vec<u8> {
    (0..SECOND_OVERWRITE_BYTES).map(|index| u8::try_from((index * 13 + 5) % 241).expect("小于 256")).collect()
}

fn parameters() -> MakeFilesystemParameters {
    e142_parameters(512, 512)
}

fn geometry_of(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

fn recorded_devices(stream: &SharedStream) -> Vec<(DeviceIdentity, RecordedSparse)> {
    (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device_number),
                    SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// H-长录完之后的四样：基镜像（mkfs 之后）、写表、段、每一步的起点（录制流下标，mkfs 之后从 0 数）、版本表、被判的根。
struct RecordedLongHistory {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    write_stream_indexes: Vec<usize>,
    step_starts: Vec<(String, usize)>,
    versions: Vec<PublishedVersion>,
    judged_root_index: usize,
}

fn version_of(output_root: (InstanceGeneration, singlefs_core::address::CheckpointTxg), content: &[u8]) -> PublishedVersion {
    PublishedVersion { instance: output_root.0, checkpoint_txg: output_root.1, content: content.to_vec() }
}

fn record_long_history() -> RecordedLongHistory {
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices(&stream);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_end = stream.operation_count();
    let base = MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
        device_size_in_bytes: DEVICE_BYTES,
    };
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let first_content = first_file_content();
    let (instance, warm_up_output, acquisition_end) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let acquisition_end = stream.operation_count();
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        (instance, warm, acquisition_end)
    };
    assert_eq!(instance, InstanceGeneration(1));
    let warm_up_end = stream.operation_count();
    let created = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile { content: &first_content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    let creation_end = stream.operation_count();
    let first_overwrite_bytes = first_overwrite_content();
    let first_overwrite = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &created,
            FirstFile { content: &first_overwrite_bytes, write_time_seconds: FIXED_WRITE_TIME_SECONDS + FIRST_OVERWRITE_SECONDS_AFTER },
            instance,
        )
        .expect("第一次覆盖写")
    };
    let first_overwrite_end = stream.operation_count();
    let sequential_bytes = sequential_write_content();
    let sequential = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        publish_sequential_write(
            &mut pool,
            &mut allocator,
            &first_overwrite,
            FirstFile { content: &sequential_bytes, write_time_seconds: FIXED_WRITE_TIME_SECONDS + SEQUENTIAL_WRITE_SECONDS_AFTER },
            instance,
        )
        .expect("顺序写")
    };
    let sequential_end = stream.operation_count();
    let second_overwrite_bytes = second_overwrite_content();
    let second_overwrite = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &sequential,
            FirstFile { content: &second_overwrite_bytes, write_time_seconds: FIXED_WRITE_TIME_SECONDS + SECOND_OVERWRITE_SECONDS_AFTER },
            instance,
        )
        .expect("第二次覆盖写")
    };
    let operations = stream.retained_operations();
    let (writes, segments, write_stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations[mkfs_end..], &geometry_of(&parameters));
    let versions = vec![
        version_of((created.root.instance, created.root.checkpoint_txg), &first_content),
        version_of((first_overwrite.root.instance, first_overwrite.root.checkpoint_txg), &first_overwrite_bytes),
        version_of((sequential.root.instance, sequential.root.checkpoint_txg), &sequential_bytes),
        version_of((second_overwrite.root.instance, second_overwrite.root.checkpoint_txg), &second_overwrite_bytes),
    ];
    let judged_root_index = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .last()
        .expect("至少一条根槽写");
    let step_starts = vec![
        ("0".to_string(), 0),
        ("1".to_string(), acquisition_end - mkfs_end),
        ("2".to_string(), warm_up_end - mkfs_end),
        ("3".to_string(), creation_end - mkfs_end),
        ("4".to_string(), first_overwrite_end - mkfs_end),
        ("5".to_string(), sequential_end - mkfs_end),
    ];
    RecordedLongHistory { base, writes, segments, write_stream_indexes, step_starts, versions, judged_root_index }
}

/// 一次写（按录制流下标）落在哪一步。
fn step_of_stream_index(step_starts: &[(String, usize)], stream_index: usize) -> String {
    step_starts.iter().rev().find(|(_, start)| *start <= stream_index).map(|(label, _)| label.clone()).expect("步 0 从 0 起")
}

/// 状态键：持久了的写按写表次序叠出的扇区终值，按（盘，扇区）排好取 SHA-256。与写表下标、段怎么切都无关，跨变体可比。
fn state_key(image: &CrashImage<'_>) -> String {
    let mut sectors: BTreeMap<(u32, u64), String> = BTreeMap::new();
    for (write, persisted) in image.writes.iter().zip(&image.persisted) {
        if !persisted {
            continue;
        }
        let first_sector = write.offset.0 / SECTOR_BYTES;
        match &write.contents {
            WrittenContents::Bytes(bytes) => {
                for (sector_offset, chunk) in bytes.chunks(usize::try_from(SECTOR_BYTES).expect("512")).enumerate() {
                    sectors.insert((write.device.0, first_sector + sector_offset as u64), sha256_hexadecimal(chunk));
                }
            }
            WrittenContents::Zeros { length } => {
                for sector_offset in 0..length.div_ceil(SECTOR_BYTES) {
                    sectors.insert((write.device.0, first_sector + sector_offset), "zeros".to_string());
                }
            }
        }
    }
    let text: String = sectors.iter().map(|((device, sector), digest)| format!("{device} {sector} {digest}\n")).collect();
    sha256_hexadecimal(text.as_bytes())
}

fn recovery_text(report: &RecoveryReport) -> String {
    let outcome = match &report.outcome {
        RecoveryOutcome::NoFile { root } => format!("no_file {}:{}", root.0 .0, root.1 .0),
        RecoveryOutcome::FileRead { root, content } => format!("file {}:{} {}", root.0 .0, root.1 .0, sha256_hexadecimal(content)),
        RecoveryOutcome::Failed { root, failure } => format!("failed {root:?} {failure:?}"),
    };
    format!("{outcome} effective={:?} prefix_applied={}", report.effective_root, report.journal.prefix_applied)
}

#[test]
fn judges_every_crash_state_of_h_long() {
    let history = record_long_history();
    let original_write_count = history.writes.len();
    let mut writes_per_step: BTreeMap<String, usize> = BTreeMap::new();
    for stream_index in &history.write_stream_indexes {
        *writes_per_step.entry(step_of_stream_index(&history.step_starts, *stream_index)).or_default() += 1;
    }
    for (step, count) in &writes_per_step {
        println!("E7RESULT probe=crash_history name=step_writes step={step} writes={count}");
    }
    let segment_sizes: Vec<String> = history.segments.iter().map(|segment| segment.len().to_string()).collect();
    println!("E7RESULT probe=crash_history name=segments sizes={}", segment_sizes.join("+"));
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES;
    let mut emitted = 0u64;
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &history.base,
        &history.writes,
        &history.segments,
        history.judged_root_index,
        &history.versions,
        &expand,
        &mut |image, consulted| {
            let first_missing = (0..original_write_count).find(|index| !image.persisted[*index]);
            let step = match first_missing {
                Some(index) => step_of_stream_index(&history.step_starts, history.write_stream_indexes[index]),
                None => history.step_starts.last().expect("有步").0.clone(),
            };
            let ignored = recover(image, JournalPolicy::Ignore);
            let newest = newest_persisted_root(image.writes, &image.persisted);
            let oracle_consulted = classified_oracle_violation_for_versions(&consulted.outcome, consulted.effective_root, newest, &history.versions)
                .map_or("ok".to_string(), |violation| violation.kind.name().to_string());
            let oracle_ignored = classified_oracle_violation_for_versions(&ignored.outcome, ignored.effective_root, newest, &history.versions)
                .map_or("ok".to_string(), |violation| violation.kind.name().to_string());
            let mut violated = Vec::new();
            let mut texts = String::new();
            for (invariant, verdict) in check_pool_image(image) {
                match verdict {
                    InvariantVerdict::Violated(detail) => {
                        violated.push(invariant.to_string());
                        texts.push_str(invariant);
                        texts.push('\n');
                        texts.push_str(&detail);
                        texts.push('\n');
                    }
                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => {}
                }
            }
            let records = check_records(image, consulted.effective_root);
            let records_text = format!("{}{}", u8::from(records.root_without_record), u8::from(records.claimed_state_missing_unit));
            let red = oracle_consulted != "ok" || oracle_ignored != "ok" || !violated.is_empty() || records_text != "00";
            let torn = image.persisted.len() > original_write_count && image.persisted[original_write_count..].iter().any(|persisted| *persisted);
            println!(
                "E7RESULT probe=crash_state step={step} key={} red={} violated={} records={records_text} oracle_consulted={oracle_consulted} oracle_ignored={oracle_ignored} torn={torn} recovery={} text={}",
                &state_key(image)[..32],
                u8::from(red),
                if violated.is_empty() { "none".to_string() } else { violated.join(",") },
                &sha256_hexadecimal(recovery_text(consulted).as_bytes())[..32],
                &sha256_hexadecimal(texts.as_bytes())[..32],
            );
            emitted += 1;
        },
    );
    assert_eq!(tally.states, emitted, "每个状态都回调了一次");
    println!("E7RESULT probe=crash_history name=crash_states_end states={} emitted={emitted}", tally.states);
}
