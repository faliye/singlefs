//! 真写满的小池上，运行库 `accept` 被空间拒时当场推抬 F 再判（D16（发布语义） 已定项 1「准入」「抬 F 的上限」两行）：
//! 前台真写到被拒（320 MiB 两块盘，日志环 64 MiB，`N₀` 与水位取产品默认），另一次挂载里干净地 unlink，紧接着的那一拍维护之后 `accept` 写得进、收下的发得成、再挂载读得出来。
//! 三种填法各填一次（`OnceLock`，填法确定，只按前台调用推，钟是用例摆的），每条用例拷盘、从「进程退出、重新挂载」接着做，互不影响。
//! 崩溃只取发布边界（丢运行库、盘原样留着）。抬 F 那一串自己的崩溃状态在验证包 `crash_enumeration_floor_raise_pushed_by_admission.rs`，这里只钉 `accept` 发的段序列与它枚举的那条路相同。
mod common_pool_on_memory_devices;

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use common_pool_on_memory_devices::{
    parameters_of_the_small_pool, small_pool_after_a_sequential_write_in_an_earlier_mount,
    DeviceFaults, PoolOnMemoryDevices, RecordedDeviceOperation, RecordedMemoryDevice,
    RecordedStream, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::compaction_plan::CallerMonotonicClockReading;
use singlefs_core::mount::FloorRaiseStop;
use singlefs_core::mounted_session::{
    AcceptRefused, ChangeAccepted, PublishReportedByTheSession, UserChange, UserChangeRefused,
};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::ROOT_RING_REGIONS;
use singlefs_mount_runtime::{
    ClockOfTheRuntime, MaintenanceEvent, MountedRuntime, ReceiverOfTheMaintenanceEvents,
    RuntimeMounted, RuntimeParameters, RuntimeRefused, SessionAndDevices,
};

type DevicesOfTheTest = Vec<(DeviceIdentity, RecordedMemoryDevice)>;
type RuntimeOfTheTest = MountedRuntime<RecordedMemoryDevice>;

const ONE_PERIOD: Duration = Duration::from_secs(30);
const ONE_POLL: Duration = Duration::from_secs(1);
/// 一个 32 单元的文件占一段（数据单元两槽、一槽 16 KiB、一段 64 槽）。
const UNITS_OF_A_THIRTY_TWO_UNIT_FILE: usize = 32;
/// 填盘写的文件数上限：比这个池放得下的多，到了就是填法没走到被拒。
const FILES_AT_MOST: usize = 400;
/// 前缀文件的份数：删 1、4 个用前面几个，再留一个 32 单元的不删。
const ONE_UNIT_FILES_OF_THE_PREFIX: usize = 4;
const THIRTY_TWO_UNIT_FILES_OF_THE_PREFIX: usize = 5;
/// 从大到小的填法：每档写到被拒再换下一档。
const DESCENDING_UNITS: [usize; 6] = [1024, 256, 64, 16, 4, 1];
/// 算「最大的那个文件」的下限：这一档以上才有几十个删除步，够留出 D16（发布语义） 已定项 1 那 4 个状态。
const UNITS_OF_THE_LARGEST_FILE_AT_LEAST: usize = 64;
/// 带整理的填法每个文件之后那一拍接着推几拍。
const POLLS_OF_A_MAINTENANCE_WHILE_FILLING_AT_MOST: usize = 8;
/// 回退候选集保留的不同状态数（D16（发布语义） 已定项 1）。
const ROLLBACK_STATES_KEPT: usize = 4;
const CONTENT_WRITTEN_AFTER_THE_UNLINK: &[u8] = b"a foreground write after the unlink";
/// 比这个池的单元区还大的写的数据单元数：8192 个单元要 16384 槽，单元区 15360 槽，怎么推都放不下。
const UNITS_OF_A_WRITE_THAT_NEVER_FITS: usize = 8192;
/// 崩溃循环崩几次：放不下的写推抬 F 那一形（每次 2 串、6 次空发布）只在 F 抬得动的那几次挂载上出现，崩到这么多次把它们都走到。
const CRASHES_OF_THE_CRASH_LOOP: usize = 8;
/// 崩溃循环每次挂载试几次放不下的写。
const ACCEPTS_PER_MOUNT_OF_THE_CRASH_LOOP: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShapeOfTheFill {
    ThirtyTwoUnitFiles,
    DescendingSizes,
    DescendingSizesWithCompaction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SizeOfTheUnlinkedFile {
    OneUnit,
    ThirtyTwoUnits,
    Largest,
}

#[derive(Debug, Default)]
struct ClockMovedByTheTest {
    reading: Mutex<Duration>,
}

impl ClockMovedByTheTest {
    fn advance_by(&self, elapsed: Duration) {
        *self.reading.lock().expect("钟的锁里不 panic") += elapsed;
    }
}

impl ClockOfTheRuntime for ClockMovedByTheTest {
    fn reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(*self.reading.lock().expect("钟的锁里不 panic"))
    }
}

/// 记 `accept` 推抬 F 那一个事件：每次事件带的串数。
#[derive(Default)]
struct EventsOfTheTest {
    floor_raises_of_each_event: Mutex<Vec<usize>>,
}

impl ReceiverOfTheMaintenanceEvents for EventsOfTheTest {
    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
        match event {
            MaintenanceEvent::FloorRaisesPushedByAnAccept(floor_raises) => self
                .floor_raises_of_each_event
                .lock()
                .expect("事件的锁里不 panic")
                .push(floor_raises.len()),
            MaintenanceEvent::PendingChangesPublished(_)
            | MaintenanceEvent::PublishWhenDueRefused(_)
            | MaintenanceEvent::CompactionCallReturned(_)
            | MaintenanceEvent::CompactionCallRefused(_)
            | MaintenanceEvent::FrozenPublishResent
            | MaintenanceEvent::FrozenPublishResendFailed(_)
            | MaintenanceEvent::MaintenanceThreadPanicked { .. } => {}
        }
    }
}

struct MountedPool {
    runtime: RuntimeOfTheTest,
    stream: RecordedStream,
    faults: DeviceFaults,
    clock: Arc<ClockMovedByTheTest>,
    events: Arc<EventsOfTheTest>,
}

/// 挂上（不起维护线程，用例一拍一拍推）。
fn mounted(devices: DevicesOfTheTest, stream: RecordedStream, faults: DeviceFaults) -> MountedPool {
    let clock = Arc::new(ClockMovedByTheTest::default());
    let events = Arc::new(EventsOfTheTest::default());
    let clock_of_the_runtime: Arc<ClockMovedByTheTest> = Arc::clone(&clock);
    let receiver: Arc<EventsOfTheTest> = Arc::clone(&events);
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters_of_the_small_pool(),
        devices,
        RuntimeParameters::by_default(),
        None,
        clock_of_the_runtime,
        receiver,
    )
    .expect("写满的小池照样可写挂载");
    MountedPool {
        runtime,
        stream,
        faults,
        clock,
        events,
    }
}

/// 崩溃：盘从运行库里拿出来，丢掉运行库（只停线程，不写盘）。
fn devices_after_a_crash(runtime: RuntimeOfTheTest) -> DevicesOfTheTest {
    let devices = runtime
        .with_the_session_and_devices(|locked| std::mem::take(&mut locked.devices))
        .expect("运行库没坏");
    drop(runtime);
    devices
}

fn inode_watermark(runtime: &RuntimeOfTheTest) -> u64 {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .current
                .version_of_trees()
                .expect("有树的一版")
                .inode_number_watermark()
        })
        .expect("运行库没坏")
}

fn new_inode(runtime: &RuntimeOfTheTest) -> Option<InodeNumber> {
    let inode = InodeNumber(inode_watermark(runtime));
    runtime
        .publish_user_change(UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        })
        .ok()
        .map(|_published| inode)
}

fn whole_write_goes_through(runtime: &RuntimeOfTheTest, inode: InodeNumber, units: usize) -> bool {
    let content = vec![0x5A_u8; units * data_unit_payload_capacity()];
    runtime
        .publish_user_change(UserChange::SequentialWriteOfAnInode {
            inode,
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        })
        .is_ok()
}

/// 填法要删、要写的那几个文件。
struct FilesOfTheFill {
    one_unit: Vec<InodeNumber>,
    thirty_two_units: Vec<InodeNumber>,
    /// 从大到小的填法里最早写成的那个大文件；只写 32 单元的填法没有。
    largest: Option<InodeNumber>,
    /// 不删、拿来试 `accept` 的那一个。
    surviving: InodeNumber,
}

/// 一拍维护（带整理的填法每个文件之后那一拍），这一趟没完就接着推几拍。
fn maintenance_while_filling(mounted_pool: &MountedPool) {
    mounted_pool.clock.advance_by(ONE_PERIOD);
    let mut pass = mounted_pool
        .runtime
        .run_one_maintenance_pass()
        .expect("运行库没坏");
    let mut polls = 0;
    while pass.is_a_trip_under_way && polls < POLLS_OF_A_MAINTENANCE_WHILE_FILLING_AT_MOST {
        mounted_pool.clock.advance_by(ONE_POLL);
        pass = mounted_pool
            .runtime
            .run_one_maintenance_pass()
            .expect("运行库没坏");
        polls += 1;
    }
}

/// 清掉录制流（填盘的写用不着，留着占内存）。
fn clear_the_stream(stream: &RecordedStream) {
    stream.lock().expect("录制流的锁里不 panic").clear();
}

/// 前台真写到被拒：前缀文件之后照填法写；带整理的那种被拒之后跑一拍再试一次，还拒才换下一档。建不出 inode 就停。
fn fill_through_the_runtime(mounted_pool: &MountedPool, shape: ShapeOfTheFill) -> FilesOfTheFill {
    let runtime = &mounted_pool.runtime;
    let mut one_unit = Vec::new();
    for _file in 0..ONE_UNIT_FILES_OF_THE_PREFIX {
        let inode = new_inode(runtime).expect("空池上建 inode");
        assert!(
            whole_write_goes_through(runtime, inode, 1),
            "空池上写一单元"
        );
        one_unit.push(inode);
    }
    let mut thirty_two_units = Vec::new();
    for _file in 0..THIRTY_TWO_UNIT_FILES_OF_THE_PREFIX {
        let inode = new_inode(runtime).expect("空池上建 inode");
        assert!(
            whole_write_goes_through(runtime, inode, UNITS_OF_A_THIRTY_TWO_UNIT_FILE),
            "空池上写 32 单元"
        );
        thirty_two_units.push(inode);
    }
    let surviving = *thirty_two_units.last().expect("前缀写了 32 单元的文件");
    clear_the_stream(&mounted_pool.stream);
    let runs_compaction = shape == ShapeOfTheFill::DescendingSizesWithCompaction;
    let ladder: &[usize] = match shape {
        ShapeOfTheFill::ThirtyTwoUnitFiles => &[UNITS_OF_A_THIRTY_TWO_UNIT_FILE],
        ShapeOfTheFill::DescendingSizes | ShapeOfTheFill::DescendingSizesWithCompaction => {
            &DESCENDING_UNITS
        }
    };
    let mut largest = None;
    let mut inode_not_written_yet = None;
    let mut files_written = 0_usize;
    let mut was_refused_a_new_inode = false;
    // 迭代上界：每一档写到被拒（带整理的那种多试一次），文件数不到 FILES_AT_MOST；跨轮携带的是建好没写的 inode。
    'ladder: for &units in ladder {
        let mut retried_after_the_maintenance = false;
        loop {
            assert!(
                files_written < FILES_AT_MOST,
                "{shape:?}：这个池放不下这么多文件，填法没走到被拒"
            );
            let inode = match inode_not_written_yet.take().or_else(|| new_inode(runtime)) {
                Some(inode) => inode,
                None => {
                    if runs_compaction && !retried_after_the_maintenance {
                        retried_after_the_maintenance = true;
                        maintenance_while_filling(mounted_pool);
                        continue;
                    }
                    was_refused_a_new_inode = true;
                    break 'ladder;
                }
            };
            if whole_write_goes_through(runtime, inode, units) {
                retried_after_the_maintenance = false;
                files_written += 1;
                if largest.is_none() && units >= UNITS_OF_THE_LARGEST_FILE_AT_LEAST {
                    largest = Some(inode);
                }
                if runs_compaction {
                    maintenance_while_filling(mounted_pool);
                }
                clear_the_stream(&mounted_pool.stream);
            } else {
                inode_not_written_yet = Some(inode);
                if runs_compaction && !retried_after_the_maintenance {
                    retried_after_the_maintenance = true;
                    maintenance_while_filling(mounted_pool);
                    continue;
                }
                break;
            }
        }
    }
    clear_the_stream(&mounted_pool.stream);
    println!(
        "name=fill_finished shape={shape:?} files_written={files_written} was_refused_a_new_inode={was_refused_a_new_inode} observations={:?}",
        runtime.observations(),
    );
    assert!(
        was_refused_a_new_inode,
        "{shape:?}：填到连 inode 都建不出，才是真写满"
    );
    FilesOfTheFill {
        one_unit,
        thirty_two_units,
        largest,
        surviving,
    }
}

struct FilledPool {
    pool: PoolOnMemoryDevices,
    files: FilesOfTheFill,
}

fn filled_in_an_earlier_mount(shape: ShapeOfTheFill) -> FilledPool {
    let pool = small_pool_after_a_sequential_write_in_an_earlier_mount();
    clear_the_stream(&pool.stream);
    let mounted_pool = mounted(pool.devices, Arc::clone(&pool.stream), pool.faults.clone());
    let files = fill_through_the_runtime(&mounted_pool, shape);
    let devices = devices_after_a_crash(mounted_pool.runtime);
    FilledPool {
        pool: PoolOnMemoryDevices {
            devices,
            stream: pool.stream,
            faults: pool.faults,
        },
        files,
    }
}

/// 一种填法只填一次，各用例拷它的盘。
fn filled_pool(shape: ShapeOfTheFill) -> &'static FilledPool {
    static THIRTY_TWO_UNIT_FILES: OnceLock<FilledPool> = OnceLock::new();
    static DESCENDING_SIZES: OnceLock<FilledPool> = OnceLock::new();
    static DESCENDING_SIZES_WITH_COMPACTION: OnceLock<FilledPool> = OnceLock::new();
    let cell = match shape {
        ShapeOfTheFill::ThirtyTwoUnitFiles => &THIRTY_TWO_UNIT_FILES,
        ShapeOfTheFill::DescendingSizes => &DESCENDING_SIZES,
        ShapeOfTheFill::DescendingSizesWithCompaction => &DESCENDING_SIZES_WITH_COMPACTION,
    };
    cell.get_or_init(|| filled_in_an_earlier_mount(shape))
}

fn mounted_copy_of(filled: &FilledPool) -> MountedPool {
    let copy = filled.pool.copied_with_an_empty_stream();
    mounted(copy.devices, copy.stream, copy.faults)
}

fn files_to_unlink(
    files: &FilesOfTheFill,
    size: SizeOfTheUnlinkedFile,
    count: usize,
) -> Vec<InodeNumber> {
    match size {
        SizeOfTheUnlinkedFile::OneUnit => files.one_unit[..count].to_vec(),
        SizeOfTheUnlinkedFile::ThirtyTwoUnits => files.thirty_two_units[..count].to_vec(),
        SizeOfTheUnlinkedFile::Largest => {
            assert_eq!(count, 1, "最大的那个文件只有一个");
            vec![files.largest.expect("这种填法写了大文件")]
        }
    }
}

fn unlink(inode: InodeNumber) -> UserChange<'static> {
    UserChange::Unlink {
        inode,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 10,
    }
}

fn overwrite_of_the_surviving_file(surviving: InodeNumber) -> UserChange<'static> {
    UserChange::OverwriteOfAnInode {
        inode: surviving,
        content: CONTENT_WRITTEN_AFTER_THE_UNLINK,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 20,
    }
}

/// 干净地 unlink，交回发成的每一次发布。
fn unlink_cleanly(
    runtime: &RuntimeOfTheTest,
    inode: InodeNumber,
) -> Vec<PublishReportedByTheSession> {
    runtime
        .publish_user_change(unlink(inode))
        .unwrap_or_else(|refused| panic!("写满之后 unlink 过得去：{refused:?}"))
}

/// 这一刻读 `inode` 的全部内容。
fn content_of_the_inode(runtime: &RuntimeOfTheTest, inode: InodeNumber) -> Vec<u8> {
    let length_in_bytes =
        u64::try_from(CONTENT_WRITTEN_AFTER_THE_UNLINK.len()).expect("内容长度装得进 u64");
    runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            session.session().read_the_file(
                devices.as_slice(),
                inode,
                FileOffsetInBytes(0),
                length_in_bytes,
            )
        })
        .expect("运行库没坏")
        .unwrap_or_else(|refused| panic!("读得出那个文件：{refused:?}"))
        .bytes
}

/// 拒绝的调试文字开头这一段：拒绝里带着推过的发布，全文能有上 GiB，不进测试输出。
fn leading_text_of(refused: &impl std::fmt::Debug) -> String {
    format!("{refused:?}").chars().take(800).collect()
}

/// 这一条历史有没有非要靠 `accept` 推抬 F：只写 32 单元与带整理的填法那一拍可能已经腾出来了，不钉。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FloorRaisesOfTheAccept {
    AtLeastOneSeries,
    NotPinned,
}

/// 同一条历史：干净地 unlink 这几个文件，紧接着的那一拍维护之后 `accept` 写得进，收下的发得成，崩了再挂载读得出来。
fn accept_goes_through_on_the_pass_right_after_the_unlink(
    shape: ShapeOfTheFill,
    size: SizeOfTheUnlinkedFile,
    count: usize,
    floor_raises: FloorRaisesOfTheAccept,
) {
    let filled = filled_pool(shape);
    let mounted_pool = mounted_copy_of(filled);
    let runtime = &mounted_pool.runtime;
    for inode in files_to_unlink(&filled.files, size, count) {
        unlink_cleanly(runtime, inode);
    }
    mounted_pool.clock.advance_by(ONE_POLL);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    let accepted = runtime
        .accept(overwrite_of_the_surviving_file(filled.files.surviving))
        .unwrap_or_else(|refused| {
            panic!(
                "{shape:?} {size:?} {count}：删过文件之后紧接着的那一拍写得进：{}",
                leading_text_of(&refused)
            )
        });
    match floor_raises {
        FloorRaisesOfTheAccept::AtLeastOneSeries => assert!(
            !accepted
                .floor_raises_pushed_for_space
                .floor_raises
                .is_empty(),
            "{shape:?} {size:?} {count}：这一拍什么都没腾，写得进靠的是这一次 accept 推的抬 F"
        ),
        FloorRaisesOfTheAccept::NotPinned => {}
    }
    runtime
        .publish_pending_now()
        .unwrap_or_else(|refused| panic!("收下的发得成：{refused:?}"));
    assert_eq!(
        content_of_the_inode(runtime, filled.files.surviving),
        CONTENT_WRITTEN_AFTER_THE_UNLINK,
        "发成之后现行那一版读出收下的内容"
    );
    let MountedPool {
        runtime: runtime_before_the_crash,
        stream,
        faults,
        ..
    } = mounted_pool;
    let devices = devices_after_a_crash(runtime_before_the_crash);
    let remounted = mounted(devices, stream, faults);
    assert_eq!(
        content_of_the_inode(&remounted.runtime, filled.files.surviving),
        CONTENT_WRITTEN_AFTER_THE_UNLINK,
        "崩了再挂载读得出收下的内容"
    );
}

/// 写满之后只删 1 个文件：差额压在 defer 里，D16（发布语义） 回收门槛压着差额（保留最近 4 个状态），D28（挂载期承诺量） 已定项 1 的承诺量定下差额，
/// `accept` 推抬 F 推到上限还放不下，报空间不够而不是写进去；再删 3 个文件之后同一个 `accept` 写得进、发得成，4 是这条用例选的保守数，不是门槛。
fn one_unlink_stops_at_the_floor_ceiling_and_four_go_through(
    shape: ShapeOfTheFill,
    size: SizeOfTheUnlinkedFile,
) {
    let filled = filled_pool(shape);
    let mounted_pool = mounted_copy_of(filled);
    let runtime = &mounted_pool.runtime;
    let inodes = files_to_unlink(&filled.files, size, 4);
    let (first, the_others) = inodes.split_first().expect("这条臂删 4 个文件");
    unlink_cleanly(runtime, *first);
    mounted_pool.clock.advance_by(ONE_POLL);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    let Err(refused_at_the_ceiling) =
        runtime.accept(overwrite_of_the_surviving_file(filled.files.surviving))
    else {
        panic!("{shape:?} {size:?}：只删 1 个文件，放出来的槽还在回退窗口里：收不下");
    };
    let RuntimeRefused::Refused(AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
        refusal, ..
    }) = &refused_at_the_ceiling
    else {
        panic!(
            "{shape:?} {size:?}：推过抬 F 还放不下：{}",
            leading_text_of(&refused_at_the_ceiling)
        );
    };
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = &**refusal else {
        panic!(
            "{shape:?} {size:?}：报空间不够：{}",
            leading_text_of(refusal)
        );
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
        ),
        "{shape:?} {size:?}：停在抬 F 的上限：{:?}",
        no_space.stop
    );
    for inode in the_others {
        unlink_cleanly(runtime, *inode);
    }
    mounted_pool.clock.advance_by(ONE_POLL);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    runtime
        .accept(overwrite_of_the_surviving_file(filled.files.surviving))
        .unwrap_or_else(|refused| {
            panic!(
                "{shape:?} {size:?}：删够 4 个之后写得进：{}",
                leading_text_of(&refused)
            )
        });
    runtime
        .publish_pending_now()
        .unwrap_or_else(|refused| panic!("收下的发得成：{}", leading_text_of(&refused)));
    assert_eq!(
        content_of_the_inode(runtime, filled.files.surviving),
        CONTENT_WRITTEN_AFTER_THE_UNLINK,
        "发成之后现行那一版读出收下的内容"
    );
}

#[test]
fn thirty_two_unit_files_one_one_unit_file_unlinked_accept_goes_through_on_the_next_pass() {
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::ThirtyTwoUnitFiles,
        SizeOfTheUnlinkedFile::OneUnit,
        1,
        FloorRaisesOfTheAccept::NotPinned,
    );
}

#[test]
fn thirty_two_unit_files_four_one_unit_files_unlinked_accept_goes_through_on_the_next_pass() {
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::ThirtyTwoUnitFiles,
        SizeOfTheUnlinkedFile::OneUnit,
        4,
        FloorRaisesOfTheAccept::NotPinned,
    );
}

#[test]
fn thirty_two_unit_files_one_thirty_two_unit_file_unlinked_accept_goes_through_on_the_next_pass() {
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::ThirtyTwoUnitFiles,
        SizeOfTheUnlinkedFile::ThirtyTwoUnits,
        1,
        FloorRaisesOfTheAccept::NotPinned,
    );
}

#[test]
fn thirty_two_unit_files_four_thirty_two_unit_files_unlinked_accept_goes_through_on_the_next_pass()
{
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::ThirtyTwoUnitFiles,
        SizeOfTheUnlinkedFile::ThirtyTwoUnits,
        4,
        FloorRaisesOfTheAccept::NotPinned,
    );
}

#[test]
fn descending_sizes_one_one_unit_file_unlinked_accept_stops_at_the_floor_ceiling_and_four_go_through(
) {
    one_unlink_stops_at_the_floor_ceiling_and_four_go_through(
        ShapeOfTheFill::DescendingSizes,
        SizeOfTheUnlinkedFile::OneUnit,
    );
}

#[test]
fn descending_sizes_four_one_unit_files_unlinked_accept_goes_through_on_the_next_pass() {
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::DescendingSizes,
        SizeOfTheUnlinkedFile::OneUnit,
        4,
        FloorRaisesOfTheAccept::AtLeastOneSeries,
    );
}

#[test]
fn descending_sizes_one_thirty_two_unit_file_unlinked_accept_stops_at_the_floor_ceiling_and_four_go_through(
) {
    one_unlink_stops_at_the_floor_ceiling_and_four_go_through(
        ShapeOfTheFill::DescendingSizes,
        SizeOfTheUnlinkedFile::ThirtyTwoUnits,
    );
}

#[test]
fn descending_sizes_four_thirty_two_unit_files_unlinked_accept_goes_through_on_the_next_pass() {
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::DescendingSizes,
        SizeOfTheUnlinkedFile::ThirtyTwoUnits,
        4,
        FloorRaisesOfTheAccept::AtLeastOneSeries,
    );
}

#[test]
fn descending_sizes_the_largest_file_unlinked_accept_goes_through_on_the_next_pass() {
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::DescendingSizes,
        SizeOfTheUnlinkedFile::Largest,
        1,
        FloorRaisesOfTheAccept::AtLeastOneSeries,
    );
}

#[test]
fn descending_sizes_with_compaction_one_one_unit_file_unlinked_accept_stops_at_the_floor_ceiling_and_four_go_through(
) {
    one_unlink_stops_at_the_floor_ceiling_and_four_go_through(
        ShapeOfTheFill::DescendingSizesWithCompaction,
        SizeOfTheUnlinkedFile::OneUnit,
    );
}

#[test]
fn descending_sizes_with_compaction_four_one_unit_files_unlinked_accept_goes_through_on_the_next_pass(
) {
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::DescendingSizesWithCompaction,
        SizeOfTheUnlinkedFile::OneUnit,
        4,
        FloorRaisesOfTheAccept::NotPinned,
    );
}

#[test]
fn descending_sizes_with_compaction_one_thirty_two_unit_file_unlinked_accept_stops_at_the_floor_ceiling_and_four_go_through(
) {
    one_unlink_stops_at_the_floor_ceiling_and_four_go_through(
        ShapeOfTheFill::DescendingSizesWithCompaction,
        SizeOfTheUnlinkedFile::ThirtyTwoUnits,
    );
}

#[test]
fn descending_sizes_with_compaction_four_thirty_two_unit_files_unlinked_accept_goes_through_on_the_next_pass(
) {
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::DescendingSizesWithCompaction,
        SizeOfTheUnlinkedFile::ThirtyTwoUnits,
        4,
        FloorRaisesOfTheAccept::NotPinned,
    );
}

#[test]
fn descending_sizes_with_compaction_the_largest_file_unlinked_accept_goes_through_on_the_next_pass()
{
    accept_goes_through_on_the_pass_right_after_the_unlink(
        ShapeOfTheFill::DescendingSizesWithCompaction,
        SizeOfTheUnlinkedFile::Largest,
        1,
        FloorRaisesOfTheAccept::NotPinned,
    );
}

fn current_txg(runtime: &RuntimeOfTheTest) -> CheckpointTxg {
    runtime
        .with_the_session_and_devices(|locked| {
            locked.session.session().current.root().checkpoint_txg
        })
        .expect("运行库没坏")
}

fn current_rollback_floor(runtime: &RuntimeOfTheTest) -> CheckpointTxg {
    runtime
        .with_the_session_and_devices(|locked| {
            locked.session.session().current.root().rollback_floor
        })
        .expect("运行库没坏")
}

/// 还留在根环里、不低于 F 的根属于几个用户状态：某次用户改动的 txg 起到下一次用户改动之前算一个状态（判法同 `mount_runtime_rollback_gate_margin_follows_the_root_ring_slots.rs` 的 `user_states_left_after_the_next_publishes`）。
fn user_states_left(
    runtime: &RuntimeOfTheTest,
    txgs_of_the_user_changes: &[CheckpointTxg],
) -> usize {
    let pool_parameters = parameters_of_the_small_pool();
    let root_ring_slots =
        ROOT_RING_REGIONS * pool_parameters.geometry.root_ring_slots_per_region.count();
    let oldest_root_kept =
        CheckpointTxg((current_txg(runtime).0 + 1).saturating_sub(root_ring_slots));
    runtime
        .with_the_session_and_devices(|locked| {
            let roots = readable_roots(
                locked.devices.as_slice(),
                &pool_parameters.region_devices,
                &pool_parameters.geometry,
                &pool_parameters.filesystem_identifier,
            );
            let floor = effective_rollback_floor(
                locked.devices.as_slice(),
                &pool_parameters.region_devices,
                &pool_parameters.geometry,
                &pool_parameters.filesystem_identifier,
            );
            txgs_of_the_user_changes
                .iter()
                .enumerate()
                .filter(|(index, txg)| {
                    let next = txgs_of_the_user_changes.get(index + 1);
                    roots.iter().any(|root| {
                        root.checkpoint_txg >= floor
                            && root.checkpoint_txg >= oldest_root_kept
                            && root.checkpoint_txg >= **txg
                            && next.is_none_or(|next_txg| root.checkpoint_txg < *next_txg)
                    })
                })
                .count()
        })
        .expect("运行库没坏")
}

/// 回退状态：抬 F 那一串的空发布不算状态，`accept` 推到上限之后 D16（发布语义） 已定项 1 那 4 个状态还在。大文件的 unlink 发了几十步、每步都改用户可见的树，F 抬得动也压得住。
#[test]
fn the_floor_raises_of_an_accept_leave_four_user_states_in_the_root_ring() {
    let filled = filled_pool(ShapeOfTheFill::DescendingSizes);
    let mounted_pool = mounted_copy_of(filled);
    let runtime = &mounted_pool.runtime;
    let floor_before = current_rollback_floor(runtime);
    let published = unlink_cleanly(
        runtime,
        filled.files.largest.expect("从大到小的填法写了大文件"),
    );
    let txgs_of_the_user_changes: Vec<CheckpointTxg> = published
        .iter()
        .flat_map(|reported| reported.published.versions_published.iter())
        .map(|version| version.root.checkpoint_txg)
        .collect();
    assert!(
        txgs_of_the_user_changes.len() > ROLLBACK_STATES_KEPT,
        "这个 unlink 发的步比要留的状态多：{txgs_of_the_user_changes:?}"
    );
    let accepted = runtime
        .accept(overwrite_of_the_surviving_file(filled.files.surviving))
        .unwrap_or_else(|refused| panic!("大文件删了：这一次收得下：{refused:?}"));
    assert!(
        !accepted
            .floor_raises_pushed_for_space
            .floor_raises
            .is_empty(),
        "这一次是推了抬 F 才收下的"
    );
    assert!(current_rollback_floor(runtime) > floor_before, "F 抬上去了");
    assert!(
        user_states_left(runtime, &txgs_of_the_user_changes) >= ROLLBACK_STATES_KEPT,
        "推完之后还留着 {ROLLBACK_STATES_KEPT} 个用户状态"
    );
}

/// 段序列：`accept` 推的那一串与 `publish_user_change` 同一个状态上同一次改动推的那一串逐个动作相同，所以它在盘上没有那条路（`crash_enumeration_floor_raise_pushed_by_admission.rs` 枚举过的）之外的状态：
/// 待发只在内存里。`publish_user_change` 接着把改动发出去，比前缀。
#[test]
fn an_accept_writes_the_same_operations_as_the_floor_raises_of_a_publish_of_the_same_change() {
    let filled = filled_pool(ShapeOfTheFill::DescendingSizes);
    let largest = filled.files.largest.expect("从大到小的填法写了大文件");
    let change = overwrite_of_the_surviving_file(filled.files.surviving);

    let accepting = mounted_copy_of(filled);
    unlink_cleanly(&accepting.runtime, largest);
    clear_the_stream(&accepting.stream);
    let accepted = accepting
        .runtime
        .accept(change)
        .unwrap_or_else(|refused| panic!("大文件删了：这一次收得下：{refused:?}"));
    let operations_of_the_accept = recorded_operations(&accepting.stream);

    let publishing = mounted_copy_of(filled);
    unlink_cleanly(&publishing.runtime, largest);
    clear_the_stream(&publishing.stream);
    let published = publishing
        .runtime
        .publish_user_change(change)
        .unwrap_or_else(|refused| panic!("大文件删了：这一次发得成：{refused:?}"));
    let operations_of_the_publish = recorded_operations(&publishing.stream);

    assert!(
        !operations_of_the_accept.is_empty(),
        "这一次 accept 推了抬 F，录下了写"
    );
    assert_eq!(
        accepted.floor_raises_pushed_for_space.floor_raises.len(),
        published
            .iter()
            .map(|reported| reported.published.floor_raises.len())
            .sum::<usize>(),
        "两条路推的串数相同"
    );
    assert!(
        operations_of_the_publish.len() > operations_of_the_accept.len(),
        "publish_user_change 推完接着发这次改动"
    );
    assert!(
        operations_of_the_publish[..operations_of_the_accept.len()] == operations_of_the_accept[..],
        "accept 发的段序列是 publish_user_change 发的前缀"
    );
}

fn recorded_operations(stream: &RecordedStream) -> Vec<RecordedDeviceOperation> {
    stream.lock().expect("录制流的锁里不 panic").clone()
}

/// 事件与读数：推了抬 F 的那一次 `accept`，空发布另列、不进 `publishes`，事件与观测照推的数记。
#[test]
fn an_accept_that_raises_the_floor_reports_the_series_apart_from_the_publishes_and_in_the_event() {
    let filled = filled_pool(ShapeOfTheFill::DescendingSizes);
    let mounted_pool = mounted_copy_of(filled);
    let runtime = &mounted_pool.runtime;
    unlink_cleanly(
        runtime,
        filled.files.largest.expect("从大到小的填法写了大文件"),
    );
    let ChangeAccepted {
        publishes,
        floor_raises_pushed_for_space,
        ..
    } = runtime
        .accept(overwrite_of_the_surviving_file(filled.files.surviving))
        .unwrap_or_else(|refused| panic!("大文件删了：这一次收得下：{refused:?}"));
    assert!(
        publishes.is_empty(),
        "钟没走、脏量没到：改动只收进待发，空发布不进这一项：{publishes:?}"
    );
    let series = floor_raises_pushed_for_space.floor_raises.len();
    assert!(series > 0, "这一次推了抬 F");
    assert_eq!(
        floor_raises_pushed_for_space
            .refusals_that_pushed_the_floor_raises
            .len(),
        series,
        "每一串对应一次被拒"
    );
    let empty_publishes: usize = floor_raises_pushed_for_space
        .floor_raises
        .iter()
        .map(|raised| raised.publishes.len())
        .sum();
    let observations = runtime.observations();
    assert_eq!(
        observations.floor_raises_pushed_by_accepts,
        u64::try_from(series).expect("串数装得进 u64"),
        "读数记串数"
    );
    assert_eq!(
        observations.empty_publishes_pushed_by_accepts,
        u64::try_from(empty_publishes).expect("次数装得进 u64"),
        "读数记空发布次数"
    );
    assert_eq!(
        *mounted_pool
            .events
            .floor_raises_of_each_event
            .lock()
            .expect("事件的锁里不 panic"),
        [series],
        "事件报一次，带这次推的串数"
    );
}

/// 真写满的池上 unlink 的意图写盘报错、冻结：交回运行库与这个文件。
fn mounted_copy_with_an_unlink_frozen() -> (MountedPool, InodeNumber) {
    let filled = filled_pool(ShapeOfTheFill::DescendingSizes);
    let mounted_pool = mounted_copy_of(filled);
    mounted_pool
        .faults
        .writes_left_to_fail
        .store(1, Ordering::SeqCst);
    assert!(
        mounted_pool
            .runtime
            .publish_user_change(unlink(filled.files.one_unit[0]))
            .is_err(),
        "摆好的故障：这次 unlink 写盘报错"
    );
    assert_eq!(
        mounted_pool
            .faults
            .writes_left_to_fail
            .load(Ordering::SeqCst),
        0,
        "故障打在了这次 unlink 上"
    );
    (mounted_pool, filled.files.surviving)
}

/// 冻结时 `accept` 先拒，不推抬 F：一个写都不发。
#[test]
fn an_accept_while_a_publish_is_frozen_is_refused_without_raising_the_floor() {
    let (mounted_pool, surviving) = mounted_copy_with_an_unlink_frozen();
    clear_the_stream(&mounted_pool.stream);
    let refused = mounted_pool
        .runtime
        .accept(overwrite_of_the_surviving_file(surviving));
    assert!(
        matches!(
            refused,
            Err(RuntimeRefused::Refused(
                AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
            ))
        ),
        "冻结着：accept 先拒，实际 {refused:?}"
    );
    assert!(
        recorded_operations(&mounted_pool.stream).is_empty(),
        "没推抬 F：一个写都不发"
    );
    assert_eq!(
        mounted_pool
            .runtime
            .observations()
            .floor_raises_pushed_by_accepts,
        0,
        "读数里没有推的串"
    );
}

/// 队列不空时的拒绝不动：重发成了、队列还没走，`accept` 照旧拒，不推抬 F。
#[test]
fn an_accept_while_the_deletion_queue_is_not_walked_is_refused_without_raising_the_floor() {
    let (mounted_pool, surviving) = mounted_copy_with_an_unlink_frozen();
    let is_resent = mounted_pool
        .runtime
        .with_the_session_and_devices(|locked| {
            let SessionAndDevices { session, devices } = locked;
            session.resend_the_frozen_publish(devices)
        })
        .expect("运行库没坏")
        .expect("故障只报一次：重发做得成");
    assert!(is_resent, "冻结着一次发布");
    clear_the_stream(&mounted_pool.stream);
    let refused = mounted_pool
        .runtime
        .accept(overwrite_of_the_surviving_file(surviving));
    assert!(
        matches!(
            refused,
            Err(RuntimeRefused::Refused(
                AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion
            ))
        ),
        "意图落盘、队列没走：accept 照旧拒，实际 {refused:?}"
    );
    assert!(
        recorded_operations(&mounted_pool.stream).is_empty(),
        "没推抬 F：一个写都不发"
    );
}

/// 比这个池的单元区还大的整份写。
fn write_that_never_fits(inode: InodeNumber, content: &[u8]) -> UserChange<'_> {
    UserChange::SequentialWriteOfAnInode {
        inode,
        content,
        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 30,
    }
}

fn content_that_never_fits() -> Vec<u8> {
    vec![0x7C_u8; UNITS_OF_A_WRITE_THAT_NEVER_FITS * data_unit_payload_capacity()]
}

/// 运行库这一刻盘上写过的扇区：盘从会话那里借出来读一遍，再放回去。
fn written_sectors_of_the_runtime(
    mounted_pool: &MountedPool,
) -> Vec<(DeviceIdentity, BTreeMap<u64, Vec<u8>>)> {
    mounted_pool
        .runtime
        .with_the_session_and_devices(|locked| {
            let pool = PoolOnMemoryDevices {
                devices: std::mem::take(&mut locked.devices),
                stream: Arc::clone(&mounted_pool.stream),
                faults: mounted_pool.faults.clone(),
            };
            let sectors = pool.written_sectors_of_every_device();
            locked.devices = pool.devices;
            sectors
        })
        .expect("运行库没坏")
}

/// 这条历史上每一次调用的拒绝都读这几样：放不下的写短在段外的槽对上，全放出来也差得远。
fn assert_refused_with_nothing_pushed(
    refusal: &UserChangeRefused,
    mounted_pool: &MountedPool,
    label: &str,
) {
    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) = refusal
    else {
        panic!(
            "{label}：放不下的写一串都不推就报空间不够：{}",
            leading_text_of(refusal)
        );
    };
    assert!(
        !no_space.devices_short.is_empty()
            && no_space.floor_raises.is_empty()
            && no_space.refusals_that_pushed_the_floor_raises.is_empty()
            && no_space.versions_published.is_empty(),
        "{label}：点名放不下的盘，一串都没推、一版都没发：{:?}",
        no_space.devices_short
    );
    assert_eq!(
        mounted_pool
            .stream
            .lock()
            .expect("录制流的锁里不 panic")
            .len(),
        0,
        "{label}：一个写都不发（录制流记每个写、写零、discard 与屏障）"
    );
    assert_eq!(
        mounted_pool
            .events
            .floor_raises_of_each_event
            .lock()
            .expect("事件的锁里不 panic")
            .len(),
        0,
        "{label}：没有推抬 F 的事件"
    );
}

/// 写满的池上崩了再挂载，每次挂载试几次放不下的写：每一次都一串不推、一个写不发，同一次挂载前后盘上逐字节不变。
/// 判别力：不判放不下时，同一条历史上 F 抬得动的那几次挂载每次推 2 串、6 次空发布。
fn write_that_never_fits_pushes_no_series_on_any_mount_of_a_crash_loop(shape: ShapeOfTheFill) {
    let filled = filled_pool(shape);
    let content = content_that_never_fits();
    let mut mounted_pool = mounted_copy_of(filled);
    for crash in 0..=CRASHES_OF_THE_CRASH_LOOP {
        clear_the_stream(&mounted_pool.stream);
        let sectors_before = written_sectors_of_the_runtime(&mounted_pool);
        for attempt in 0..ACCEPTS_PER_MOUNT_OF_THE_CRASH_LOOP {
            let label = format!("第 {crash} 次崩溃之后第 {attempt} 次 accept");
            let refused = mounted_pool
                .runtime
                .accept(write_that_never_fits(filled.files.surviving, &content))
                .err()
                .unwrap_or_else(|| panic!("{label}：比单元区还大的写收不下"));
            let RuntimeRefused::Refused(AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
                refusal,
                ..
            }) = &refused
            else {
                panic!("{label}：被空间拒：{}", leading_text_of(&refused));
            };
            assert_refused_with_nothing_pushed(refusal, &mounted_pool, &label);
        }
        assert!(
            written_sectors_of_the_runtime(&mounted_pool) == sectors_before,
            "第 {crash} 次崩溃之后：这次挂载的几次 accept 前后盘上逐字节不变"
        );
        let MountedPool {
            runtime,
            stream,
            faults,
            ..
        } = mounted_pool;
        mounted_pool = mounted(devices_after_a_crash(runtime), stream, faults);
    }
}

// 不切片并行：每次挂载接着上一次崩溃留下的盘，次序本身就是被测对象，没有可切的区间；两种填法各是一个测试函数，由测试框架并行。
#[test]
fn descending_sizes_a_write_that_never_fits_pushes_no_series_on_any_mount_of_a_crash_loop() {
    write_that_never_fits_pushes_no_series_on_any_mount_of_a_crash_loop(
        ShapeOfTheFill::DescendingSizes,
    );
}

// 不切片并行：每次挂载接着上一次崩溃留下的盘，次序本身就是被测对象，没有可切的区间；两种填法各是一个测试函数，由测试框架并行。
#[test]
fn thirty_two_unit_files_a_write_that_never_fits_pushes_no_series_on_any_mount_of_a_crash_loop() {
    write_that_never_fits_pushes_no_series_on_any_mount_of_a_crash_loop(
        ShapeOfTheFill::ThirtyTwoUnitFiles,
    );
}

/// `publish_user_change` 与 `accept` 共用一处判：同一条放不下的写走单发那个入口，同样一串不推、一个写不发，交回同一个成员。
#[test]
fn publish_of_a_write_that_never_fits_pushes_no_series() {
    let filled = filled_pool(ShapeOfTheFill::DescendingSizes);
    let content = content_that_never_fits();
    let mounted_pool = mounted_copy_of(filled);
    clear_the_stream(&mounted_pool.stream);
    let sectors_before = written_sectors_of_the_runtime(&mounted_pool);
    let refused = mounted_pool
        .runtime
        .publish_user_change(write_that_never_fits(filled.files.surviving, &content))
        .err()
        .unwrap_or_else(|| panic!("比单元区还大的写发不成"));
    let RuntimeRefused::Refused(refused_after_the_pending_changes) = &refused else {
        panic!("被空间拒：{}", leading_text_of(&refused));
    };
    assert!(
        refused_after_the_pending_changes
            .publishes_before_the_refusal
            .is_empty(),
        "待发是空的：拒绝之前什么都没发"
    );
    assert_refused_with_nothing_pushed(
        &refused_after_the_pending_changes.refusal,
        &mounted_pool,
        "单发",
    );
    assert!(
        written_sectors_of_the_runtime(&mounted_pool) == sectors_before,
        "盘上逐字节不变"
    );
}
