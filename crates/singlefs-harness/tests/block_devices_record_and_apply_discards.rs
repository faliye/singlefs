//! discard 那个口子在 harness 档里的几样（`research/prompts/m4-gc-r1-main-verification.md` 第三节 G5「trait 形态」）：
//! 内存盘把它当整段清零应用、录制器把交给了底层的那一次记成一步、不认 discard 的后端交回「不支持」且看得见、
//! 切段把它当一次普通写、种类自成一种 `discard`（C829（discard 与 mkfs 清零在段种类串里同名）；崩溃点重放按「范围写零」重建，
//! 取几态照 D13（验证路线） 已定项 4，长于一个扇区、罩住旧内容的第三态在 checker 档 `crash` 的用例里）、长度 0 的经录制器不记一步、
//! 故障注入设备转给里面那块盘并注入得了失败。
//! 产品路径这一批还不发 discard（谁发、什么时候发归下一批），这里直接调块设备。

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, DiscardOutcomeCounts, PhysicalBlockSizeInBytes,
    WriteDurability,
};
use singlefs_core::recovery::PoolReader;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, SparseBlockDevice,
    WrittenContents,
};
use singlefs_harness::segments::{
    segment_kinds_text, segment_sizes_text, split_into_segments, FixedGeometry, StepKind,
};
use singlefs_harness::{
    fnv1a_64, fnv1a_64_of_zeros, RecordedOperationKind, RecordingBlockDevice, SharedStream,
};

const SECTOR: usize = 512;
const DIRT_BYTE: u8 = 0xC3;
/// 内存盘的大小：要装得下单元区里的落点（第一版几何里单元区从槽 50176 起，槽宽 16 KiB）。
const DEVICE_BYTES: u64 = 1 << 30;
/// 单元区里的一个落点：切段时普通写落在这里归「写单元」。
const UNIT_OFFSET: u64 = 50176 * 16384;

fn geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: 768 << 20,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    }
}

fn sector_of(device: &impl BlockDevice, offset: u64) -> Vec<u8> {
    let mut sector = vec![0x5Au8; SECTOR];
    device
        .read_at(DeviceOffsetInBytes(offset), &mut sector)
        .expect("读一扇区");
    sector
}

/// 内存盘：前四扇区先写脏，再 discard 第二、三扇区。
fn memory_disk_with_two_sectors_discarded() -> (SparseBlockDevice, DiscardOutcome) {
    let mut device = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
    device
        .write_at(
            DeviceOffsetInBytes(0),
            &[DIRT_BYTE; 4 * SECTOR],
            WriteDurability::Plain,
        )
        .expect("写脏");
    let outcome = device
        .discard_at(DeviceOffsetInBytes(512), 1024)
        .expect("discard");
    (device, outcome)
}

/// 内存盘认 discard、当整段清零应用：段内读回全 0，段外一个字节不动，交回「交给了底层」。
#[test]
fn a_discard_on_the_memory_disk_reads_back_zero_and_leaves_the_neighbours() {
    let (device, outcome) = memory_disk_with_two_sectors_discarded();
    assert_eq!(outcome, DiscardOutcome::Applied);
    for offset in [512u64, 1024] {
        assert_eq!(
            sector_of(&device, offset),
            vec![0u8; SECTOR],
            "偏移 {offset} 在 discard 段内，读回全 0"
        );
    }
    for offset in [0u64, 1536] {
        assert_eq!(
            sector_of(&device, offset),
            vec![DIRT_BYTE; SECTOR],
            "偏移 {offset} 在 discard 段外，不该被动"
        );
    }
}

/// 录制器把交给了底层的 discard 记成一步：种类 `discard`、偏移长度照发、哈希按全 0 记、开了内容保留也不留字节；
/// 这条流数到一次「交给了底层」。
#[test]
fn the_recorder_records_an_applied_discard_as_one_step_without_contents() {
    let stream = SharedStream::retaining_contents();
    let mut recorder = RecordingBlockDevice::with_shared_stream(
        DeviceIdentity(0),
        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
        stream.clone(),
    );
    recorder
        .write_at(
            DeviceOffsetInBytes(0),
            &[DIRT_BYTE; 2 * SECTOR],
            WriteDurability::Plain,
        )
        .expect("写");
    let outcome = recorder
        .discard_at(DeviceOffsetInBytes(512), 512)
        .expect("discard");
    assert_eq!(outcome, DiscardOutcome::Applied);
    let operations = stream.operations();
    assert_eq!(operations.len(), 2, "一次写 + 一次 discard = 两步");
    assert_eq!(operations[1].kind, RecordedOperationKind::Discard);
    assert_eq!(operations[1].device, DeviceIdentity(0));
    assert_eq!(operations[1].offset, DeviceOffsetInBytes(512));
    assert_eq!(operations[1].length, 512);
    assert_eq!(operations[1].content_hash, fnv1a_64(&[0u8; SECTOR]));
    assert_eq!(
        operations[1].to_stream_line(),
        format!(
            "device=0 kind=discard offset=512 length=512 hash={:016x}",
            fnv1a_64_of_zeros(512)
        )
    );
    let retained = stream.retained_operations();
    assert!(retained[1].contents.is_none(), "discard 不留内容");
    assert_eq!(
        retained[1].written_bytes_rebuilding_zero_fills(),
        Some(vec![0u8; SECTOR]),
        "重建 discard 写下的字节：按长度铺 0（「范围写零」）"
    );
    assert_eq!(
        stream.discard_outcome_counts(),
        DiscardOutcomeCounts {
            applied_calls: 1,
            unsupported_calls: 0
        }
    );
    assert_eq!(
        sector_of(&recorder, 512),
        vec![0u8; SECTOR],
        "录制器把 discard 转给了里面那块盘"
    );
}

/// 只实现六个旧动作、discard 用默认实现的一块盘（不认 discard 的后端）。
struct DeviceKeepingTheDefaultDiscard {
    bytes: Vec<u8>,
}

impl BlockDevice for DeviceKeepingTheDefaultDiscard {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let start = usize::try_from(offset.0).expect("测试偏移装得进 usize");
        buffer.copy_from_slice(&self.bytes[start..start + buffer.len()]);
        Ok(())
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        _durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let start = usize::try_from(offset.0).expect("测试偏移装得进 usize");
        self.bytes[start..start + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let start = usize::try_from(offset.0).expect("测试偏移装得进 usize");
        let end = start + usize::try_from(length).expect("测试长度装得进 usize");
        self.bytes[start..end].fill(0);
        Ok(())
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(512)
    }
    fn size_in_bytes(&self) -> u64 {
        u64::try_from(self.bytes.len()).expect("测试镜像装得进 u64")
    }
}

/// 默认实现：交回「不支持」、一个字节不动；录制器不给它记一步，只把它数进「不支持」；没对齐的照样拒、不数。
#[test]
fn a_backend_keeping_the_default_answers_unsupported_and_the_recorder_counts_it_without_a_step() {
    let stream = SharedStream::new();
    let mut recorder = RecordingBlockDevice::with_shared_stream(
        DeviceIdentity(0),
        DeviceKeepingTheDefaultDiscard {
            bytes: vec![DIRT_BYTE; 4 * SECTOR],
        },
        stream.clone(),
    );
    let outcome = recorder
        .discard_at(DeviceOffsetInBytes(512), 1024)
        .expect("discard");
    assert_eq!(outcome, DiscardOutcome::Unsupported);
    assert!(
        stream.operations().is_empty(),
        "「不支持」的 discard 不进录制流"
    );
    assert_eq!(
        sector_of(&recorder, 512),
        vec![DIRT_BYTE; SECTOR],
        "不支持 discard 的后端一个字节不动"
    );
    assert!(matches!(
        recorder.discard_at(DeviceOffsetInBytes(100), 512),
        Err(BlockDeviceError::Unaligned { .. })
    ));
    assert_eq!(
        stream.discard_outcome_counts(),
        DiscardOutcomeCounts {
            applied_calls: 0,
            unsupported_calls: 1
        },
        "交回「不支持」的那一次看得见；被拒的那一次两边都不数"
    );
}

/// 内存盘上长度 0 的 discard 没有动作：交回「交给了底层」，周围一个字节不动（判决 m4-batch2-code-r1 S5，四个后端同一结局；
/// 内存盘没有 discard 计数）。
#[test]
fn a_zero_length_discard_on_the_memory_disk_is_applied_and_changes_nothing() {
    let mut device = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
    device
        .write_at(
            DeviceOffsetInBytes(512),
            &[DIRT_BYTE; SECTOR],
            WriteDurability::Plain,
        )
        .expect("写脏");
    let outcome = device
        .discard_at(DeviceOffsetInBytes(512), 0)
        .expect("长度 0 的 discard");
    assert_eq!(outcome, DiscardOutcome::Applied, "长度 0 交回交给了底层");
    assert_eq!(sector_of(&device, 512), vec![DIRT_BYTE; SECTOR]);
}

/// discard 用默认实现的后端上长度 0 的 discard 没有动作：交回「交给了底层」（不是「不支持」），一个字节不动
/// （判决 m4-batch2-code-r1 S5；默认实现没有状态、不记数）。
#[test]
fn a_zero_length_discard_on_a_backend_keeping_the_default_is_applied_and_changes_nothing() {
    let mut device = DeviceKeepingTheDefaultDiscard {
        bytes: vec![DIRT_BYTE; 4 * SECTOR],
    };
    let outcome = device
        .discard_at(DeviceOffsetInBytes(512), 0)
        .expect("长度 0 的 discard");
    assert_eq!(
        outcome,
        DiscardOutcome::Applied,
        "长度 0 交回交给了底层，不是不支持"
    );
    assert_eq!(device.bytes, vec![DIRT_BYTE; 4 * SECTOR]);
}

/// 一次写单元、一次 discard（落在已经写过旧字节的地方）、一道屏障的流，带内容；前面先有一次落了盘的旧写当基线。
fn base_pool_and_stream_with_one_discard() -> (MemoryPool, Vec<singlefs_harness::RetainedOperation>)
{
    let stream = SharedStream::retaining_contents();
    let mut recorder = RecordingBlockDevice::with_shared_stream(
        DeviceIdentity(0),
        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
        stream.clone(),
    );
    recorder
        .write_at(
            DeviceOffsetInBytes(UNIT_OFFSET + 16384),
            &[DIRT_BYTE; SECTOR],
            WriteDurability::Plain,
        )
        .expect("旧写");
    recorder.barrier().expect("旧写的屏障");
    let base_operations = stream.retained_operations();
    recorder
        .write_at(
            DeviceOffsetInBytes(UNIT_OFFSET),
            &[7u8; SECTOR],
            WriteDurability::Plain,
        )
        .expect("写单元");
    recorder
        .discard_at(DeviceOffsetInBytes(UNIT_OFFSET + 16384), 512)
        .expect("discard");
    recorder.barrier().expect("屏障");
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0)], DEVICE_BYTES);
    base.apply(&base_operations);
    let operations = stream.retained_operations()[base_operations.len()..].to_vec();
    (base, operations)
}

/// 切段把 discard 当一次普通写：种类按动作自成一种（`discard`），不放行、不关段，与前面的单元写同段；
/// 写表里它是一次长 512 的全 0 写，不长于一个扇区、只取两态（D13（验证路线） 已定项 4）——持久时那一段读回 0，没持久时是旧字节。
#[test]
fn segments_count_a_discard_as_one_plain_write_with_two_crash_states() {
    let (base, operations) = base_pool_and_stream_with_one_discard();
    let recorded: Vec<_> = operations
        .iter()
        .map(|retained| retained.operation.clone())
        .collect();
    let kinds_by_segment = split_into_segments(&recorded, &geometry());
    assert_eq!(segment_sizes_text(&kinds_by_segment), "2");
    assert_eq!(
        segment_kinds_text(&kinds_by_segment),
        "[discard,unit_write,barrier]"
    );
    let (writes, segments) = writes_and_segments(&operations, &geometry());
    assert_eq!(segments, vec![vec![0, 1]], "单元写与 discard 同段");
    assert_eq!(writes[1].kind, StepKind::Discard);
    assert_eq!(writes[1].contents, WrittenContents::Zeros { length: 512 });
    assert!(!writes[1].is_force_unit_access);
    assert_eq!(
        closed_form_state_count(&segments),
        1 + ((1 << 2) - 1),
        "两次写、每次两态：1 + (2^2 − 1)"
    );
    let discard_persisted = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![false, true],
    };
    assert_eq!(
        PoolReader::read(
            &discard_persisted,
            DeviceIdentity(0),
            DeviceOffsetInBytes(UNIT_OFFSET + 16384),
            SECTOR
        ),
        Some(vec![0u8; SECTOR]),
        "discard 持久那一态：那一段读回 0"
    );
    let discard_not_persisted = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true, false],
    };
    assert_eq!(
        PoolReader::read(
            &discard_not_persisted,
            DeviceIdentity(0),
            DeviceOffsetInBytes(UNIT_OFFSET + 16384),
            SECTOR
        ),
        Some(vec![DIRT_BYTE; SECTOR]),
        "discard 没持久那一态：那一段是旧字节"
    );
}

/// 把录到的流整个施加到内存池上（「整段流都落了」那一份镜像）：discard 那一段清成 0。
#[test]
fn applying_a_recorded_discard_to_a_memory_pool_zero_fills_its_range() {
    let (mut pool, operations) = base_pool_and_stream_with_one_discard();
    pool.apply(&operations);
    assert_eq!(
        PoolReader::read(
            &pool,
            DeviceIdentity(0),
            DeviceOffsetInBytes(UNIT_OFFSET + 16384),
            SECTOR
        ),
        Some(vec![0u8; SECTOR])
    );
    assert_eq!(
        PoolReader::read(
            &pool,
            DeviceIdentity(0),
            DeviceOffsetInBytes(UNIT_OFFSET),
            SECTOR
        ),
        Some(vec![7u8; SECTOR]),
        "discard 之外的写照落"
    );
}

/// 故障注入设备不注入时把 discard 原样转给里面那块盘、交回它的结局，记进这块盘的 discard 计数，不算进写数。
#[test]
fn the_fault_injecting_device_forwards_a_discard_to_the_device_inside() {
    let plan = SharedFaultPlan::unarmed(geometry());
    let mut device = FaultInjectingBlockDevice::new(
        DeviceIdentity(0),
        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
        plan.clone(),
    );
    device
        .write_at(
            DeviceOffsetInBytes(2048),
            &[DIRT_BYTE; SECTOR],
            WriteDurability::Plain,
        )
        .expect("写脏");
    let outcome = device
        .discard_at(DeviceOffsetInBytes(2048), 512)
        .expect("discard");
    assert_eq!(outcome, DiscardOutcome::Applied);
    assert_eq!(sector_of(&device, 2048), vec![0u8; SECTOR]);
    let counts = plan.counts_of_device(DeviceIdentity(0));
    assert_eq!(counts.discards_applied, 1);
    assert_eq!(counts.discards_answered_unsupported, 0);
    assert_eq!(counts.writes, 1, "discard 不算进写数");
}

/// 故障注入设备让 discard 报错：报块设备错，里面那块盘一个字节不动，也不记进 discard 计数。
#[test]
fn the_fault_injecting_device_makes_a_discard_fail_and_leaves_the_device_inside_untouched() {
    let plan = SharedFaultPlan::armed(
        geometry(),
        FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, 1),
    );
    let mut sparse = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
    sparse
        .write_at(
            DeviceOffsetInBytes(512),
            &[DIRT_BYTE; SECTOR],
            WriteDurability::Plain,
        )
        .expect("写脏（注入层外面）");
    let mut device = FaultInjectingBlockDevice::new(DeviceIdentity(0), sparse, plan.clone());
    let outcome = device.discard_at(DeviceOffsetInBytes(512), 512);
    assert!(
        matches!(outcome, Err(BlockDeviceError::InputOutput(_))),
        "注入的失败交回块设备错：{outcome:?}"
    );
    assert_eq!(sector_of(&device, 512), vec![DIRT_BYTE; SECTOR]);
    assert_eq!(plan.fired_count(), 1);
    assert_eq!(plan.counts_of_device(DeviceIdentity(0)).discards_applied, 0);
}

/// 录制流的种类名：discard 是 `discard`，与另四种互不相同。
#[test]
fn the_discard_kind_has_its_own_stream_name() {
    let names = [
        RecordedOperationKind::Write.name(),
        RecordedOperationKind::WriteForceUnitAccess.name(),
        RecordedOperationKind::WriteZeroes.name(),
        RecordedOperationKind::Barrier.name(),
    ];
    assert_eq!(RecordedOperationKind::Discard.name(), "discard");
    assert!(!names.contains(&RecordedOperationKind::Discard.name()));
}

/// 经录制器发长度 0 的 discard：照样转给里面那块盘核请求（偏移不对齐照报错），核过了没有动作——不记一步、不计数，
/// 与后端同一口径（判决 m4-batch2-code-r1 S5）。
#[test]
fn a_zero_length_discard_through_the_recorder_records_no_step_and_counts_nothing() {
    let stream = SharedStream::retaining_contents();
    let mut recorder = RecordingBlockDevice::with_shared_stream(
        DeviceIdentity(0),
        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
        stream.clone(),
    );
    let outcome = recorder
        .discard_at(DeviceOffsetInBytes(512), 0)
        .expect("长度 0 的 discard");
    assert_eq!(outcome, DiscardOutcome::Applied, "长度 0 交回交给了底层");
    assert_eq!(stream.operation_count(), 0, "长度 0 的 discard 不记一步");
    assert_eq!(
        stream.discard_outcome_counts(),
        DiscardOutcomeCounts::NONE,
        "长度 0 的 discard 不计数"
    );
    assert!(
        matches!(
            recorder.discard_at(DeviceOffsetInBytes(100), 0),
            Err(BlockDeviceError::Unaligned { .. })
        ),
        "偏移不对齐的长度 0 照样报错：请求由里面那块盘核"
    );
    assert_eq!(stream.operation_count(), 0, "报错的也不记");
}

/// 段种类串分得出 discard 与 mkfs 的整段清零（C829（discard 与 mkfs 清零在段种类串里同名））：同一块盘上一次整段清零、
/// 一次 discard、一道屏障切成一段，种类串是 `[zero_fill,discard,barrier]`，两种步各有各的名字。
#[test]
fn segment_kind_strings_tell_a_discard_from_a_make_filesystem_zero_fill() {
    let stream = SharedStream::retaining_contents();
    let mut recorder = RecordingBlockDevice::with_shared_stream(
        DeviceIdentity(0),
        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
        stream.clone(),
    );
    recorder
        .write_zeroes_at(DeviceOffsetInBytes(1024 * 16384), 4096)
        .expect("整段清零");
    recorder
        .discard_at(DeviceOffsetInBytes(UNIT_OFFSET), 16384)
        .expect("discard");
    recorder.barrier().expect("屏障");
    let kinds_by_segment = split_into_segments(&stream.operations(), &geometry());
    assert_eq!(
        kinds_by_segment,
        vec![vec![
            StepKind::ZeroFill,
            StepKind::Discard,
            StepKind::Barrier
        ]]
    );
    assert_eq!(
        segment_kinds_text(&kinds_by_segment),
        "[zero_fill,discard,barrier]"
    );
    assert_ne!(StepKind::ZeroFill.name(), StepKind::Discard.name());
}
