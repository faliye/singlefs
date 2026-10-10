
// ====================== 攻方原型（m3-batch10z-tu-code-r1，opus 腿）：只进内存的盘与抄的对齐判 ======================
// 只拼进攻方量法副本的 crates/singlefs-core/src/mounted_session.rs 末尾（副本里另把 block_device.rs 的 check_aligned_and_in_range 放成 pub(crate)）。
#[cfg(test)]
mod atk_overlay_tests {
    use super::*;

    /// 参照盘：一整块字节，读写都先过文件后端那一份判据（`block_device::check_aligned_and_in_range`）。
    #[derive(Clone)]
    struct AtkVecDevice {
        bytes: Vec<u8>,
        block: PhysicalBlockSizeInBytes,
    }

    impl BlockDevice for AtkVecDevice {
        fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
            crate::block_device::check_aligned_and_in_range(offset, buffer.len() as u64, self.block, self.bytes.len() as u64)?;
            let start = offset.0 as usize;
            buffer.copy_from_slice(&self.bytes[start..start + buffer.len()]);
            Ok(())
        }
        fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], _durability: WriteDurability) -> Result<(), BlockDeviceError> {
            crate::block_device::check_aligned_and_in_range(offset, bytes.len() as u64, self.block, self.bytes.len() as u64)?;
            let start = offset.0 as usize;
            self.bytes[start..start + bytes.len()].copy_from_slice(bytes);
            Ok(())
        }
        fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
            crate::block_device::check_aligned_and_in_range(offset, length, self.block, self.bytes.len() as u64)?;
            let start = offset.0 as usize;
            self.bytes[start..start + length as usize].fill(0);
            Ok(())
        }
        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            self.block
        }
        fn size_in_bytes(&self) -> u64 {
            self.bytes.len() as u64
        }
    }

    struct AtkRandom(u64);
    impl AtkRandom {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn below(&mut self, bound: u64) -> u64 {
            self.next() % bound.max(1)
        }
    }

    /// 一次请求的偏移与长度：多半对齐、落在盘内，少数不对齐、越界、贴着 u64 顶。
    fn atk_request(random: &mut AtkRandom, block: u64, size: u64) -> (u64, u64) {
        let blocks_in_size = size / block;
        match random.below(10) {
            0 => (random.below(size + block), random.below(4 * block)),
            1 => (u64::MAX - random.below(3) * block, random.below(3) * block),
            2 => (random.below(blocks_in_size + 2) * block, random.below(4) * block + block),
            _ => {
                let first = random.below(blocks_in_size);
                let count = random.below((blocks_in_size - first).min(16) + 1);
                (first * block, count * block)
            }
        }
    }

    #[test]
    fn atk_overlay_reads_back_like_a_plain_device_and_never_writes_through() {
        let mut compared = 0u64;
        for block in [512u32, 4096] {
            let block_bytes = u64::from(block);
            for size in [block_bytes * 64, block_bytes * 64 + 100, block_bytes * 7] {
                for seed in 1..=300u64 {
                    let mut random = AtkRandom(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(u64::from(block)).wrapping_add(size) | 1);
                    let initial: Vec<u8> = (0..size).map(|_| random.next() as u8).collect();
                    let base = AtkVecDevice { bytes: initial.clone(), block: PhysicalBlockSizeInBytes(block) };
                    let mut reference = base.clone();
                    let mut overlay = DeviceKeepingWritesInMemory::over_the_disk(&base);
                    for _ in 0..200 {
                        let (offset, length) = atk_request(&mut random, block_bytes, size);
                        let offset = DeviceOffsetInBytes(offset);
                        let kind = random.below(4);
                        let length_usize = usize::try_from(length.min(1 << 20)).expect("短");
                        let (got, want) = match kind {
                            0 => {
                                let bytes: Vec<u8> = (0..length_usize).map(|_| random.next() as u8).collect();
                                (format!("{:?}", overlay.write_at(offset, &bytes, WriteDurability::Plain)), format!("{:?}", reference.write_at(offset, &bytes, WriteDurability::Plain)))
                            }
                            1 => (format!("{:?}", overlay.write_zeroes_at(offset, length)), format!("{:?}", reference.write_zeroes_at(offset, length))),
                            2 => (format!("{:?}", overlay.barrier()), format!("{:?}", reference.barrier())),
                            _ => {
                                let mut got_buffer = vec![0xAAu8; length_usize];
                                let mut want_buffer = vec![0xAAu8; length_usize];
                                let got = overlay.read_at(offset, &mut got_buffer);
                                let want = reference.read_at(offset, &mut want_buffer);
                                (format!("{got:?} {got_buffer:?}"), format!("{want:?} {want_buffer:?}"))
                            }
                        };
                        assert_eq!(got, want, "block={block} size={size} seed={seed} kind={kind} offset={} length={length}", offset.0);
                        compared += 1;
                    }
                    assert_eq!(base.bytes, initial, "底下那块盘一个字节都没被写");
                }
            }
        }
        println!("ATK overlay compared={compared}");
    }

    #[test]
    fn atk_overlay_check_is_the_file_backend_check() {
        let mut compared = 0u64;
        let mut differ = 0u64;
        for block in [1u32, 512, 520, 4096, 65536] {
            let block_bytes = u64::from(block);
            for size in [0u64, block_bytes, block_bytes * 64, block_bytes * 64 + 100, u64::MAX - 7] {
                let disk = AtkVecDevice { bytes: Vec::new(), block: PhysicalBlockSizeInBytes(block) };
                // 底下那块盘的大小：参照盘的字节数取不到 u64 那么大，借一层只报大小的盘。
                struct SizedDisk(PhysicalBlockSizeInBytes, u64);
                impl BlockDevice for SizedDisk {
                    fn read_at(&self, _: DeviceOffsetInBytes, _: &mut [u8]) -> Result<(), BlockDeviceError> { unreachable!() }
                    fn write_at(&mut self, _: DeviceOffsetInBytes, _: &[u8], _: WriteDurability) -> Result<(), BlockDeviceError> { unreachable!() }
                    fn write_zeroes_at(&mut self, _: DeviceOffsetInBytes, _: u64) -> Result<(), BlockDeviceError> { unreachable!() }
                    fn barrier(&mut self) -> Result<(), BlockDeviceError> { unreachable!() }
                    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes { self.0 }
                    fn size_in_bytes(&self) -> u64 { self.1 }
                }
                let _ = disk;
                let sized = SizedDisk(PhysicalBlockSizeInBytes(block), size);
                let overlay = DeviceKeepingWritesInMemory::over_the_disk(&sized);
                let mut random = AtkRandom((u64::from(block) * 31).wrapping_add(size).wrapping_add(7) | 1);
                let mut requests: Vec<(u64, u64)> = vec![(0, 0), (size, 0), (size.wrapping_add(1), 0), (u64::MAX, 0), (u64::MAX, 1), (0, u64::MAX), (u64::MAX - block_bytes + 1, block_bytes), (size.saturating_sub(block_bytes), block_bytes), (size.saturating_sub(block_bytes), 2 * block_bytes)];
                for _ in 0..2000 {
                    requests.push((random.next(), random.next()));
                    requests.push((random.below(size.saturating_add(3 * block_bytes).max(1)) / block_bytes * block_bytes, random.below(8) * block_bytes));
                }
                for (offset, length) in requests {
                    let got = format!("{:?}", overlay.check_aligned_and_in_range(DeviceOffsetInBytes(offset), length));
                    let want = format!("{:?}", crate::block_device::check_aligned_and_in_range(DeviceOffsetInBytes(offset), length, PhysicalBlockSizeInBytes(block), size));
                    compared += 1;
                    if got != want {
                        differ += 1;
                        println!("ATK k5 differ block={block} size={size} offset={offset} length={length} overlay={got} file_backend={want}");
                    }
                }
            }
        }
        println!("ATK k5 compared={compared} differ={differ}");
        assert_eq!(differ, 0);
    }
}
