//! 验证包模块：crash、crash_state_enumeration
//! 代码审阅第 1、3 条（A 合并，按设备记屏障）与第 4 条（原地覆写补第三态，全量也跑）：
//! 录制器按设备记屏障；切段时当前段里每块有写的盘都被自己的屏障或 FUA 放行了才关段，FUA 只放行它自己那块盘；
//! 原地覆写的写取第三态「新旧都读不出」，那一态真生成镜像喂 checker 与记录核对器。

mod common;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_verification::crash::{
    crash_state_count, crash_state_count_with_torn_in_place_overwrites, full_expansion,
    in_place_subsets_with_copy_on_write_none_or_all_expansion, CrashPointReplayObserverCounts,
    CrashPointReplaySegmentExpansion, CrashPointReplayVersionTable, CrashStateJudgement,
    TearableInPlaceOverwrites,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_selecting, enumerate_crash_states_with_expansion,
    sample_crash_states_on_the_cpu_with_threads_against_the_version_table, CrashPointReplayItem,
    CrashPointReplayObserver,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-verification/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs";
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{PoolReader, RecoveryOutcome, RecoveryReport};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_verification::memory_pool::{
    closed_form_state_count_of_write_index_segments, root_identity_written_by, writes_and_segments,
    CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice, WrittenContents,
};
use singlefs_verification::segments::{
    segment_kinds_text, segment_sizes_text, split_into_segments, StepKind,
};
use singlefs_verification::{
    RecordedOperation, RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

// ---- 第 1 条：录制器按设备记屏障 ----

/// 包在录制器外面的一块盘：第 `swallowed_barrier_call` 次（从 0 数）屏障不往里传，录制器与底下的盘都看不见那一道；
/// 另记下每次屏障调用那一刻录制流有几步，好在干跑里找出要吞的是第几次。
struct DeviceThatCanSwallowOneBarrier {
    recorder: RecordingBlockDevice<SparseBlockDevice>,
    stream: SharedStream,
    swallowed_barrier_call: Option<usize>,
    stream_lengths_at_each_barrier_call: Vec<usize>,
}

impl BlockDevice for DeviceThatCanSwallowOneBarrier {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.recorder.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.recorder.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.recorder.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let this_call = self.stream_lengths_at_each_barrier_call.len();
        self.stream_lengths_at_each_barrier_call
            .push(self.stream.operation_count());
        if self.swallowed_barrier_call == Some(this_call) {
            return Ok(());
        }
        self.recorder.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.recorder.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.recorder.size_in_bytes()
    }
}

const UNIT_OFFSET: u64 = 50176 * 16384;
const ROOT_SLOT_OFFSET: u64 = 1 << 20;

/// 两块内存盘共用一条录制流；盘 1 外面可以吞一道屏障。
fn two_recorded_devices(
    swallowed_barrier_call_on_device_one: Option<usize>,
) -> (
    SharedStream,
    Vec<(DeviceIdentity, DeviceThatCanSwallowOneBarrier)>,
) {
    let stream = SharedStream::retaining_contents();
    let devices = [0u32, 1]
        .into_iter()
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                DeviceThatCanSwallowOneBarrier {
                    recorder: RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                    stream: stream.clone(),
                    swallowed_barrier_call: match device_number {
                        1 => swallowed_barrier_call_on_device_one,
                        _ => None,
                    },
                    stream_lengths_at_each_barrier_call: Vec::new(),
                },
            )
        })
        .collect();
    (stream, devices)
}

/// 池写路径的形状：两块盘各写一个单元 → 逐盘屏障 → 盘 0 根槽 FUA。
fn unit_on_each_device_then_a_barrier_on_each_then_a_root_slot(
    swallowed_barrier_call_on_device_one: Option<usize>,
) -> Vec<RecordedOperation> {
    let (stream, mut devices) = two_recorded_devices(swallowed_barrier_call_on_device_one);
    for (_, device) in &mut devices {
        device
            .write_at(
                DeviceOffsetInBytes(UNIT_OFFSET),
                &[1u8; 512],
                WriteDurability::Plain,
            )
            .expect("单元写");
    }
    for (_, device) in &mut devices {
        device.barrier().expect("屏障");
    }
    devices[0]
        .1
        .write_at(
            DeviceOffsetInBytes(ROOT_SLOT_OFFSET),
            &[2u8; 512],
            WriteDurability::ForceUnitAccess,
        )
        .expect("根槽 FUA");
    stream.operations()
}

/// 盘 1 那道屏障被吞在录制器外面：录下来的流要与每盘都发屏障的不同——每块盘的屏障各记一步，不并成一道池屏障。
#[test]
fn a_device_that_misses_its_barrier_records_a_different_stream_than_every_device_barriering() {
    let every_device_barriers = unit_on_each_device_then_a_barrier_on_each_then_a_root_slot(None);
    let device_one_misses_its_barrier =
        unit_on_each_device_then_a_barrier_on_each_then_a_root_slot(Some(0));
    let barrier_devices = |operations: &[RecordedOperation]| -> Vec<DeviceIdentity> {
        operations
            .iter()
            .filter(|operation| operation.kind == RecordedOperationKind::Barrier)
            .map(|operation| operation.device)
            .collect()
    };
    assert_eq!(
        barrier_devices(&every_device_barriers),
        vec![DeviceIdentity(0), DeviceIdentity(1)],
        "每块盘的屏障各记一步：两块盘各一道"
    );
    assert_eq!(
        barrier_devices(&device_one_misses_its_barrier),
        vec![DeviceIdentity(0)],
        "盘 1 那道没发：流里只剩盘 0 那一道"
    );
    assert_ne!(
        every_device_barriers, device_one_misses_its_barrier,
        "盘 1 漏发屏障的录制流要与每盘都发屏障的不同"
    );
}

/// 同一块盘上连着的几道屏障（中间一个写都没有）照旧记成一道：与每块盘各记一道并不矛盾。
#[test]
fn back_to_back_barriers_on_one_device_are_recorded_once_per_device() {
    let (stream, mut devices) = two_recorded_devices(None);
    devices[0]
        .1
        .write_at(
            DeviceOffsetInBytes(UNIT_OFFSET),
            &[1u8; 512],
            WriteDurability::Plain,
        )
        .expect("单元写");
    for _ in 0..2 {
        for (_, device) in &mut devices {
            device.barrier().expect("屏障");
        }
    }
    let kinds_and_devices: Vec<(RecordedOperationKind, DeviceIdentity)> = stream
        .operations()
        .iter()
        .map(|operation| (operation.kind, operation.device))
        .collect();
    assert_eq!(
        kinds_and_devices,
        vec![
            (RecordedOperationKind::Write, DeviceIdentity(0)),
            (RecordedOperationKind::Barrier, DeviceIdentity(0)),
            (RecordedOperationKind::Barrier, DeviceIdentity(1)),
        ],
        "两轮逐盘屏障之间没有写：每块盘一道"
    );
}

// ---- 第 3 条：FUA 只放行自己那块盘；切段照 A ----

fn retained(device: u32, kind: RecordedOperationKind, offset: u64) -> RetainedOperation {
    RetainedOperation {
        operation: RecordedOperation {
            device: DeviceIdentity(device),
            kind,
            offset: DeviceOffsetInBytes(offset),
            length: match kind {
                RecordedOperationKind::Barrier => 0,
                RecordedOperationKind::Write
                | RecordedOperationKind::WriteForceUnitAccess
                | RecordedOperationKind::WriteZeroes
                | RecordedOperationKind::Discard => 512,
            },
            content_hash: 0,
        },
        contents: match kind {
            RecordedOperationKind::Barrier
            | RecordedOperationKind::WriteZeroes
            | RecordedOperationKind::Discard => None,
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                Some(vec![0u8; 512])
            }
        },
    }
}

fn plain_operations(stream: &[RetainedOperation]) -> Vec<RecordedOperation> {
    stream
        .iter()
        .map(|retained| retained.operation.clone())
        .collect()
}

/// 盘 1 一次普通写 → 盘 0 根槽 FUA → 盘 0 一次普通写：FUA 只让盘 0 上它之前的写持久，盘 1 那次写在盘 0 后一次写落盘时可以还没落
/// ⇒ 三次写同一段（A：盘 1 没被放行，FUA 不关段）。两条切段的实现（写表下标那一份、种类串那一份）切得一样。
#[test]
fn a_force_unit_access_write_releases_only_its_own_device() {
    let stream = [
        retained(1, RecordedOperationKind::Write, UNIT_OFFSET),
        retained(
            0,
            RecordedOperationKind::WriteForceUnitAccess,
            ROOT_SLOT_OFFSET,
        ),
        retained(0, RecordedOperationKind::Write, 0),
    ];
    let (_writes, segments) = writes_and_segments(&stream, &geometry());
    assert_eq!(
        segments,
        vec![vec![0, 1, 2]],
        "要枚举得出「盘 0 后一次写已落、盘 1 那次写没落」：盘 1 那次写不能在更早的段里"
    );
    assert_eq!(
        segment_sizes_text(&split_into_segments(
            &plain_operations(&stream),
            &geometry()
        )),
        "3"
    );
    // 盘 1 先被自己的屏障放行：FUA 关得上段，与今天一块盘上的切法相同。
    let device_one_released_first = [
        retained(1, RecordedOperationKind::Write, UNIT_OFFSET),
        retained(1, RecordedOperationKind::Barrier, 0),
        retained(
            0,
            RecordedOperationKind::WriteForceUnitAccess,
            ROOT_SLOT_OFFSET,
        ),
        retained(0, RecordedOperationKind::Write, 0),
    ];
    assert_eq!(
        writes_and_segments(&device_one_released_first, &geometry()).1,
        vec![vec![0], vec![1], vec![2]]
    );
    assert_eq!(
        segment_sizes_text(&split_into_segments(
            &plain_operations(&device_one_released_first),
            &geometry()
        )),
        "1+1+1"
    );
    // 一块盘：FUA 关掉它与前面没被屏障隔开的写所在的那一段（D13（验证路线） 已定项 4 原句），不变。
    let one_device = [
        retained(0, RecordedOperationKind::Write, UNIT_OFFSET),
        retained(
            0,
            RecordedOperationKind::WriteForceUnitAccess,
            ROOT_SLOT_OFFSET,
        ),
        retained(0, RecordedOperationKind::Write, 0),
    ];
    assert_eq!(
        writes_and_segments(&one_device, &geometry()).1,
        vec![vec![0, 1], vec![2]]
    );
}

/// 一块盘的屏障只放行这块盘：盘 0 发了屏障、盘 1 没发时，这道屏障不关段，前后的写同段（A 合并）；盘 1 的屏障一到，两块盘都放行了才关。
#[test]
fn a_barrier_closes_the_segment_only_once_every_device_with_a_write_in_it_is_released() {
    let stream = [
        retained(0, RecordedOperationKind::Write, UNIT_OFFSET),
        retained(1, RecordedOperationKind::Write, UNIT_OFFSET),
        retained(0, RecordedOperationKind::Barrier, 0),
        retained(0, RecordedOperationKind::Write, UNIT_OFFSET + 32768),
        retained(1, RecordedOperationKind::Barrier, 0),
        retained(0, RecordedOperationKind::Barrier, 0),
        retained(1, RecordedOperationKind::Write, UNIT_OFFSET + 32768),
    ];
    assert_eq!(
        writes_and_segments(&stream, &geometry()).1,
        vec![vec![0, 1, 2], vec![3]],
        "盘 0 那道屏障时盘 1 还没放行：不关段；盘 1 放行时盘 0 又有了新写：不关；盘 0 再放行才关"
    );
    let segments = split_into_segments(&plain_operations(&stream), &geometry());
    assert_eq!(segment_sizes_text(&segments), "3+1");
    assert_eq!(
        segment_kinds_text(&segments),
        "[unit_write×3,barrier×3]|[unit_write]"
    );
}

/// 每块盘都发屏障（今天池写路径的形状）时，按设备切出的段与补这一条之前把那几道并成一道池屏障切出的逐段相同：段序列、每段的写、
/// 闭式都与今天一样（下面钉的段就是之前的切法在这条流上切出来的）；种类串里屏障按设备各数一个。
#[test]
fn a_barrier_on_every_device_cuts_the_same_segments_as_one_pool_barrier() {
    let per_device = [
        retained(0, RecordedOperationKind::Write, UNIT_OFFSET),
        retained(1, RecordedOperationKind::Write, UNIT_OFFSET),
        retained(0, RecordedOperationKind::Barrier, 0),
        retained(1, RecordedOperationKind::Barrier, 0),
        retained(0, RecordedOperationKind::Write, 16 << 20),
        retained(1, RecordedOperationKind::Write, 16 << 20),
        retained(0, RecordedOperationKind::Barrier, 0),
        retained(1, RecordedOperationKind::Barrier, 0),
        retained(
            0,
            RecordedOperationKind::WriteForceUnitAccess,
            ROOT_SLOT_OFFSET,
        ),
        retained(0, RecordedOperationKind::Write, 0),
        retained(1, RecordedOperationKind::Write, 0),
    ];
    let (_, per_device_segments) = writes_and_segments(&per_device, &geometry());
    assert_eq!(
        per_device_segments,
        vec![vec![0, 1], vec![2, 3], vec![4], vec![5, 6]]
    );
    assert_eq!(
        closed_form_state_count_of_write_index_segments(&per_device_segments),
        1 + 3 + 3 + 1 + 3
    );
    let kind_segments = split_into_segments(&plain_operations(&per_device), &geometry());
    assert_eq!(segment_sizes_text(&kind_segments), "2+2+1+2");
    assert_eq!(
        segment_kinds_text(&kind_segments),
        "[unit_write×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]"
    );
}

/// 真流上的判别力（第 1 条）：只吞盘 1 在一处的屏障——新池新建文件里「journal 记录 → 根槽」那一组里盘 1 那一道，吞在录制器外面。
/// 盘 1 那份记录没被放行，按 A 那道屏障不关段：记录两份、根槽与之后的两次系统配置槽写并成一段，崩溃点重放小流（只展开 5 个写以内的段）里
/// 摆得出「根在案而记录一份都不在」，记录核对器报出来；每盘都发屏障的同一条流报 0。合并屏障的录制器把盘 0 那一道记成整个池的，
/// 两条流逐步相同，报不出来。
#[test]
// crash-case-check:not-a-crash-case 新池新建文件流只展开 5 个写以内的段、24 写那一段不展开，一百来个状态，单跑几秒
fn one_device_missing_its_barrier_before_the_root_slot_is_reported_by_the_record_checker() {
    let (every_device_barriers, stream_lengths_at_barrier_calls_on_device_one) =
        new_pool_file_creation_stream(None);
    let root_position = every_device_barriers
        .operations
        .iter()
        .rposition(|retained| geometry().classify(&retained.operation) == StepKind::RootRecordFua)
        .expect("流里有新池新建文件的根槽写");
    let barrier_call_before_the_root = stream_lengths_at_barrier_calls_on_device_one
        .iter()
        .rposition(|stream_length| *stream_length < root_position)
        .expect("盘 1 在根槽写之前发过屏障");
    let (device_one_misses_one_barrier, _) =
        new_pool_file_creation_stream(Some(barrier_call_before_the_root));
    let record_checker_reports = |recorded: &RecordedNewPoolFileCreationStream,
                                  item_label: &str| {
        let (writes, segments) = writes_and_segments(
            &recorded.operations[recorded.mkfs_operation_count..],
            &geometry(),
        );
        let root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("根槽");
        let item_name = format!("segments_per_device::barriers::{item_label}");
        let tally = enumerate_crash_states_selecting(
            CrashPointReplayItem {
                name: &item_name,
                setting_file: THIS_FILE,
            },
            &recorded.base,
            &writes,
            &segments,
            root_index,
            &file_content(),
            &|_segment_index, segment| segment.len() <= 5,
        )
        .judged()?;
        let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
        Some((sizes, tally.record_root_without_record, tally.violations))
    };
    // 两条流都先过一次库（只录那一趟两条都录下），再判
    let every_device = record_checker_reports(&every_device_barriers, "every_device_barriers");
    let missing = record_checker_reports(
        &device_one_misses_one_barrier,
        "device_one_misses_one_barrier",
    );
    let (
        Some((every_device_sizes, every_device_reports, every_device_violations)),
        Some((missing_sizes, missing_reports, _)),
    ) = (every_device, missing)
    else {
        return;
    };
    assert_eq!(
        every_device_sizes,
        vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2],
        "每盘都发屏障：与今天的段序列相同（取号两轮各一段；C577 之后暖机第二次的轮换自成一段，改之前与 A 的单元写并成 26）"
    );
    assert_eq!(
        (every_device_reports, every_device_violations),
        (0, 0),
        "每盘都发屏障：记录核对器与 oracle 都不报"
    );
    assert_eq!(
        missing_sizes,
        vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 5],
        "盘 1 漏发那一道：记录两份、根槽、两次系统配置槽写并成一段（轮换之后那道屏障两块盘都发，这一段在那里关）"
    );
    assert!(
        missing_reports > 0,
        "盘 1 漏发屏障：崩溃点重放要摆出根在案而记录一份都不在的状态，记录核对器报出来（报了 {missing_reports} 个）"
    );
}

struct RecordedNewPoolFileCreationStream {
    operations: Vec<RetainedOperation>,
    mkfs_operation_count: usize,
    base: MemoryPool,
}

/// 两块内存盘上 mkfs → 取号 → 暖机 → 新池新建文件（与 `common::build_pool` 同一串调用），盘 1 外面可以吞掉第几次屏障；
/// 交回录制流、mkfs 占了几步、mkfs 之后的基镜像，与盘 1 每次屏障调用那一刻录制流有几步。
fn new_pool_file_creation_stream(
    swallowed_barrier_call_on_device_one: Option<usize>,
) -> (RecordedNewPoolFileCreationStream, Vec<usize>) {
    let parameters = parameters();
    let (stream, mut devices) = two_recorded_devices(swallowed_barrier_call_on_device_one);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operation_count();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
    allocator.install_root_ring_occupancy(
        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
            &parameters,
            &genesis,
        ),
    );
    let content = file_content();
    {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warmed_up = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warmed_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed_up.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..mkfs_operation_count]);
    let stream_lengths_at_barrier_calls_on_device_one =
        devices[1].1.stream_lengths_at_each_barrier_call.clone();
    (
        RecordedNewPoolFileCreationStream {
            operations,
            mkfs_operation_count,
            base,
        },
        stream_lengths_at_barrier_calls_on_device_one,
    )
}

// ---- 第 4 条：原地覆写补第三态「新旧都读不出」 ----

const SYSTEM_CONFIGURATION_SLOT_OFFSET: u64 = 4096;
const JOURNAL_SLOT_OFFSET: u64 = 16 << 20;
const UNIT_SLOT_OFFSET: u64 = 784 << 20;

/// 手摆一次普通写（`length_in_bytes` 个 `fill` 字节）。
fn plain_write(
    device: u32,
    kind: StepKind,
    offset: u64,
    length_in_bytes: usize,
    fill: u8,
) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(device),
        kind,
        is_force_unit_access: kind == StepKind::RootRecordFua,
        offset: DeviceOffsetInBytes(offset),
        contents: WrittenContents::Bytes(vec![fill; length_in_bytes]),
    }
}

/// 认法三条（`crash::TearableInPlaceOverwrites`）：系统配置槽写罩住基镜像上的旧槽、第二次 journal 写罩住第一次，取三态；
/// 第一次 journal 写（那里原来是 0）、单元写（COW）、一个扇区的根槽写取两态。状态数：全量第一段 3 · 2 · 2 · 2 − 1、
/// 第二段 2 − 1、第三段 3 − 1，甲二第一段原地写 3 · 2 × 单元写全不落或全落 2 − 1；没有原地覆写时两式就是补第三态之前的闭式。
#[test]
fn in_place_overwrites_take_three_states_and_every_other_write_two() {
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 1 << 30);
    let old_device_zero = base.devices.get_mut(&DeviceIdentity(0)).expect("盘 0");
    old_device_zero.write(
        DeviceOffsetInBytes(SYSTEM_CONFIGURATION_SLOT_OFFSET),
        &[0x5Au8; 4096],
    );
    old_device_zero.write(DeviceOffsetInBytes(ROOT_SLOT_OFFSET), &[0x6Bu8; 512]);
    let writes = vec![
        plain_write(
            0,
            StepKind::SystemConfigurationSlot,
            SYSTEM_CONFIGURATION_SLOT_OFFSET,
            4096,
            1,
        ),
        plain_write(0, StepKind::JournalRecord, JOURNAL_SLOT_OFFSET, 4096, 2),
        plain_write(0, StepKind::UnitWrite, UNIT_SLOT_OFFSET, 32768, 3),
        plain_write(1, StepKind::UnitWrite, UNIT_SLOT_OFFSET, 32768, 3),
        plain_write(0, StepKind::RootRecordFua, ROOT_SLOT_OFFSET, 512, 4),
        plain_write(0, StepKind::JournalRecord, JOURNAL_SLOT_OFFSET, 4096, 5),
    ];
    let segments = vec![vec![0, 1, 2, 3], vec![4], vec![5]];
    assert_eq!(
        TearableInPlaceOverwrites::of(&base, &writes).write_indexes(),
        vec![0, 5],
        "系统配置槽写与第二次 journal 写是原地覆写"
    );
    assert_eq!(
        crash_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion),
        1 + 23 + 1 + 2
    );
    assert_eq!(
        crash_state_count_with_torn_in_place_overwrites(
            &base,
            &writes,
            &segments,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion
        ),
        1 + 11 + 1 + 2
    );
    assert_eq!(
        closed_form_state_count_of_write_index_segments(&segments),
        1 + 15 + 1 + 1
    );
    assert_eq!(
        crash_state_count(
            &writes,
            &segments,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion
        ),
        1 + 7 + 1 + 1
    );
    let empty_base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 1 << 30);
    let first_writes_only = &writes[..5];
    let first_segments = vec![vec![0, 1, 2, 3], vec![4]];
    assert!(
        TearableInPlaceOverwrites::of(&empty_base, first_writes_only)
            .write_indexes()
            .is_empty(),
        "基镜像全 0、没有更早的写与它重叠：一次原地覆写都没有"
    );
    assert_eq!(
        crash_state_count_with_torn_in_place_overwrites(
            &empty_base,
            first_writes_only,
            &first_segments,
            &full_expansion
        ),
        closed_form_state_count_of_write_index_segments(&first_segments)
    );
    assert_eq!(
        crash_state_count_with_torn_in_place_overwrites(
            &empty_base,
            first_writes_only,
            &first_segments,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion
        ),
        crash_state_count(
            first_writes_only,
            &first_segments,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion
        )
    );
}

/// 新池新建文件那条流（崩溃点重放从 mkfs 之后枚举的整条流）：每盘都发屏障，段序列与补第三态之前的闭式与今天逐字相同；
/// 取三态的只有系统配置槽写（10 次，每次都罩住更早写下的旧槽），全量 16777268、甲二 54 个状态（取号写一轮时 8 次、16777260、46；
/// C577 之前暖机第二次的轮换与 A 的单元写同段，150994980、54）。
#[test]
fn the_new_pool_file_creation_stream_keeps_its_segments_and_takes_the_third_state_on_its_system_configuration_writes(
) {
    let pool = common::build_pool("torn-third-state-first-stream");
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &geometry(),
    );
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]
    );
    assert_eq!(
        closed_form_state_count_of_write_index_segments(&segments),
        16_777_243
    );
    assert_eq!(
        crash_state_count(
            &writes,
            &segments,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion
        ),
        29
    );
    let tearable = TearableInPlaceOverwrites::of(&base, &writes);
    let tearable_kinds: Vec<StepKind> = tearable
        .write_indexes()
        .into_iter()
        .map(|write_index| writes[write_index].kind)
        .collect();
    assert_eq!(
        tearable_kinds,
        vec![StepKind::SystemConfigurationSlot; 10],
        "取号两轮各 2 + 两次暖机各 2 + A 的轮换 2"
    );
    assert_eq!(
        crash_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion),
        16_777_268,
        "24 写的单元段 2²⁴ − 1、五个系统配置槽段（取号两轮、三次轮换）各 3² − 1、其余同今天"
    );
    assert_eq!(
        crash_state_count_with_torn_in_place_overwrites(
            &base,
            &writes,
            &segments,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion
        ),
        54,
        "24 写的单元段全不落 1、五个系统配置槽段各 3² − 1、其余同今天"
    );
}

/// 观察者「撕裂的系统配置槽」（`torn_slots`）：轮换那一段每次槽写在这个状态的盘上既不是旧槽也不是新槽的算撕裂，撕裂的那一槽新旧都读不出、
/// 在持久集合里记成「原来那次写没持久」；数撕裂的状态与槽。观察者的身份是这个 fn 的闭包键（入口登记在
/// `singlefs_verification::judging_units_entries::OBSERVER_ENTRIES`），另读的输入（轮换的写与每槽新旧两份）按摘要进它的版本。
fn count_the_torn_system_configuration_slots(
    rotation_writes: &[usize],
    old_and_new_of_each_rotation_write: &[(Vec<u8>, Vec<u8>)],
    writes: &[RetainedWrite],
    image: &CrashImage<'_>,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let mut torn_in_this_state = 0u64;
    for (write_index, (old, new)) in rotation_writes
        .iter()
        .zip(old_and_new_of_each_rotation_write)
    {
        let write = &writes[*write_index];
        let on_disk = PoolReader::read(image, write.device, write.offset, 4096).expect("槽读得出");
        if on_disk != *old && on_disk != *new {
            torn_in_this_state += 1;
            assert!(
                SystemConfiguration::parse_slot(&on_disk).is_err(),
                "撕裂的那一槽新旧都读不出（盘 {}）",
                write.device.0
            );
            assert!(
                !image.persisted[*write_index],
                "撕裂那一态在持久集合里是「原来那次写没持久」"
            );
        }
    }
    if torn_in_this_state > 0 {
        counts.add("states_with_a_torn_slot", 1);
        counts.add("torn_slots", torn_in_this_state);
    }
}

/// 撕裂那一态真生成镜像、喂给恢复、checker 与记录核对器：新池新建文件流只展开最后一段（A 的根之后两块盘的系统配置槽轮换），
/// 3² − 1 个状态再加全部持久那一个。每个状态按盘上字节认那两次写怎么落（与这次写的字节相同是落了、与写之前的相同是没落、
/// 都不同是撕裂）：撕裂的那一槽解不开（新旧都读不出），五个状态里至少一槽撕裂；恢复读出文件、oracle、池级 checker、
/// 记录核对器都不报，checker 在每个状态上都跑过（评估过的 + 不适用的 = 状态数）。
#[test]
// crash-case-check:not-a-crash-case 新池新建文件流只展开最后那一段两个写，九个状态，单跑几秒
fn the_torn_state_of_a_system_configuration_rotation_is_generated_and_fed_to_the_checker_and_the_record_checker(
) {
    let pool = common::build_pool("torn-third-state-generated");
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &geometry(),
    );
    let last_segment = segments.len() - 1;
    let rotation_writes = segments[last_segment].clone();
    assert!(
        rotation_writes
            .iter()
            .all(|write_index| writes[*write_index].kind == StepKind::SystemConfigurationSlot),
        "最后一段是 A 的两块盘的系统配置槽轮换"
    );
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("A 的根槽写");
    let (instance, checkpoint_txg) =
        root_identity_written_by(writes[root_index].bytes().expect("根槽写带着字节"));
    let versions = [PublishedVersion {
        instance,
        checkpoint_txg,
        content: file_content(),
    }];
    let mut image_before_the_rotation = base.clone();
    image_before_the_rotation.apply_writes(&writes[..rotation_writes[0]]);
    let old_and_new_of_each_rotation_write: Vec<(Vec<u8>, Vec<u8>)> = rotation_writes
        .iter()
        .map(|write_index| {
            let write = &writes[*write_index];
            let old =
                PoolReader::read(&image_before_the_rotation, write.device, write.offset, 4096)
                    .expect("旧槽读得出");
            (old, write.bytes().expect("系统配置槽写带着字节").to_vec())
        })
        .collect();
    let only_the_last_segment = |segment_index: usize, _segment: &[usize]| {
        if segment_index == last_segment {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    // 观察者经崩溃判定库那一路按块看：要记的数全经观察者计数。记录核对器不在观察者里跑：判段从库里读回每个状态判
    // （配了 GPU 在卡上判），撕裂的状态在内，下面照 tally 的两格判
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _consulted_report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        count_the_torn_system_configuration_slots(
            &rotation_writes,
            &old_and_new_of_each_rotation_write,
            &writes,
            image,
            counts,
        );
    };
    // 观察者另读的输入：轮换那几次写的下标与每一次写的旧、新两份字节（写表本身在流的输入里）
    let observer_extra_inputs_digest = singlefs_verification::sha256::sha256_digest(
        format!(
            "rotation_writes={rotation_writes:?}\nold_and_new_of_each_rotation_write={old_and_new_of_each_rotation_write:?}\n"
        )
        .as_bytes(),
    );
    let Some(tally) = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: "segments_per_device::torn_system_configuration_rotation",
            setting_file: THIS_FILE,
        },
        &base,
        &writes,
        &segments,
        root_index,
        CrashPointReplayVersionTable::with_a_file_only(&versions),
        &only_the_last_segment,
        &[CrashPointReplayObserver {
            name: "torn_slots",
            extra_inputs_digest: extra_inputs_digest_that_was_computed(
                observer_extra_inputs_digest,
            ),
            observe: &observe,
        }],
    )
    .judged() else {
        return;
    };
    let states_with_a_torn_slot = tally.observer_counts.count_named("states_with_a_torn_slot");
    let torn_slots = tally.observer_counts.count_named("torn_slots");
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        9u64,
        "最后一段 3² − 1，再加全部持久那一个",
    );
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &base,
            &writes,
            &segments,
            &only_the_last_segment
        ),
        "crash_segments_per_device_and_torn_in_place_overwrites：用例自己的展开下的闭式",
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        (states_with_a_torn_slot, torn_slots),
        (5u64, 6u64),
        "撕裂 × 没落 / 落了 各两个状态、两槽都撕裂一个状态",
    );
    assert_eq!(
        (tally.violations, tally.failed_states),
        (0, 0),
        "oracle 不红、走读不失败（缩过域照判；第一处违例：{:?}）",
        tally.first_violation
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        tally.file_read_states,
        9u64,
        "A 的根已落：每个状态都读出文件",
    );
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器在撕裂镜像上照判、不报（判段从库里读回每个状态判）"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant}：{:?}",
            tally.checker_first_violation.get(invariant)
        );
        assert_eq!(
            tally
                .checker_evaluated_states
                .get(invariant)
                .copied()
                .unwrap_or(0)
                + tally
                    .checker_not_applicable_states
                    .get(invariant)
                    .copied()
                    .unwrap_or(0),
            tally.states,
            "{invariant}：checker 在每个状态（撕裂的在内）上都跑过"
        );
    }
    // 批 19 进清单的四条按这几个状态的盘面分两类（推的，没量；九个状态都在 A 的根落了盘之后，盘面是新池新建文件那条流根已落的那一类）。
    // 前件在、撕裂的状态上照样评估：I-8.14（新实例第一个 txg 高于更早实例的根与记录）——环里有实例 1 的根与记录、mkfs 的种子根（实例 0），
    // 撕裂只出在系统配置槽、根环与 journal 环没有读不出的槽；I-3.20（空闲、已分配、不可回收之和等于容量，已承诺预留等于闭式）——最新根 A 下面有记账树，
    // 与 I-3.1、I-5.2 同一支。两条推的都是 9。
    for must_evaluate in [
        singlefs_checker::image::FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES,
        singlefs_checker::image::FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM,
        // I-3.21（走到的引用起点是最新根计入的分配记录的起点，C944）与 I-3.20 同一类：最新根的树表带分配记录树（有记账树的那几版都带）就判，
        // 树表 0 条的种子根、暖机根上报不适用（推的，没量；判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节 Q2 新立）。
        singlefs_checker::image::WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
    ] {
        singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0),
            &format!("{must_evaluate} 在撕裂的那几个状态所在的这一段上真被评估过：{}", tally.checker_counts_by_invariant()),
        );
    }
    // 前件不在、一个状态都评估不到：I-3.19（实例表行的 W 是一次恢复施加得出的最大事务号）——新池新建文件那条流上取号不写行，实例表里一行都没有；
    // I-5.7（由记录施加出来的一版的分配记录树根出生在这一版）——A 的根已落，没有下一次挂载先施加的那一版，也没有由记录施加出来的那一版。
    for evaluated_on_no_state in [
        singlefs_checker::image::INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED,
        singlefs_checker::image::ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION,
    ] {
        assert_eq!(
            tally
                .checker_evaluated_states
                .get(evaluated_on_no_state)
                .copied()
                .unwrap_or(0),
            0,
            "{evaluated_on_no_state} 在这一段上一个状态都评估不到（前件不在）：{}",
            tally.checker_counts_by_invariant()
        );
    }
}

/// 新池新建文件流只展开最后一段（A 的根之后两块盘的系统配置槽轮换）时撕裂那一态要的几样：起点镜像、写表、段、最后一段的下标与它的写、
/// 被判的根槽写、版本表，与轮换那几次写各自写之前、写之后那一槽的字节。
struct RotationAfterTheFileCreation {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    last_segment: usize,
    rotation_writes: Vec<usize>,
    root_index: usize,
    versions: Vec<PublishedVersion>,
    old_and_new_of_each_rotation_write: Vec<(Vec<u8>, Vec<u8>)>,
}

impl RotationAfterTheFileCreation {
    /// 录一条新池新建文件流（`common::build_pool`），取最后一段的轮换。
    fn recorded(tag: &str) -> Self {
        let pool = common::build_pool(tag);
        let base = pool.memory_pool_after_mkfs();
        let (writes, segments) = writes_and_segments(
            &pool.retained_operations()[pool.mkfs_operation_count..],
            &geometry(),
        );
        let last_segment = segments.len() - 1;
        let rotation_writes = segments[last_segment].clone();
        assert!(
            rotation_writes
                .iter()
                .all(|write_index| writes[*write_index].kind == StepKind::SystemConfigurationSlot),
            "最后一段是 A 的两块盘的系统配置槽轮换"
        );
        let root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("A 的根槽写");
        let (instance, checkpoint_txg) =
            root_identity_written_by(writes[root_index].bytes().expect("根槽写带着字节"));
        let versions = vec![PublishedVersion {
            instance,
            checkpoint_txg,
            content: file_content(),
        }];
        let mut image_before_the_rotation = base.clone();
        image_before_the_rotation.apply_writes(&writes[..rotation_writes[0]]);
        let old_and_new_of_each_rotation_write: Vec<(Vec<u8>, Vec<u8>)> = rotation_writes
            .iter()
            .map(|write_index| {
                let write = &writes[*write_index];
                let old =
                    PoolReader::read(&image_before_the_rotation, write.device, write.offset, 4096)
                        .expect("旧槽读得出");
                (old, write.bytes().expect("系统配置槽写带着字节").to_vec())
            })
            .collect();
        Self {
            base,
            writes,
            segments,
            last_segment,
            rotation_writes,
            root_index,
            versions,
            old_and_new_of_each_rotation_write,
        }
    }

    /// 只展开最后一段（轮换那一段每次写取三态，3² − 1 个状态），别的段整段持久进入后面的状态。
    fn expansion_of(&self, segment_index: usize) -> CrashPointReplaySegmentExpansion {
        if segment_index == self.last_segment {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    }
}

/// 只展开最后一段的那条流整条在 CPU 判器上逐状态判：取样那一支（`sample_crash_states_on_the_cpu_with_threads_against_the_version_table`）
/// 不经选路，多机配置开不开 GPU、编没编进 gpu 特性都在 CPU 上判——判段逐个序号照 `CrashStatePlan::persisted_writes_of_state` 摆持久集合、
/// 撕裂那一态叠上撕裂镜像，交 CPU 判器。撕裂那一条（[`the_torn_state_of_a_system_configuration_rotation_is_generated_and_fed_to_the_checker_and_the_record_checker`]）
/// 在门禁开 GPU 时判段走 GPU 判器、观察者经录入段看，CPU 那一路摆持久集合的代码它跑不到，变异表里改坏它的行点名这里的用例。
/// 每个状态交 `see` 取出要核的那几样；交回整条流的状态数与按序号排好的 (序号, 取出的那几样)。只录那一趟（分片）交 `None`。线程数取每台默认重活线程数。
// gpu-exempt: cpu-judge-mutation-rows
fn rotation_after_the_file_creation_judged_on_the_cpu_judge<Seen: Send>(
    rotation: &RotationAfterTheFileCreation,
    see: &(dyn Fn(&CrashImage<'_>, &CrashStateJudgement) -> Seen + Sync),
) -> Option<(u64, Vec<(u64, Seen)>)> {
    let only_the_last_segment =
        |segment_index: usize, _segment: &[usize]| rotation.expansion_of(segment_index);
    let state_count = crash_state_count_with_torn_in_place_overwrites(
        &rotation.base,
        &rotation.writes,
        &rotation.segments,
        &only_the_last_segment,
    );
    let seen_by_ordinal: std::sync::Mutex<Vec<(u64, Seen)>> = std::sync::Mutex::new(Vec::new());
    let states_of_the_flow = sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
        CrashPointReplayItem {
            name: "segments_per_device::torn_system_configuration_rotation_on_the_cpu_judge",
            setting_file: THIS_FILE,
        },
        &rotation.base,
        &rotation.writes,
        &rotation.segments,
        rotation.root_index,
        CrashPointReplayVersionTable::with_a_file_only(&rotation.versions),
        &only_the_last_segment,
        0..state_count,
        singlefs_verification::heavy_threads::default_heavy_threads(),
        &|ordinal, image, judgement| {
            let seen = see(image, judgement);
            seen_by_ordinal
                .lock()
                .expect("判定线程不 panic")
                .push((ordinal, seen));
        },
    )
    .judged()?;
    let mut seen_by_ordinal = seen_by_ordinal.into_inner().expect("判定线程不 panic");
    seen_by_ordinal.sort_by_key(|(ordinal, _seen)| *ordinal);
    Some((states_of_the_flow, seen_by_ordinal))
}

/// CPU 判器那一路上一个状态要核的几样：持久集合、这张镜像上数出的撕裂槽（[`count_the_torn_system_configuration_slots`] 同一个判法）、
/// 判红的各项、看 journal 那一遍恢复读没读出文件。
struct TornRotationStateOnTheCpuJudge {
    persisted: Vec<bool>,
    torn_slot_counts: CrashPointReplayObserverCounts,
    red_items: Vec<String>,
    file_read: bool,
}

/// 撕裂那一态在 CPU 判器那一路上真叠了撕裂镜像：只展开最后一段的九个状态逐个照 CPU 判器那一路摆持久集合、交镜像，
/// 交回的镜像上按盘上字节数撕裂的系统配置槽，五个状态有撕裂、共六槽（撕裂 × 没落 / 落了各两个状态、两槽都撕裂一个状态），
/// 九个持久集合两两不同；每个状态都读出文件，oracle、池级 checker、记录核对器都不报。
/// 被改坏的 `CrashStatePlan::persisted_writes_of_state` 在 CPU 判器那一路上逐个序号调（`CrashStatesOfTheReadBackFlow::visit_the_states_in_the_ordinals`）：
/// 撕裂那一态不在持久集合里记撕裂镜像时，盘上那一槽还是写之前的字节，与那次写没持久是同一个镜像、一槽都数不出撕裂
/// （变异表「实审 B3a-2 第 4 条：撕裂那一态不叠撕裂镜像」）。
#[test]
fn cpu_judge_lays_the_torn_image_of_a_system_configuration_rotation_over_its_slot_and_judges_it_clean(
) {
    let rotation = RotationAfterTheFileCreation::recorded("torn-third-state-on-the-cpu-judge");
    let Some((states_of_the_flow, states_by_ordinal)) =
        rotation_after_the_file_creation_judged_on_the_cpu_judge(&rotation, &|image, judgement| {
            let mut torn_slot_counts = CrashPointReplayObserverCounts::default();
            count_the_torn_system_configuration_slots(
                &rotation.rotation_writes,
                &rotation.old_and_new_of_each_rotation_write,
                &rotation.writes,
                image,
                &mut torn_slot_counts,
            );
            TornRotationStateOnTheCpuJudge {
                persisted: image.persisted.clone(),
                torn_slot_counts,
                red_items: judgement.red_items(),
                file_read: matches!(
                    judgement.consulted.outcome,
                    RecoveryOutcome::FileRead { .. }
                ),
            }
        })
    else {
        return;
    };
    assert_eq!(states_of_the_flow, 9, "最后一段 3² − 1，再加全部持久那一个");
    assert_eq!(
        states_by_ordinal
            .iter()
            .map(|(ordinal, _state)| *ordinal)
            .collect::<Vec<u64>>(),
        (0..9).collect::<Vec<u64>>(),
        "每个序号各判一次"
    );
    let total_named = |counter: &str| {
        states_by_ordinal
            .iter()
            .map(|(_ordinal, state)| state.torn_slot_counts.count_named(counter))
            .sum::<u64>()
    };
    assert_eq!(
        (
            total_named("states_with_a_torn_slot"),
            total_named("torn_slots")
        ),
        (5, 6),
        "撕裂 × 没落 / 落了 各两个状态、两槽都撕裂一个状态：CPU 判器那一路交回的镜像上真叠了撕裂镜像"
    );
    let distinct_persisted_sets: std::collections::BTreeSet<&Vec<bool>> = states_by_ordinal
        .iter()
        .map(|(_ordinal, state)| &state.persisted)
        .collect();
    assert_eq!(
        distinct_persisted_sets.len(),
        9,
        "九个持久集合两两不同（撕裂那一态与没持久那一态是两个持久集合）"
    );
    let red_states: Vec<(u64, &Vec<String>)> = states_by_ordinal
        .iter()
        .filter(|(_ordinal, state)| !state.red_items.is_empty())
        .map(|(ordinal, state)| (*ordinal, &state.red_items))
        .collect();
    assert!(
        red_states.is_empty(),
        "撕裂的状态在内，oracle、池级 checker、记录核对器都不报：{red_states:?}"
    );
    assert!(
        states_by_ordinal
            .iter()
            .all(|(_ordinal, state)| state.file_read),
        "A 的根已落：每个状态都读出文件"
    );
}

/// 观察者另读的输入的摘要要真算：全零就是没算（另读输入不同的两趟会共用看过的块）。
fn extra_inputs_digest_that_was_computed(
    digest: [u8; singlefs_verification::sha256::DIGEST_BYTES],
) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
    assert_ne!(
        digest,
        [0; singlefs_verification::sha256::DIGEST_BYTES],
        "观察者另读的输入的摘要真算（全零就是没算：另读输入不同的两趟会共用看过的块）"
    );
    digest
}
