//! 只进内存的盘（[`DeviceKeepingWritesInMemory`]）：预演时写与写零只进内存、盖在之后的读上，读没写过的块穿到底下那块盘，底下那块盘一字不动。

use super::*;

/// 预演用的盘：读穿到底下那块盘（这一刻盘上的字节），写与写零只记进内存、盖在之后的读上，屏障什么都不做——一个写都不发到底下那块盘。
/// 要出路时预演推环与推完之后这次改动的准入用（[`rehearse_the_push_then_the_remainder_of_the_change`]）；可写挂载取号之前预演续做、
/// 续做要推抬 F 时，取号之后那一串连同续做也在它上面走一遍（`mount` 的 `walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`）：
/// 那一处底下那块盘是这次挂载共用的那一份读（[`DeviceReadOnceInThisMount`]），这里读成的块在那里记下，真做再读取的是同一份。
/// 底下那块盘只读借成 `&dyn BlockDevice`：改盘的三个动作要 `&mut`，类型上调不到。请求先过两个后端那一份判据
/// （`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒。
pub(crate) struct DeviceKeepingWritesInMemory<'disk> {
    disk: &'disk dyn BlockDevice,
    physical_block_size: PhysicalBlockSizeInBytes,
    /// 写过的物理块：块号（按 `physical_block_size`）→ 那一块现在的字节。写零也逐块记成全 0（发布路径不写零，只有 mkfs 写零）。
    written_blocks: BTreeMap<u64, Vec<u8>>,
}

impl<'disk> DeviceKeepingWritesInMemory<'disk> {
    pub(crate) fn over_the_disk(disk: &'disk dyn BlockDevice) -> Self {
        DeviceKeepingWritesInMemory {
            disk,
            physical_block_size: disk.probe_physical_block_size(),
            written_blocks: BTreeMap::new(),
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
        check_aligned_and_in_range(
            offset,
            length,
            self.physical_block_size,
            self.disk.size_in_bytes(),
        )
    }

    /// 块号 `first_unwritten_block` 起、到 `end_block` 之前那一段没写过的块从底下那块盘读进 `buffer` 里对应的那一截
    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）；`pass` 是当场重读那一遍时走底下那块盘的
    /// [`BlockDevice::reread_at_after_a_bad_read`]。
    fn read_unwritten_blocks_from_the_disk(
        &self,
        first_unwritten_block: u64,
        end_block: u64,
        first_block_of_the_buffer: u64,
        buffer: &mut [u8],
        pass: ReadPassOverThePool,
    ) -> Result<(), BlockDeviceError> {
        let block_bytes = self.block_bytes();
        let start_in_buffer =
            usize::try_from((first_unwritten_block - first_block_of_the_buffer) * block_bytes)
                .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let end_in_buffer = usize::try_from((end_block - first_block_of_the_buffer) * block_bytes)
            .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let offset = DeviceOffsetInBytes(first_unwritten_block * block_bytes);
        let range_of_the_buffer = &mut buffer[start_in_buffer..end_in_buffer];
        match pass {
            ReadPassOverThePool::FirstRead => self.disk.read_at(offset, range_of_the_buffer),
            ReadPassOverThePool::RereadAfterABadRead => self
                .disk
                .reread_at_after_a_bad_read(offset, range_of_the_buffer),
        }
    }

    /// 读一段：写过的块取内存里那一份，没写过的那几段从底下那块盘读（`pass` 定走 [`BlockDevice::read_at`] 还是当场重读那一读）。
    fn read_over_the_blocks_kept_in_memory(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
        pass: ReadPassOverThePool,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length)?;
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length / block_bytes;
        // 迭代上界是这次读的块数；跨轮带着还没读的那一段没写过的块从哪一块起；出口只有底下那块盘读错（原样交回）。
        let mut unwritten_run_start: Option<u64> = None;
        for block_index in first_block..end_block {
            match self.written_blocks.get(&block_index) {
                None => {
                    unwritten_run_start.get_or_insert(block_index);
                }
                Some(written) => {
                    if let Some(run_start) = unwritten_run_start.take() {
                        self.read_unwritten_blocks_from_the_disk(
                            run_start,
                            block_index,
                            first_block,
                            buffer,
                            pass,
                        )?;
                    }
                    let start_in_buffer = usize::try_from(block_index - first_block)
                        .expect("块数装得进 usize")
                        * block_length;
                    buffer[start_in_buffer..start_in_buffer + block_length]
                        .copy_from_slice(written);
                }
            }
        }
        if let Some(run_start) = unwritten_run_start {
            self.read_unwritten_blocks_from_the_disk(
                run_start,
                end_block,
                first_block,
                buffer,
                pass,
            )?;
        }
        Ok(())
    }

    fn keep_in_memory(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
            self.written_blocks
                .insert(first_block + block_offset, block.to_vec());
        }
    }
}

impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_over_the_blocks_kept_in_memory(offset, buffer, ReadPassOverThePool::FirstRead)
    }

    /// 写过的块照旧取内存里那一份（这一趟写的，盘上没有）；没写过的走底下那块盘的当场重读那一读。
    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_over_the_blocks_kept_in_memory(
            offset,
            buffer,
            ReadPassOverThePool::RereadAfterABadRead,
        )
    }

    /// 没写过的那几段交给底下那块盘（它们是从那里重读来的）；写过的块是这一趟写在内存里的，不交。
    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
        for (block_index, block) in (first_block..).zip(bytes.chunks(block_length)) {
            if !self.written_blocks.contains_key(&block_index) {
                self.disk.keep_the_self_verified_reread(
                    DeviceOffsetInBytes(block_index * block_bytes),
                    block,
                );
            }
        }
    }

    /// 没写过的那几段交给底下那块盘（读坏的字节是从那里读来的，那一层记进读坏账、丢掉记下的那几块），一段连着的没写过的块交一次；
    /// 写过的块是这一趟写在内存里的，不交。
    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        let block_bytes = self.block_bytes();
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length_in_bytes / block_bytes;
        let hand_the_unwritten_run_to_the_disk = |run_start: u64, run_end: u64| {
            self.disk
                .note_the_read_failing_self_verification_and_forget_its_bytes(
                    DeviceOffsetInBytes(run_start * block_bytes),
                    (run_end - run_start) * block_bytes,
                );
        };
        // 迭代上界是这一段的块数；跨轮带着还没交的那一段没写过的块从哪一块起；没有提前出口。
        let mut unwritten_run_start: Option<u64> = None;
        for block_index in first_block..end_block {
            if self.written_blocks.contains_key(&block_index) {
                if let Some(run_start) = unwritten_run_start.take() {
                    hand_the_unwritten_run_to_the_disk(run_start, block_index);
                }
            } else {
                unwritten_run_start.get_or_insert(block_index);
            }
        }
        if let Some(run_start) = unwritten_run_start {
            hand_the_unwritten_run_to_the_disk(run_start, end_block);
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        _durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"))?;
        self.keep_in_memory(offset, bytes);
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        let zeros = vec![0u8; usize::try_from(length).expect("写零的长度装得进 usize")];
        self.keep_in_memory(offset, &zeros);
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.disk.size_in_bytes()
    }
}

#[cfg(test)]
mod tests {
    //! 只进内存的盘（[`DeviceKeepingWritesInMemory`]）：写与写零只进内存、盖在之后的读上，读没写过的块穿到底下那块盘，底下那块盘一字不动；
    //! 请求判据与两个后端是同一份（`block_device::check_aligned_and_in_range`）。差分对拍照正推腿的做法
    //! （`research/prompts/m3-batch10z-tu-code-r1-sonnet-model/src/core-mounted-session-debug.diff` 的 `sonnet_k5_differential`），
    //! 参照换成一块把每个写直接写进去的字节盘（同一份请求判据）。
    use super::*;

    /// 物理块 512 字节、16 块的盘。
    const PHYSICAL_BLOCK_BYTES: u32 = 512;
    const BLOCKS_OF_THE_DISK: u64 = 16;

    /// 底下那块盘（也是差分对拍的参照）：整块字节放内存，请求判据照两个后端那一份。
    struct DeviceOfBytes {
        bytes: Vec<u8>,
        physical_block_size: PhysicalBlockSizeInBytes,
    }

    impl DeviceOfBytes {
        /// 物理块 [`PHYSICAL_BLOCK_BYTES`]、[`BLOCKS_OF_THE_DISK`] 块的盘，每个字节按位置与 `seed` 填一个图样：读回哪一块看得出来。
        fn filled_with_a_pattern(seed: u8) -> Self {
            Self::filled_with_a_pattern_on_blocks_of(
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                BLOCKS_OF_THE_DISK,
                seed,
            )
        }

        /// 物理块 `physical_block_size`、`blocks_of_the_disk` 块的盘，图样同 [`Self::filled_with_a_pattern`]。
        fn filled_with_a_pattern_on_blocks_of(
            physical_block_size: PhysicalBlockSizeInBytes,
            blocks_of_the_disk: u64,
            seed: u8,
        ) -> Self {
            let device_bytes =
                usize::try_from(u64::from(physical_block_size.0) * blocks_of_the_disk)
                    .expect("盘长装得进 usize");
            DeviceOfBytes {
                bytes: (0..device_bytes)
                    .map(|position| {
                        u8::try_from(position % 251)
                            .expect("小于 256")
                            .wrapping_add(seed)
                    })
                    .collect(),
                physical_block_size,
            }
        }

        fn range_of(
            &self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<std::ops::Range<usize>, BlockDeviceError> {
            check_aligned_and_in_range(
                offset,
                length,
                self.physical_block_size,
                self.size_in_bytes(),
            )?;
            let start = usize::try_from(offset.0).expect("判过不越过盘尾");
            let end = start + usize::try_from(length).expect("判过不越过盘尾");
            Ok(start..end)
        }
    }

    impl BlockDevice for DeviceOfBytes {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, u64::try_from(buffer.len()).expect("读长"))?;
            buffer.copy_from_slice(&self.bytes[range]);
            Ok(())
        }

        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            _durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, u64::try_from(bytes.len()).expect("写长"))?;
            self.bytes[range].copy_from_slice(bytes);
            Ok(())
        }

        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, length)?;
            self.bytes[range].fill(0);
            Ok(())
        }

        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }

        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            self.physical_block_size
        }

        fn size_in_bytes(&self) -> u64 {
            u64::try_from(self.bytes.len()).expect("盘长装得进 u64")
        }
    }

    fn offset_of_block(block: u64) -> DeviceOffsetInBytes {
        DeviceOffsetInBytes(block * u64::from(PHYSICAL_BLOCK_BYTES))
    }

    fn bytes_of_blocks(blocks: u64) -> usize {
        usize::try_from(blocks * u64::from(PHYSICAL_BLOCK_BYTES)).expect("块数装得进 usize")
    }

    /// 从 `first_block` 起读 `blocks` 块；读缓冲先填 0xAA，没读到的那一截露得出来。
    fn read_blocks(device: &dyn BlockDevice, first_block: u64, blocks: u64) -> Vec<u8> {
        let mut buffer = vec![0xAA_u8; bytes_of_blocks(blocks)];
        device
            .read_at(offset_of_block(first_block), &mut buffer)
            .expect("对齐、没越过盘尾的读做得成");
        buffer
    }

    fn bytes_of_the_disk_blocks(disk: &DeviceOfBytes, first_block: u64, blocks: u64) -> Vec<u8> {
        let start = bytes_of_blocks(first_block);
        disk.bytes[start..start + bytes_of_blocks(blocks)].to_vec()
    }

    #[test]
    fn write_kept_in_memory_reads_back_and_leaves_the_disk_below_unchanged() {
        let disk = DeviceOfBytes::filled_with_a_pattern(0);
        let bytes_before = disk.bytes.clone();
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let written = vec![0x5A_u8; bytes_of_blocks(1)];
        kept.write_at(
            offset_of_block(3),
            &written,
            WriteDurability::ForceUnitAccess,
        )
        .expect("对齐、没越过盘尾的写做得成");
        assert_eq!(
            read_blocks(&kept, 3, 1),
            written,
            "写过的块读回写进去的字节"
        );
        assert!(
            disk.bytes == bytes_before,
            "写只进内存：底下那块盘一个字节都没变"
        );
    }

    #[test]
    fn read_of_blocks_never_written_passes_through_to_the_disk_below() {
        let disk = DeviceOfBytes::filled_with_a_pattern(7);
        let kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        assert_eq!(
            read_blocks(&kept, 5, 3),
            bytes_of_the_disk_blocks(&disk, 5, 3),
            "没写过的块读穿到底下那块盘"
        );
    }

    #[test]
    fn read_across_written_and_unwritten_blocks_stitches_the_kept_blocks_with_the_disk_below() {
        let disk = DeviceOfBytes::filled_with_a_pattern(11);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let block_two = vec![0x22_u8; bytes_of_blocks(1)];
        let block_four = vec![0x44_u8; bytes_of_blocks(1)];
        let blocks_seven_and_eight: Vec<u8> = (0..bytes_of_blocks(2))
            .map(|position| u8::try_from(position % 199).expect("小于 256"))
            .collect();
        for (first_block, bytes) in [
            (2, &block_two),
            (4, &block_four),
            (7, &blocks_seven_and_eight),
        ] {
            kept.write_at(offset_of_block(first_block), bytes, WriteDurability::Plain)
                .expect("对齐、没越过盘尾的写做得成");
        }
        let mut expected = bytes_of_the_disk_blocks(&disk, 1, 9);
        for (first_block, bytes) in [
            (2, &block_two),
            (4, &block_four),
            (7, &blocks_seven_and_eight),
        ] {
            let start = bytes_of_blocks(first_block - 1);
            expected[start..start + bytes.len()].copy_from_slice(bytes);
        }
        assert!(
            read_blocks(&kept, 1, 9) == expected,
            "块 1 到 9 一次读：写过的块（2、4、7、8）读回写进去的字节，没写过的（1、3、5、6、9，含末尾那一段）读穿到底下那块盘"
        );
    }

    #[test]
    fn zeroes_written_in_memory_read_back_as_zero_over_the_disk_below() {
        let disk = DeviceOfBytes::filled_with_a_pattern(13);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        kept.write_zeroes_at(offset_of_block(3), u64::from(PHYSICAL_BLOCK_BYTES) * 3)
            .expect("对齐、没越过盘尾的写零做得成");
        let mut expected = bytes_of_the_disk_blocks(&disk, 2, 5);
        expected[bytes_of_blocks(1)..bytes_of_blocks(4)].fill(0);
        assert!(
            read_blocks(&kept, 2, 5) == expected,
            "写零的块 3 到 5 读回全 0，两头的块 2、6 读穿到底下那块盘"
        );
        assert!(
            bytes_of_the_disk_blocks(&disk, 3, 3)
                .iter()
                .any(|byte| *byte != 0),
            "底下那块盘那一段不是全 0（阳性对照：读回的 0 是写零记下的）"
        );
    }

    #[test]
    fn request_off_the_physical_block_boundaries_is_refused_before_anything_is_kept() {
        let disk = DeviceOfBytes::filled_with_a_pattern(17);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let unaligned_offset = DeviceOffsetInBytes(u64::from(PHYSICAL_BLOCK_BYTES) + 100);
        let one_block = vec![0x33_u8; bytes_of_blocks(1)];
        let short_of_a_block = vec![0x33_u8; 100];
        let refusals = [
            (
                "偏移不在物理块边界上的写",
                kept.write_at(unaligned_offset, &one_block, WriteDurability::Plain),
                unaligned_offset,
                u64::from(PHYSICAL_BLOCK_BYTES),
            ),
            (
                "长度不是物理块整数倍的写",
                kept.write_at(
                    offset_of_block(2),
                    &short_of_a_block,
                    WriteDurability::Plain,
                ),
                offset_of_block(2),
                100,
            ),
            (
                "长度不是物理块整数倍的写零",
                kept.write_zeroes_at(offset_of_block(4), 100),
                offset_of_block(4),
                100,
            ),
            (
                "偏移不在物理块边界上的读",
                kept.read_at(unaligned_offset, &mut vec![0_u8; bytes_of_blocks(1)]),
                unaligned_offset,
                u64::from(PHYSICAL_BLOCK_BYTES),
            ),
        ];
        for (request, outcome, offset, length) in refusals {
            assert!(
                matches!(
                    outcome,
                    Err(BlockDeviceError::Unaligned {
                        offset: refused_offset,
                        length: refused_length,
                        physical_block_size: PHYSICAL_BLOCK_BYTES,
                    }) if refused_offset == offset && refused_length == length
                ),
                "{request}：拒成 Unaligned，带偏移 {} 与长度 {length}，而交回 {outcome:?}",
                offset.0
            );
        }
        assert!(
            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == disk.bytes,
            "被拒的写一个块都没记：整盘读回底下那块盘的字节"
        );
    }

    #[test]
    fn request_past_the_end_of_the_disk_below_is_refused_and_one_ending_at_the_end_is_kept() {
        let disk = DeviceOfBytes::filled_with_a_pattern(19);
        let device_size = disk.size_in_bytes();
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let last_block = vec![0x77_u8; bytes_of_blocks(1)];
        kept.write_at(
            offset_of_block(BLOCKS_OF_THE_DISK - 1),
            &last_block,
            WriteDurability::Plain,
        )
        .expect("最后一块正好写到盘尾：放行");
        assert_eq!(
            read_blocks(&kept, BLOCKS_OF_THE_DISK - 1, 1),
            last_block,
            "最后一块读回写进去的字节"
        );
        let two_blocks = vec![0x66_u8; bytes_of_blocks(2)];
        let offset_near_the_top =
            DeviceOffsetInBytes(u64::MAX - u64::from(PHYSICAL_BLOCK_BYTES) + 1);
        let refusals = [
            (
                "从最后一块起写两块（越过盘尾一块）",
                kept.write_at(
                    offset_of_block(BLOCKS_OF_THE_DISK - 1),
                    &two_blocks,
                    WriteDurability::Plain,
                ),
                offset_of_block(BLOCKS_OF_THE_DISK - 1),
                u64::from(PHYSICAL_BLOCK_BYTES) * 2,
            ),
            (
                "从盘尾起写零一块",
                kept.write_zeroes_at(
                    offset_of_block(BLOCKS_OF_THE_DISK),
                    u64::from(PHYSICAL_BLOCK_BYTES),
                ),
                offset_of_block(BLOCKS_OF_THE_DISK),
                u64::from(PHYSICAL_BLOCK_BYTES),
            ),
            (
                "偏移加长度越过 u64 的读",
                kept.read_at(offset_near_the_top, &mut vec![0_u8; bytes_of_blocks(2)]),
                offset_near_the_top,
                u64::from(PHYSICAL_BLOCK_BYTES) * 2,
            ),
        ];
        for (request, outcome, offset, length) in refusals {
            assert!(
                matches!(
                    outcome,
                    Err(BlockDeviceError::OutOfRange {
                        offset: refused_offset,
                        length: refused_length,
                        device_size: refused_device_size,
                    }) if refused_offset == offset && refused_length == length && refused_device_size == device_size
                ),
                "{request}：拒成 OutOfRange，带偏移 {}、长度 {length} 与盘长 {device_size}，而交回 {outcome:?}",
                offset.0
            );
        }
        let mut expected = disk.bytes.clone();
        expected[bytes_of_blocks(BLOCKS_OF_THE_DISK - 1)..].copy_from_slice(&last_block);
        assert!(
            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == expected,
            "被拒的写一个块都没记：整盘读回只有最后一块是写进去的"
        );
    }

    /// 差分对拍那一串请求的个数。
    const REQUESTS_OF_THE_RANDOM_STREAM: u64 = 20_000;

    /// 线性同余，取高 33 位（只要确定、够散）。
    fn next_random(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *state >> 33
    }

    /// 偏移：多半落在物理块边界上、盘以内，少数不对齐、正好在盘尾、越过盘尾、贴着 u64 的顶。
    fn random_offset(state: &mut u64, device_size: u64, block_bytes: u64) -> u64 {
        match next_random(state) % 8 {
            0 => next_random(state) % (device_size + 2 * block_bytes),
            1 => u64::MAX - next_random(state) % 1024,
            2 => device_size,
            _ => (next_random(state) % (device_size / block_bytes)) * block_bytes,
        }
    }

    /// 长度：多半是一到几个物理块，少数是 0、不对齐、跨过盘尾。
    fn random_length(state: &mut u64, device_size: u64, block_bytes: u64) -> u64 {
        match next_random(state) % 8 {
            0 => 0,
            1 => next_random(state) % (2 * device_size),
            _ => (1 + next_random(state) % 4) * block_bytes,
        }
    }

    /// 一串随机请求（读、写、写零，偶尔一道屏障）同时交给盖在底下那块盘上的只进内存的盘与一块把写直接写进去的同样的盘之后看到的。
    struct RandomRequestStreamAgainstADeviceWrittenThrough {
        /// 两边对不上的请求，逐条写明第几个、什么请求、哪一样不同（读回的字节、交回值）。
        disagreements: Vec<String>,
        refused_requests: u64,
        kept_writes: u64,
        /// 一串请求之后整盘读回的字节与写穿的那块盘相同。
        whole_disk_reads_back_as_written_through: bool,
        /// 一串请求之后底下那块盘一个字节都没变。
        disk_below_is_unchanged: bool,
    }

    /// 物理块 `physical_block_size`、`blocks_of_the_disk` 块、图样种子 `seed` 的盘上跑 [`REQUESTS_OF_THE_RANDOM_STREAM`] 个随机请求。
    fn random_request_stream_against_a_device_written_through(
        physical_block_size: PhysicalBlockSizeInBytes,
        blocks_of_the_disk: u64,
        seed: u8,
    ) -> RandomRequestStreamAgainstADeviceWrittenThrough {
        let disk = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(
            physical_block_size,
            blocks_of_the_disk,
            seed,
        );
        let bytes_before = disk.bytes.clone();
        let device_size = disk.size_in_bytes();
        let block_bytes = u64::from(physical_block_size.0);
        let mut written_through = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(
            physical_block_size,
            blocks_of_the_disk,
            seed,
        );
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let mut state = 1_u64;
        let mut disagreements: Vec<String> = Vec::new();
        let mut refused_requests = 0_u64;
        let mut kept_writes = 0_u64;
        // 迭代上界是请求个数；跨轮带着两块盘、随机数的状态与两个计数；没有提前出口。
        for request_number in 1..=REQUESTS_OF_THE_RANDOM_STREAM {
            let offset = DeviceOffsetInBytes(random_offset(&mut state, device_size, block_bytes));
            let length = random_length(&mut state, device_size, block_bytes);
            let length_in_memory = usize::try_from(length).expect("长度不过两倍盘长");
            let (request, through, in_memory) = match next_random(&mut state) % 3 {
                0 => {
                    let mut through_buffer = vec![0xAA_u8; length_in_memory];
                    let mut in_memory_buffer = vec![0xAA_u8; length_in_memory];
                    let through = written_through.read_at(offset, &mut through_buffer);
                    let in_memory = kept.read_at(offset, &mut in_memory_buffer);
                    if through_buffer != in_memory_buffer {
                        disagreements.push(format!(
                            "第 {request_number} 个请求（读 {} 起 {length} 字节）：读回的字节不同",
                            offset.0
                        ));
                    }
                    ("读", through, in_memory)
                }
                1 => {
                    let bytes: Vec<u8> = (0..length_in_memory)
                        .map(|_| u8::try_from(next_random(&mut state) % 256).expect("小于 256"))
                        .collect();
                    (
                        "写",
                        written_through.write_at(offset, &bytes, WriteDurability::Plain),
                        kept.write_at(offset, &bytes, WriteDurability::ForceUnitAccess),
                    )
                }
                _ => (
                    "写零",
                    written_through.write_zeroes_at(offset, length),
                    kept.write_zeroes_at(offset, length),
                ),
            };
            let (through_returned, in_memory_returned) =
                (format!("{through:?}"), format!("{in_memory:?}"));
            if through_returned != in_memory_returned {
                disagreements.push(format!(
                    "第 {request_number} 个请求（{request} {} 起 {length} 字节）：写穿的盘交回 {through_returned}，只进内存的盘交回 {in_memory_returned}",
                    offset.0
                ));
            }
            match through {
                Ok(()) => kept_writes += u64::from(request != "读"),
                Err(_) => refused_requests += 1,
            }
            if next_random(&mut state).is_multiple_of(50) {
                kept.barrier().expect("屏障什么都不做");
            }
        }
        let mut whole_disk =
            vec![0xAA_u8; usize::try_from(device_size).expect("盘长装得进 usize")];
        kept.read_at(DeviceOffsetInBytes(0), &mut whole_disk)
            .expect("整盘一次读：对齐、没越过盘尾");
        RandomRequestStreamAgainstADeviceWrittenThrough {
            disagreements,
            refused_requests,
            kept_writes,
            whole_disk_reads_back_as_written_through: whole_disk == written_through.bytes,
            disk_below_is_unchanged: disk.bytes == bytes_before,
        }
    }

    /// 物理块 512 字节的盘上：只进内存的盘与写穿的盘逐个请求交回值、读回的字节相同，整盘读回相同，底下那块盘一字不动。
    #[test]
    fn the_device_keeping_writes_in_memory_agrees_with_a_device_written_through_on_a_random_request_stream(
    ) {
        let stream = random_request_stream_against_a_device_written_through(
            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            BLOCKS_OF_THE_DISK,
            23,
        );
        assert!(
            stream.disagreements.is_empty(),
            "每个请求两边交回值与读回的字节相同，而 {} 个对不上，头几个：{:?}",
            stream.disagreements.len(),
            stream.disagreements.iter().take(5).collect::<Vec<_>>()
        );
        assert!(
            stream.refused_requests > 0 && stream.kept_writes > 0,
            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {} 个、写 {} 个",
            stream.refused_requests,
            stream.kept_writes
        );
        assert!(
            stream.whole_disk_reads_back_as_written_through,
            "整盘读回与写穿的那块盘相同"
        );
        assert!(
            stream.disk_below_is_unchanged,
            "一串写之后底下那块盘一个字节都没变"
        );
    }

    /// 物理块 4096 字节的盘（正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md` K2：入库用例原来只在 512 的盘上，
    /// 判据里把物理块写死 512 照绿）：同一串随机请求按 4096 的块取偏移与长度，两边逐个请求相同、整盘读回相同、底下那块盘一字不动。
    #[test]
    fn the_device_keeping_writes_in_memory_on_4096_byte_physical_blocks_agrees_with_a_device_written_through_on_a_random_request_stream(
    ) {
        let stream = random_request_stream_against_a_device_written_through(
            PhysicalBlockSizeInBytes(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES),
            BLOCKS_OF_THE_DISK,
            29,
        );
        assert!(
            stream.disagreements.is_empty(),
            "每个请求两边交回值与读回的字节相同，而 {} 个对不上，头几个：{:?}",
            stream.disagreements.len(),
            stream.disagreements.iter().take(5).collect::<Vec<_>>()
        );
        assert!(
            stream.refused_requests > 0 && stream.kept_writes > 0,
            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {} 个、写 {} 个",
            stream.refused_requests,
            stream.kept_writes
        );
        assert!(
            stream.whole_disk_reads_back_as_written_through,
            "整盘读回与写穿的那块盘相同"
        );
        assert!(
            stream.disk_below_is_unchanged,
            "一串写之后底下那块盘一个字节都没变"
        );
    }

    /// 4096 字节的物理块。
    const FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES: u32 = 4096;

    /// 物理块 4096 字节的盘上，落在 512 字节边界上、不在 4096 边界上的请求（偏移或长度）照两个后端那一份判据拒成 `Unaligned`、
    /// 带 4096 这个物理块，一个块都不记；同一块盘上按 4096 对齐的写读得回来。判据里物理块写死 512 时这几个请求都放行。
    #[test]
    fn requests_on_512_byte_boundaries_off_the_4096_byte_physical_blocks_are_refused_before_anything_is_kept(
    ) {
        let block = PhysicalBlockSizeInBytes(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES);
        let block_bytes = u64::from(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES);
        let disk = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(block, BLOCKS_OF_THE_DISK, 31);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let one_block = vec![0x3C_u8; usize::try_from(block_bytes).expect("一块装得进 usize")];
        let one_sector = vec![0x3C_u8; 512];
        let offset_on_a_sector_inside_block_one = DeviceOffsetInBytes(block_bytes + 512);
        let refusals = [
            (
                "偏移在 512 边界上、不在 4096 边界上的写",
                kept.write_at(
                    offset_on_a_sector_inside_block_one,
                    &one_block,
                    WriteDurability::Plain,
                ),
                offset_on_a_sector_inside_block_one,
                block_bytes,
            ),
            (
                "长度 512 的写",
                kept.write_at(
                    DeviceOffsetInBytes(2 * block_bytes),
                    &one_sector,
                    WriteDurability::Plain,
                ),
                DeviceOffsetInBytes(2 * block_bytes),
                512,
            ),
            (
                "长度 512 的写零",
                kept.write_zeroes_at(DeviceOffsetInBytes(3 * block_bytes), 512),
                DeviceOffsetInBytes(3 * block_bytes),
                512,
            ),
            (
                "偏移在 512 边界上、不在 4096 边界上的读",
                kept.read_at(offset_on_a_sector_inside_block_one, &mut vec![0_u8; 512]),
                offset_on_a_sector_inside_block_one,
                512,
            ),
        ];
        for (request, outcome, offset, length) in refusals {
            assert!(
                matches!(
                    outcome,
                    Err(BlockDeviceError::Unaligned {
                        offset: refused_offset,
                        length: refused_length,
                        physical_block_size: FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES,
                    }) if refused_offset == offset && refused_length == length
                ),
                "{request}：拒成 Unaligned，带偏移 {}、长度 {length} 与物理块 4096，而交回 {outcome:?}",
                offset.0
            );
        }
        kept.write_at(
            DeviceOffsetInBytes(5 * block_bytes),
            &one_block,
            WriteDurability::Plain,
        )
        .expect("按 4096 对齐的一块写做得成（阳性对照）");
        let mut expected = disk.bytes.clone();
        let start_of_block_five = usize::try_from(5 * block_bytes).expect("偏移装得进 usize");
        expected[start_of_block_five..start_of_block_five + one_block.len()]
            .copy_from_slice(&one_block);
        let mut whole_disk = vec![0xAA_u8; disk.bytes.len()];
        kept.read_at(DeviceOffsetInBytes(0), &mut whole_disk)
            .expect("整盘一次读：对齐、没越过盘尾");
        assert!(
            whole_disk == expected,
            "被拒的请求一个块都没记：整盘读回只有按 4096 对齐写的那一块是写进去的"
        );
    }
}
