//! 复用键的两样输入：一步带屏障的写表哈希、它起步的镜像（扇区终值表），以及写表哈希里一次写的内容的写法。

use super::*;

// ============================== 录入的属性：写表与父输出 ==============================

/// 一次写的内容在写表哈希里的写法：普通写取字节的 SHA-256，整段清零只写长度。崩溃放量算判器另读的输入时
/// （`crash_amplification` 的计划哈希）也照这一份写，两处一个写法。
pub(crate) fn content_digest_text(contents: &WrittenContents) -> String {
    match contents {
        WrittenContents::Bytes(bytes) => hexadecimal_text(&sha256_digest(bytes)),
        WrittenContents::Zeros { length } => format!("zeros:{length}"),
    }
}

/// 一步的写表哈希（带屏障）：这一步的写按写表次序逐条，每条记盘、种类、FUA、偏移、长度、内容摘要、这次写在这一步里落在第几段。
/// 段由调用方按 D13（验证路线） 已定项 4 切好（`segments` 是整条流的段，写表下标）：屏障挪了、少了，段号就变，哈希跟着变
/// （E164 F1：只按字节算会漏掉去掉一道屏障的改动）。
///
/// # Panics
/// `step_writes` 里有写不属于任何段。
#[must_use]
pub fn step_write_table_with_barriers(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    step_writes: std::ops::Range<usize>,
) -> [u8; DIGEST_BYTES] {
    let mut segment_of_write: BTreeMap<usize, usize> = BTreeMap::new();
    for (segment_index, segment) in segments.iter().enumerate() {
        for write_index in segment {
            segment_of_write.insert(*write_index, segment_index);
        }
    }
    let first_segment_of_step = step_writes
        .clone()
        .filter_map(|write_index| segment_of_write.get(&write_index).copied())
        .min()
        .unwrap_or(0);
    let mut text = String::new();
    for write_index in step_writes {
        let write = &writes[write_index];
        let segment = segment_of_write
            .get(&write_index)
            .unwrap_or_else(|| panic!("写 {write_index} 不属于任何段"));
        text.push_str(&format!(
            "W {} {} {} {} {} {} {}\n",
            write.device.0,
            write.kind.name(),
            u8::from(write.is_force_unit_access),
            write.offset.0,
            write.length_in_bytes(),
            content_digest_text(&write.contents),
            segment - first_segment_of_step
        ));
    }
    sha256_digest(text.as_bytes())
}

/// 扇区终值表：（盘，扇区号）→ 那个扇区最后留下的内容摘要。父输出 = 基镜像上叠这一步之前全部写之后的这张表。
#[derive(Clone, Default)]
pub struct SectorImage {
    sectors: BTreeMap<(u32, u64), [u8; DIGEST_BYTES]>,
}

impl SectorImage {
    /// 基镜像（mkfs 之后）里写过的每个扇区。
    #[must_use]
    pub fn of_base(base: &MemoryPool) -> Self {
        let mut sectors = BTreeMap::new();
        for (device, image) in &base.devices {
            for (sector, bytes) in image.written_sectors() {
                sectors.insert((device.0, *sector), sha256_digest(bytes));
            }
        }
        Self { sectors }
    }

    /// 按写表次序叠几次写。
    pub fn overlay(&mut self, writes: &[RetainedWrite]) {
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        for write in writes {
            let first_sector = write.offset.0 / SECTOR_BYTES;
            match &write.contents {
                WrittenContents::Bytes(bytes) => {
                    for (sector_offset, chunk) in bytes.chunks(sector_bytes).enumerate() {
                        let sector = first_sector
                            + u64::try_from(sector_offset).expect("扇区序号装得进 u64");
                        self.sectors
                            .insert((write.device.0, sector), sha256_digest(chunk));
                    }
                }
                WrittenContents::Zeros { length } => {
                    let zero_sector = sha256_digest(&vec![0u8; sector_bytes]);
                    for sector_offset in 0..length.div_ceil(SECTOR_BYTES) {
                        self.sectors
                            .insert((write.device.0, first_sector + sector_offset), zero_sector);
                    }
                }
            }
        }
    }

    /// 按（盘，扇区）升序每行 `<盘> <扇区> <摘要>` 拼起来取 SHA-256。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        let mut text = String::new();
        for ((device, sector), digest) in &self.sectors {
            text.push_str(&format!("{device} {sector} {}\n", hexadecimal_text(digest)));
        }
        sha256_digest(text.as_bytes())
    }
}
