//! 内存盘与录制摘要：按块懒分配、块照两种存法存，读写记进同一份录制摘要。

use super::*;

// ---- 内存盘与录制摘要（登记第五节「负载」第 1 条、「装置约束」第三行）----

/// 改盘的一个动作（录制摘要按先后喂这一种、盘、偏移、长度与字节）。
#[derive(Clone, Copy, Debug)]
enum RecordedDeviceOperation<'bytes> {
    Write {
        device: DeviceIdentity,
        offset_in_bytes: u64,
        bytes: &'bytes [u8],
        durability: WriteDurability,
    },
    WriteZeroes {
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    },
    Discard {
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    },
    Barrier {
        device: DeviceIdentity,
    },
}

/// 把一个改盘动作喂进录制摘要：种类、盘、偏移、长度，写的另喂持久方式与字节（第九节变异「录制摘要只喂长度」改的是这里）。
fn feed_the_operation_into_the_digest(
    digest: &mut Sha256Digest,
    operation: RecordedDeviceOperation<'_>,
) {
    match operation {
        RecordedDeviceOperation::Write {
            device,
            offset_in_bytes,
            bytes,
            durability,
        } => {
            digest.feed(&[1]);
            digest.feed(&device.0.to_le_bytes());
            digest.feed_u64(offset_in_bytes);
            digest.feed_u64(u64::try_from(bytes.len()).expect("写长装得进 u64"));
            digest.feed(&[match durability {
                WriteDurability::Plain => 0,
                WriteDurability::ForceUnitAccess => 1,
            }]);
            digest.feed(bytes);
        }
        RecordedDeviceOperation::WriteZeroes {
            device,
            offset_in_bytes,
            length_in_bytes,
        } => {
            digest.feed(&[2]);
            digest.feed(&device.0.to_le_bytes());
            digest.feed_u64(offset_in_bytes);
            digest.feed_u64(length_in_bytes);
        }
        RecordedDeviceOperation::Discard {
            device,
            offset_in_bytes,
            length_in_bytes,
        } => {
            digest.feed(&[3]);
            digest.feed(&device.0.to_le_bytes());
            digest.feed_u64(offset_in_bytes);
            digest.feed_u64(length_in_bytes);
        }
        RecordedDeviceOperation::Barrier { device } => {
            digest.feed(&[4]);
            digest.feed(&device.0.to_le_bytes());
        }
    }
}

/// 一个池两块盘共用的录制摘要，连同喂它花掉的线程 CPU（`name=recording_overhead`）与读 CPU 钟失败的次数。
pub(super) struct RecordingDigest {
    is_fed: bool,
    pub(super) digest: Sha256Digest,
    pub(super) operations: u64,
    pub(super) cpu_spent_feeding: Duration,
    pub(super) clock_read_failures: u64,
}

impl RecordingDigest {
    fn empty() -> RecordingDigest {
        RecordingDigest {
            is_fed: true,
            digest: Sha256Digest::new(),
            operations: 0,
            cpu_spent_feeding: Duration::ZERO,
            clock_read_failures: 0,
        }
    }

    /// 造起步镜像时不喂摘要（造池不比录制流，省下给几 GiB 字节算摘要的 CPU）。
    pub(super) fn not_fed() -> RecordingDigest {
        RecordingDigest {
            is_fed: false,
            digest: Sha256Digest::new(),
            operations: 0,
            cpu_spent_feeding: Duration::ZERO,
            clock_read_failures: 0,
        }
    }

    fn note(&mut self, operation: RecordedDeviceOperation<'_>) {
        if !self.is_fed {
            return;
        }
        let digest = &mut self.digest;
        let ((), spent) =
            thread_cpu_spent_in(|| feed_the_operation_into_the_digest(digest, operation));
        self.operations += 1;
        match spent {
            Some(spent) => self.cpu_spent_feeding += spent,
            None => self.clock_read_failures += 1,
        }
    }
}

pub(super) type SharedRecordingDigest = Arc<Mutex<RecordingDigest>>;

/// 内存盘的块怎么存（登记第五节「负载」第 1 条的稀疏内存盘）。两种存法读写的字节逐个相同，只差占多少内存。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChunkStorage {
    /// 每块盘各存各的块，拷镜像整份复制：改前的存法，只留给 `construction-timing` 对照与同包测试对拍。
    EachDeviceCopiesItsChunks,
    /// 同一个池里同号、逐字节相同的块共用一份：核心层把每个单元与每条 journal 记录同一份字节写到每块盘的同一偏移
    /// （`singlefs_core::transaction` 的 `CommitStep::WriteUnitToEveryDevice`、`WriteJournalRecordToEveryDevice`），后写的那块盘认出同号块与
    /// 先写的那块相同就改指那一份；全是零的块不存（没写过的读回 0）；拷镜像只拷块的引用，哪块盘写到共用的块才复制那一块。
    SharedWhenTheBytesAreEqual,
}

/// 一个池两块盘共用的「每个块号最近写成的那一份」：后写同号块的那块盘拿它比字节，相同就共用。每个块号只留一项，项数不超过盘的块数。
type LatestChunksOfThePool = Arc<Mutex<HashMap<u64, Weak<Vec<u8>>>>>;

/// 内存里的一块稀疏盘：按 64 KiB 块懒分配，没写过的读回 0；每个改盘动作喂进这个池共用的录制摘要。
pub(super) struct RecordingMemoryDevice {
    identity: DeviceIdentity,
    pub(super) size_in_bytes: u64,
    pub(super) chunks: HashMap<u64, Arc<Vec<u8>>>,
    pub(super) storage: ChunkStorage,
    latest_chunks: LatestChunksOfThePool,
    recording: SharedRecordingDigest,
}

impl RecordingMemoryDevice {
    fn note(&self, operation: RecordedDeviceOperation<'_>) {
        self.recording
            .lock()
            .expect("录制摘要只在借着盘的那条线程上喂，没在别处 panic 过")
            .note(operation);
    }

    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) -> Result<(), BlockDeviceError> {
        if !offset.0.is_multiple_of(SECTOR_BYTES) || !length_in_bytes.is_multiple_of(SECTOR_BYTES) {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length: length_in_bytes,
                physical_block_size: u32::try_from(SECTOR_BYTES).expect("512"),
            });
        }
        if offset
            .0
            .checked_add(length_in_bytes)
            .is_none_or(|end| end > self.size_in_bytes)
        {
            return Err(BlockDeviceError::OutOfRange {
                offset,
                length: length_in_bytes,
                device_size: self.size_in_bytes,
            });
        }
        Ok(())
    }

    /// `[offset, offset + length)` 按块切开：块号、块内起点、这一截的长度、这一截在请求里的起点。
    fn pieces_of_the_range(
        offset_in_bytes: u64,
        length_in_bytes: u64,
    ) -> Vec<(u64, usize, usize, usize)> {
        let mut pieces = Vec::new();
        let mut position = offset_in_bytes;
        let end = offset_in_bytes + length_in_bytes;
        while position < end {
            let chunk_index = position / MEMORY_DEVICE_CHUNK_BYTES;
            let start_in_the_chunk = position % MEMORY_DEVICE_CHUNK_BYTES;
            let length_in_the_chunk =
                (MEMORY_DEVICE_CHUNK_BYTES - start_in_the_chunk).min(end - position);
            pieces.push((
                chunk_index,
                usize::try_from(start_in_the_chunk).expect("块内偏移"),
                usize::try_from(length_in_the_chunk).expect("块内长度"),
                usize::try_from(position - offset_in_bytes).expect("请求里的偏移"),
            ));
            position += length_in_the_chunk;
        }
        pieces
    }

    fn zero_the_range(&mut self, offset_in_bytes: u64, length_in_bytes: u64) {
        for (chunk_index, start_in_the_chunk, length_in_the_chunk, _start_in_the_request) in
            Self::pieces_of_the_range(offset_in_bytes, length_in_bytes)
        {
            if length_in_the_chunk == usize::try_from(MEMORY_DEVICE_CHUNK_BYTES).expect("64 KiB") {
                self.chunks.remove(&chunk_index);
            } else if let Some(chunk) = self.chunks.get_mut(&chunk_index) {
                Arc::make_mut(chunk)[start_in_the_chunk..start_in_the_chunk + length_in_the_chunk]
                    .fill(0);
                self.settle_the_chunk(chunk_index);
            }
        }
    }

    /// 写过块 `chunk_index` 之后按存法收拾它（[`ChunkStorage::SharedWhenTheBytesAreEqual`]）：全是零就不存；同号块最近写成的那一份
    /// 与它逐字节相同就改指那一份，不同就把它记成这个块号最近写成的那一份。各存各的那一种什么都不做。
    fn settle_the_chunk(&mut self, chunk_index: u64) {
        match self.storage {
            ChunkStorage::EachDeviceCopiesItsChunks => {}
            ChunkStorage::SharedWhenTheBytesAreEqual => {
                let Some(chunk) = self.chunks.get_mut(&chunk_index) else {
                    return;
                };
                if chunk.iter().all(|byte| *byte == 0) {
                    self.chunks.remove(&chunk_index);
                    return;
                }
                let mut latest_chunks = self
                    .latest_chunks
                    .lock()
                    .expect("块表只在借着盘的那条线程上改，没在别处 panic 过");
                let latest_of_the_index = latest_chunks.get(&chunk_index).and_then(Weak::upgrade);
                match latest_of_the_index {
                    Some(latest) if Arc::ptr_eq(&latest, chunk) => {}
                    Some(latest) if latest.as_slice() == chunk.as_slice() => *chunk = latest,
                    Some(_) | None => {
                        latest_chunks.insert(chunk_index, Arc::downgrade(chunk));
                    }
                }
            }
        }
    }

    /// 同一份字节的另一块盘，记进 `recording`、共用 `latest_chunks`（拷起步镜像：登记第五节「量挂载」从起步镜像拷一份）。
    /// 共用的存法只拷块的引用（写到哪一块才复制那一块）；各存各的存法整份复制。
    fn copied_into(
        &self,
        recording: &SharedRecordingDigest,
        latest_chunks: &LatestChunksOfThePool,
    ) -> RecordingMemoryDevice {
        let chunks = match self.storage {
            ChunkStorage::EachDeviceCopiesItsChunks => self
                .chunks
                .iter()
                .map(|(chunk_index, chunk)| (*chunk_index, Arc::new(Vec::clone(chunk))))
                .collect(),
            ChunkStorage::SharedWhenTheBytesAreEqual => self.chunks.clone(),
        };
        RecordingMemoryDevice {
            identity: self.identity,
            size_in_bytes: self.size_in_bytes,
            chunks,
            storage: self.storage,
            latest_chunks: Arc::clone(latest_chunks),
            recording: Arc::clone(recording),
        }
    }
}

/// 一个池的内存盘此刻装着的块字节：各块盘各自装着的（每块盘各数一遍），与真占内存的（共用的一份只数一次）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ChunkBytesOfThePool {
    pub(super) held_by_the_devices: u64,
    pub(super) stored: u64,
}

pub(super) fn chunk_bytes_of_the_pool(devices: &PoolDevices) -> ChunkBytesOfThePool {
    let mut stored_chunks: HashSet<*const Vec<u8>> = HashSet::new();
    let mut held_by_the_devices = 0_u64;
    for (_identity, device) in devices {
        for chunk in device.chunks.values() {
            held_by_the_devices += MEMORY_DEVICE_CHUNK_BYTES;
            stored_chunks.insert(Arc::as_ptr(chunk));
        }
    }
    ChunkBytesOfThePool {
        held_by_the_devices,
        stored: u64::try_from(stored_chunks.len()).expect("块数装得进 u64")
            * MEMORY_DEVICE_CHUNK_BYTES,
    }
}

impl BlockDevice for RecordingMemoryDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length_in_bytes)?;
        for (chunk_index, start_in_the_chunk, length_in_the_chunk, start_in_the_request) in
            Self::pieces_of_the_range(offset.0, length_in_bytes)
        {
            let target =
                &mut buffer[start_in_the_request..start_in_the_request + length_in_the_chunk];
            match self.chunks.get(&chunk_index) {
                Some(chunk) => target.copy_from_slice(
                    &chunk[start_in_the_chunk..start_in_the_chunk + length_in_the_chunk],
                ),
                None => target.fill(0),
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(bytes.len()).expect("写长装得进 u64");
        self.check_request(offset, length_in_bytes)?;
        let chunk_bytes = usize::try_from(MEMORY_DEVICE_CHUNK_BYTES).expect("64 KiB");
        for (chunk_index, start_in_the_chunk, length_in_the_chunk, start_in_the_request) in
            Self::pieces_of_the_range(offset.0, length_in_bytes)
        {
            let chunk = self
                .chunks
                .entry(chunk_index)
                .or_insert_with(|| Arc::new(vec![0u8; chunk_bytes]));
            Arc::make_mut(chunk)[start_in_the_chunk..start_in_the_chunk + length_in_the_chunk]
                .copy_from_slice(
                    &bytes[start_in_the_request..start_in_the_request + length_in_the_chunk],
                );
            self.settle_the_chunk(chunk_index);
        }
        self.note(RecordedDeviceOperation::Write {
            device: self.identity,
            offset_in_bytes: offset.0,
            bytes,
            durability,
        });
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_the_range(offset.0, length);
        self.note(RecordedDeviceOperation::WriteZeroes {
            device: self.identity,
            offset_in_bytes: offset.0,
            length_in_bytes: length,
        });
        Ok(())
    }

    /// 内存盘认 discard，当整段清零应用，交回「交给了底层」（同产品 GPU 包测试共用的内存盘）。
    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_the_range(offset.0, length);
        self.note(RecordedDeviceOperation::Discard {
            device: self.identity,
            offset_in_bytes: offset.0,
            length_in_bytes: length,
        });
        Ok(DiscardOutcome::Applied)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.note(RecordedDeviceOperation::Barrier {
            device: self.identity,
        });
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(u32::try_from(SECTOR_BYTES).expect("512"))
    }

    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }
}

pub(super) type PoolDevices = Vec<(DeviceIdentity, RecordingMemoryDevice)>;

/// 两块空盘，记进同一份录制摘要，块照共用的存法存（[`ChunkStorage::SharedWhenTheBytesAreEqual`]）。只有同包测试用。
#[cfg(test)]
pub(super) fn empty_pool_devices(
    device_bytes: u64,
    recording: &SharedRecordingDigest,
) -> PoolDevices {
    empty_pool_devices_storing_the_chunks(
        device_bytes,
        recording,
        ChunkStorage::SharedWhenTheBytesAreEqual,
    )
}

/// 两块空盘，记进同一份录制摘要，块照 `storage` 存；两块盘共用一份块表。
pub(super) fn empty_pool_devices_storing_the_chunks(
    device_bytes: u64,
    recording: &SharedRecordingDigest,
    storage: ChunkStorage,
) -> PoolDevices {
    let latest_chunks: LatestChunksOfThePool = Arc::new(Mutex::new(HashMap::new()));
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                RecordingMemoryDevice {
                    identity,
                    size_in_bytes: device_bytes,
                    chunks: HashMap::new(),
                    storage,
                    latest_chunks: Arc::clone(&latest_chunks),
                    recording: Arc::clone(recording),
                },
            )
        })
        .collect()
}

/// 同一份字节的另一个池，记进一份新的空录制摘要（「同一历史」的另一个副本），两块盘另共用一份新的块表。
pub(super) fn copied_pool_devices(devices: &PoolDevices) -> (PoolDevices, SharedRecordingDigest) {
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
    let latest_chunks: LatestChunksOfThePool = Arc::new(Mutex::new(HashMap::new()));
    let copied = devices
        .iter()
        .map(|(identity, device)| (*identity, device.copied_into(&recording, &latest_chunks)))
        .collect();
    (copied, recording)
}

#[cfg(test)]
mod tests;
