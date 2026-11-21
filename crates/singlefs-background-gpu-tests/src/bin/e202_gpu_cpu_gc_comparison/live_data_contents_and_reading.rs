//! 活数据单元的内容与读回：每个数据单元的字节由单元下标与第几次写定，读回算内容摘要。

use super::*;

// ---- 活数据单元的内容（登记第五节「负载」「前台那串用户改动」）：每个数据单元的字节由单元下标与第几次写定 ----

/// 造数据挂载写活数据单元是第 0 次，垫片文件的单元是第 1 次（按垫片里的单元序号定内容），前台第 k 次用户改动写的是第 2 + k 次。
pub(super) const WRITE_GENERATION_OF_THE_DATA_MOUNT: u64 = 0;
pub(super) const WRITE_GENERATION_OF_THE_SPACER_FILE: u64 = 1;
pub(super) const WRITE_GENERATION_OF_THE_FIRST_USER_CHANGE: u64 = 2;

fn splitmix64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^ (mixed >> 31)
}

/// 活数据单元第 `unit_index` 个（垫片文件里按它自己的单元序号）第 `write_generation` 次写的内容，填满 `buffer`（一个数据单元的净荷容量那么长）。
pub(super) fn fill_the_content_of_a_data_unit(
    unit_index: u64,
    write_generation: u64,
    buffer: &mut [u8],
) {
    let mut state = FOREGROUND_SEED ^ unit_index.rotate_left(24) ^ write_generation.rotate_left(48);
    for eight_bytes in buffer.chunks_mut(8) {
        let word = splitmix64_next(&mut state).to_le_bytes();
        eight_bytes.copy_from_slice(&word[..eight_bytes.len()]);
    }
}

/// 前台第 `change_number` 次用户改动覆盖写哪一个单元：种子 0xE202 与序号定，在 P ÷ K 个活数据文件里均匀取一个文件，覆盖写它的最后一个单元
/// （K = [`LIVE_DATA_UNITS_PER_FILE`]）。按偏移写从偏移所在的单元重写到文件末尾（`transaction::publish_partial_write` 文档 R4），
/// 取最后一个单元才只重写这一个；取文件里别的单元一次要重写好几个，不是登记第五节「覆盖写那一个单元」。
pub(super) fn unit_index_of_the_user_change(change_number: u64, live_data_units: u64) -> u64 {
    let mut state = FOREGROUND_SEED.wrapping_add(change_number.wrapping_mul(0x1000_0000_0001));
    let drawn = splitmix64_next(&mut state);
    let file =
        u64::try_from((u128::from(drawn) * u128::from(live_data_files(live_data_units))) >> 64)
            .expect("小于活数据文件数");
    file * LIVE_DATA_UNITS_PER_FILE + (LIVE_DATA_UNITS_PER_FILE - 1)
}

/// 活数据单元按每个单元最后一次写算出的期望内容的摘要（按单元下标次序，即 inode 号次序）。
pub(super) fn expected_content_digest(write_generation_of_each_unit: &[u64]) -> String {
    let mut digest = Sha256Digest::new();
    let mut buffer = vec![0u8; data_unit_payload_capacity()];
    for (unit_index, write_generation) in (0_u64..).zip(write_generation_of_each_unit) {
        fill_the_content_of_a_data_unit(unit_index, *write_generation, &mut buffer);
        digest.feed(&buffer);
    }
    digest.finish_as_hex()
}

/// 读盘那一侧（读路径）看两块内存盘：读不到（没有这块盘、越界）交 `None`，不给 journal 偏移提示。
pub(super) struct ReaderOfTheMemoryPool<'devices> {
    pub(super) devices: &'devices [(DeviceIdentity, RecordingMemoryDevice)],
}

impl PoolReader for ReaderOfTheMemoryPool<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.devices
            .iter()
            .map(|(identity, _device)| *identity)
            .collect()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.devices
            .iter()
            .find(|(identity, _device)| *identity == device)
            .map(|(_identity, memory_device)| memory_device.size_in_bytes)
    }

    /// 内存盘自己答（[`BlockDevice::probe_physical_block_size`]）；没有这块盘交 `None`。
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        self.devices
            .iter()
            .find(|(identity, _device)| *identity == device)
            .map(|(_identity, memory_device)| memory_device.probe_physical_block_size())
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let (_identity, memory_device) = self
            .devices
            .iter()
            .find(|(identity, _device)| *identity == device)?;
        let mut bytes = vec![0u8; length];
        memory_device.read_at(offset, &mut bytes).ok()?;
        Some(bytes)
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

/// 按现行那一版读回活数据单元那 P ÷ K 个文件（inode `lowest_live_data_inode` 起，每个 K 个单元），按单元下标次序逐个喂摘要
/// （登记第五节「装置约束」第三行：跑完读每个文件的内容摘要）。哪个文件不是恰好 K 个数据单元、读回来不是 K 个单元的净荷那么长，就交回读不成。
pub(super) fn read_content_digest(
    devices: &PoolDevices,
    version: &TransactionOutput,
    lowest_live_data_inode: u64,
    live_data_units: u64,
) -> Result<String, String> {
    let reader = ReaderOfTheMemoryPool {
        devices: devices.as_slice(),
    };
    let reader: &dyn PoolReader = &reader;
    let opened_pool = open_pool_for_read(reader, &version.root)
        .map_err(|failure| format!("打开池读：{failure:?}"))?;
    let capacity = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
    let file_bytes = capacity * LIVE_DATA_UNITS_PER_FILE;
    let mut digest = Sha256Digest::new();
    for file in 0..live_data_files(live_data_units) {
        let inode = lowest_live_data_inode + file;
        let opened_file = opened_pool
            .open_file(reader, InodeNumber(inode))
            .map_err(|failure| format!("打开 inode {inode}：{failure:?}"))?;
        if opened_file.data_unit_count() != LIVE_DATA_UNITS_PER_FILE {
            return Err(format!(
                "inode {inode} 有 {} 个数据单元，不是 {LIVE_DATA_UNITS_PER_FILE} 个",
                opened_file.data_unit_count()
            ));
        }
        let read = opened_file
            .read_at(reader, FileOffsetInBytes(0), file_bytes)
            .map_err(|failure| format!("读 inode {inode}：{failure:?}"))?;
        if u64::try_from(read.bytes.len()).expect("读回的字节数装得进 u64") != file_bytes {
            return Err(format!(
                "inode {inode} 读回 {} 字节，不是 {file_bytes} 字节",
                read.bytes.len()
            ));
        }
        digest.feed(&read.bytes);
    }
    Ok(digest.finish_as_hex())
}

#[cfg(test)]
mod tests;
