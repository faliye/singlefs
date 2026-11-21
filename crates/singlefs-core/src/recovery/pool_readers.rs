//! [`PoolReader`] 的各个实现：数只坏一份的读、只在映射落点上记只坏一份、一遍读与读坏之后当场重读的那一遍、
//! 盘与一串盘自己答、这一刻直接读盘。

use super::*;

/// 套在一个读者外面数「只坏一份的读」（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]）：读与别的几样报告原样交给里面那一份，
/// 只坏一份的那一报另加一次计数。计数只在内存里，随这个套子活着：冷恢复读不挂载，只计数、报给读者，不搬
/// （C946（只坏一份的读没有计数也没有搬迁修复）的取数接口）。
/// 同一个单元的同一份坏副本（同一组位置条目）套着的这一程里只计一次：可写挂载建上一版时实例表链经两处读两遍
/// （判决 `research/prompts/m4-close-code-r3-main-verification.md` 第二节 T2 ①），退到的那一版建不出、再建所选那一版时又读一遍同几个单元，
/// 都是同一份坏副本，一次挂载只算一次。
pub struct PoolReaderCountingUnitReadsWithOneCopyFailing<'reader, Reader: PoolReader + ?Sized> {
    reader: &'reader Reader,
    unit_reads_with_one_copy_failing: Cell<u64>,
    /// 计过的那几组位置条目，按第一次报的先后。
    locations_of_the_units_counted: RefCell<Vec<[LocationEntry; 2]>>,
}

impl<'reader, Reader: PoolReader + ?Sized>
    PoolReaderCountingUnitReadsWithOneCopyFailing<'reader, Reader>
{
    #[must_use]
    pub fn counting_from_zero(reader: &'reader Reader) -> Self {
        PoolReaderCountingUnitReadsWithOneCopyFailing {
            reader,
            unit_reads_with_one_copy_failing: Cell::new(0),
            locations_of_the_units_counted: RefCell::new(Vec::new()),
        }
    }

    /// 套上之后读到只坏一份的次数：同一组位置条目只算一次。
    #[must_use]
    pub fn unit_reads_with_one_copy_failing(&self) -> u64 {
        self.unit_reads_with_one_copy_failing.get()
    }

    /// 套上之后读到只坏一份的那几个单元的位置条目（一个单元一组，按第一次报的先后）：可写挂载交回之前按它们认映射 key，
    /// 交给会话下一次用户改动发成之后另发的那一批搬迁（`mounted_session::MountedSession::publish_user_change`）。
    #[must_use]
    pub fn locations_of_the_units_read_with_one_copy_failing(&self) -> Vec<[LocationEntry; 2]> {
        self.locations_of_the_units_counted.borrow().clone()
    }
}

impl<Reader: PoolReader + ?Sized> PoolReader
    for PoolReaderCountingUnitReadsWithOneCopyFailing<'_, Reader>
{
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.reader.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.reader.device_size_in_bytes(device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        self.reader.physical_block_size_probed_on_the_device(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reader.read(device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.reader
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reader.reread_after_a_bad_read(device, offset, length)
    }
    fn keep_the_self_verified_reread(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        self.reader
            .keep_the_self_verified_reread(device, offset, bytes);
    }
    fn note_that_the_bytes_read_self_verify(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        self.reader
            .note_that_the_bytes_read_self_verify(device, offset, bytes);
    }
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    ) {
        self.reader.note_that_the_bytes_read_fail_self_verification(
            device,
            offset,
            length_in_bytes,
        );
    }
    fn note_that_a_unit_was_read_with_one_copy_failing(&self, locations: &[LocationEntry; 2]) {
        self.reader
            .note_that_a_unit_was_read_with_one_copy_failing(locations);
        if self
            .locations_of_the_units_counted
            .borrow()
            .contains(locations)
        {
            return;
        }
        self.locations_of_the_units_counted
            .borrow_mut()
            .push(*locations);
        self.unit_reads_with_one_copy_failing
            .set(self.unit_reads_with_one_copy_failing.get() + 1);
    }
}

/// 套在一个读者外面按位置条目读一个进映射的单元（`mapping_key` 那一个）：只坏一份那一报（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]）
/// 只在读的那一组位置条目就是这个单元此刻的权威落点——中央映射里它那两条位置条目（D19（块指针的结构与宽度预算） 已定项 8）——时往里转；
/// 别的读与报告原样交给里面那一份。提示是盘上 extent 叶、父条目里记的，单元被整理搬走之后提示还指旧槽，旧槽上一份被盖掉、另一份还是旧字节，
/// 按提示读就是「只坏一份」，而映射里的权威落点两份都好：旧槽上坏的那一份不归这个单元计（判决 `research/prompts/m4-close-code-r3-main-verification.md`
/// 第二节 T2 ②、第三节第 4 条）。只有查到了、落点不是这一组才判得出提示过期；映射里查不出（没有这把 key、查映射本身判红）判不出，这一份是真读到的坏副本，
/// 照转（D4（校验和位置） 已定项 4「读到只坏一份时计数」，只有造坏的镜像走得到，判红留给走到映射那一步的读者照报）。
/// 经映射回退的那一读读的就是映射里那两条，照转。没转的那一报记下（[`Self::dropped_a_one_copy_failing_note_off_the_mapping_position`]）。
pub(crate) struct PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition<'reader> {
    reader: &'reader dyn PoolReader,
    mapping_key: &'reader [u8],
    central_mapping_locations_of_key: &'reader CentralMappingLocationsOfKey<'reader>,
    one_copy_failing_note_dropped: Cell<bool>,
}

impl<'reader> PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition<'reader> {
    pub(crate) fn over(
        reader: &'reader dyn PoolReader,
        mapping_key: &'reader [u8],
        central_mapping_locations_of_key: &'reader CentralMappingLocationsOfKey<'reader>,
    ) -> Self {
        Self {
            reader,
            mapping_key,
            central_mapping_locations_of_key,
            one_copy_failing_note_dropped: Cell::new(false),
        }
    }

    /// 有一报只坏一份因为读的那一组不是映射里的落点而没往里转。
    pub(crate) fn dropped_a_one_copy_failing_note_off_the_mapping_position(&self) -> bool {
        self.one_copy_failing_note_dropped.get()
    }
}

impl PoolReader for PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.reader.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.reader.device_size_in_bytes(device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        self.reader.physical_block_size_probed_on_the_device(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reader.read(device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.reader
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reader.reread_after_a_bad_read(device, offset, length)
    }
    fn keep_the_self_verified_reread(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        self.reader
            .keep_the_self_verified_reread(device, offset, bytes);
    }
    fn note_that_the_bytes_read_self_verify(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        self.reader
            .note_that_the_bytes_read_self_verify(device, offset, bytes);
    }
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    ) {
        self.reader.note_that_the_bytes_read_fail_self_verification(
            device,
            offset,
            length_in_bytes,
        );
    }
    fn note_that_a_unit_was_read_with_one_copy_failing(&self, locations: &[LocationEntry; 2]) {
        match (self.central_mapping_locations_of_key)(self.mapping_key) {
            Ok(Some(mapped_locations)) if mapped_locations != *locations => {
                self.one_copy_failing_note_dropped.set(true);
            }
            Ok(Some(_) | None) | Err(_) => self
                .reader
                .note_that_a_unit_was_read_with_one_copy_failing(locations),
        }
    }
}

/// 一遍读是第一次读，还是读坏之后当场重读的那一遍（D19（块指针的结构与宽度预算） 已定项 5、D16（发布语义） 已定项 1「根槽这一次读坏」那一行
/// 与「重读一次」：只重读一次，重读要再读一次盘）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadPassOverThePool {
    FirstRead,
    /// 读走 [`PoolReader::reread_after_a_bad_read`]：可写挂载里绕过这次挂载记下的那一份、直接读底下那块盘
    /// （主 agent 判定：抬 F 与续做里「读坏当场重读」那一读不许被预演与真做共用的那一份吃掉）；读到自证过的换掉记下的那一份。
    RereadAfterABadRead,
}

/// 一遍读的读者：第一次读照原样读；读坏之后当场重读的那一遍，读走 [`PoolReader::reread_after_a_bad_read`]，
/// 读的一方核过自证报上来的字节（[`PoolReader::note_that_the_bytes_read_self_verify`]）转成 [`PoolReader::keep_the_self_verified_reread`]。
pub struct PoolReaderOfOneReadPass<'reader, Reader: PoolReader + ?Sized> {
    reader: &'reader Reader,
    pass: ReadPassOverThePool,
}

impl<'reader, Reader: PoolReader + ?Sized> PoolReaderOfOneReadPass<'reader, Reader> {
    #[must_use]
    pub fn reading_for_the_first_time(reader: &'reader Reader) -> Self {
        PoolReaderOfOneReadPass {
            reader,
            pass: ReadPassOverThePool::FirstRead,
        }
    }

    #[must_use]
    pub fn rereading_after_a_bad_read(reader: &'reader Reader) -> Self {
        PoolReaderOfOneReadPass {
            reader,
            pass: ReadPassOverThePool::RereadAfterABadRead,
        }
    }

    #[must_use]
    pub fn in_the_pass(reader: &'reader Reader, pass: ReadPassOverThePool) -> Self {
        PoolReaderOfOneReadPass { reader, pass }
    }
}

impl<Reader: PoolReader + ?Sized> PoolReader for PoolReaderOfOneReadPass<'_, Reader> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.reader.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.reader.device_size_in_bytes(device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        self.reader.physical_block_size_probed_on_the_device(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        match self.pass {
            ReadPassOverThePool::FirstRead => self.reader.read(device, offset, length),
            ReadPassOverThePool::RereadAfterABadRead => {
                self.reader.reread_after_a_bad_read(device, offset, length)
            }
        }
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.reader
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reader.reread_after_a_bad_read(device, offset, length)
    }
    fn keep_the_self_verified_reread(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        self.reader
            .keep_the_self_verified_reread(device, offset, bytes);
    }
    fn note_that_the_bytes_read_self_verify(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        match self.pass {
            ReadPassOverThePool::FirstRead => self
                .reader
                .note_that_the_bytes_read_self_verify(device, offset, bytes),
            ReadPassOverThePool::RereadAfterABadRead => self
                .reader
                .keep_the_self_verified_reread(device, offset, bytes),
        }
    }
    /// 两遍一样转过去：重读那一读同样是共用的那一段里的一次读，读坏同样记账（重读不记下读到的字节，丢掉的是第一次读记下的那一份）。
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    ) {
        self.reader.note_that_the_bytes_read_fail_self_verification(
            device,
            offset,
            length_in_bytes,
        );
    }
    /// 两遍一样转过去：只坏一份的读不管是哪一遍读到的都计一次。
    fn note_that_a_unit_was_read_with_one_copy_failing(&self, locations: &[LocationEntry; 2]) {
        self.reader
            .note_that_a_unit_was_read_with_one_copy_failing(locations);
    }
}

impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.iter().map(|(identity, _)| *identity).collect()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
        Some(block_device.size_in_bytes())
    }
    /// 盘自己答（[`BlockDevice::probe_physical_block_size`]）：只调探测，不读盘。
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
        Some(block_device.probe_physical_block_size())
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
        let mut buffer = vec![0u8; length];
        block_device.read_at(offset, &mut buffer).ok()?;
        Some(buffer)
    }
    /// 盘自己答（[`BlockDevice::journal_record_offsets_hint`]）：真盘答 `None`、全环扫描；只记写过的扇区的内存盘答它写过的记录槽。
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
        block_device.journal_record_offsets_hint(ring_start, ring_bytes)
    }
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
        let mut buffer = vec![0u8; length];
        block_device
            .reread_at_after_a_bad_read(offset, &mut buffer)
            .ok()?;
        Some(buffer)
    }
    fn keep_the_self_verified_reread(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        if let Some((_, block_device)) = self.iter().find(|(identity, _)| *identity == device) {
            block_device.keep_the_self_verified_reread(offset, bytes);
        }
    }
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    ) {
        if let Some((_, block_device)) = self.iter().find(|(identity, _)| *identity == device) {
            block_device.note_the_read_failing_self_verification_and_forget_its_bytes(
                offset,
                u64::try_from(length_in_bytes).expect("读长装得进 u64"),
            );
        }
    }
}

/// 文件后端的池就是一串盘：读法与切片那一份相同。
impl<Device: BlockDevice> PoolReader for Vec<(DeviceIdentity, Device)> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.as_slice())
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.as_slice(), device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        PoolReader::physical_block_size_probed_on_the_device(self.as_slice(), device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        PoolReader::read(self.as_slice(), device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        PoolReader::journal_record_offsets_hint(self.as_slice(), device, ring_start, ring_bytes)
    }
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        PoolReader::reread_after_a_bad_read(self.as_slice(), device, offset, length)
    }
    fn keep_the_self_verified_reread(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        PoolReader::keep_the_self_verified_reread(self.as_slice(), device, offset, bytes);
    }
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    ) {
        PoolReader::note_that_the_bytes_read_fail_self_verification(
            self.as_slice(),
            device,
            offset,
            length_in_bytes,
        );
    }
}

/// 这一刻读盘的读者（[`BlockDevice::read_at_straight_from_the_disk`]）：写系统配置槽算世代号、覆写哪一槽那一读用它
/// （`transaction::PoolWriter` 的系统配置槽写）。读的一方报上来的自证结论（[`PoolReader::note_that_the_bytes_read_self_verify`]、
/// [`PoolReader::note_that_the_bytes_read_fail_self_verification`]）不转给盘那一层：读到的字节没进这次挂载记下的那一份。
pub struct PoolReaderStraightFromTheDisk<'devices, Device: BlockDevice> {
    devices: &'devices [(DeviceIdentity, Device)],
}

impl<'devices, Device: BlockDevice> PoolReaderStraightFromTheDisk<'devices, Device> {
    #[must_use]
    pub fn over(devices: &'devices [(DeviceIdentity, Device)]) -> Self {
        PoolReaderStraightFromTheDisk { devices }
    }
}

impl<Device: BlockDevice> PoolReader for PoolReaderStraightFromTheDisk<'_, Device> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.devices)
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.devices, device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        PoolReader::physical_block_size_probed_on_the_device(self.devices, device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let (_, block_device) = self
            .devices
            .iter()
            .find(|(identity, _)| *identity == device)?;
        let mut buffer = vec![0u8; length];
        block_device
            .read_at_straight_from_the_disk(offset, &mut buffer)
            .ok()?;
        Some(buffer)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        PoolReader::journal_record_offsets_hint(self.devices, device, ring_start, ring_bytes)
    }
}

#[cfg(test)]
mod counting_unit_reads_with_one_copy_failing_tests {
    use super::{
        PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
        PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition, RecoveryFailure,
    };
    use crate::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
    use crate::pointer::LocationEntry;

    /// 一块盘都没有的读者：这几条用例只往套子里报「只坏一份」，不读盘。
    struct PoolReaderWithoutDevices;

    impl PoolReader for PoolReaderWithoutDevices {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            Vec::new()
        }
        fn device_size_in_bytes(&self, _device: DeviceIdentity) -> Option<u64> {
            None
        }
        fn physical_block_size_probed_on_the_device(
            &self,
            _device: DeviceIdentity,
        ) -> Option<crate::block_device::PhysicalBlockSizeInBytes> {
            None
        }
        fn read(
            &self,
            _device: DeviceIdentity,
            _offset: DeviceOffsetInBytes,
            _length: usize,
        ) -> Option<Vec<u8>> {
            None
        }
        fn journal_record_offsets_hint(
            &self,
            _device: DeviceIdentity,
            _ring_start: DeviceOffsetInBytes,
            _ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            None
        }
    }

    /// 两块盘同一个槽上的一个单元的两条位置条目。
    fn locations_of_a_unit_at(slot: u64) -> [LocationEntry; 2] {
        [DeviceIdentity(0), DeviceIdentity(1)].map(|device| LocationEntry {
            device,
            slot: SlotNumber(slot),
            unit_checksum: 7,
        })
    }

    /// 按提示读到只坏一份：映射里查不到这把 key、查映射本身判红时判不出提示过期，照转、照计（只有造坏的镜像走得到）；
    /// 查到了而落点不是这一组是提示过期，不转、记下没转。
    #[test]
    fn note_is_dropped_only_when_the_mapping_names_another_position() {
        let key = [1u8; 27];
        let noted_when_the_mapping_answers =
            |answer: Result<Option<[LocationEntry; 2]>, RecoveryFailure>| {
                let counting = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(
                    &PoolReaderWithoutDevices,
                );
                let lookup = |_key: &[u8]| answer.clone();
                let noting = PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition::over(
                    &counting, &key, &lookup,
                );
                noting
                    .note_that_a_unit_was_read_with_one_copy_failing(&locations_of_a_unit_at(100));
                (
                    counting.unit_reads_with_one_copy_failing(),
                    noting.dropped_a_one_copy_failing_note_off_the_mapping_position(),
                )
            };
        assert_eq!(
            noted_when_the_mapping_answers(Ok(None)),
            (1, false),
            "映射里没有这把 key：照计"
        );
        assert_eq!(
            noted_when_the_mapping_answers(Err(RecoveryFailure::NoValidRoot)),
            (1, false),
            "查映射本身判红：照计"
        );
        assert_eq!(
            noted_when_the_mapping_answers(Ok(Some(locations_of_a_unit_at(100)))),
            (1, false),
            "提示就是映射里的落点：照计"
        );
        assert_eq!(
            noted_when_the_mapping_answers(Ok(Some(locations_of_a_unit_at(102)))),
            (0, true),
            "映射里的落点是别处：提示过期，不计"
        );
    }

    /// 同一个单元的同一份坏副本报两次（实例表链经两处读两遍那一形，判决 m4-close-code-r3 第二节 T2 ①）只计一次；另一个单元再报一次计第二次；
    /// 交出的位置条目按第一次报的先后、一个单元一组。
    #[test]
    fn the_same_unit_reported_twice_is_counted_once_and_another_unit_once_more() {
        let counting = PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(
            &PoolReaderWithoutDevices,
        );
        counting.note_that_a_unit_was_read_with_one_copy_failing(&locations_of_a_unit_at(100));
        counting.note_that_a_unit_was_read_with_one_copy_failing(&locations_of_a_unit_at(100));
        assert_eq!(
            counting.unit_reads_with_one_copy_failing(),
            1,
            "同一份坏副本读两遍只计一次"
        );
        counting.note_that_a_unit_was_read_with_one_copy_failing(&locations_of_a_unit_at(102));
        assert_eq!(
            counting.unit_reads_with_one_copy_failing(),
            2,
            "另一个单元另计一次"
        );
        assert_eq!(
            counting.locations_of_the_units_read_with_one_copy_failing(),
            vec![locations_of_a_unit_at(100), locations_of_a_unit_at(102)],
            "一个单元一组、按第一次报的先后"
        );
    }
}
