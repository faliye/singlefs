use super::*;

// ───────────────────────────── 真盘臂的镜像（登记第五节：起始件、每轮拷回、核摘要） ─────────────────────────────

/// 一块 4 MiB 块的 SHA-256。
pub(super) type ChunkDigest = [u8; 32];

fn image_chunk_length() -> usize {
    usize::try_from(IMAGE_CHUNK_BYTES).expect("4 MiB 装得进 usize")
}

/// 全 0 那一块的摘要（算一次）。
fn digest_of_a_zero_chunk() -> ChunkDigest {
    static DIGEST: std::sync::OnceLock<ChunkDigest> = std::sync::OnceLock::new();
    *DIGEST.get_or_init(|| sha256_digest(&vec![0_u8; image_chunk_length()]))
}

/// 镜像的分块摘要：每 4 MiB 一块的 SHA-256 依次接起来再算一次 SHA-256。任何一个字节不同，摘要就不同；内存盘与镜像文件按同一个定义算。
fn digest_of_the_chunk_digests(chunk_digests: &[ChunkDigest]) -> ChunkDigest {
    let joined: Vec<u8> = chunk_digests
        .iter()
        .flat_map(|digest| digest.iter().copied())
        .collect();
    sha256_digest(&joined)
}

pub(super) fn hexadecimal_of(digest: &ChunkDigest) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 一块内存盘的分块摘要（没写过扇区的块就是全 0 块）。
pub(super) fn digest_of_the_memory_device(device: &SparseBlockDevice) -> ChunkDigest {
    let mut buffer = vec![0_u8; image_chunk_length()];
    let chunk_digests: Vec<ChunkDigest> = (0..device.size_in_bytes() / IMAGE_CHUNK_BYTES)
        .map(|chunk| {
            let offset = DeviceOffsetInBytes(chunk * IMAGE_CHUNK_BYTES);
            if device
                .image
                .written_sectors_in(offset, IMAGE_CHUNK_BYTES)
                .is_empty()
            {
                digest_of_a_zero_chunk()
            } else {
                device.image.read_into(offset, &mut buffer);
                sha256_digest(&buffer)
            }
        })
        .collect();
    digest_of_the_chunk_digests(&chunk_digests)
}

/// 一个镜像文件的分块摘要。
fn digest_of_the_image_file(path: &Path) -> std::io::Result<ChunkDigest> {
    let file = File::open(path)?;
    let length = file.metadata()?.len();
    let mut buffer = vec![0_u8; image_chunk_length()];
    let mut chunk_digests = Vec::new();
    for chunk in 0..length / IMAGE_CHUNK_BYTES {
        file.read_exact_at(&mut buffer, chunk * IMAGE_CHUNK_BYTES)?;
        chunk_digests.push(if buffer.iter().all(|byte| *byte == 0) {
            digest_of_a_zero_chunk()
        } else {
            sha256_digest(&buffer)
        });
    }
    Ok(digest_of_the_chunk_digests(&chunk_digests))
}

/// 建一个整份写满零并 `fsync` 的镜像文件（不是稀疏、不是 `fallocate` 的未写区段），再核占用块数 × 512 ≥ 文件字节数
/// （登记第五节；`stat -c '%b %B %s'` 的同一组数）。
/// 镜像文件的文件名（不带目录）：报错进产物行的 `reason=`，目录是执行的那台机器的私有路径，不进产物。
fn name_of_the_image_file(path: &Path) -> String {
    path.file_name().map_or_else(
        || "unnamed_image".to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

pub(super) fn create_a_zero_filled_image(path: &Path, bytes: u64) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let zeros = vec![0_u8; image_chunk_length()];
    for _chunk in 0..bytes / IMAGE_CHUNK_BYTES {
        file.write_all(&zeros)?;
    }
    file.sync_all()?;
    let metadata = file.metadata()?;
    if metadata.blocks() * 512 < metadata.len() {
        return Err(std::io::Error::other(format!(
            "{} 占用块数 {} × 512 小于文件字节数 {}：是稀疏文件",
            name_of_the_image_file(path),
            metadata.blocks(),
            metadata.len()
        )));
    }
    Ok(())
}

/// 把一块内存盘写过的扇区原样写进一个全零的镜像文件（相邻扇区并成一次写），再 `fsync`。
fn write_the_memory_device_into(path: &Path, device: &SparseBlockDevice) -> std::io::Result<()> {
    let file = OpenOptions::new().write(true).open(path)?;
    let sector_bytes = u64::from(PHYSICAL_BLOCK_BYTES);
    let mut run_start: Option<u64> = None;
    let mut run_bytes: Vec<u8> = Vec::new();
    for (sector, bytes) in device.image.written_sectors() {
        let offset = sector * sector_bytes;
        let continues_the_run =
            run_start.is_some_and(|start| start + length_in_bytes(&run_bytes) == offset);
        if !continues_the_run || run_bytes.len() >= image_chunk_length() {
            if let Some(start) = run_start {
                file.write_all_at(&run_bytes, start)?;
            }
            run_start = Some(offset);
            run_bytes.clear();
        }
        run_bytes.extend_from_slice(bytes);
    }
    if let Some(start) = run_start {
        file.write_all_at(&run_bytes, start)?;
    }
    file.sync_all()
}

/// 从起始件把点名的块拷回工作件，再 `fsync`；交回拷了几块。
fn restore_the_chunks(start: &Path, work: &Path, chunks: &BTreeSet<u64>) -> std::io::Result<u64> {
    let source = File::open(start)?;
    let destination = OpenOptions::new().write(true).open(work)?;
    let mut buffer = vec![0_u8; image_chunk_length()];
    for chunk in chunks {
        source.read_exact_at(&mut buffer, chunk * IMAGE_CHUNK_BYTES)?;
        destination.write_all_at(&buffer, chunk * IMAGE_CHUNK_BYTES)?;
    }
    destination.sync_all()?;
    Ok(u64::try_from(chunks.len()).expect("块数"))
}

/// Linux 的 `POSIX_FADV_DONTNEED`（`<linux/fadvise.h>` 里是 4）：把文件这一段干净的页缓存扔掉。
const POSIX_FADVISE_ADVICE_DONT_NEED: libc::c_int = libc::POSIX_FADV_DONTNEED;

// `libc::posix_fadvise` 照 POSIX `int posix_fadvise(int fd, off_t offset, off_t len, int advice)`；出错交回错误号（不设 errno）。

/// 先 `fsync` 再把整个文件的页缓存扔掉（脏页扔不掉，所以先同步）。真盘臂每次测量之前对工作件做一次，量的那一刻工作件一页都不在页缓存里，
/// 与工作件在这之前被谁读过、内存紧不紧（准备阶段并行时同时攥着几份池）无关（登记第十二节 12.10）。
#[allow(
    unsafe_code,
    reason = "std 没有 posix_fadvise，只扔一个文件的页缓存只能经 libc 调 posix_fadvise(2) 的 POSIX_FADV_DONTNEED；已有依赖没有安全封装，写 /proc/sys/vm/drop_caches 要 root、扔的是全机的缓存"
)]
fn drop_the_page_cache_of_the_file(path: &Path) -> std::io::Result<()> {
    use std::os::fd::AsRawFd as _;
    let file = File::open(path)?;
    file.sync_all()?;
    // SAFETY: 文件描述符来自这个函数里打开、这一帧里还活着的 `file`；偏移与长度都是 0（整个文件），不传指针，内核不往本进程内存里写。
    let status =
        unsafe { libc::posix_fadvise(file.as_raw_fd(), 0, 0, POSIX_FADVISE_ADVICE_DONT_NEED) };
    if status == 0 {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(status))
    }
}

/// 单元区之前（系统配置、根环、journal 环）写过的扇区抄进一块新的内存盘：真盘臂挂载时照它答 journal 记录偏移的提示。
fn image_of_the_area_before_the_units(
    device: &SparseBlockDevice,
    unit_area_start_slot: u64,
) -> SparseBlockDevice {
    let boundary = unit_area_start_slot * LOCAL_SLOT_BYTES;
    let sector_bytes = u64::from(PHYSICAL_BLOCK_BYTES);
    let mut copy =
        SparseBlockDevice::new(device.size_in_bytes(), device.probe_physical_block_size());
    for (sector, bytes) in device.image.written_sectors() {
        let offset = sector * sector_bytes;
        if offset < boundary {
            copy.image.write(DeviceOffsetInBytes(offset), bytes);
        }
    }
    copy
}

/// 一个取样点在真盘上的两份镜像：起始件（造好的池，写满零之后写进去）与工作件（每次测量之前把上一次改过的块从起始件拷回、核摘要）。
pub(super) struct RealDiskImages {
    start_paths: Vec<PathBuf>,
    work_paths: Vec<PathBuf>,
    start_digests: Vec<ChunkDigest>,
    chunks_to_restore: Vec<BTreeSet<u64>>,
    hint_images: Vec<SparseBlockDevice>,
    pub(super) unit_area_start_slot: u64,
}

impl RealDiskImages {
    /// 建起始件与工作件：两份都先整份写满零并 `fsync`，起始件再写进造好的池；核起始件的摘要与内存盘相同。
    pub(super) fn create(
        directory: &Path,
        point: SamplingPoint,
        pool: &ConstructedPool,
    ) -> std::io::Result<RealDiskImages> {
        std::fs::create_dir_all(directory)?;
        let mut images = RealDiskImages {
            start_paths: Vec::new(),
            work_paths: Vec::new(),
            start_digests: Vec::new(),
            chunks_to_restore: Vec::new(),
            hint_images: Vec::new(),
            unit_area_start_slot: pool.unit_area_start_slot,
        };
        for (identity, device) in &pool.devices {
            let stem = format!(
                "e185-fraction-{}-pool-data-units-{}-device-{}",
                point.fraction.label(),
                point.pool_data_units,
                identity.0
            );
            let start_path = directory.join(format!("{stem}-start.img"));
            let work_path = directory.join(format!("{stem}-work.img"));
            for path in [&start_path, &work_path] {
                if path.exists() {
                    std::fs::remove_file(path)?;
                }
                create_a_zero_filled_image(path, device.size_in_bytes())?;
            }
            write_the_memory_device_into(&start_path, device)?;
            let start_digest = digest_of_the_image_file(&start_path)?;
            if start_digest != digest_of_the_memory_device(device) {
                return Err(std::io::Error::other(format!(
                    "起始件 {} 的摘要与内存盘不同",
                    name_of_the_image_file(&start_path)
                )));
            }
            images.start_paths.push(start_path);
            images.work_paths.push(work_path);
            images.start_digests.push(start_digest);
            images
                .chunks_to_restore
                .push((0..device.size_in_bytes() / IMAGE_CHUNK_BYTES).collect());
            images.hint_images.push(image_of_the_area_before_the_units(
                device,
                pool.unit_area_start_slot,
            ));
        }
        Ok(images)
    }

    /// 把上一次改过的块从起始件拷回工作件，再核工作件的摘要与起始件相同（登记第十一节「起始镜像不同」那一轮作废），最后扔掉工作件的页缓存
    /// （[`drop_the_page_cache_of_the_file`]）。交回各盘拷回的块数。
    pub(super) fn restore_and_check(&mut self) -> Result<Vec<u64>, String> {
        let mut restored = Vec::new();
        for device_index in 0..self.work_paths.len() {
            let chunks = std::mem::take(&mut self.chunks_to_restore[device_index]);
            restored.push(
                restore_the_chunks(
                    &self.start_paths[device_index],
                    &self.work_paths[device_index],
                    &chunks,
                )
                .map_err(|error| field_token(&format!("restore_failed:{error}")))?,
            );
            let work_digest = digest_of_the_image_file(&self.work_paths[device_index])
                .map_err(|error| field_token(&format!("digest_failed:{error}")))?;
            if work_digest != self.start_digests[device_index] {
                self.chunks_to_restore[device_index] =
                    (0..LOCAL_DEVICE_BYTES / IMAGE_CHUNK_BYTES).collect();
                return Err("start_image_digest_differs".to_string());
            }
            drop_the_page_cache_of_the_file(&self.work_paths[device_index])
                .map_err(|error| field_token(&format!("page_cache_drop_failed:{error}")))?;
        }
        Ok(restored)
    }

    /// 以 `O_DIRECT` 打开两份工作件、套上计时包装与 journal 提示的照抄。
    pub(super) fn open_timed(
        &self,
        ledger: &Rc<RefCell<CallLedger>>,
    ) -> Result<
        Vec<(
            DeviceIdentity,
            TimedBlockDevice<DirectInputOutputBlockDevice>,
        )>,
        String,
    > {
        let mut devices = Vec::new();
        for (device_index, path) in self.work_paths.iter().enumerate() {
            let identity = DeviceIdentity(u32::try_from(device_index).expect("两块盘"));
            let inner = DirectInputOutputBlockDevice::open_existing(
                path,
                PageCachePolicy::BypassWithDirectInputOutput,
                PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)),
            )
            .map_err(|error| field_token(&format!("open_failed:{error:?}")))?;
            let mut device = TimedBlockDevice::new(inner, identity, ledger);
            let mut hint_image = SparseBlockDevice::new(
                self.hint_images[device_index].size_in_bytes(),
                self.hint_images[device_index].probe_physical_block_size(),
            );
            hint_image.image = self.hint_images[device_index].image.clone();
            device.journal_hint_mirror = Some(JournalHintMirror {
                image: hint_image,
                below_offset: self.unit_area_start_slot * LOCAL_SLOT_BYTES,
                is_mirroring: true,
            });
            devices.push((identity, device));
        }
        Ok(devices)
    }

    /// 一次测量之后，把它改过的块记下，下一次之前拷回。
    pub(super) fn note_the_changed_chunks(&mut self, ledger: &Rc<RefCell<CallLedger>>) {
        let changed = std::mem::take(&mut ledger.borrow_mut().chunks_written_since_the_copy);
        for (device, chunk) in changed {
            let device_index = usize::try_from(device.0).expect("盘号装得进 usize");
            self.chunks_to_restore[device_index].insert(chunk);
        }
    }

    /// 一个取样点量完删掉两份镜像（草稿目录里自己建的东西）。
    pub(super) fn remove(self) -> std::io::Result<()> {
        for path in self.start_paths.iter().chain(&self.work_paths) {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

/// 在造好的池的一份拷贝上量一条意图（内存臂）。
pub(super) fn measure_on_the_memory_device(
    pool: &ConstructedPool,
    switches: CompactionSwitches,
    injection: WriteInjection,
    capture: ReplayCapture,
) -> IntentReading {
    let ledger = Rc::new(RefCell::new(CallLedger::new(injection)));
    let mut devices = timed_devices(copy_of_the_memory_devices(&pool.devices), &ledger);
    measure_one_intent(&mut devices, &ledger, switches, capture)
}

/// 在真盘上量一条意图：工作件拷回、核摘要、打开、量、记下改过的块。拷回或打开不成交回原因（那一轮作废）。
pub(super) fn measure_on_the_real_disk(
    images: &mut RealDiskImages,
    switches: CompactionSwitches,
    injection: WriteInjection,
) -> Result<IntentReading, String> {
    images.restore_and_check()?;
    let ledger = Rc::new(RefCell::new(CallLedger::new(injection)));
    let mut devices = images.open_timed(&ledger)?;
    let reading = measure_one_intent(&mut devices, &ledger, switches, ReplayCapture::Off);
    drop(devices);
    images.note_the_changed_chunks(&ledger);
    Ok(reading)
}

#[cfg(test)]
mod tests;
