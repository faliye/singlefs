//! 可写挂载这次挂载里的读（[`DeviceReadOnceInThisMount`]）：每一块第一次读才穿到盘上、读到的记下，预演与真做共用；读走哪一路看
//! [`ReadRouteOfThisMount`]，读错与读坏各记一本账。

use super::*;

/// 可写挂载这次挂载里的读，取号之前的预演与取号之后的真做共用一份（岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一
/// 「读一次两边共用」那一行，用户定）：共用的那一段里每一块在这次挂载里第一次读时才穿到底下那块盘，读到的字节记下；之后再读这一块——预演
/// （分配器拷贝上那一趟、只进内存那一趟，[`DeviceKeepingWritesInMemory`] 读没写过的块穿到它这里）还是真做（判号、写行、暖机、抬 F、续做）——
/// 都取记下的那一份，不再读盘。照旧读盘的几段不在共用的那一段里：取号之前逐盘核、真做取号写之前那几次读、实例表清扫
/// （[`ReadRouteOfThisMount::StraightToTheDisk`]），取号写与轮换写算世代号、覆写哪一槽那一读（[`BlockDevice::read_at_straight_from_the_disk`]），
/// 读坏之后当场重读那一读（见下）。这次挂载写成的块记成写下的字节，之后读它的读写下的。
/// 写报错的那几块原来记着的丢掉（盘上那几块现在是什么不知道），之后读它们穿到盘上。
/// 读错（盘报了错）的那一块不记，下一次读照样穿到盘上，读错记进 `reads_failed_on_the_disk`；读坏（读成了、读的一方核出自证不过，
/// [`BlockDevice::note_the_read_failing_self_verification_and_forget_its_bytes`]）的那几块从记下的那一份里丢掉（坏字节不留，下一次读穿到盘上），
/// 读坏记进 `reads_failing_self_verification`（岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路一「两样都做」那一行，用户定）。
/// 两本账几块盘各共用一份：取号之前预演没走到续做的结局时拿它们判（`mount::establish_instance`）。
/// 记下的那一份活到这次挂载交回为止，读过的、写过的块都在里面，没有上限（岔路二「先量再定」归计数实验）。
/// 读走哪一路看几块盘共用的 `read_route`（[`ReadRouteOfThisMount`]，`mount::establish_instance` 按段切）。
/// 读坏之后当场重读那一读（[`BlockDevice::reread_at_after_a_bad_read`]）不取记下的那一份、穿到盘上读；调用方核过自证交回来的那几字节
/// （[`BlockDevice::keep_the_self_verified_reread`]）换掉记下的那一份（[`ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies`]）。
/// 请求先过两个后端那一份判据（`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒、不发到盘上。
pub(crate) struct DeviceReadOnceInThisMount<'disk, 'log, Device> {
    identity: DeviceIdentity,
    disk: &'disk mut Device,
    physical_block_size: PhysicalBlockSizeInBytes,
    /// 这次挂载读成或写成的物理块：块号（按 `physical_block_size`）→ 那一块的字节。
    blocks_read_or_written_in_this_mount: RefCell<BTreeMap<u64, Vec<u8>>>,
    reads_failed_on_the_disk: &'log RefCell<Vec<ReadFailedOnTheDisk>>,
    reads_failing_self_verification: &'log RefCell<Vec<ReadFailingSelfVerification>>,
    read_route: &'log Cell<ReadRouteOfThisMount>,
}

/// [`DeviceReadOnceInThisMount`] 的读此刻走哪一路。写不分路：写成的块一律记成写下的字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReadRouteOfThisMount {
    /// 取记下的那一份，没记下的穿到盘上读、读成记下：预演与真做共用的那一份（岔路一「读一次两边共用」那一行）。
    SharedBetweenTheRehearsalAndTheRealRun,
    /// 穿到盘上读，不取也不改记下的那一份：条款要这一刻再读一次盘的那几段——取号之前逐盘核（读阶段之后的瞬时读错在这一遍报，
    /// D18（块里携带什么信息） 已定项 11）、真做取号（屏障前后各读一次见证值，C554 乙-配置续 Q1）、实例表清扫（抹之前重读每个头，
    /// 用户 2026-10-04 定 K1-b）。它们不在岔路单那一行说的「抬 F 与续做要读的东西」里，预演也不替它们读。
    StraightToTheDisk,
    /// 共用的那一段里读坏（读不出或自证不过）之后当场重读的那一读（[`BlockDevice::reread_at_after_a_bad_read`]；D19（块指针的结构与宽度预算）
    /// 已定项 5、D16（发布语义） 已定项 1「根槽这一次读坏」那一行要它再读一次盘）：绕过记下的那一份、穿到盘上读，读错照样记进读错账；
    /// 读到的字节调用方核过自证才交回来（[`BlockDevice::keep_the_self_verified_reread`]），换掉记下的那一份——预演里换了，真做再读取的就是它。
    /// 第一次读回来的坏字节记下了也挡不住这一读（主 agent 判定：抬 F 与续做里「读坏当场重读」那一读不许被共用的那一份吃掉）。
    /// 这一路只按每一次重读走，不装进几块盘共用的那一格。
    RereadStraightToTheDiskKeepingWhatSelfVerifies,
}

impl ReadRouteOfThisMount {
    /// 这一段里当场重读那一读走哪一路：共用的那一段改走穿到盘上、核过自证再换掉记下的那一份；照旧读盘的那几段本来就穿到盘上，
    /// 也不改记下的那一份。
    fn for_the_reread_after_a_bad_read(self) -> ReadRouteOfThisMount {
        match self {
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies
            }
            ReadRouteOfThisMount::StraightToTheDisk => ReadRouteOfThisMount::StraightToTheDisk,
        }
    }
}

impl<'disk, 'log, Device: BlockDevice> DeviceReadOnceInThisMount<'disk, 'log, Device> {
    pub(crate) fn over_the_disk(
        identity: DeviceIdentity,
        disk: &'disk mut Device,
        reads_failed_on_the_disk: &'log RefCell<Vec<ReadFailedOnTheDisk>>,
        reads_failing_self_verification: &'log RefCell<Vec<ReadFailingSelfVerification>>,
        read_route: &'log Cell<ReadRouteOfThisMount>,
    ) -> Self {
        let physical_block_size = disk.probe_physical_block_size();
        DeviceReadOnceInThisMount {
            identity,
            disk,
            physical_block_size,
            blocks_read_or_written_in_this_mount: RefCell::new(BTreeMap::new()),
            reads_failed_on_the_disk,
            reads_failing_self_verification,
            read_route,
        }
    }

    fn block_bytes(&self) -> u64 {
        u64::from(self.physical_block_size.0)
    }

    /// 两个后端那一份判据（`block_device::check_aligned_and_in_range`），设备大小取底下那块盘的。
    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let device_size = self.disk.size_in_bytes();
        check_aligned_and_in_range(offset, length, self.physical_block_size, device_size)
    }

    /// 块号 `first_unrecorded_block` 起、到 `end_block` 之前那一段没记下的块从底下那块盘读进 `buffer` 里对应的那一截
    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）：读成就逐块记下；读错记进读错账、原样交回。
    fn read_unrecorded_blocks_from_the_disk(
        &self,
        first_unrecorded_block: u64,
        end_block: u64,
        first_block_of_the_buffer: u64,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let start_in_buffer =
            usize::try_from((first_unrecorded_block - first_block_of_the_buffer) * block_bytes)
                .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let end_in_buffer = usize::try_from((end_block - first_block_of_the_buffer) * block_bytes)
            .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let offset = DeviceOffsetInBytes(first_unrecorded_block * block_bytes);
        if let Err(error) = self
            .disk
            .read_at(offset, &mut buffer[start_in_buffer..end_in_buffer])
        {
            self.reads_failed_on_the_disk
                .borrow_mut()
                .push(ReadFailedOnTheDisk {
                    device: self.identity,
                    offset,
                    length_in_bytes: (end_block - first_unrecorded_block) * block_bytes,
                    error: block_device_error_with_the_same_content(&error),
                });
            return Err(error);
        }
        let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
        // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
        for (block_index, block) in (first_unrecorded_block..end_block)
            .zip(buffer[start_in_buffer..end_in_buffer].chunks(block_length))
        {
            recorded.insert(block_index, block.to_vec());
        }
        Ok(())
    }

    /// 穿到盘上读、不取也不改记下的那一份（当场重读那一读）：读错记进读错账、原样交回。
    fn reread_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        if let Err(error) = self.disk.read_at(offset, buffer) {
            self.reads_failed_on_the_disk
                .borrow_mut()
                .push(ReadFailedOnTheDisk {
                    device: self.identity,
                    offset,
                    length_in_bytes,
                    error: block_device_error_with_the_same_content(&error),
                });
            return Err(error);
        }
        Ok(())
    }

    /// 读一段，照 `route` 走（[`ReadRouteOfThisMount`]）。
    fn read_by_the_route(
        &self,
        route: ReadRouteOfThisMount,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length)?;
        match route {
            ReadRouteOfThisMount::StraightToTheDisk => return self.disk.read_at(offset, buffer),
            ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                return self.reread_straight_from_the_disk(offset, buffer)
            }
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun => {}
        }
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length / block_bytes;
        // 迭代上界是这次读的块数；跨轮带着还没读的那一段没记下的块从哪一块起；出口只有底下那块盘读错（原样交回）。
        let mut unrecorded_run_start: Option<u64> = None;
        for block_index in first_block..end_block {
            let is_recorded_in_this_mount = self
                .blocks_read_or_written_in_this_mount
                .borrow()
                .contains_key(&block_index);
            if !is_recorded_in_this_mount {
                unrecorded_run_start.get_or_insert(block_index);
                continue;
            }
            if let Some(run_start) = unrecorded_run_start.take() {
                self.read_unrecorded_blocks_from_the_disk(
                    run_start,
                    block_index,
                    first_block,
                    buffer,
                )?;
            }
            let start_in_buffer = usize::try_from(block_index - first_block)
                .expect("块数装得进 usize")
                * block_length;
            let recorded = self.blocks_read_or_written_in_this_mount.borrow();
            buffer[start_in_buffer..start_in_buffer + block_length].copy_from_slice(
                recorded
                    .get(&block_index)
                    .expect("这一块刚判过记着：判与取之间没有写"),
            );
        }
        if let Some(run_start) = unrecorded_run_start {
            self.read_unrecorded_blocks_from_the_disk(run_start, end_block, first_block, buffer)?;
        }
        Ok(())
    }

    /// 写成的那几块记成写下的字节。
    fn record_the_written_blocks(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        let recorded = self.blocks_read_or_written_in_this_mount.get_mut();
        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
            recorded.insert(first_block + block_offset, block.to_vec());
        }
    }

    /// 写报错的那几块：原来记着的丢掉，之后读它们穿到盘上。
    fn forget_the_blocks_of_a_failed_write(&mut self, offset: DeviceOffsetInBytes, length: u64) {
        let block_bytes = self.block_bytes();
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length / block_bytes;
        let recorded = self.blocks_read_or_written_in_this_mount.get_mut();
        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
        for block_index in first_block..end_block {
            recorded.remove(&block_index);
        }
    }
}

/// 读错记进读错账要一份自己的：盘报的错交回给读的那一方，账里另存一份内容相同的（`BlockDeviceError` 带着 `io::Error`，不能 `Clone`）。
fn block_device_error_with_the_same_content(error: &BlockDeviceError) -> BlockDeviceError {
    match error {
        BlockDeviceError::OutOfRange {
            offset,
            length,
            device_size,
        } => BlockDeviceError::OutOfRange {
            offset: *offset,
            length: *length,
            device_size: *device_size,
        },
        BlockDeviceError::Unaligned {
            offset,
            length,
            physical_block_size,
        } => BlockDeviceError::Unaligned {
            offset: *offset,
            length: *length,
            physical_block_size: *physical_block_size,
        },
        BlockDeviceError::InputOutput(input_output_error) => BlockDeviceError::InputOutput(
            std::io::Error::new(input_output_error.kind(), input_output_error.to_string()),
        ),
        BlockDeviceError::ProbeFailed { attribute } => BlockDeviceError::ProbeFailed { attribute },
        BlockDeviceError::ImageFileAlreadyExists { path } => {
            BlockDeviceError::ImageFileAlreadyExists { path: path.clone() }
        }
        BlockDeviceError::ImageFileMissing { path } => {
            BlockDeviceError::ImageFileMissing { path: path.clone() }
        }
    }
}

impl<Device: BlockDevice> BlockDevice for DeviceReadOnceInThisMount<'_, '_, Device> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_by_the_route(self.read_route.get(), offset, buffer)
    }

    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_by_the_route(
            self.read_route.get().for_the_reread_after_a_bad_read(),
            offset,
            buffer,
        )
    }

    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        match self.read_route.get().for_the_reread_after_a_bad_read() {
            ReadRouteOfThisMount::StraightToTheDisk => {}
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                let length = u64::try_from(bytes.len()).expect("重读的长度装得进 u64");
                assert!(
                    self.check_request(offset, length).is_ok(),
                    "交回来的是一次读成的重读：偏移与长度过了同一份对齐与越界判据"
                );
                let block_bytes = self.block_bytes();
                let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
                let first_block = offset.0 / block_bytes;
                let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
                // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
                for (block_index, block) in (first_block..).zip(bytes.chunks(block_length)) {
                    recorded.insert(block_index, block.to_vec());
                }
            }
        }
    }

    /// 共用的那一段里（当场重读那一读也算）：记进读坏账、丢掉记下的那几块，下一次读它们穿到盘上（坏字节不留）。
    /// 照旧读盘的那几段里读到的字节本来没记下，什么都不做。
    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        match self.read_route.get().for_the_reread_after_a_bad_read() {
            ReadRouteOfThisMount::StraightToTheDisk => {}
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                assert!(
                    self.check_request(offset, length_in_bytes).is_ok(),
                    "报读坏的是一次读成的读：偏移与长度过了同一份对齐与越界判据"
                );
                self.reads_failing_self_verification.borrow_mut().push(
                    ReadFailingSelfVerification {
                        device: self.identity,
                        offset,
                        length_in_bytes,
                    },
                );
                let block_bytes = self.block_bytes();
                let first_block = offset.0 / block_bytes;
                let end_block = first_block + length_in_bytes / block_bytes;
                let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
                // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
                for block_of_the_bad_read in first_block..end_block {
                    recorded.remove(&block_of_the_bad_read);
                }
            }
        }
    }

    /// 不取也不改记下的那一份、直接读底下那块盘；读错不记进读错账（与 [`ReadRouteOfThisMount::StraightToTheDisk`] 那一路同一个读法）。
    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_by_the_route(ReadRouteOfThisMount::StraightToTheDisk, offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        self.check_request(offset, length)?;
        match self.disk.write_at(offset, bytes, durability) {
            Ok(()) => {
                self.record_the_written_blocks(offset, bytes);
                Ok(())
            }
            Err(error) => {
                self.forget_the_blocks_of_a_failed_write(offset, length);
                Err(error)
            }
        }
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        match self.disk.write_zeroes_at(offset, length) {
            Ok(()) => {
                let zeros =
                    vec![0u8; usize::try_from(length).expect("写零的长度装得进 usize")];
                self.record_the_written_blocks(offset, &zeros);
                Ok(())
            }
            Err(error) => {
                self.forget_the_blocks_of_a_failed_write(offset, length);
                Err(error)
            }
        }
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.disk.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.disk.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.disk
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}
