use super::*;

// ============================================================================
// 九、单盘恢复的读者：崩溃镜像上只留一块盘（另一块整块掉了）；甲一、甲二避开某条根时，把那条根的根槽读成全零（再择就取下一条）。
// ============================================================================

pub(super) struct KeptDiskView<'image, 'base> {
    pub(super) image: &'image CrashImage<'base>,
    pub(super) device: DeviceIdentity,
    /// 被避开的根的根槽字节区间 (偏移, 长度)，读回全零。
    pub(super) masked_root_slots: Vec<(u64, u64)>,
}

impl PoolReader for KeptDiskView<'_, '_> {
    /// 掉了的那块盘的身份还在盘表里，只是读不到（与验证包「盘整块掉了」的两种形态同一个口径：
    /// `crates/singlefs-verification/tests/whole_device_loss_after_warm_up.rs`）。把它的身份也拿掉，分配记录树的走读会按一块盘的几何判，
    /// 每个状态都报 `InvariantViolated` I-1.1（装置第一次样本里的读数），那不是单盘恢复。
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.image)
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.image, device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        PoolReader::physical_block_size_probed_on_the_device(self.image, device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        if device != self.device {
            return None;
        }
        let mut bytes = PoolReader::read(self.image, device, offset, length)?;
        let read_end = offset.0 + u64::try_from(length).expect("读长装得进 u64");
        for (masked_offset, masked_length) in &self.masked_root_slots {
            let start = (*masked_offset).max(offset.0);
            let end = (masked_offset + masked_length).min(read_end);
            if start < end {
                let first = usize::try_from(start - offset.0).expect("缓冲区内偏移");
                let last = usize::try_from(end - offset.0).expect("缓冲区内偏移");
                bytes[first..last].fill(0);
            }
        }
        Some(bytes)
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        if device != self.device {
            return Some(Vec::new());
        }
        PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes)
    }
}

impl ImageReader for KeptDiskView<'_, '_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.image)
    }

    fn device_bytes(&self, device: u32) -> Option<u64> {
        PoolReader::device_size_in_bytes(self, DeviceIdentity(device))
    }

    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        PoolReader::read(
            self,
            DeviceIdentity(device),
            DeviceOffsetInBytes(offset),
            length,
        )
    }

    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        if device == self.device.0 {
            ImageReader::candidate_unit_slots(self.image, device)
        } else {
            Some(Vec::new())
        }
    }

    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        if device == self.device.0 {
            ImageReader::candidate_journal_slots(self.image, device)
        } else {
            Some(Vec::new())
        }
    }
}

pub(super) fn kept_disk<'image, 'base>(
    image: &'image CrashImage<'base>,
    device: DeviceIdentity,
) -> KeptDiskView<'image, 'base> {
    KeptDiskView {
        image,
        device,
        masked_root_slots: Vec::new(),
    }
}

// ----------------------------------------------------------------------------
// 被盖：恢复择到的根引用的某个单元，在用到的盘上的字节与全知日志里这条根写下时的字节不同（逐字节比，经读）。
// ----------------------------------------------------------------------------

pub(super) fn unit_differs_in_the_image(image: &CrashImage<'_>, unit: &UnitLog) -> bool {
    match PoolReader::read(
        image,
        DeviceIdentity(unit.device),
        DeviceOffsetInBytes(unit.offset_in_bytes),
        unit.bytes.len(),
    ) {
        Some(read_back) => read_back != unit.bytes,
        None => true,
    }
}

/// 这条根引用的单元里有没有被盖的；`only_on_device` 给了盘号就只看落在那块盘上的单元（单盘恢复用得到的那一份），不给就两块盘都看。
pub(super) fn some_unit_of_the_root_is_covered(
    image: &CrashImage<'_>,
    log: &RootLog,
    only_on_device: Option<u32>,
) -> bool {
    log.units
        .iter()
        .filter(|unit| only_on_device.is_none_or(|device| unit.device == device))
        .any(|unit| unit_differs_in_the_image(image, unit))
}

#[cfg(test)]
mod tests;
