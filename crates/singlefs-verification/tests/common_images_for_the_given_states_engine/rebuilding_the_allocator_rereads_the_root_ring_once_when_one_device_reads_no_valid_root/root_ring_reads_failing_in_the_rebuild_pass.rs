//! 重建分配器那一遍读根环时注入的读错：哪块盘、哪几槽报读错、读回全零或读回乱字节，几次读之后才坏，与按这份计划读坏的盘。

use super::*;

/// 重建分配器整环重读那一遍是每个槽的第几次读（紧接着重建那一遍）。
pub const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD: u64 =
    READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD + 1;

/// 读回乱字节那一类填的字节（攻方原型 `k.rs` 同一个值）：不是全 0，自证不过。
pub const GARBAGE_BYTE: u8 = 0xA5;

/// 重建那一遍点名的槽这一次读成什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootRingSlotReadFault {
    /// 报块设备错（攻方计划的 `e`）。
    ReadFails,
    /// 报成功、读回全 0（`z`）。
    ReadsBackAllZero,
    /// 报成功、读回不是全 0 而自证不过的乱字节（`g`）。
    ReadsBackGarbage,
}

/// 一块盘上重建那一遍点名哪几个根环槽。
#[derive(Clone, Copy, Debug)]
pub enum RootRingSlotsOfOneDevice {
    /// `-`：一个都不点名。
    NoSlot,
    /// `all`：每一个。
    Every,
    /// `but1`：除了这块盘上最新那条自证过的根所在的槽之外的每一个。
    EveryButTheSlotOfTheNewestRoot,
}

/// 攻方的一份读坏计划：两块盘各点名哪几个槽、点名的槽读成什么。
#[derive(Clone, Copy, Debug)]
pub struct RebuildPassPlan {
    pub device_zero: RootRingSlotsOfOneDevice,
    pub device_one: RootRingSlotsOfOneDevice,
    pub fault: RootRingSlotReadFault,
}

/// `P5:all/-:e`
pub const DEVICE_ZERO_REPORTS_READ_ERRORS: RebuildPassPlan = RebuildPassPlan {
    device_zero: RootRingSlotsOfOneDevice::Every,
    device_one: RootRingSlotsOfOneDevice::NoSlot,
    fault: RootRingSlotReadFault::ReadFails,
};

/// `P5:-/all:e`
pub const DEVICE_ONE_REPORTS_READ_ERRORS: RebuildPassPlan = RebuildPassPlan {
    device_zero: RootRingSlotsOfOneDevice::NoSlot,
    device_one: RootRingSlotsOfOneDevice::Every,
    fault: RootRingSlotReadFault::ReadFails,
};

/// `P5:but1/all:z`
pub const DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS: RebuildPassPlan =
    RebuildPassPlan {
        device_zero: RootRingSlotsOfOneDevice::EveryButTheSlotOfTheNewestRoot,
        device_one: RootRingSlotsOfOneDevice::Every,
        fault: RootRingSlotReadFault::ReadsBackAllZero,
    };

/// `P5:all/but1:g`
pub const DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE: RebuildPassPlan =
    RebuildPassPlan {
        device_zero: RootRingSlotsOfOneDevice::Every,
        device_one: RootRingSlotsOfOneDevice::EveryButTheSlotOfTheNewestRoot,
        fault: RootRingSlotReadFault::ReadsBackGarbage,
    };

/// 这个进程写过一个根环槽之后，点名读坏它的那几次还照不照坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultsOnARootRingSlotThisProcessWrote {
    /// 照坏（坏道写不好）。
    KeepFailing,
    /// 不再坏（写把坏道换掉了；攻方原型 `research/prompts/m3-batch10i-code-r1-opus-model/main.rs` 的 `FaultDevice` 同一个做法）。
    StopFailing,
}

/// 根环每个槽被读的次数与点名读坏的那几次（几块盘共用一份）。
pub struct RootRingReadsWithFaultsState {
    pub slot_spacing_in_bytes: u64,
    /// 每个根环槽在哪块盘、盘内从哪个偏移起。
    pub slots: Vec<(RootRingSlot, DeviceIdentity, u64)>,
    /// （槽，这个槽第几次被读，从 1 数）→ 这一次读成什么。
    pub faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
    /// 槽 →（从这个槽第几次被读起，每一次都读成什么）：坏道从这一刻起一直坏。`faulted_reads_of_each_slot` 点名的那一次先照它。
    pub slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
    pub reads_of_each_slot: BTreeMap<RootRingSlot, u64>,
    pub faulted_reads: u64,
    pub faults_on_a_slot_this_process_wrote: FaultsOnARootRingSlotThisProcessWrote,
    pub slots_this_process_wrote: std::collections::BTreeSet<RootRingSlot>,
    /// 每一次根环槽读（槽，这个槽第几次被读）按发生的次序，中间每一段写记成一项 `None`（连着的写只记一项）。
    pub reads_and_writes_in_order: Vec<Option<(RootRingSlot, u64)>>,
}

#[derive(Clone)]
pub struct SharedRootRingReadsWithFaults(Rc<RefCell<RootRingReadsWithFaultsState>>);

impl SharedRootRingReadsWithFaults {
    /// 这一次读落在哪个根环槽里就给那个槽记一次；是点名读坏的那一次，交回这一次读成什么与那个槽在盘内的起点。
    pub fn note_a_read_and_tell_its_fault(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
    ) -> Option<(RootRingSlotReadFault, u64)> {
        let mut state = self.0.borrow_mut();
        let spacing = state.slot_spacing_in_bytes;
        let (ring_slot, slot_start) = state
            .slots
            .iter()
            .find(|(_, slot_device, start)| {
                *slot_device == device && offset.0 >= *start && offset.0 < *start + spacing
            })
            .map(|(ring_slot, _, start)| (*ring_slot, *start))?;
        let reads_of_this_slot = {
            let reads = state.reads_of_each_slot.entry(ring_slot).or_insert(0);
            *reads += 1;
            *reads
        };
        state
            .reads_and_writes_in_order
            .push(Some((ring_slot, reads_of_this_slot)));
        if state.faults_on_a_slot_this_process_wrote
            == FaultsOnARootRingSlotThisProcessWrote::StopFailing
            && state.slots_this_process_wrote.contains(&ring_slot)
        {
            return None;
        }
        let fault = state
            .faulted_reads_of_each_slot
            .get(&(ring_slot, reads_of_this_slot))
            .copied()
            .or_else(|| {
                state
                    .slots_failing_from_a_read_onward
                    .get(&ring_slot)
                    .filter(|(first_failing_read, _)| reads_of_this_slot >= *first_failing_read)
                    .map(|(_, fault)| *fault)
            })?;
        state.faulted_reads += 1;
        Some((fault, slot_start))
    }

    pub fn reads_of_each_slot(&self) -> BTreeMap<RootRingSlot, u64> {
        self.0.borrow().reads_of_each_slot.clone()
    }

    pub fn faulted_reads(&self) -> u64 {
        self.0.borrow().faulted_reads
    }

    /// 记一次写：写盖到的根环槽记成这个进程写过的；读写次序里记一项写（紧接着上一项写的不重复记）。
    pub fn note_a_write(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: u64) {
        let mut state = self.0.borrow_mut();
        let spacing = state.slot_spacing_in_bytes;
        let written: Vec<RootRingSlot> = state
            .slots
            .iter()
            .filter(|(_, slot_device, start)| {
                *slot_device == device && offset.0 < *start + spacing && offset.0 + length > *start
            })
            .map(|(ring_slot, _, _)| *ring_slot)
            .collect();
        state.slots_this_process_wrote.extend(written);
        if state.reads_and_writes_in_order.last() != Some(&None) {
            state.reads_and_writes_in_order.push(None);
        }
    }

    pub fn reads_and_writes_in_order(&self) -> Vec<Option<(RootRingSlot, u64)>> {
        self.0.borrow().reads_and_writes_in_order.clone()
    }
}

/// 读的时候按 [`SharedRootRingReadsWithFaults`] 让点名的那几次根环槽读读坏的盘；写、写零、屏障原样交给里面那块盘。
pub struct DeviceFailingTheRebuildPass {
    pub identity: DeviceIdentity,
    pub inner: SparseBlockDevice,
    pub root_ring_reads: SharedRootRingReadsWithFaults,
}

impl BlockDevice for DeviceFailingTheRebuildPass {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let fault = self
            .root_ring_reads
            .note_a_read_and_tell_its_fault(self.identity, offset);
        let filler = match fault {
            None => return self.inner.read_at(offset, buffer),
            Some((RootRingSlotReadFault::ReadFails, _)) => {
                return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                    "用例点名的这一次根环槽读坏",
                )));
            }
            Some((RootRingSlotReadFault::ReadsBackAllZero, slot_start)) => (0u8, slot_start),
            Some((RootRingSlotReadFault::ReadsBackGarbage, slot_start)) => {
                (GARBAGE_BYTE, slot_start)
            }
        };
        self.inner.read_at(offset, buffer)?;
        let (byte, slot_start) = filler;
        let spacing = self.root_ring_reads.0.borrow().slot_spacing_in_bytes;
        let first_byte_in_the_buffer = usize::try_from(slot_start.saturating_sub(offset.0))
            .expect("缓冲区里的偏移装得进 usize");
        let end_in_the_buffer = usize::try_from(
            (slot_start + spacing - offset.0).min(u64::try_from(buffer.len()).expect("缓冲区长度")),
        )
        .expect("缓冲区里的偏移装得进 usize");
        buffer[first_byte_in_the_buffer..end_in_the_buffer].fill(byte);
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.root_ring_reads.note_a_write(
            self.identity,
            offset,
            u64::try_from(bytes.len()).expect("写的长度装得进 u64"),
        );
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.root_ring_reads
            .note_a_write(self.identity, offset, length);
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 攻方的两个起点，与一个丢任一块盘之后仍有活文件的起点。
#[derive(Clone, Copy, Debug)]
pub enum StartingPoint {
    /// `192:9:3:1` n = 8。
    UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
    /// `216:7:5:4` n = now。
    UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
    /// `264:6:4`：不补环窗口、准入关着造的窄盘（单元区 264 槽），第一个文件之后覆盖写 6 次、顺序写长到 4 个单元就崩
    /// （攻方第一次抽样 B 段的 `264:6:4-live-crash` 起点）；重开可写（准入判着、影子账开）做完就崩：两块盘上最新的根都是那次暖机的，
    /// 带着同一个活文件。
    UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
    /// `narrow-384-20-2`（`narrow_start(384, 20, 2)` 的造法）：
    /// 不补环窗口、准入关着造的窄盘（单元区 384 槽），第一个文件之后覆盖写 20 次、顺序写长到 2 个单元，进程退出（不再挂）。
    /// 再可写挂载时准入在取号之前不够，写行与暖机之后推抬 F。
    UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
}

/// 再挂之前的盘面，与「卸载前」那一份（丢盘整读拿它比；n = now 那一个起点两份相同）。
pub struct PoolBeforeTheRemount {
    pub image_before_the_crash: MemoryPool,
    pub image_to_remount: MemoryPool,
    pub parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
}

pub fn root_ring_reads_failing(
    image: &MemoryPool,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
) -> SharedRootRingReadsWithFaults {
    root_ring_reads_failing_also_from_a_read_onward(
        image,
        faulted_reads_of_each_slot,
        BTreeMap::new(),
    )
}

/// 同 [`root_ring_reads_failing`]，另有几个槽从它第几次被读起每一次都读坏。
pub fn root_ring_reads_failing_also_from_a_read_onward(
    image: &MemoryPool,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
    slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
) -> SharedRootRingReadsWithFaults {
    root_ring_reads_failing_with(
        image,
        faulted_reads_of_each_slot,
        slots_failing_from_a_read_onward,
        FaultsOnARootRingSlotThisProcessWrote::KeepFailing,
    )
}

/// 同 [`root_ring_reads_failing_also_from_a_read_onward`]，另按 `faults_on_a_slot_this_process_wrote` 定写过的槽还坏不坏。
pub fn root_ring_reads_failing_with(
    image: &MemoryPool,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
    slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
    faults_on_a_slot_this_process_wrote: FaultsOnARootRingSlotThisProcessWrote,
) -> SharedRootRingReadsWithFaults {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = system_configuration.immutable;
    let slots = every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .map(|ring_slot| {
            (
                ring_slot,
                immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
                slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing).0,
            )
        })
        .collect();
    SharedRootRingReadsWithFaults(Rc::new(RefCell::new(RootRingReadsWithFaultsState {
        slot_spacing_in_bytes: u64::from(immutable.sizes.fixed_structure_slot_spacing),
        slots,
        faulted_reads_of_each_slot,
        slots_failing_from_a_read_onward,
        reads_of_each_slot: BTreeMap::new(),
        faulted_reads: 0,
        faults_on_a_slot_this_process_wrote,
        slots_this_process_wrote: std::collections::BTreeSet::new(),
        reads_and_writes_in_order: Vec::new(),
    })))
}

impl DeviceOverASparseImage for DeviceFailingTheRebuildPass {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}
