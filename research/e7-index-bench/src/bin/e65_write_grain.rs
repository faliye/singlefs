//! E65：校验和粒度的写侧代价 —— D4 已定项 2（补齐）与已定项 3（读—改—写）付多少。
//!
//! E58 三条臂全是读臂，写侧零覆盖。D4 已定项 3 定了「凑不满一个单元的写走读—改—写」，
//! 而 RMW 的代价一个数都没有。本实验补写侧。
//!
//! ## 臂
//!
//! | 臂 | 一次「写 4 KiB 用户数据」做什么 | G 影响什么 |
//! |---|---|---|
//! | `rmw` | 读回旧单元（整 G）+ 验 MAC → 改 4 KiB → 重算 MAC → 整单元写到新位置 | 读放大与写放大同时随 G 涨 |
//! | `full` | 整单元是新的（追加 / 新建），**不读** | 只有写放大 |
//! | `rmwsync` | 同 `rmw`，每次 `fdatasync` | 加一次持久点，看固定开销把倍数摊平多少 |
//! | `pad` | 纯算术：补齐到 G 的空间放大 | D4 已定项 2 的代价 |
//!
//! **写位置按 COW 前推**（游标每次加一个单元，到区域末尾回绕），不原地覆盖——
//! 原地覆盖不是本工程的写路径，量它等于量另一个文件系统。
//!
//! ## 跑前写死的解析预测
//!
//! 字节口径是恒等式，不是预测：写放大 = G/4096，`rmw` 还要另读 G 字节。
//! **时间口径没有预测**——NVMe 的写被 SLC 吸收，8 倍字节不等于 8 倍时间，那正是要量的。
//!
//! ## 失败条款（跑前写死，跑完不许改）
//!
//! 1. **阳性对照，逐臂跑**：内核记的 `/proc/self/io` `write_bytes` 增量 = `ops × G`（±2%）。
//! 2. **判别力对照**：`full` 臂的 `read_bytes` 必须**恒为 0**，而 `rmw` 臂必须 = `ops × G`。
//!    两者分不开 ⇒ 这套度量看不见 RMW 那次读，整轮作废。
//! 3. **读不到 ≠ 读到 0**：计时为 0、计数取不到，一律整轮作废。
//! 4. N=5 轮，判「通过」要 5 轮全通过。
//! 5. **反过来的结果接不接受**（跑前写下）：若 32 KiB 的 `rmw` 单次耗时相对 16 KiB
//!    **涨不到 5%**，就写「写侧分不出 16 与 32」，不许因为字节倍数是 2 倍就说它贵一倍。
//!
//! ## 它答不了什么
//!
//! 不含事务与 checkpoint 摊销（真实写路径会把多个改动并进一次发布）；不含分配器；
//! 不含并发；`pad` 那一臂是算术不是实测；换设备要重跑。

use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Nonce,
};
use e7_index_bench::Emitter;
use std::alloc::{alloc, dealloc, Layout};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::time::Instant;

const O_DIRECT: i32 = 0o40000;
const ALIGN: usize = 4096;
/// 用户一次写多少。与 E58 的读侧同口径，两个实验的倍数才可比。
const USER_WRITE: u64 = 4096;
const GRAINS: [usize; 6] = [4096, 8192, 16384, 32768, 65536, 131072];
/// D4 已定项 1 定的数据单元。`pad` 那一臂拿它当默认档。
const DATA_UNIT_BYTES: usize = 32768;
const SEQ_IO: usize = 1024 * 1024;

struct Aligned { ptr: *mut u8, len: usize, layout: Layout }
impl Aligned {
    fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len, ALIGN).expect("对齐参数非法");
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null(), "分配失败");
        unsafe { std::ptr::write_bytes(ptr, 0xA5, len) };
        Self { ptr, len, layout }
    }
    fn as_slice(&self) -> &[u8] { unsafe { std::slice::from_raw_parts(self.ptr, self.len) } }
    fn as_mut_slice(&mut self) -> &mut [u8] { unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) } }
}
impl Drop for Aligned {
    fn drop(&mut self) { unsafe { dealloc(self.ptr, self.layout) } }
}

fn next_rand(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x >> 12; x ^= x << 25; x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn proc_io(key: &str) -> Option<u64> {
    let t = std::fs::read_to_string("/proc/self/io").ok()?;
    for line in t.lines() {
        if let Some(v) = line.strip_prefix(key) {
            return v.trim().parse().ok();
        }
    }
    None
}

/// 写放大：写 4 KiB 用户数据实际要写的倍数。补齐之后它是恒等式。
fn write_amp(g: usize) -> f64 { g as f64 / USER_WRITE as f64 }

/// D4 已定项 2 的空间放大：一个 `size` 字节的 extent 补齐到 `g` 之后占多少倍。
fn pad_amp(size: u64, g: usize) -> f64 {
    let used = size.div_ceil(g as u64) * g as u64;
    used as f64 / size.max(1) as f64
}

struct Arm { elapsed_ns: u64, verify_ns: u64, ops: u64, sink: u64 }

fn open(path: &str) -> std::fs::File {
    OpenOptions::new().read(true).write(true).custom_flags(O_DIRECT).open(path)
        .unwrap_or_else(|e| { eprintln!("打不开 {path}：{e}"); std::process::exit(3) })
}

/// `rmw` / `full` / `rmwsync` 三条臂共用一段：`read_old` 决定读不读、`sync` 决定同不同步。
fn write_arm(path: &str, g: usize, units: u64, ops: u64, seed: u64, read_old: bool, sync: bool) -> Arm {
    let f = open(path);
    let c = Aes256Gcm::new_from_slice(&[0x42u8; 32]).unwrap();
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut buf = Aligned::new(g);
    let mut st = seed | 1;
    let (mut sink, mut verify_ns) = (0u64, 0u64);
    // COW 游标：新单元往前推，不原地覆盖。
    let mut cursor = 0u64;
    let t0 = Instant::now();
    for _ in 0..ops {
        if read_old {
            let unit = next_rand(&mut st) % units;
            f.read_exact_at(&mut buf.as_mut_slice()[..g], unit * g as u64).expect("RMW 读失败");
            let tv = Instant::now();
            // 读回来的旧单元先验一遍：D4 的 Merkle 要求改之前先确认它没坏
            sink = sink.wrapping_add(
                c.encrypt_in_place_detached(nonce, b"", &mut buf.as_mut_slice()[..g])
                    .expect("验旧单元失败")[0] as u64);
            verify_ns += tv.elapsed().as_nanos() as u64;
        }
        // 改掉其中 4 KiB
        let w = next_rand(&mut st);
        for (i, b) in buf.as_mut_slice()[..USER_WRITE as usize].iter_mut().enumerate() {
            *b = (w >> (i % 8 * 8)) as u8;
        }
        let tv = Instant::now();
        sink = sink.wrapping_add(
            c.encrypt_in_place_detached(nonce, b"", &mut buf.as_mut_slice()[..g])
                .expect("算新 MAC 失败")[0] as u64);
        verify_ns += tv.elapsed().as_nanos() as u64;
        f.write_all_at(buf.as_slice(), cursor * g as u64).expect("写失败");
        if sync { f.sync_data().expect("fdatasync 失败"); }
        cursor = (cursor + 1) % units;
    }
    Arm { elapsed_ns: t0.elapsed().as_nanos() as u64, verify_ns, ops, sink }
}

fn fill(path: &str, region: u64) {
    let mut f = OpenOptions::new().read(true).write(true).create(true).truncate(false)
        .custom_flags(O_DIRECT).open(path)
        .unwrap_or_else(|e| { eprintln!("建不了测试区 {path}：{e}"); std::process::exit(3) });
    if f.seek(SeekFrom::End(0)).expect("取不到大小") >= region { return; }
    eprintln!("填充测试区 {} MiB …", region / (1024 * 1024));
    let mut buf = Aligned::new(SEQ_IO);
    for (i, b) in buf.as_mut_slice().iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(31).wrapping_add(7);
    }
    f.seek(SeekFrom::Start(0)).expect("seek 失败");
    for _ in 0..(region / SEQ_IO as u64) { f.write_all(buf.as_slice()).expect("填充失败"); }
    f.sync_all().expect("sync 失败");
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e65-write-grain <块设备或文件> [种子] [ops] [区域 MiB]");
        std::process::exit(2)
    });
    let seed: u64 = std::env::args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(0x6161_1234);
    let ops: u64 = std::env::args().nth(3).and_then(|x| x.parse().ok()).unwrap_or(2048);
    let region_mb: u64 = std::env::args().nth(4).and_then(|x| x.parse().ok()).unwrap_or(8192);
    let mut region = region_mb * 1024 * 1024;

    if std::fs::metadata(&path).map(|m| m.is_file()).unwrap_or(true) { fill(&path, region); }
    let size = open(&path).seek(SeekFrom::End(0)).expect("取不到大小");
    if size == 0 { eprintln!("大小为 0 —— 判定不明，整轮作废"); std::process::exit(5); }
    region = region.min(size);

    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config dev={path} size={size} region={region} ops={ops} user_write={USER_WRITE} \
         data_unit={DATA_UNIT_BYTES} seed={seed} grains={GRAINS:?}")));

    for &g in GRAINS.iter() {
        let units = region / g as u64;
        for (arm, read_old, sync) in [("rmw", true, false), ("full", false, false), ("rmwsync", true, true)] {
            let (r0, w0) = (proc_io("read_bytes:"), proc_io("write_bytes:"));
            let a = write_arm(&path, g, units, ops, seed, read_old, sync);
            let (r1, w1) = (proc_io("read_bytes:"), proc_io("write_bytes:"));
            let d = |x: Option<u64>, y: Option<u64>| match (x, y) {
                (Some(x), Some(y)) => format!("{}", y.saturating_sub(x)),
                _ => "NA".into(),
            };
            let want = a.ops * g as u64;
            let wr = match (w0, w1) {
                (Some(x), Some(y)) => format!("{:.4}", y.saturating_sub(x) as f64 / want as f64),
                _ => "NA".into(),
            };
            println!("{}", em.emit_raw(&format!(
                "name={arm}_g{g} grain={g} ops={} user_bytes={} dev_write_bytes={want} \
                 elapsed_ns={} verify_ns={} ns_per_op={:.1} write_amp={:.4} \
                 proc_read_bytes={} proc_write_bytes={} pw_over_want={wr} sink={}",
                a.ops, a.ops * USER_WRITE, a.elapsed_ns, a.verify_ns,
                a.elapsed_ns as f64 / a.ops.max(1) as f64, write_amp(g),
                d(r0, r1), d(w0, w1), a.sink)));
        }
        println!("{}", em.emit_raw(&format!(
            "name=pad_g{g} grain={g} write_amp={:.4} pad_amp_1k={:.4} pad_amp_4k={:.4} \
             pad_amp_g_minus_one={:.6}",
            write_amp(g), pad_amp(1024, g), pad_amp(4096, g), pad_amp(g as u64 - 1, g))));
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言 1：常量与网格。**
    #[test]
    fn constants_are_pinned() {
        assert_eq!(GRAINS, [4096, 8192, 16384, 32768, 65536, 131072]);
        assert_eq!(USER_WRITE, 4096);
        assert_eq!(DATA_UNIT_BYTES, 32768, "D4 已定项 1 定的数据单元");
        assert_eq!(SEQ_IO, 1048576);
    }

    /// **绝对值断言 2：写放大恰为 G/4096，逐档钉死。**
    #[test]
    fn write_amplification_is_exactly_grain_over_four_k() {
        assert_eq!(write_amp(4096), 1.0);
        assert_eq!(write_amp(16384), 4.0);
        assert_eq!(write_amp(32768), 8.0);
        assert_eq!(write_amp(131072), 32.0);
    }

    /// **绝对值断言 3：D4 已定项 2 的空间放大。**
    /// 32 KiB 单元下：1 KiB 文件占 32 倍、4 KiB 占 8 倍；恰好 1 字节不足时是最坏点。
    #[test]
    fn padding_amplification_at_the_settled_unit() {
        assert_eq!(pad_amp(1024, 32768), 32.0);
        assert_eq!(pad_amp(4096, 32768), 8.0);
        assert_eq!(pad_amp(32768, 32768), 1.0);
        // 32769 字节要占 2 个单元 = 65536，倍数 65536/32769 = 1.99994（不是恰好 2）
        assert!((pad_amp(32769, 32768) - 1.9999390).abs() < 1e-6, "{}", pad_amp(32769, 32768));
        // 最坏浪费：单元大小减一
        let worst = pad_amp(32767, 32768);
        assert!((worst - 1.0000305).abs() < 1e-5, "{worst}");
    }

    /// **16 KiB 与 32 KiB 在小文件上的差，绝对值。**
    /// 一个 1 KiB 文件：16 KiB 单元占 16 倍、32 KiB 占 32 倍 —— 差恰好一倍。
    #[test]
    fn thirty_two_doubles_the_small_file_waste_versus_sixteen() {
        assert_eq!(pad_amp(1024, 16384), 16.0);
        assert_eq!(pad_amp(1024, 32768), 32.0);
        assert_eq!(pad_amp(1024, 32768) / pad_amp(1024, 16384), 2.0);
    }

    #[test]
    fn prng_is_deterministic() {
        let (mut a, mut b) = (999u64, 999u64);
        let xs: Vec<u64> = (0..8).map(|_| next_rand(&mut a)).collect();
        let ys: Vec<u64> = (0..8).map(|_| next_rand(&mut b)).collect();
        assert_eq!(xs, ys);
        assert_eq!(xs.len(), 8);
        assert!(xs.windows(2).all(|w| w[0] != w[1]));
    }
}
