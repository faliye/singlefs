//! 记账三项的读数、镜像字节里的记账条目，与只改盘 0 那一份记账节点的造坏。

use super::*;

/// 一块盘的记账第 1/2/5 项，槽数：直接读真实 `PoolAllocator`（K1-1 与单测用）。
pub(super) fn accounting_row_slots(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
) -> (u64, u64, u64) {
    let map = allocator
        .devices
        .iter()
        .find(|map| map.device == device)
        .expect("两块盘都在");
    (
        map.allocated_slots(),
        map.free_slots(),
        map.deferred_slots(),
    )
}

/// G27 的「最新根走读引用」：`current.units` 是这一版全部角色的落点，实例表单元在它第一次被重写之前不进这张表
/// （`transaction.rs` 文档字面），这里据此补上它——与字节级走读（`walk::check_pool_image`）是两条独立路径。
pub(super) fn referenced_slots(current: &TransactionOutput) -> u64 {
    let has_instance_table = current
        .units
        .iter()
        .any(|unit| matches!(unit.identity, TransactionUnit::InstanceTable));
    let mut total: u64 = current
        .units
        .iter()
        .map(|unit| unit.identity.span_slots())
        .sum();
    if !has_instance_table {
        total += INSTANCE_TABLE_SPAN_SLOTS;
    }
    total
}

/// 内存读法（R3 的反面）：只给 `crates/mutations.tsv` 的 M21 当反面用（U8 的调用点被换成它），
/// 正式判定路径谁都不叫它。
#[cfg(test)]
fn allocated_minus_deferred_matches_referenced(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
    referenced: u64,
) -> bool {
    let (allocated, _free, deferred) = accounting_row_slots(allocator, device);
    allocated == deferred + referenced
}

pub(super) fn mp_read(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(
        pool,
        DeviceIdentity(device),
        DeviceOffsetInBytes(offset),
        length,
    )
    .expect("读")
}
pub(super) fn mp_write(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    pool.devices
        .get_mut(&DeviceIdentity(device))
        .expect("盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

/// 记账树叶里 (统计量, 设备 0) 那一行的值，**单位字节**：直接从**镜像字节**解析（与 `adjust_accounting_entry`
/// 同一份布局：条目区从 115 + 2 × 22 = 159 起，每条 34：统计量 2、树 8、设备 4、代 8、值 8、seq 4），只读不改。
fn read_accounting_entry_bytes_from_mirror(
    pool: &MemoryPool,
    accounting_slot: u64,
    statistic: u16,
) -> u64 {
    let bytes = mp_read(pool, 0, accounting_slot * SLOT_BYTES, 16384);
    let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
    for index in 0..count {
        let row = 159 + 34 * index;
        if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
            && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == 0
        {
            return u64::from_le_bytes(bytes[row + 22..row + 30].try_into().expect("8"));
        }
    }
    panic!("记账树里没有 (统计量 {statistic}, 设备 0) 这一行");
}

/// R3：G27 一律读镜像上的记账行（第 1/2/5 项），单位槽。记账单元是镜像对（w = 2），
/// `corrupt_device_zero_accounting` 只改盘 0 那一份，这里也只读盘 0，两处同一个读法。
pub(super) fn mirror_accounting_row_slots(
    pool: &MemoryPool,
    accounting_slot: u64,
) -> (u64, u64, u64) {
    (
        read_accounting_entry_bytes_from_mirror(pool, accounting_slot, 1) / SLOT_BYTES,
        read_accounting_entry_bytes_from_mirror(pool, accounting_slot, 2) / SLOT_BYTES,
        read_accounting_entry_bytes_from_mirror(pool, accounting_slot, 5) / SLOT_BYTES,
    )
}

/// G27 正式判定（R3）：逐盘（只判盘 0，两盘镜像同值）`第 1 项 − 第 5 项 == 最新根走读引用`，一律读镜像字节。
/// 第 4 次重跑的 `main` 不量岔路 7，只剩单测 U8 在用。
#[cfg(test)]
fn allocated_minus_deferred_mismatches_referenced(
    pool: &MemoryPool,
    accounting_slot: u64,
    referenced: u64,
) -> bool {
    let (allocated, _free, deferred) = mirror_accounting_row_slots(pool, accounting_slot);
    allocated != deferred + referenced
}

/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
/// 与 `crates/singlefs-verification/tests/checker_known_bad_images.rs` 的 `reseal_unit` 同一份布局（两处各自独立实现，字段表出自
/// `singlefs-format`，不是互相抄）。
fn reseal_unit(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[6] {
        1 => (105usize, 101usize),
        3 => (107, 89),
        _ => (
            86 + 2 * usize::from(bytes[51]),
            76 + 2 * usize::from(bytes[51]),
        ),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

/// 记账树叶里 (统计量, 设备) 那一行的值加 `delta_bytes`（与 `checker_known_bad_images.rs` 的 `adjust_accounting` 同一份布局：
/// 条目区从 115 + 2 × 22 = 159 起，每条 34：统计量 2、树 8、设备 4、代 8、值 8、seq 4）。
fn adjust_accounting_entry(bytes: &mut [u8], statistic: u16, device: u32, delta_bytes: i64) {
    let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
    for index in 0..count {
        let row = 159 + 34 * index;
        if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
            && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
        {
            let value = u64::from_le_bytes(bytes[row + 22..row + 30].try_into().expect("8"));
            let new_value = value.wrapping_add_signed(delta_bytes);
            bytes[row + 22..row + 30].copy_from_slice(&new_value.to_le_bytes());
            return;
        }
    }
    panic!("记账树里没有 ({statistic}, {device}) 这一行");
}

fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    [
        device.to_le_bytes().to_vec(),
        slot.to_le_bytes()[..6].to_vec(),
        checksum.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
    let mut changed = false;
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        if haystack[index..index + needle.len()] == *needle {
            haystack[index..index + needle.len()].copy_from_slice(replacement);
            changed = true;
            index += needle.len();
        } else {
            index += 1;
        }
    }
    changed
}

/// 根环全部槽的 (设备, 偏移)：按公开的 `root_ring` 几何现算，不写死字节偏移（β0 与 β_syn 那一段的 S = 8 池）。
fn root_ring_slots(region_devices: &[DeviceIdentity; 3], spacing: u32) -> Vec<(u32, u64)> {
    (0..E156_ROOT_RING_REGIONS)
        .flat_map(|region| {
            let device = region_devices[usize::try_from(region).expect("区域号落在 3 以内")].0;
            (0..E156_SLOTS_PER_REGION).map(move |slot| {
                (
                    device,
                    slot_offset(RootRingSlot { region, slot }, spacing).0,
                )
            })
        })
        .collect()
}

/// 只改盘 0 那一份记账节点：重封盘 0 那一份的两道校验和，把新的整单元校验和补进盘 0 自己那份树表与根槽——
/// 盘 1 一个字节都不碰（`checker_known_bad_images.rs` 的 `mutate_one_device_copy_of_unit` 同一条改法，这里独立实现）。
/// `entries` 是这次要同时改的 (统计量, delta 字节) 列表。
pub(super) fn corrupt_device_zero_accounting(
    base: &MemoryPool,
    accounting_slot: u64,
    tree_table_slot: u64,
    region_devices: &[DeviceIdentity; 3],
    spacing: u32,
    entries: &[(u16, i64)],
) -> MemoryPool {
    let mut pool = base.clone();
    let device = 0u32;
    let before = mp_read(&pool, device, accounting_slot * SLOT_BYTES, 16384);
    let mut bytes = before.clone();
    for (statistic, delta_bytes) in entries.iter().copied() {
        adjust_accounting_entry(&mut bytes, statistic, device, delta_bytes);
    }
    reseal_unit(&mut bytes);
    mp_write(&mut pool, device, accounting_slot * SLOT_BYTES, &bytes);
    let (old_checksum, new_checksum) = (crc32_castagnoli(&before), crc32_castagnoli(&bytes));

    // 树表：只改盘 0 那一份，重封，再把新校验和补进盘 0 与盘 1 各自的树表条目里指着「盘 0 那份记账」的那一条位置条目
    // （两块盘的树表内容原本相同，位置条目本身按设备分两条，各自独立记着盘 0 与盘 1 各一份的校验和）。
    let mut tree_table_changed_on = Vec::new();
    for tree_table_device in [0u32, 1u32] {
        let tree_table_before = mp_read(
            &pool,
            tree_table_device,
            tree_table_slot * SLOT_BYTES,
            16384,
        );
        let mut tree_table_bytes = tree_table_before.clone();
        if replace_all(
            &mut tree_table_bytes,
            &location_pattern(device, accounting_slot, old_checksum),
            &location_pattern(device, accounting_slot, new_checksum),
        ) {
            reseal_unit(&mut tree_table_bytes);
            mp_write(
                &mut pool,
                tree_table_device,
                tree_table_slot * SLOT_BYTES,
                &tree_table_bytes,
            );
            tree_table_changed_on.push((
                tree_table_device,
                crc32_castagnoli(&tree_table_before),
                crc32_castagnoli(&tree_table_bytes),
            ));
        }
    }
    for (root_device, offset) in root_ring_slots(region_devices, spacing) {
        let mut root_bytes = mp_read(&pool, root_device, offset, 512);
        let mut touched = false;
        for (tree_table_device, old_tt, new_tt) in tree_table_changed_on.iter().copied() {
            touched |= replace_all(
                &mut root_bytes,
                &location_pattern(tree_table_device, tree_table_slot, old_tt),
                &location_pattern(tree_table_device, tree_table_slot, new_tt),
            );
        }
        if touched {
            let digest = wide_checksum_with_field_zeroed(&root_bytes, 512, 138);
            root_bytes[138..170].copy_from_slice(&digest);
            mp_write(&mut pool, root_device, offset, &root_bytes);
        }
    }
    pool
}

#[cfg(test)]
mod tests;
