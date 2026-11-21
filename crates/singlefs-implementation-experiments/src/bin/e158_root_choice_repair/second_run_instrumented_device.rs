//! 第 2 次跑的仪表盘：按落点让读失败或读成零、按次数让写与屏障失败，连同按写的前缀截出的崩溃状态。

use super::*;

// ---------------------------------------------------------------------------
// 10.1 读法写死表「故障」的落地：一块盘一层、一次装完这块盘的全部落点；读落点按区间罩（不只认起点偏移）；
//      每个落点在这次调用里被拦下的读次数单独记（V2）；这次调用自己写到落点上之后那一处不再拦
//      （写出的字节此后照常读得出，同 `crates/singlefs-verification/src/history.rs` 的 `DeviceReadingZerosOverHiddenRanges`）；
//      另可让整池第 n 次写或第 n 次屏障报错（H1d 的「注入写失败」）。设备一层真落了的写按次序记下（K3 / K4 与崩溃状态靠它）。
// ---------------------------------------------------------------------------

/// 一个落点这次调用里读不出的样子（H1d 两种造法都做：注入读错、读回清零）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UnreadableForm {
    ReadFails,
    ReadsZeros,
}

impl UnreadableForm {
    pub(super) fn name(self) -> &'static str {
        match self {
            UnreadableForm::ReadFails => "read_fails",
            UnreadableForm::ReadsZeros => "reads_zeros",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct UnreadableRange {
    pub(super) device: DeviceIdentity,
    pub(super) offset: u64,
    pub(super) length: u64,
    pub(super) form: UnreadableForm,
    pub(super) label: String,
}

impl UnreadableRange {
    pub(super) fn overlaps(&self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
        self.device == device && self.offset < offset + length && offset < self.offset + self.length
    }
}

/// 设备一层真落了的一步（按整池的先后）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AppliedDeviceStep {
    Write {
        device: DeviceIdentity,
        offset: u64,
        bytes: Vec<u8>,
    },
    ZeroFill {
        device: DeviceIdentity,
        offset: u64,
        length: u64,
    },
    Barrier {
        device: DeviceIdentity,
    },
}

#[derive(Default)]
struct InstrumentationState {
    unreadable: Vec<UnreadableRange>,
    written_over: Vec<bool>,
    intercepted_reads: Vec<u64>,
    /// 整池发出的写（含报了错的那一次），从 0 数。
    write_calls: u64,
    barrier_calls: u64,
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
    injected_failure_fired: bool,
    applied: Vec<AppliedDeviceStep>,
}

impl InstrumentationState {
    pub(super) fn new(
        unreadable: Vec<UnreadableRange>,
        failing_write_call: Option<u64>,
        failing_barrier_call: Option<u64>,
    ) -> Self {
        let count = unreadable.len();
        InstrumentationState {
            unreadable,
            written_over: vec![false; count],
            intercepted_reads: vec![0; count],
            failing_write_call,
            failing_barrier_call,
            ..InstrumentationState::default()
        }
    }

    pub(super) fn mark_written_over(&mut self, device: DeviceIdentity, offset: u64, length: u64) {
        for (range, written_over) in self.unreadable.iter().zip(self.written_over.iter_mut()) {
            if range.overlaps(device, offset, length) {
                *written_over = true;
            }
        }
    }
}

struct InstrumentedDevice {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    state: Rc<RefCell<InstrumentationState>>,
}

impl BlockDevice for InstrumentedDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let mut fails = false;
        let mut zeroed: Vec<(u64, u64)> = Vec::new();
        {
            let mut state = self.state.borrow_mut();
            let state = &mut *state;
            for ((range, written_over), intercepted) in state
                .unreadable
                .iter()
                .zip(state.written_over.iter())
                .zip(state.intercepted_reads.iter_mut())
            {
                if *written_over || !range.overlaps(self.identity, offset.0, length) {
                    continue;
                }
                *intercepted += 1;
                match range.form {
                    UnreadableForm::ReadFails => fails = true,
                    UnreadableForm::ReadsZeros => zeroed.push((range.offset, range.length)),
                }
            }
        }
        if fails {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)?;
        for (range_offset, range_length) in zeroed {
            let start = range_offset.saturating_sub(offset.0);
            let end = (range_offset + range_length - offset.0).min(length);
            buffer[usize::try_from(start).expect("缓冲区里")
                ..usize::try_from(end).expect("缓冲区里")]
                .fill(0);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.write_calls;
        state.write_calls += 1;
        if state.failing_write_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_at(offset, bytes, durability)?;
        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        state.mark_written_over(self.identity, offset.0, length);
        state.applied.push(AppliedDeviceStep::Write {
            device: self.identity,
            offset: offset.0,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.write_calls;
        state.write_calls += 1;
        if state.failing_write_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_zeroes_at(offset, length)?;
        state.mark_written_over(self.identity, offset.0, length);
        state.applied.push(AppliedDeviceStep::ZeroFill {
            device: self.identity,
            offset: offset.0,
            length,
        });
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let mut state = self.state.borrow_mut();
        let ordinal = state.barrier_calls;
        state.barrier_calls += 1;
        if state.failing_barrier_call == Some(ordinal) {
            state.injected_failure_fired = true;
            return Err(injected_block_device_error("屏障"));
        }
        self.inner.barrier()?;
        state.applied.push(AppliedDeviceStep::Barrier {
            device: self.identity,
        });
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一次受观测的可写挂载交回的东西。
pub(super) struct InstrumentedMount {
    pub(super) result: Result<Mounted, singlefs_core::mount::MountError>,
    /// 整池发出的写次数（含报了错的那一次）：K3 / K4 按它是不是 0 分（跑前登记前提 3 的答案）。
    pub(super) write_calls: u64,
    pub(super) applied: Vec<AppliedDeviceStep>,
    pub(super) intercepted_reads: Vec<u64>,
    pub(super) injected_failure_fired: bool,
    pub(super) pool_after: MemoryPool,
}

pub(super) fn run_instrumented_mount_writable(
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    unreadable: &[UnreadableRange],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
) -> InstrumentedMount {
    let state = Rc::new(RefCell::new(InstrumentationState::new(
        unreadable.to_vec(),
        failing_write_call,
        failing_barrier_call,
    )));
    let mut devices: Vec<(DeviceIdentity, InstrumentedDevice)> =
        devices_from_pool(base, IMAGE_BYTES)
            .into_iter()
            .map(|(identity, inner)| {
                (
                    identity,
                    InstrumentedDevice {
                        identity,
                        inner,
                        state: Rc::clone(&state),
                    },
                )
            })
            .collect();
    let result = mount_writable(parameters, &mut devices);
    let plain: Vec<(DeviceIdentity, SparseBlockDevice)> = devices
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    let pool_after = memory_pool_of(&plain, IMAGE_BYTES);
    let state = state.borrow();
    InstrumentedMount {
        result,
        write_calls: state.write_calls,
        applied: state.applied.clone(),
        intercepted_reads: state.intercepted_reads.clone(),
        injected_failure_fired: state.injected_failure_fired,
        pool_after,
    }
}

/// 一次可写挂载的结局按 K0–K4 分（跑前登记前提 3 的答案）：K0 做成且取号之前判过（或测试开关没判）、K1 写行之后推过抬 F 够了、
/// K2 推满仍不够而挂载照样做成、K3 报错且这次调用设备一层 0 次写、K4 报错且写过。
pub(super) fn mount_outcome_class(
    result: &Result<Mounted, singlefs_core::mount::MountError>,
    write_calls: u64,
) -> &'static str {
    use singlefs_core::mount::MountSpaceAdmission;
    match result {
        Ok(mounted) => match mounted.output.space_admission {
            MountSpaceAdmission::AdmittedBeforeAcquisition
            | MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => "K0",
            MountSpaceAdmission::AdmittedAfterTheFloorRaises { .. } => "K1",
            MountSpaceAdmission::StillShortAfterTheFloorRaises { .. } => "K2",
        },
        Err(_) if write_calls == 0 => "K3",
        Err(_) => "K4",
    }
}

pub(super) fn mount_error_member_or_none(
    result: &Result<Mounted, singlefs_core::mount::MountError>,
) -> String {
    match result {
        Ok(_) => "none".to_string(),
        Err(error) => error_member_of_debug(&format!("{error:?}")),
    }
}

/// `base` 上依次施加 `applied` 里前 `write_count` 个写（屏障不改字节），交回那份镜像：崩溃在第 `write_count` 个写之前、
/// 之前的写都已持久的那一个崩溃状态（前缀）。
pub(super) fn image_with_the_first_writes(
    base: &MemoryPool,
    applied: &[AppliedDeviceStep],
    write_count: usize,
) -> MemoryPool {
    let mut devices = devices_from_pool(base, IMAGE_BYTES);
    let mut written = 0usize;
    for step in applied {
        if written == write_count {
            break;
        }
        match step {
            AppliedDeviceStep::Write {
                device,
                offset,
                bytes,
            } => {
                if let Some((_, target)) =
                    devices.iter_mut().find(|(identity, _)| identity == device)
                {
                    target
                        .write_at(DeviceOffsetInBytes(*offset), bytes, WriteDurability::Plain)
                        .expect("内存盘写不报错");
                }
                written += 1;
            }
            AppliedDeviceStep::ZeroFill {
                device,
                offset,
                length,
            } => {
                if let Some((_, target)) =
                    devices.iter_mut().find(|(identity, _)| identity == device)
                {
                    target
                        .write_zeroes_at(DeviceOffsetInBytes(*offset), *length)
                        .expect("内存盘清零不报错");
                }
                written += 1;
            }
            AppliedDeviceStep::Barrier { .. } => {}
        }
    }
    memory_pool_of(&devices, IMAGE_BYTES)
}

/// 根环每个槽在哪块盘的哪个偏移（checker 的独立解码，`checker_image::root_slot_positions`）。
pub(super) fn root_slot_device_offsets(geometry: &PoolGeometry) -> Vec<(DeviceIdentity, u64)> {
    checker_image::root_slot_positions(geometry)
        .into_iter()
        .map(|(_, _, device, offset)| (DeviceIdentity(device), offset))
        .collect()
}

/// 设备一层的第几个写（从 0 数）是第一个落到根环槽上的：可写挂载里第一个根槽写就是写行那次发布的根（取号只写系统配置）。
pub(super) fn index_of_the_first_root_slot_write(
    applied: &[AppliedDeviceStep],
    root_slots: &[(DeviceIdentity, u64)],
    root_slot_bytes: u64,
) -> Option<usize> {
    let mut write_index = 0usize;
    for step in applied {
        let (device, offset, length) = match step {
            AppliedDeviceStep::Write {
                device,
                offset,
                bytes,
            } => (*device, *offset, u64::try_from(bytes.len()).expect("写长")),
            AppliedDeviceStep::ZeroFill {
                device,
                offset,
                length,
            } => (*device, *offset, *length),
            AppliedDeviceStep::Barrier { .. } => continue,
        };
        if root_slots.iter().any(|(slot_device, slot_offset)| {
            *slot_device == device
                && *slot_offset < offset + length
                && offset < slot_offset + root_slot_bytes
        }) {
            return Some(write_index);
        }
        write_index += 1;
    }
    None
}

/// 前 `write_count` 个写之间发过几次屏障（整池，从 0 数的屏障序号都小于它）。
pub(super) fn barriers_before_write(applied: &[AppliedDeviceStep], write_count: usize) -> u64 {
    let mut writes = 0usize;
    let mut barriers = 0u64;
    for step in applied {
        match step {
            AppliedDeviceStep::Barrier { .. } => {
                if writes < write_count {
                    barriers += 1;
                }
            }
            AppliedDeviceStep::Write { .. } | AppliedDeviceStep::ZeroFill { .. } => {
                writes += 1;
                if writes >= write_count {
                    break;
                }
            }
        }
    }
    barriers
}

#[cfg(test)]
mod tests;
