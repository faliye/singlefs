//! E53：丢一整块盘之后根环还挂不挂得上 —— **本工程第一个真的碰设备的失败域实验**。
//!
//! E46 / E47 / E48 / E50 全是纯算术模型（文件操作 0 处、故障注入 0 处），判的是几何。
//! 本实验在虚机的**真块设备**上写根、**真的抹掉一整块盘**、再走一遍择根路径。
//!
//! ## 它验哪一条
//!
//! E48 的可行点结论：**跨区轮转 + 每区落在不同的失败域上 ⇒ 丢一整块盘后仍读得到有效根，
//! 且最坏回退 1 代**。虚机里 4 块独立裸盘、区域与盘一一对应——
//! 那正是素数步长在条带阵列上要达成的理想放置。**放置那一半仍是算术（E48），本实验不验。**
//!
//! ## 判据（跑前写死）
//!
//! 1. 丢一块盘后 `survivors >= 1`；2. `rollback <= 1`；3. 四块盘逐块各丢一次，四次都要满足。
//!
//! ## 失败条款
//!
//! - 阳性对照：不注入 ⇒ rollback = 0 且 survivors = 8。不是 ⇒ 整轮作废。
//! - 阴性对照：四块全抹 ⇒ survivors = 0。非 0 ⇒ 它在读缓存不是读盘，整轮作废。
//! - rollback > 1 ⇒ **如实记录**，那是模型结论在真设备上不成立。
//!
//! ## 口径
//!
//! 全程 O_DIRECT，绕开来宾页缓存——否则「抹掉之后还读得到」会变成读缓存的假通过。
//! 只报计数，不报时间（本机唯一可读的内核带 lockdep，虚机时间不可与宿主比）。

use e7_index_bench::Emitter;
use std::alloc::{alloc, dealloc, Layout};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;

const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名
const ALIGNMENT_BYTES: usize = 4096;
const SLOT_BYTES: u64 = 4096; // 一个根槽占一个 4 KiB 单元
const REGION_OFFSET_BYTES: u64 = 1 << 20; // 区域在盘上的起点：1 MiB，避开盘头
const SLOTS_PER_REGION: u64 = 2;
const GENERATIONS: u64 = 24; // 8 槽转 3 圈
const MAGIC: u32 = 0x5352_4b52; // "SRKR"

struct AlignedBuffer {
    pointer: *mut u8,
    length_in_bytes: usize,
    allocation_layout: Layout,
}
impl AlignedBuffer {
    fn new(length_in_bytes: usize) -> Self {
        let allocation_layout = Layout::from_size_align(length_in_bytes, ALIGNMENT_BYTES).unwrap();
        let pointer = unsafe { alloc(allocation_layout) };
        unsafe { std::ptr::write_bytes(pointer, 0, length_in_bytes) };
        AlignedBuffer { pointer, length_in_bytes, allocation_layout }
    }
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length_in_bytes) }
    }
    fn as_ref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.pointer, self.length_in_bytes) }
    }
}
impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.pointer, self.allocation_layout) }
    }
}

fn cyclic_redundancy_check_32(data: &[u8]) -> u32 {
    let mut checksum_state = 0xFFFF_FFFFu32;
    for &byte in data {
        checksum_state ^= byte as u32;
        for _ in 0..8 {
            let mask = (checksum_state & 1).wrapping_neg();
            checksum_state = (checksum_state >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !checksum_state
}

/// 一条根记录的字节形态：magic(4) + gen(8) + slot(4) + csum(4)，其余补零。
fn encode_root(generation: u64, slot: u64, buffer: &mut [u8]) {
    for byte in buffer.iter_mut() {
        *byte = 0;
    }
    buffer[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    buffer[4..12].copy_from_slice(&generation.to_le_bytes());
    buffer[12..16].copy_from_slice(&(slot as u32).to_le_bytes());
    let checksum = cyclic_redundancy_check_32(&buffer[0..16]);
    buffer[16..20].copy_from_slice(&checksum.to_le_bytes());
}

/// 解一条根记录。magic 或 CRC 对不上 ⇒ None（**这就是「验得过」的定义**）。
fn decode_root(buffer: &[u8]) -> Option<(u64, u64)> {
    if u32::from_le_bytes(buffer[0..4].try_into().ok()?) != MAGIC {
        return None;
    }
    let generation = u64::from_le_bytes(buffer[4..12].try_into().ok()?);
    let slot = u32::from_le_bytes(buffer[12..16].try_into().ok()?) as u64;
    let expected_checksum = u32::from_le_bytes(buffer[16..20].try_into().ok()?);
    if cyclic_redundancy_check_32(&buffer[0..16]) != expected_checksum {
        return None;
    }
    Some((generation, slot))
}

/// **跨区轮转**（E48 的可行点）：第 t 代落在区域 `t % R` 的槽 `(t / R) % S`。
fn placement_of_generation(generation: u64, region_count: u64, slots_per_region: u64) -> (u64, u64) {
    (generation % region_count, (generation / region_count) % slots_per_region)
}

fn slot_offset(slot_in_region: u64) -> u64 {
    REGION_OFFSET_BYTES + slot_in_region * SLOT_BYTES
}

fn write_slot(device_path: &str, slot_in_region: u64, generation: u64, global_slot: u64) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).custom_flags(O_DIRECT).open(device_path)?;
    let mut buffer = AlignedBuffer::new(SLOT_BYTES as usize);
    encode_root(generation, global_slot, buffer.as_mut());
    file.seek(SeekFrom::Start(slot_offset(slot_in_region)))?;
    file.write_all(buffer.as_ref())?;
    file.sync_all()?;
    Ok(())
}

fn read_slot(device_path: &str, slot_in_region: u64) -> std::io::Result<Option<(u64, u64)>> {
    let mut file = OpenOptions::new().read(true).custom_flags(O_DIRECT).open(device_path)?;
    let mut buffer = AlignedBuffer::new(SLOT_BYTES as usize);
    file.seek(SeekFrom::Start(slot_offset(slot_in_region)))?;
    file.read_exact(buffer.as_mut())?;
    Ok(decode_root(buffer.as_ref()))
}

/// **真注入**：把这块盘承载的整个区域写零。真写，走同一条 O_DIRECT 路径。
fn wipe_region(device_path: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).custom_flags(O_DIRECT).open(device_path)?;
    let zero_buffer = AlignedBuffer::new((SLOT_BYTES * SLOTS_PER_REGION) as usize);
    file.seek(SeekFrom::Start(REGION_OFFSET_BYTES))?;
    file.write_all(zero_buffer.as_ref())?;
    file.sync_all()?;
    Ok(())
}

/// 择根：先逐个验证全部候选，再在有效者中按代号择新（D22 已定的次序）。
fn survey(device_paths: &[String]) -> std::io::Result<(u64, Option<u64>)> {
    let mut survivors = 0u64;
    let mut newest: Option<u64> = None;
    for device_path in device_paths {
        for slot_in_region in 0..SLOTS_PER_REGION {
            if let Some((generation, _)) = read_slot(device_path, slot_in_region)? {
                survivors += 1;
                newest = Some(newest.map_or(generation, |newest_so_far: u64| newest_so_far.max(generation)));
            }
        }
    }
    Ok((survivors, newest))
}

fn lay_down(device_paths: &[String]) -> std::io::Result<()> {
    let region_count = device_paths.len() as u64;
    for generation in 0..GENERATIONS {
        let (region, slot) = placement_of_generation(generation, region_count, SLOTS_PER_REGION);
        write_slot(&device_paths[region as usize], slot, generation, region * SLOTS_PER_REGION + slot)?;
    }
    Ok(())
}

fn main() {
    let device_paths: Vec<String> = std::env::args().skip(1).filter(|argument| argument.starts_with("/dev/")).collect();
    let mut emitter = Emitter::new();
    if device_paths.len() < 2 {
        println!("{}", emitter.emit_raw("name=fatal reason=need_at_least_two_devices"));
        println!("{}", emitter.finish());
        std::process::exit(9);
    }
    let region_count = device_paths.len() as u64;
    let latest_generation = GENERATIONS - 1;
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config devices={} regions={region_count} slots_per_region={SLOTS_PER_REGION} \
             generations={GENERATIONS} slot_bytes={SLOT_BYTES} region_off={REGION_OFFSET_BYTES}",
            device_paths.len()
        ))
    );

    // ── 阳性对照：不注入 ──
    if let Err(error) = lay_down(&device_paths) {
        println!("{}", emitter.emit_raw(&format!("name=fatal reason=write_failed err={error}")));
        println!("{}", emitter.finish());
        std::process::exit(10);
    }
    match survey(&device_paths) {
        Ok((survivor_count, newest)) => {
            let rollback = newest.map(|newest_generation| latest_generation - newest_generation);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=poscontrol_nofault survivors={survivor_count} chosen_gen={} rollback={} \
                     expect_survivors={} expect_rollback=0",
                    newest.map(|value| value.to_string()).unwrap_or_else(|| "NA".into()),
                    rollback.map(|value| value.to_string()).unwrap_or_else(|| "NA".into()),
                    region_count * SLOTS_PER_REGION,
                ))
            );
        }
        Err(error) => {
            println!("{}", emitter.emit_raw(&format!("name=fatal reason=read_failed err={error}")));
            println!("{}", emitter.finish());
            std::process::exit(11);
        }
    }

    // ── 主判据：逐块盘各丢一次 ──
    let mut worst_rollback = 0u64;
    let mut all_mountable = true;
    for wiped_device_index in 0..device_paths.len() {
        if lay_down(&device_paths).is_err() {
            println!("{}", emitter.emit_raw("name=fatal reason=relay_failed"));
            println!("{}", emitter.finish());
            std::process::exit(12);
        }
        if let Err(error) = wipe_region(&device_paths[wiped_device_index]) {
            println!("{}", emitter.emit_raw(&format!("name=fatal reason=wipe_failed err={error}")));
            println!("{}", emitter.finish());
            std::process::exit(13);
        }
        match survey(&device_paths) {
            Ok((survivor_count, newest)) => {
                let rollback = newest.map(|newest_generation| latest_generation - newest_generation);
                if survivor_count == 0 {
                    all_mountable = false;
                }
                if let Some(rollback_generations) = rollback {
                    worst_rollback = worst_rollback.max(rollback_generations);
                }
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=lost_one_device wiped={} survivors={survivor_count} chosen_gen={} \
                         rollback={} mountable={}",
                        device_paths[wiped_device_index],
                        newest.map(|value| value.to_string()).unwrap_or_else(|| "NA".into()),
                        rollback.map(|value| value.to_string()).unwrap_or_else(|| "NA".into()),
                        u8::from(survivor_count > 0),
                    ))
                );
            }
            Err(error) => {
                println!("{}", emitter.emit_raw(&format!("name=fatal reason=survey_failed err={error}")));
                println!("{}", emitter.finish());
                std::process::exit(14);
            }
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=verdict worst_rollback={worst_rollback} all_mountable={} \
             criterion_rollback_le_1={} criterion_mountable={}",
            u8::from(all_mountable),
            u8::from(worst_rollback <= 1),
            u8::from(all_mountable),
        ))
    );

    // ── 阴性对照：全抹 ⇒ 一个都不剩 ──
    if lay_down(&device_paths).is_ok() {
        let mut all_wipes_succeeded = true;
        for device_path in &device_paths {
            if wipe_region(device_path).is_err() {
                all_wipes_succeeded = false;
            }
        }
        if all_wipes_succeeded {
            if let Ok((survivor_count, _)) = survey(&device_paths) {
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=negcontrol_wipe_all survivors={survivor_count} expect=0"
                    ))
                );
            }
        }
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **跨区轮转的绝对值**：4 区 2 槽，第 0..7 代逐个钉死；第 8 代绕回区域 0 槽 0。
    #[test]
    fn placement_rotates_across_regions_first() {
        let expected_placements = [(0, 0), (1, 0), (2, 0), (3, 0), (0, 1), (1, 1), (2, 1), (3, 1)];
        for (generation_index, expected_placement) in expected_placements.iter().enumerate() {
            assert_eq!(placement_of_generation(generation_index as u64, 4, 2), *expected_placement, "gen={generation_index}");
        }
        assert_eq!(placement_of_generation(8, 4, 2), (0, 0));
        // 最新一代 23 落在区域 3 槽 1
        assert_eq!(placement_of_generation(23, 4, 2), (3, 1));
        // 丢掉区域 3 之后，最新的幸存代是 22（区域 2）⇒ 回退恰好 1
        assert_eq!(placement_of_generation(22, 4, 2), (2, 1));
    }

    /// **编解码的往返与判别力**：改一位 CRC 必须判不过。
    #[test]
    fn encode_decode_roundtrip_and_checksum_catches_a_flipped_bit() {
        let mut buffer = vec![0u8; SLOT_BYTES as usize];
        encode_root(23, 7, &mut buffer);
        assert_eq!(decode_root(&buffer), Some((23, 7)));
        buffer[5] ^= 1;
        assert_eq!(decode_root(&buffer), None, "改一位必须判不过");
        // 全零（被抹掉的槽）也必须判不过——magic 就对不上
        let zero_buffer = vec![0u8; SLOT_BYTES as usize];
        assert_eq!(decode_root(&zero_buffer), None);
    }

    /// **magic 那道闸要单独考**：被抹的槽是全零，CRC 就把它拦了 ⇒ 全零那条测试
    /// **考不出 magic**（变异 M3 实测：去掉 magic 检查，一个测试都不红）。
    /// 这里造一条 **magic 错、而 CRC 自洽**的记录——只有 magic 拦得住它。
    #[test]
    fn foreign_block_with_valid_checksum_is_rejected_by_magic() {
        let mut buffer = vec![0u8; SLOT_BYTES as usize];
        // 别人的块：magic 不是 SRKR，但它自己的 CRC 是对的
        buffer[0..4].copy_from_slice(&0xDEAD_BEEFu32.to_le_bytes());
        buffer[4..12].copy_from_slice(&99u64.to_le_bytes());
        buffer[12..16].copy_from_slice(&5u32.to_le_bytes());
        let checksum = cyclic_redundancy_check_32(&buffer[0..16]);
        buffer[16..20].copy_from_slice(&checksum.to_le_bytes());
        // CRC 自洽 —— 先证明这一点，否则下面那条断言是空的
        assert_eq!(cyclic_redundancy_check_32(&buffer[0..16]), checksum);
        assert_eq!(decode_root(&buffer), None, "magic 不对必须判不过");
        // 对照：把 magic 换成对的，同一条记录就该被接受
        buffer[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        let checksum_with_right_magic = cyclic_redundancy_check_32(&buffer[0..16]);
        buffer[16..20].copy_from_slice(&checksum_with_right_magic.to_le_bytes());
        assert_eq!(decode_root(&buffer), Some((99, 5)));
    }

    /// **CRC32 的绝对值**：空串与 "123456789" 的标准值。
    #[test]
    fn cyclic_redundancy_check_32_matches_known_vectors() {
        assert_eq!(cyclic_redundancy_check_32(b""), 0);
        assert_eq!(cyclic_redundancy_check_32(b"123456789"), 0xCBF4_3926);
    }

    /// **槽偏移的绝对值**：区域起点 1 MiB，两个槽相距 4096。
    #[test]
    fn slot_offsets_are_where_we_say() {
        assert_eq!(slot_offset(0), 1_048_576);
        assert_eq!(slot_offset(1), 1_052_672);
        assert_eq!(slot_offset(1) - slot_offset(0), SLOT_BYTES);
    }

    /// **代数与槽数的关系**：24 代在 8 个槽上恰好转 3 圈。
    #[test]
    fn generations_wrap_exactly_three_laps() {
        assert_eq!(GENERATIONS, 24);
        assert_eq!(4 * SLOTS_PER_REGION, 8);
        assert_eq!(GENERATIONS / (4 * SLOTS_PER_REGION), 3);
    }
}
