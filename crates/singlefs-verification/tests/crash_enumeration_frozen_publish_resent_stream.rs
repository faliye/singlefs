//! 验证包模块：crash、crash_state_enumeration、crash_amplification
//! 一次发布写盘报错、冻结，原样重发，在重发的每个写上崩（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」；D16（发布语义） 已定项 17）：
//! 第一个文件之后崩了再可写挂载，覆盖写一次（基线），下一次覆盖写在落盘途中报错、冻结，再 `MountedSession::resend_the_frozen_publish`。录失败那一遍与重发那一遍，
//! 枚举每个崩溃状态：恢复 + 多版本 oracle（基线那一版或这次覆盖写那一版）+ 池级 checker + 记录核对器；另在每个状态上可写挂载一次，挂完的盘过池级 checker。两种报错位置各一条用例：
//! - 第二块盘的日志记录写报错：单元已过屏障，第一块盘的记录已落；重发把单元、记录原地再写一遍。
//! - 最后一道屏障报错：根与系统配置轮换都已写过；重发把这些原地再写一遍，根槽在同一条崩溃状态里可以是旧的、新的或撕裂的。
//!
//! **比已有的流多罩了什么**：已有的流里每次发布只落一遍（`publish_failure_resent_unchanged` 那一格核的是重发的字节、不枚举崩溃状态）。这两条第一次枚举失败那一遍留下的写与重发那一遍的写同在一段里的状态：
//! 记录已落、单元正在重写；根与轮换已写过、重发的单元与记录正在原地再写。多跑的步是每个状态上的可写挂载与挂完的盘上那一遍池级 checker。
//!
//! 每一段按甲二展开（[`in_place_subsets_with_copy_on_write_none_or_all_expansion`]）：原地写任意组合、单元写全不落或全落。重发的单元写逐字节同失败那一遍、失败那一遍已过屏障，
//! 只落一部分读出来与全落相同，所以这里的单元写不展开不丢状态；失败那一遍之前的单元写只落一部分归已有的全量流。
//!
//! 登记成崩溃枚举用例；判段的工作线程取登记值（`crash_state_enumeration::crash_point_replay_worker_threads`），片并行在枚举引擎里。
//! 多机配置开着 GPU 时那两条用例的判段走 GPU 判器，没编进 gpu 特性的二进制不判；每条流另有一条在 CPU 判器上逐状态判的小用例（取样那一支不经选路），改坏发布路径的变异行点名它。

mod common;
mod common_admission_pool;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use common_admission_pool::{
    content_of, device_bytes_of_the_384_slot_small_devices,
    fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_devices_on, DeviceOverASparseImage, PoolUnderTest, FIRST_WRITE_TIME_SECONDS,
    OVERWRITE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::system_configuration::JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION;
use singlefs_core::transaction::FirstFile;
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites,
    in_place_subsets_with_copy_on_write_none_or_all_expansion, CrashPointReplayObserverCounts,
    CrashPointReplaySegmentExpansion, CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_verification::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObserverDeduplication,
    ReadRangeOnTheCrashImage,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer,
    sample_crash_states_on_the_cpu_with_threads_against_the_version_table, CrashPointReplayItem,
    CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_verification::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice, SparseDevice,
};
use singlefs_verification::mount_transparency::{
    image_after_the_refusal_into_the_undecidable_witness,
    print_the_writable_mount_refusals_into_the_undecidable_witness,
    WritableMountRefusedOnTheCrashDisks,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-verification/tests/crash_enumeration_frozen_publish_resent_stream.rs";

/// 挂载之后观察者判干净时加的计数器名。
const MOUNTED_AND_CHECKER_CLEAN_COUNTER: &str = "mounted_and_checker_clean";

/// 一次发布在哪里报错。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WhereThePublishFails {
    JournalWriteOnTheSecondDevice,
    LastBarrier,
}

impl WhereThePublishFails {
    /// 这条流的名字（崩溃点重放项名第一个 `::` 之前那一截，来历表按它查）。
    const fn flow_name(self) -> &'static str {
        match self {
            WhereThePublishFails::JournalWriteOnTheSecondDevice => {
                "frozen_publish_resent_after_a_failed_journal_write"
            }
            WhereThePublishFails::LastBarrier => {
                "frozen_publish_resent_after_a_failed_last_barrier"
            }
        }
    }

    /// 录下的写里根槽写有几次：失败那一遍走到根槽写的多一次。
    const fn root_writes_recorded(self) -> usize {
        match self {
            WhereThePublishFails::JournalWriteOnTheSecondDevice => 1,
            WhereThePublishFails::LastBarrier => 2,
        }
    }
}

/// 用例摆的故障，两块盘共用一份。
#[derive(Debug, Default)]
struct FaultsOfTheFlow {
    /// 落在第二块盘日志环里的下一个写报错一次。
    next_journal_write_on_the_second_device_fails: AtomicBool,
    /// 下面这么多道屏障放过之后的那一道报错一次。
    barrier_fails_after_those_let_through: AtomicBool,
    barriers_let_through: AtomicU64,
    /// 两块盘过了几道屏障：量一次发布发几道。
    barriers_passed: AtomicU64,
}

/// 用例摆的块设备错。
fn error_injected_by_the_test(what: &str) -> BlockDeviceError {
    BlockDeviceError::InputOutput(std::io::Error::other(format!("用例摆的{what}错")))
}

/// 录制盘外面包一层用例摆的故障：报错时什么都不写、不记进录制流。
struct RecordingDeviceFailingOnRequest {
    identity: DeviceIdentity,
    recording: RecordingBlockDevice<SparseBlockDevice>,
    faults: Arc<FaultsOfTheFlow>,
    journal_ring_start_in_bytes: u64,
    journal_ring_bytes: u64,
}

impl RecordingDeviceFailingOnRequest {
    fn is_a_write_into_the_journal_ring(&self, offset: DeviceOffsetInBytes) -> bool {
        offset.0 >= self.journal_ring_start_in_bytes
            && offset.0 - self.journal_ring_start_in_bytes < self.journal_ring_bytes
    }
}

impl BlockDevice for RecordingDeviceFailingOnRequest {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.recording.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if self.identity == DeviceIdentity(1)
            && self.is_a_write_into_the_journal_ring(offset)
            && self
                .faults
                .next_journal_write_on_the_second_device_fails
                .swap(false, Ordering::SeqCst)
        {
            return Err(error_injected_by_the_test("写"));
        }
        self.recording.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.recording.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.recording.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        if self
            .faults
            .barrier_fails_after_those_let_through
            .load(Ordering::SeqCst)
        {
            let is_let_through = self
                .faults
                .barriers_let_through
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                    left.checked_sub(1)
                })
                .is_ok();
            if !is_let_through {
                self.faults
                    .barrier_fails_after_those_let_through
                    .store(false, Ordering::SeqCst);
                return Err(error_injected_by_the_test("屏障"));
            }
        }
        self.faults.barriers_passed.fetch_add(1, Ordering::SeqCst);
        self.recording.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.recording.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.recording.size_in_bytes()
    }
}

impl DeviceOverASparseImage for RecordingDeviceFailingOnRequest {
    fn sparse_image(&self) -> &SparseDevice {
        &self.recording.wrapped_device().image
    }
}

/// 录好的这一段：报错那一次之前的镜像、失败那一遍与重发那一遍的写表与段、最后一次根槽写、每一版的第一个文件。
struct RecordedResend {
    parameters: MakeFilesystemParameters,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    last_root_write: usize,
    versions: Vec<PublishedVersion>,
}

/// 起池（第一个文件）、崩了再可写挂载、覆盖写一次（基线，顺带量一次发布几道屏障），照 `fails_at` 摆好故障再覆盖写一次（报错、冻结），重发。录失败那一遍与重发那一遍。
fn record_the_failed_publish_and_its_resend(fails_at: WhereThePublishFails) -> RecordedResend {
    let stream = SharedStream::retaining_contents();
    let stream_for_the_devices = stream.clone();
    let faults = Arc::new(FaultsOfTheFlow::default());
    let faults_of_the_devices = Arc::clone(&faults);
    let parameters = parameters_of_the_384_slot_small_devices();
    let journal_ring_start_in_bytes = JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION
        .to_device_offset()
        .0;
    let journal_ring_bytes = parameters.geometry.journal_ring_bytes;
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters,
        device_bytes_of_the_384_slot_small_devices(),
        move |identity, device| RecordingDeviceFailingOnRequest {
            identity,
            recording: RecordingBlockDevice::with_shared_stream(
                identity,
                device,
                stream_for_the_devices.clone(),
            ),
            faults: Arc::clone(&faults_of_the_devices),
            journal_ring_start_in_bytes,
            journal_ring_bytes,
        },
    );
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    let mut session = pool.session.take().expect("挂载之后的会话");
    let baseline_content = content_of(OVERWRITE_BYTES, 701);
    let barriers_before_the_baseline = faults.barriers_passed.load(Ordering::SeqCst);
    session
        .publish_user_change(
            &mut pool.devices,
            UserChange::Overwrite(FirstFile {
                content: &baseline_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 701,
            }),
        )
        .expect("基线那一次覆盖写");
    let barriers_of_one_publish =
        faults.barriers_passed.load(Ordering::SeqCst) - barriers_before_the_baseline;
    pool.content_of_the_current_version = baseline_content;
    let root_of_the_baseline = *session.current.root();
    pool.note_root_of_the_current_content(&root_of_the_baseline);
    let base = pool.image();
    let operations_before_the_failure = stream.operation_count();
    match fails_at {
        WhereThePublishFails::JournalWriteOnTheSecondDevice => {
            faults
                .next_journal_write_on_the_second_device_fails
                .store(true, Ordering::SeqCst);
        }
        WhereThePublishFails::LastBarrier => {
            faults
                .barriers_let_through
                .store(barriers_of_one_publish - 1, Ordering::SeqCst);
            faults
                .barrier_fails_after_those_let_through
                .store(true, Ordering::SeqCst);
        }
    }
    let resent_content = content_of(OVERWRITE_BYTES, 702);
    session
        .publish_user_change(
            &mut pool.devices,
            UserChange::Overwrite(FirstFile {
                content: &resent_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 702,
            }),
        )
        .expect_err("摆好的故障：这次发布落盘途中报错");
    assert!(
        session.allocator.frozen_publish().is_some(),
        "落盘途中报错：那一次冻结在分配器上"
    );
    let is_resent = session
        .resend_the_frozen_publish(&mut pool.devices)
        .expect("故障只报一次：重发这一遍做得成");
    assert!(is_resent, "冻结着一次发布");
    pool.content_of_the_current_version = resent_content;
    let root_of_the_resend = *session.current.root();
    pool.note_root_of_the_current_content(&root_of_the_resend);
    let recorded = stream.retained_operations()[operations_before_the_failure..].to_vec();
    let (writes, segments) =
        writes_and_segments(&recorded, &fixed_geometry_of_the_384_slot_small_devices());
    let root_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_writes.len(),
        fails_at.root_writes_recorded(),
        "失败那一遍走到根槽写的多一次"
    );
    let last_root_write = *root_writes.last().expect("重发那一遍有根槽写");
    let versions = pool.published_versions();
    RecordedResend {
        parameters: pool.parameters.clone(),
        base,
        writes,
        segments,
        last_root_write,
        versions,
    }
}

/// 观察者「崩溃之后可写挂载」：把这个状态持久的写落到基线上、可写挂载一次（空间准入判着），可写挂载被拒时只认乙入判不出那一拒（`mount_transparency::image_after_the_refusal_into_the_undecidable_witness`），别的拒当场 panic；挂完的盘（被拒的是没改过的盘）交回，池级 checker 在判段判。
// gpu-exempt: writable-mount
fn mount_writable_after_a_crash_in_the_resend(
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let mut materialized = base.clone();
    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
        if *is_persisted {
            write.contents.apply_to(
                materialized
                    .devices
                    .get_mut(&write.device)
                    .expect("写落在池里的盘上"),
                write.offset,
            );
        }
    }
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = plain_devices_on(&materialized)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    match mount_writable_with_space_admission(
        parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    ) {
        Ok(_) => {
            let disks_after_the_observer = MemoryPool {
                devices: devices
                    .iter()
                    .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                    .collect(),
                device_size_in_bytes: materialized.device_size_in_bytes,
            };
            let (writes_on_the_crash_image, _segments_of_the_mount) = writes_and_segments(
                &stream.retained_operations(),
                &fixed_geometry_of_the_384_slot_small_devices(),
            );
            ImageAfterTheObserver {
                writes_on_the_crash_image,
                disks_after_the_observer,
                ranges_read_on_the_crash_image:
                    ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(&stream.read_ranges()),
                invariants_judged:
                    InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
                counter_when_the_pool_checker_is_clean: MOUNTED_AND_CHECKER_CLEAN_COUNTER
                    .to_string(),
            }
        }
        Err(refusal) => image_after_the_refusal_into_the_undecidable_witness(
            &WritableMountRefusedOnTheCrashDisks {
                refusal,
                devices,
                stream,
            },
            &materialized,
            report,
            &parameters.filesystem_identifier,
            &fixed_geometry_of_the_384_slot_small_devices(),
            MOUNTED_AND_CHECKER_CLEAN_COUNTER,
            counts,
        ),
    }
}

/// 按 `expansion` 枚举失败那一遍与重发那一遍的崩溃状态，挂可写挂载那个观察者；只录那一趟交回 `None`。
fn enumerate_the_failed_publish_and_its_resend(
    recorded: &RecordedResend,
    flow_name: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let base = &recorded.base;
    let parameters = &recorded.parameters;
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        mount_writable_after_a_crash_in_the_resend(base, parameters, image, report, counts)
    };
    let item_name = format!("{flow_name}::in_place_subsets");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.last_root_write,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        expansion,
        &[],
        // 观察者另读的输入：挂载参数（不随状态变，进版本与去重键时只取它的调试写法的摘要）
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "writable_mount_after_the_crash",
            extra_inputs_digest: singlefs_verification::sha256::sha256_digest(
                format!("{parameters:?}").as_bytes(),
            ),
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        }],
    )
    .judged()
}

/// 判器先判、计数后核：恢复之后 oracle 与池级 checker 0 违例、每个状态都判过：挂得上的挂完干净、被乙入判不出拒的原样干净。
fn assert_every_state_recovers_a_published_version_and_remounts_clean(
    flow_name: &str,
    tally: &CrashPointReplayTally,
    states: u64,
) {
    print_the_writable_mount_refusals_into_the_undecidable_witness(flow_name, tally);
    assert_eq!(
        (tally.violations, tally.first_violation.clone()),
        (0, None),
        "恢复之后 oracle 0 违例：读出的是基线那一版或这次覆盖写那一版，崩在重发的任何一个写上都一样"
    );
    assert_eq!(
        tally.checker_violated_states.values().sum::<u64>(),
        0,
        "恢复之后池级 checker 0 违例：{:?}",
        tally.checker_first_violation
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        tally
            .observer_counts
            .count_named(MOUNTED_AND_CHECKER_CLEAN_COUNTER),
        states,
        "每个崩溃状态都判过：挂得上的挂载之后、被乙入判不出拒的原样，池级 checker 0 违例",
    );
}

/// 录、枚举、打计数行 `count_line`、判。
fn judge_every_crash_state_of_the_resend(fails_at: WhereThePublishFails, count_line: &str) {
    let flow_name = fails_at.flow_name();
    let recorded = record_the_failed_publish_and_its_resend(fails_at);
    let states = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &in_place_subsets_with_copy_on_write_none_or_all_expansion,
    );
    let Some(tally) = enumerate_the_failed_publish_and_its_resend(
        &recorded,
        flow_name,
        &in_place_subsets_with_copy_on_write_none_or_all_expansion,
    ) else {
        return;
    };
    println!(
        "{count_line} states={} closed_form={states} exhaustive={} violations={} failed={} checker_violations={} mounted_and_checker_clean={} checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states == states,
        tally.violations,
        tally.failed_states,
        tally.checker_violated_states.values().sum::<u64>(),
        tally
            .observer_counts
            .count_named(MOUNTED_AND_CHECKER_CLEAN_COUNTER),
        tally.checker_counts_by_invariant()
    );
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        states,
        "每一段按甲二展开，状态数与按崩溃点重放口径另算的相同",
    );
    assert_every_state_recovers_a_published_version_and_remounts_clean(flow_name, &tally, states);
}

/// 失败那一遍与重发那一遍按同一个甲二展开在 CPU 判器上逐状态判：多机配置开不开 GPU、编没编进 gpu 特性都在 CPU 上判——每个状态两遍恢复、oracle、池级 checker。
/// 崩溃之后的可写挂载在登记的那一条用例上判。只录那一趟（分片）什么都不核。线程数取每台默认重活线程数。
// gpu-exempt: cpu-judge-mutation-rows
fn judge_the_states_of_the_resend_on_the_cpu_judge(fails_at: WhereThePublishFails) {
    let flow_name = fails_at.flow_name();
    let recorded = record_the_failed_publish_and_its_resend(fails_at);
    let state_count = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &in_place_subsets_with_copy_on_write_none_or_all_expansion,
    );
    let item_name = format!("{flow_name}::in_place_subsets_on_the_cpu_judge");
    let red_states: Mutex<Vec<(u64, Vec<String>)>> = Mutex::new(Vec::new());
    let Some(states_of_the_flow) =
        sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
            CrashPointReplayItem {
                name: &item_name,
                setting_file: THIS_FILE,
            },
            &recorded.base,
            &recorded.writes,
            &recorded.segments,
            recorded.last_root_write,
            CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
            &in_place_subsets_with_copy_on_write_none_or_all_expansion,
            0..state_count,
            singlefs_verification::heavy_threads::default_heavy_threads(),
            &|ordinal, _image, judgement| {
                let is_red = judgement.consulted_oracle.is_some()
                    || judgement.pool_checker.iter().any(|(_invariant, verdict)| {
                        matches!(verdict, InvariantVerdict::Violated(_))
                    });
                if is_red {
                    red_states
                        .lock()
                        .expect("判定线程不 panic")
                        .push((ordinal, judgement.red_items()));
                }
            },
        )
        .judged()
    else {
        return;
    };
    assert_eq!(states_of_the_flow, state_count, "甲二的状态数");
    let mut red_states = red_states.into_inner().expect("判定线程不 panic");
    red_states.sort();
    assert_eq!(
        red_states.len(),
        0,
        "每个状态恢复出基线那一版或这次覆盖写那一版、池级 checker 不红（判法同登记的那两条用例）：{:?}",
        red_states.first()
    );
}

/// 第二块盘的日志记录写报错、冻结、重发：每个状态在 CPU 判器上判（不经选路）。
#[test]
fn the_resend_after_a_failed_journal_write_judged_on_the_cpu_judge_keeps_every_state_on_a_published_version(
) {
    judge_the_states_of_the_resend_on_the_cpu_judge(
        WhereThePublishFails::JournalWriteOnTheSecondDevice,
    );
}

/// 最后一道屏障报错、冻结、重发：每个状态在 CPU 判器上判（不经选路）。
#[test]
fn the_resend_after_a_failed_last_barrier_judged_on_the_cpu_judge_keeps_every_state_on_a_published_version(
) {
    judge_the_states_of_the_resend_on_the_cpu_judge(WhereThePublishFails::LastBarrier);
}

/// 第二块盘的日志记录写报错、冻结、重发：判完打计数行 `CRASH_POINT_REPLAY_FROZEN_PUBLISH_RESENT_AFTER_A_FAILED_JOURNAL_WRITE`，
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:frozen-publish-resent-after-a-failed-journal-write`）的 `count-line=`、`exhaustive=`、`threads=` 都按它判。
#[test]
fn every_crash_state_of_the_resend_after_a_failed_journal_write_recovers_a_published_version_and_remounts_clean(
) {
    judge_every_crash_state_of_the_resend(
        WhereThePublishFails::JournalWriteOnTheSecondDevice,
        "CRASH_POINT_REPLAY_FROZEN_PUBLISH_RESENT_AFTER_A_FAILED_JOURNAL_WRITE",
    );
}

/// 最后一道屏障报错、冻结、重发：判完打计数行 `CRASH_POINT_REPLAY_FROZEN_PUBLISH_RESENT_AFTER_A_FAILED_LAST_BARRIER`，
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:frozen-publish-resent-after-a-failed-last-barrier`）的 `count-line=`、`exhaustive=`、`threads=` 都按它判。
#[test]
fn every_crash_state_of_the_resend_after_a_failed_last_barrier_recovers_a_published_version_and_remounts_clean(
) {
    judge_every_crash_state_of_the_resend(
        WhereThePublishFails::LastBarrier,
        "CRASH_POINT_REPLAY_FROZEN_PUBLISH_RESENT_AFTER_A_FAILED_LAST_BARRIER",
    );
}
