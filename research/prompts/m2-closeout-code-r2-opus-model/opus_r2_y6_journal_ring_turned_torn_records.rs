//! 代码轮第二轮云端攻方腿（m2-closeout-code-r2）Y6：journal 环转过圈之后，记录写是原地覆写（覆写的是环里更早的记录），
//! 层 0 的枚举域给它第三态（新旧都读不出）。这里在 1 MiB 环（在飞上限 85）的池上把环写转圈，再只录一小段（段都小，
//! 状态数照层 0 口径另算、控制在 10⁶ 以内），按层 0 的枚举域（每一段都展开，原地覆写三态）枚举，恢复之后 oracle 与池级 checker 判；
//! 每个崩溃状态再可写挂载一次、挂载之后的镜像跑池级 checker。名字里不带 layer0：它不是层 0 的全量流。
mod common_admission;

use std::collections::BTreeMap;

use common_admission::{checker_violations_on, content_of, PoolUnderTest};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, ParametersAndDeviceTableOfTheMount, ShadowLedger};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::crash::{
    enumerate_layer0_in_state_slices, full_expansion, layer0_state_count_with_torn_in_place_overwrites, writes_and_segments_with_stream_indexes_and_entries,
    Layer0Parallelism, MemoryPool, RetainedWrite, SparseBlockDevice, TearableInPlaceOverwrites,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::layer0_progress::Layer0Resume;
use singlefs_harness::segments::{segment_sizes_text, split_into_segments, FixedGeometry, StepKind};
use singlefs_harness::{RecordedEntrySpan, RecordedPublishEntry, RecordingBlockDevice, SharedStream};

type Recorded = RecordingBlockDevice<SparseBlockDevice>;

const RING_BYTES: u64 = 1024 * 1024;
const STATES_AT_MOST: u64 = 1_000_000;

fn parameters() -> MakeFilesystemParameters {
    let mut parameters = HistoryDeviceWidth::FourGibibytes.parameters();
    parameters.geometry.journal_ring_bytes = RING_BYTES;
    parameters
}

fn geometry(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

/// 同 `PoolUnderTest::start_after_the_first_file`，几何换成 1 MiB 环；每块盘包录制（留内容）。
fn start(stream: &SharedStream) -> PoolUnderTest<Recorded> {
    let parameters = parameters();
    let device_bytes = HistoryDeviceWidth::FourGibibytes.device_bytes();
    let mut devices: Vec<(DeviceIdentity, Recorded)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (identity, RecordingBlockDevice::with_shared_stream(identity, SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)), stream.clone()))
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("1 MiB 环上 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let first_content = content_of(3000, 0);
    let (instance, first_version) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_version = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("两条根"),
            FirstFile { content: &first_content, write_time_seconds: common_admission::FIRST_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件");
        (instance, first_version)
    };
    let mut content_of_each_root = BTreeMap::new();
    content_of_each_root.insert(common_admission::root_key(&first_version.root), first_content.clone());
    let parameters_and_device_table = ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&devices).expect("盘表");
    PoolUnderTest {
        parameters,
        device_bytes,
        devices,
        session: Some(MountedSession {
            allocator,
            current: PoolVersion::WithFile(first_version),
            instance,
            shadow_ledger: ShadowLedger::On,
            parameters_and_device_table,
        }),
        write_time_seconds: common_admission::FIRST_WRITE_TIME_SECONDS,
        content_of_the_current_version: first_content,
        content_of_each_root,
    }
}

/// 只做过 mkfs 的池（1 MiB 环），每块盘包录制；之后靠一次次崩了再可写挂载（取号、写行、暖机都是零单元或只写实例表的发布）把环写转圈。
fn start_without_file(stream: &SharedStream) -> PoolUnderTest<Recorded> {
    let parameters = parameters();
    let device_bytes = HistoryDeviceWidth::FourGibibytes.device_bytes();
    let mut devices: Vec<(DeviceIdentity, Recorded)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (identity, RecordingBlockDevice::with_shared_stream(identity, SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)), stream.clone()))
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("1 MiB 环上 mkfs");
    PoolUnderTest {
        parameters,
        device_bytes,
        devices,
        session: None,
        write_time_seconds: common_admission::FIRST_WRITE_TIME_SECONDS,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    }
}

fn last_counter(pool: &PoolUnderTest<Recorded>) -> u64 {
    pool.session().current.record().counter
}

/// 把环写到计数器越过 `past`（环 256 槽，越过 256 之后每条记录写都覆写环里一条更早的记录）。
fn turn_the_ring(pool: &mut PoolUnderTest<Recorded>, past: u64) -> usize {
    let mut overwrites = 0;
    while last_counter(pool) <= past {
        pool.overwrite(2999).unwrap_or_else(|refusal| panic!("第 {} 次覆盖写：{refusal:?}", overwrites + 1));
        overwrites += 1;
    }
    overwrites
}

/// 窗口里的写表与段：录制流从 `before` 起那一截，入口段照整条流记的平移过来（卸载记号只许落在卸载入口里，C557）。
fn writes_and_segments_of_the_window(
    stream: &SharedStream,
    before: usize,
    recorded: &[singlefs_harness::RetainedOperation],
    geometry: &FixedGeometry,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let spans: Vec<RecordedEntrySpan> = stream
        .entry_spans()
        .into_iter()
        .filter(|span| span.operations.start >= before)
        .map(|span| RecordedEntrySpan { entry: span.entry, operations: span.operations.start - before..span.operations.end - before })
        .collect();
    let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(recorded, &spans, geometry);
    (writes, segments)
}

/// 探一下各入口在转过圈的环上录出来的段序列（只数，不枚举）。
#[test]
fn probe_segment_sizes_of_the_entries_on_a_turned_ring() {
    let stream = SharedStream::retaining_contents();
    let mut pool = start(&stream);
    let geometry = geometry(&pool.parameters);
    let overwrites = turn_the_ring(&mut pool, 300);
    println!("Y6 probe overwrites_to_turn={overwrites} last_counter={}", last_counter(&pool));
    let windows: [&str; 3] = ["overwrite", "unmount", "crash_remount"];
    for window in windows {
        let base = pool.image();
        let before = stream.operation_count();
        match window {
            "overwrite" => {
                pool.overwrite(2999).expect("覆盖写");
            }
            "unmount" => {
                let mut session = pool.session.take().expect("会话");
                let parameters = pool.parameters.clone();
                let devices = &mut pool.devices;
                stream.record_entry(RecordedPublishEntry::Unmount, || {
                    singlefs_core::mount::unmount(&parameters, devices, &mut session.allocator, &mut session.current, ShadowLedger::On).expect("卸载");
                });
                pool.session = Some(session);
            }
            "crash_remount" => {
                pool.crash_and_mount_writable().expect("崩了再挂");
            }
            _ => unreachable!(),
        }
        let recorded = stream.retained_operations()[before..].to_vec();
        let operations = stream.operations()[before..].to_vec();
        let segments = split_into_segments(&operations, &geometry);
        let (writes, write_segments) = writes_and_segments_of_the_window(&stream, before, &recorded, &geometry);
        let tearable = TearableInPlaceOverwrites::of(&base, &writes);
        let tearable_journal = tearable.write_indexes().iter().filter(|index| writes[**index].kind == StepKind::JournalRecord).count();
        let journal = writes.iter().filter(|write| write.kind == StepKind::JournalRecord).count();
        let states = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &write_segments, &full_expansion);
        println!(
            "Y6 probe window={window} segments={} journal_writes={journal} tearable_journal_writes={tearable_journal} tearable_total={} layer0_states={states} counter_after={}",
            segment_sizes_text(&segments),
            tearable.write_indexes().len(),
            last_counter(&pool)
        );
    }
}

/// 环转过圈之后录一个入口（`OPUS_Y6_WINDOW`：unmount 或 crash_remount），按层 0 的枚举域全量枚举（原地覆写三态），
/// 恢复之后 oracle 与池级 checker 由枚举器判；每个崩溃状态再可写挂载一次、挂载之后的镜像跑池级 checker。
/// `OPUS_Y6_ESTIMATE=1` 时只展开写数不超过 2 的段（估时用，不算结果）。
#[test]
fn crash_states_of_an_entry_on_a_turned_journal_ring() {
    let window = std::env::var("OPUS_Y6_WINDOW").unwrap_or_else(|_| "unmount".to_string());
    let estimate = std::env::var("OPUS_Y6_ESTIMATE").is_ok();
    let started = std::time::Instant::now();
    let stream = SharedStream::retaining_contents();
    let (mut pool, overwrites) = if window == "nofile_remount" {
        let mut pool = start_without_file(&stream);
        let mut mounts = 0;
        loop {
            pool.crash_and_mount_writable().expect("只做过 mkfs 的池上崩了再挂");
            mounts += 1;
            if last_counter(&pool) > 300 {
                break;
            }
        }
        (pool, mounts)
    } else {
        let mut pool = start(&stream);
        let overwrites = turn_the_ring(&mut pool, 300);
        (pool, overwrites)
    };
    let geometry = geometry(&pool.parameters);
    let base = pool.image();
    let before = stream.operation_count();
    match window.as_str() {
        "unmount" => {
            let mut session = pool.session.take().expect("会话");
            let parameters = pool.parameters.clone();
            let devices = &mut pool.devices;
            let raised = stream.record_entry(RecordedPublishEntry::Unmount, || {
                singlefs_core::mount::unmount(&parameters, devices, &mut session.allocator, &mut session.current, ShadowLedger::On).expect("卸载")
            });
            if let singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised) = &raised {
                let roots: Vec<_> = raised.publishes.iter().map(|publish| publish.root).collect();
                for root in &roots {
                    pool.note_root_of_the_current_content(root);
                }
            }
            pool.session = Some(session);
        }
        "crash_remount" | "nofile_remount" => {
            pool.crash_and_mount_writable().expect("崩了再挂");
        }
        other => panic!("没有这个窗口 {other}"),
    }
    let recorded = stream.retained_operations()[before..].to_vec();
    let (writes, segments) = writes_and_segments_of_the_window(&stream, before, &recorded, &geometry);
    let tearable = TearableInPlaceOverwrites::of(&base, &writes);
    let tearable_journal = tearable.write_indexes().iter().filter(|index| writes[**index].kind == StepKind::JournalRecord).count();
    let expand_all = |_segment_index: usize, _segment: &[usize]| singlefs_harness::crash::Layer0SegmentExpansion::EveryProperSubset;
    let expand_small = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= 2 { singlefs_harness::crash::Layer0SegmentExpansion::EveryProperSubset } else { singlefs_harness::crash::Layer0SegmentExpansion::NotExpanded }
    };
    let expansion: &dyn Fn(usize, &[usize]) -> singlefs_harness::crash::Layer0SegmentExpansion = if estimate { &expand_small } else { &expand_all };
    let states = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, expansion);
    let full_states = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    println!(
        "Y6 window={window} overwrites_to_turn={overwrites} writes={} segments={} tearable_journal_writes={tearable_journal} tearable_total={} states_this_run={states} full_states={full_states} estimate={estimate}",
        writes.len(),
        segments.len(),
        tearable.write_indexes().len()
    );
    assert!(full_states <= STATES_AT_MOST, "一段历史的全量状态数 {full_states} 不超过 10⁶");
    let judged_root = writes.iter().enumerate().filter(|(_, write)| write.kind == StepKind::RootRecordFua).map(|(index, _)| index).last().expect("窗口里有根槽写");
    // 只做过 mkfs 的池上每一代根下面都没有文件：不给版本表（oracle 对表外的根只许报没有文件）。
    let versions = if window == "nofile_remount" { Vec::new() } else { pool.published_versions() };
    let parallelism = Layer0Parallelism::from_environment();
    let mut enumerated_writes: Option<Vec<RetainedWrite>> = None;
    let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
    let mut torn_journal_states = 0u64;
    // 再可写挂载那一遍只挂「有一条撕开的 journal 记录落了盘」的状态（`OPUS_Y6_REMOUNT_ALL=1` 时每个状态都挂）：
    // 枚举与恢复之后的 oracle、checker 照层 0 的枚举域一个不落，挂载那一遍是这条腿另加的观测，按候选取样。
    let remount_all = std::env::var("OPUS_Y6_REMOUNT_ALL").is_ok();
    let mut collect = |image: &singlefs_harness::crash::CrashImage<'_>, _report: &singlefs_core::recovery::RecoveryReport, _counts: &mut singlefs_harness::crash::Layer0ObserverCounts| {
        let table = enumerated_writes.get_or_insert_with(|| image.writes.to_vec());
        let torn_journal_landed = image.persisted.iter().enumerate().skip(writes.len()).any(|(index, persisted)| *persisted && table[index].kind == StepKind::JournalRecord);
        if torn_journal_landed {
            torn_journal_states += 1;
        }
        if torn_journal_landed || remount_all {
            persisted_sets.push(image.persisted.clone());
        }
    };
    let tally = enumerate_layer0_in_state_slices(&base, &writes, &segments, judged_root, &versions, expansion, parallelism, Some(&mut collect), &Layer0Resume::NoProgressFile);
    println!(
        "Y6 window={window} states={} violations={} first_violation={:?} checker_violated={:?} checker_first={:?} failed_states={} torn_journal_states={torn_journal_states} remount_candidates={} seconds_enumerate={:.1}",
        tally.states,
        tally.violations,
        tally.first_violation,
        tally.checker_violated_states,
        tally.checker_first_violation,
        tally.failed_states,
        persisted_sets.len(),
        started.elapsed().as_secs_f64()
    );
    let enumerated_writes = enumerated_writes.expect("至少一个状态");
    let worker_threads = parallelism.worker_threads.get();
    let chunk_length = persisted_sets.len().div_ceil(worker_threads).max(1);
    let parameters = pool.parameters.clone();
    let results: Vec<(u64, u64, u64, Vec<String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = persisted_sets
            .chunks(chunk_length)
            .map(|chunk| {
                let base = &base;
                let writes = &enumerated_writes;
                let parameters = &parameters;
                scope.spawn(move || {
                    let (mut mounted, mut refused, mut red) = (0u64, 0u64, 0u64);
                    let mut examples = Vec::new();
                    for persisted in chunk {
                        let mut image = base.clone();
                        let landed: Vec<RetainedWrite> = writes.iter().zip(persisted).filter(|(_, is)| **is).map(|(write, _)| write.clone()).collect();
                        image.apply_writes(&landed);
                        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
                            .devices
                            .iter()
                            .map(|(identity, sparse)| {
                                let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
                                device.image = sparse.clone();
                                (*identity, device)
                            })
                            .collect();
                        match mount_writable(parameters, &mut devices) {
                            Ok(_) => {
                                mounted += 1;
                                let after = MemoryPool { devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(), device_size_in_bytes: image.device_size_in_bytes };
                                let violations = checker_violations_on(&after);
                                if !violations.is_empty() {
                                    red += 1;
                                    if examples.len() < 3 { examples.push(format!("checker_after_mount {:?}", violations.first())); }
                                }
                            }
                            Err(error) => {
                                refused += 1;
                                if examples.len() < 3 { examples.push(format!("mount_refused {}", format!("{error:?}").chars().take(200).collect::<String>())); }
                            }
                        }
                    }
                    (mounted, refused, red, examples)
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程")).collect()
    });
    let mounted: u64 = results.iter().map(|result| result.0).sum();
    let refused: u64 = results.iter().map(|result| result.1).sum();
    let red: u64 = results.iter().map(|result| result.2).sum();
    let examples: Vec<&String> = results.iter().flat_map(|result| result.3.iter()).take(6).collect();
    println!("Y6 window={window} remount mounted={mounted} refused={refused} checker_red_after_mount={red} examples={examples:?} seconds_total={:.1}", started.elapsed().as_secs_f64());
}
