//! 代码审阅第 1、3 条（用户 2026-09-27 定「A 合并，按设备记屏障」）与第 4 条（用户同日定「原地覆写补第三态，全量也跑」）：
//! 录制器按设备记屏障；切段时当前段里每块有写的盘都被自己的屏障或 FUA 放行了才关段，FUA 只放行它自己那块盘；
//! 原地覆写的写取第三态「新旧都读不出」，那一态真生成镜像喂 checker 与记录核对器。

mod common;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{PoolReader, RecoveryReport};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_harness::crash::{
    check_records, closed_form_state_count, enumerate_layer0_in_state_slices,
    enumerate_layer0_selecting, full_expansion, layer0_state_count,
    layer0_state_count_with_torn_in_place_overwrites, quick_tier_expansion,
    root_identity_written_by, writes_and_segments, CrashImage, Layer0ObserverCounts,
    Layer0Parallelism, Layer0SegmentExpansion, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice, TearableInPlaceOverwrites, WrittenContents,
};
use singlefs_harness::layer0_progress::Layer0Resume;
use singlefs_harness::segments::{
    segment_kinds_text, segment_sizes_text, split_into_segments, StepKind,
};
use singlefs_harness::{
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
                | RecordedOperationKind::WriteZeroes => 512,
            },
            content_hash: 0,
        },
        contents: match kind {
            RecordedOperationKind::Barrier | RecordedOperationKind::WriteZeroes => None,
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
        closed_form_state_count(&per_device_segments),
        1 + 3 + 3 + 1 + 3
    );
    let kind_segments = split_into_segments(&plain_operations(&per_device), &geometry());
    assert_eq!(segment_sizes_text(&kind_segments), "2+2+1+2");
    assert_eq!(
        segment_kinds_text(&kind_segments),
        "[unit_write×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]"
    );
}

/// 真流上的判别力（第 1 条）：只吞盘 1 在一处的屏障——第一个事务里「journal 记录 → 根槽」那一组里盘 1 那一道，吞在录制器外面。
/// 盘 1 那份记录没被放行，按 A 那道屏障不关段：记录两份、根槽与之后的两次系统配置槽写并成一段，层 0 小流（只展开 5 个写以内的段）里
/// 摆得出「根在案而记录一份都不在」，记录核对器报出来；每盘都发屏障的同一条流报 0。合并屏障的录制器把盘 0 那一道记成整个池的，
/// 两条流逐步相同，报不出来。
#[test]
// crash-case-check:not-a-crash-case 第一条流只展开 5 个写以内的段、26 写那一段不展开，一百来个状态，单跑几秒
fn one_device_missing_its_barrier_before_the_root_slot_is_reported_by_the_record_checker() {
    let (every_device_barriers, stream_lengths_at_barrier_calls_on_device_one) =
        first_transaction_stream(None);
    let root_position = every_device_barriers
        .operations
        .iter()
        .rposition(|retained| geometry().classify(&retained.operation) == StepKind::RootRecordFua)
        .expect("流里有第一个事务的根槽写");
    let barrier_call_before_the_root = stream_lengths_at_barrier_calls_on_device_one
        .iter()
        .rposition(|stream_length| *stream_length < root_position)
        .expect("盘 1 在根槽写之前发过屏障");
    let (device_one_misses_one_barrier, _) =
        first_transaction_stream(Some(barrier_call_before_the_root));
    let record_checker_reports = |recorded: &RecordedFirstTransactionStream| {
        let (writes, segments) = writes_and_segments(
            &recorded.operations[recorded.mkfs_operation_count..],
            &geometry(),
        );
        let root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("根槽");
        let tally = enumerate_layer0_selecting(
            &recorded.base,
            &writes,
            &segments,
            root_index,
            &file_content(),
            &|_segment_index, segment| segment.len() <= 5,
        );
        let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
        (sizes, tally.record_root_without_record, tally.violations)
    };
    let (every_device_sizes, every_device_reports, every_device_violations) =
        record_checker_reports(&every_device_barriers);
    assert_eq!(
        every_device_sizes,
        vec![2, 2, 1, 2, 2, 1, 26, 2, 1, 2],
        "每盘都发屏障：与今天的段序列相同"
    );
    assert_eq!(
        (every_device_reports, every_device_violations),
        (0, 0),
        "每盘都发屏障：记录核对器与 oracle 都不报"
    );
    let (missing_sizes, missing_reports, _) =
        record_checker_reports(&device_one_misses_one_barrier);
    assert_eq!(
        missing_sizes,
        vec![2, 2, 1, 2, 2, 1, 26, 5],
        "盘 1 漏发那一道：记录两份、根槽、两次系统配置槽写并成一段"
    );
    assert!(
        missing_reports > 0,
        "盘 1 漏发屏障：层 0 要摆出根在案而记录一份都不在的状态，记录核对器报出来（报了 {missing_reports} 个）"
    );
}

struct RecordedFirstTransactionStream {
    operations: Vec<RetainedOperation>,
    mkfs_operation_count: usize,
    base: MemoryPool,
}

/// 两块内存盘上 mkfs → 取号 → 暖机 → 第一个事务（与 `common::build_pool` 同一串调用），盘 1 外面可以吞掉第几次屏障；
/// 交回录制流、mkfs 占了几步、mkfs 之后的基镜像，与盘 1 每次屏障调用那一刻录制流有几步。
fn first_transaction_stream(
    swallowed_barrier_call_on_device_one: Option<usize>,
) -> (RecordedFirstTransactionStream, Vec<usize>) {
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
        .expect("第一个事务");
    }
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..mkfs_operation_count]);
    let stream_lengths_at_barrier_calls_on_device_one =
        devices[1].1.stream_lengths_at_each_barrier_call.clone();
    (
        RecordedFirstTransactionStream {
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
        layer0_state_count_with_torn_in_place_overwrites(
            &base,
            &writes,
            &segments,
            &full_expansion
        ),
        1 + 23 + 1 + 2
    );
    assert_eq!(
        layer0_state_count_with_torn_in_place_overwrites(
            &base,
            &writes,
            &segments,
            &quick_tier_expansion
        ),
        1 + 11 + 1 + 2
    );
    assert_eq!(closed_form_state_count(&segments), 1 + 15 + 1 + 1);
    assert_eq!(
        layer0_state_count(&writes, &segments, &quick_tier_expansion),
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
        layer0_state_count_with_torn_in_place_overwrites(
            &empty_base,
            first_writes_only,
            &first_segments,
            &full_expansion
        ),
        closed_form_state_count(&first_segments)
    );
    assert_eq!(
        layer0_state_count_with_torn_in_place_overwrites(
            &empty_base,
            first_writes_only,
            &first_segments,
            &quick_tier_expansion
        ),
        layer0_state_count(first_writes_only, &first_segments, &quick_tier_expansion)
    );
}

/// 第一个事务那条流（层 0 从 mkfs 之后枚举的整条流）：每盘都发屏障，段序列与补第三态之前的闭式与今天逐字相同；
/// 取三态的只有系统配置槽写（8 次，每次都罩住更早写下的旧槽），全量 150994980、甲二 54 个状态。
#[test]
fn the_first_transaction_stream_keeps_its_segments_and_takes_the_third_state_on_its_system_configuration_writes(
) {
    let pool = common::build_pool("torn-third-state-first-stream");
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &geometry(),
    );
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        vec![2, 2, 1, 2, 2, 1, 26, 2, 1, 2]
    );
    assert_eq!(closed_form_state_count(&segments), 67_108_885);
    assert_eq!(
        layer0_state_count(&writes, &segments, &quick_tier_expansion),
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
        vec![StepKind::SystemConfigurationSlot; 8],
        "取号 2 + 两次暖机各 2 + A 的轮换 2"
    );
    assert_eq!(
        layer0_state_count_with_torn_in_place_overwrites(
            &base,
            &writes,
            &segments,
            &full_expansion
        ),
        150_994_980,
        "26 写那一段 3² · 2²⁴ − 1、四个系统配置槽段各 3² − 1、其余同今天"
    );
    assert_eq!(
        layer0_state_count_with_torn_in_place_overwrites(
            &base,
            &writes,
            &segments,
            &quick_tier_expansion
        ),
        54,
        "26 写那一段 3² · 2 − 1、四个系统配置槽段各 3² − 1、其余同今天"
    );
}

/// 撕裂那一态真生成镜像、喂给恢复、checker 与记录核对器：第一条流只展开最后一段（A 的根之后两块盘的系统配置槽轮换），
/// 3² − 1 个状态再加全部持久那一个。每个状态按盘上字节认那两次写怎么落（与这次写的字节相同是落了、与写之前的相同是没落、
/// 都不同是撕裂）：撕裂的那一槽解不开（新旧都读不出），五个状态里至少一槽撕裂；恢复读出文件、oracle、池级 checker、
/// 记录核对器都不报，checker 在每个状态上都跑过（评估过的 + 不适用的 = 状态数）。
#[test]
// crash-case-check:not-a-crash-case 第一条流只展开最后那一段两个写，九个状态，单跑几秒
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
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let mut states_with_a_torn_slot = 0u64;
    let mut torn_slots = 0u64;
    let tally = enumerate_layer0_in_state_slices(
        &base,
        &writes,
        &segments,
        root_index,
        &versions,
        &only_the_last_segment,
        Layer0Parallelism::from_environment(),
        Some(&mut |image: &CrashImage<'_>,
                   consulted_report: &RecoveryReport,
                   _counts: &mut Layer0ObserverCounts| {
            let mut torn_in_this_state = 0u64;
            for (write_index, (old, new)) in rotation_writes
                .iter()
                .zip(&old_and_new_of_each_rotation_write)
            {
                let write = &writes[*write_index];
                let on_disk =
                    PoolReader::read(image, write.device, write.offset, 4096).expect("槽读得出");
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
                states_with_a_torn_slot += 1;
                torn_slots += torn_in_this_state;
            }
            let records = check_records(image, consulted_report.effective_root);
            assert!(
                !records.root_without_record && !records.claimed_state_missing_unit,
                "记录核对器在撕裂镜像上照判、不报"
            );
        }),
        &Layer0Resume::NoProgressFile,
    );
    assert_eq!(tally.states, 9, "最后一段 3² − 1，再加全部持久那一个");
    assert_eq!(
        tally.states,
        layer0_state_count_with_torn_in_place_overwrites(
            &base,
            &writes,
            &segments,
            &only_the_last_segment
        )
    );
    assert_eq!(
        (states_with_a_torn_slot, torn_slots),
        (5, 6),
        "撕裂 × 没落 / 落了 各两个状态、两槽都撕裂一个状态"
    );
    assert_eq!(
        (
            tally.violations,
            tally.failed_states,
            tally.file_read_states
        ),
        (0, 0, 9),
        "A 的根已落：每个状态都读出文件（第一处违例：{:?}）",
        tally.first_violation
    );
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0)
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
}
