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

const O_DIRECT: i32 = 0o40000;
const ALIGN: usize = 4096;
const SLOT_BYTES: u64 = 4096; // 一个根槽占一个 4 KiB 单元
const REGION_OFF: u64 = 1 << 20; // 区域在盘上的起点：1 MiB，避开盘头
const SLOTS_PER_REGION: u64 = 2;
const GENERATIONS: u64 = 24; // 8 槽转 3 圈
const MAGIC: u32 = 0x5352_4b52; // "SRKR"

struct Aligned {
    ptr: *mut u8,
    len: usize,
    lay: Layout,
}
impl Aligned {
    fn new(len: usize) -> Self {
        let lay = Layout::from_size_align(len, ALIGN).unwrap();
        let ptr = unsafe { alloc(lay) };
        unsafe { std::ptr::write_bytes(ptr, 0, len) };
        Aligned { ptr, len, lay }
    }
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
    fn as_ref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}
impl Drop for Aligned {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.lay) }
    }
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let m = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & m);
        }
    }
    !crc
}

/// 一条根记录的字节形态：magic(4) + gen(8) + slot(4) + csum(4)，其余补零。
fn encode_root(gen: u64, slot: u64, buf: &mut [u8]) {
    for b in buf.iter_mut() {
        *b = 0;
    }
    buf[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    buf[4..12].copy_from_slice(&gen.to_le_bytes());
    buf[12..16].copy_from_slice(&(slot as u32).to_le_bytes());
    let c = crc32(&buf[0..16]);
    buf[16..20].copy_from_slice(&c.to_le_bytes());
}

/// 解一条根记录。magic 或 CRC 对不上 ⇒ None（**这就是「验得过」的定义**）。
fn decode_root(buf: &[u8]) -> Option<(u64, u64)> {
    if u32::from_le_bytes(buf[0..4].try_into().ok()?) != MAGIC {
        return None;
    }
    let gen = u64::from_le_bytes(buf[4..12].try_into().ok()?);
    let slot = u32::from_le_bytes(buf[12..16].try_into().ok()?) as u64;
    let want = u32::from_le_bytes(buf[16..20].try_into().ok()?);
    if crc32(&buf[0..16]) != want {
        return None;
    }
    Some((gen, slot))
}

/// **跨区轮转**（E48 的可行点）：第 t 代落在区域 `t % R` 的槽 `(t / R) % S`。
fn placement(gen: u64, regions: u64, slots: u64) -> (u64, u64) {
    (gen % regions, (gen / regions) % slots)
}

fn slot_offset(slot_in_region: u64) -> u64 {
    REGION_OFF + slot_in_region * SLOT_BYTES
}

fn write_slot(dev: &str, slot_in_region: u64, gen: u64, global_slot: u64) -> std::io::Result<()> {
    let mut f = OpenOptions::new().write(true).custom_flags(O_DIRECT).open(dev)?;
    let mut buf = Aligned::new(SLOT_BYTES as usize);
    encode_root(gen, global_slot, buf.as_mut());
    f.seek(SeekFrom::Start(slot_offset(slot_in_region)))?;
    f.write_all(buf.as_ref())?;
    f.sync_all()?;
    Ok(())
}

fn read_slot(dev: &str, slot_in_region: u64) -> std::io::Result<Option<(u64, u64)>> {
    let mut f = OpenOptions::new().read(true).custom_flags(O_DIRECT).open(dev)?;
    let mut buf = Aligned::new(SLOT_BYTES as usize);
    f.seek(SeekFrom::Start(slot_offset(slot_in_region)))?;
    f.read_exact(buf.as_mut())?;
    Ok(decode_root(buf.as_ref()))
}

/// **真注入**：把这块盘承载的整个区域写零。真写，走同一条 O_DIRECT 路径。
fn wipe_region(dev: &str) -> std::io::Result<()> {
    let mut f = OpenOptions::new().write(true).custom_flags(O_DIRECT).open(dev)?;
    let zero = Aligned::new((SLOT_BYTES * SLOTS_PER_REGION) as usize);
    f.seek(SeekFrom::Start(REGION_OFF))?;
    f.write_all(zero.as_ref())?;
    f.sync_all()?;
    Ok(())
}

/// 择根：先逐个验证全部候选，再在有效者中按代号择新（D22 已定的次序）。
fn survey(devs: &[String]) -> std::io::Result<(u64, Option<u64>)> {
    let mut survivors = 0u64;
    let mut newest: Option<u64> = None;
    for d in devs {
        for s in 0..SLOTS_PER_REGION {
            if let Some((gen, _)) = read_slot(d, s)? {
                survivors += 1;
                newest = Some(newest.map_or(gen, |n: u64| n.max(gen)));
            }
        }
    }
    Ok((survivors, newest))
}

fn lay_down(devs: &[String]) -> std::io::Result<()> {
    let r = devs.len() as u64;
    for gen in 0..GENERATIONS {
        let (region, slot) = placement(gen, r, SLOTS_PER_REGION);
        write_slot(&devs[region as usize], slot, gen, region * SLOTS_PER_REGION + slot)?;
    }
    Ok(())
}

fn main() {
    let devs: Vec<String> = std::env::args().skip(1).filter(|a| a.starts_with("/dev/")).collect();
    let mut em = Emitter::new();
    if devs.len() < 2 {
        println!("{}", em.emit_raw("name=fatal reason=need_at_least_two_devices"));
        println!("{}", em.finish());
        std::process::exit(9);
    }
    let r = devs.len() as u64;
    let latest = GENERATIONS - 1;
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config devices={} regions={r} slots_per_region={SLOTS_PER_REGION} \
             generations={GENERATIONS} slot_bytes={SLOT_BYTES} region_off={REGION_OFF}",
            devs.len()
        ))
    );

    // ── 阳性对照：不注入 ──
    if let Err(e) = lay_down(&devs) {
        println!("{}", em.emit_raw(&format!("name=fatal reason=write_failed err={e}")));
        println!("{}", em.finish());
        std::process::exit(10);
    }
    match survey(&devs) {
        Ok((surv, newest)) => {
            let rb = newest.map(|n| latest - n);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=poscontrol_nofault survivors={surv} chosen_gen={} rollback={} \
                     expect_survivors={} expect_rollback=0",
                    newest.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                    rb.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                    r * SLOTS_PER_REGION,
                ))
            );
        }
        Err(e) => {
            println!("{}", em.emit_raw(&format!("name=fatal reason=read_failed err={e}")));
            println!("{}", em.finish());
            std::process::exit(11);
        }
    }

    // ── 主判据：逐块盘各丢一次 ──
    let mut worst_rollback = 0u64;
    let mut all_mountable = true;
    for k in 0..devs.len() {
        if lay_down(&devs).is_err() {
            println!("{}", em.emit_raw("name=fatal reason=relay_failed"));
            println!("{}", em.finish());
            std::process::exit(12);
        }
        if let Err(e) = wipe_region(&devs[k]) {
            println!("{}", em.emit_raw(&format!("name=fatal reason=wipe_failed err={e}")));
            println!("{}", em.finish());
            std::process::exit(13);
        }
        match survey(&devs) {
            Ok((surv, newest)) => {
                let rb = newest.map(|n| latest - n);
                if surv == 0 {
                    all_mountable = false;
                }
                if let Some(v) = rb {
                    worst_rollback = worst_rollback.max(v);
                }
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=lost_one_device wiped={} survivors={surv} chosen_gen={} \
                         rollback={} mountable={}",
                        devs[k],
                        newest.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                        rb.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                        u8::from(surv > 0),
                    ))
                );
            }
            Err(e) => {
                println!("{}", em.emit_raw(&format!("name=fatal reason=survey_failed err={e}")));
                println!("{}", em.finish());
                std::process::exit(14);
            }
        }
    }
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=verdict worst_rollback={worst_rollback} all_mountable={} \
             criterion_rollback_le_1={} criterion_mountable={}",
            u8::from(all_mountable),
            u8::from(worst_rollback <= 1),
            u8::from(all_mountable),
        ))
    );

    // ── 阴性对照：全抹 ⇒ 一个都不剩 ──
    if lay_down(&devs).is_ok() {
        let mut ok = true;
        for d in &devs {
            if wipe_region(d).is_err() {
                ok = false;
            }
        }
        if ok {
            if let Ok((surv, _)) = survey(&devs) {
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=negcontrol_wipe_all survivors={surv} expect=0"
                    ))
                );
            }
        }
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **跨区轮转的绝对值**：4 区 2 槽，第 0..7 代逐个钉死；第 8 代绕回区域 0 槽 0。
    #[test]
    fn placement_rotates_across_regions_first() {
        let want = [(0, 0), (1, 0), (2, 0), (3, 0), (0, 1), (1, 1), (2, 1), (3, 1)];
        for (g, w) in want.iter().enumerate() {
            assert_eq!(placement(g as u64, 4, 2), *w, "gen={g}");
        }
        assert_eq!(placement(8, 4, 2), (0, 0));
        // 最新一代 23 落在区域 3 槽 1
        assert_eq!(placement(23, 4, 2), (3, 1));
        // 丢掉区域 3 之后，最新的幸存代是 22（区域 2）⇒ 回退恰好 1
        assert_eq!(placement(22, 4, 2), (2, 1));
    }

    /// **编解码的往返与判别力**：改一位 CRC 必须判不过。
    #[test]
    fn encode_decode_roundtrip_and_crc_catches_a_flipped_bit() {
        let mut buf = vec![0u8; SLOT_BYTES as usize];
        encode_root(23, 7, &mut buf);
        assert_eq!(decode_root(&buf), Some((23, 7)));
        buf[5] ^= 1;
        assert_eq!(decode_root(&buf), None, "改一位必须判不过");
        // 全零（被抹掉的槽）也必须判不过——magic 就对不上
        let zero = vec![0u8; SLOT_BYTES as usize];
        assert_eq!(decode_root(&zero), None);
    }

    /// **magic 那道闸要单独考**：被抹的槽是全零，CRC 就把它拦了 ⇒ 全零那条测试
    /// **考不出 magic**（变异 M3 实测：去掉 magic 检查，一个测试都不红）。
    /// 这里造一条 **magic 错、而 CRC 自洽**的记录——只有 magic 拦得住它。
    #[test]
    fn a_foreign_block_with_a_valid_checksum_is_rejected_by_magic() {
        let mut buf = vec![0u8; SLOT_BYTES as usize];
        // 别人的块：magic 不是 SRKR，但它自己的 CRC 是对的
        buf[0..4].copy_from_slice(&0xDEAD_BEEFu32.to_le_bytes());
        buf[4..12].copy_from_slice(&99u64.to_le_bytes());
        buf[12..16].copy_from_slice(&5u32.to_le_bytes());
        let c = crc32(&buf[0..16]);
        buf[16..20].copy_from_slice(&c.to_le_bytes());
        // CRC 自洽 —— 先证明这一点，否则下面那条断言是空的
        assert_eq!(crc32(&buf[0..16]), c);
        assert_eq!(decode_root(&buf), None, "magic 不对必须判不过");
        // 对照：把 magic 换成对的，同一条记录就该被接受
        buf[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        let c2 = crc32(&buf[0..16]);
        buf[16..20].copy_from_slice(&c2.to_le_bytes());
        assert_eq!(decode_root(&buf), Some((99, 5)));
    }

    /// **CRC32 的绝对值**：空串与 "123456789" 的标准值。
    #[test]
    fn crc32_matches_known_vectors() {
        assert_eq!(crc32(b""), 0);
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
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
