//! E66：补齐之后小文件怎么落 —— D4 已定项 2 把 E8 的分流判据的输入改了。
//!
//! E8 算内联收益时**没有补齐这一项**（当时 D4 还没定粒度）。现在不内联就占满一个
//! 32 KiB 单元，那一项在 E8 的模型里根本不存在 ⇒ 它的交叉点不能直接拿来用。
//!
//! ## 三条落法
//!
//! | 臂 | 小文件住哪 | 读一次 | 写一次 | 空间 |
//! |---|---|---|---|---|
//! | `extent` | 独立数据 extent，补齐到 32 KiB | 读 1 个 32 KiB 单元 | **纯写** 32 KiB（新文件不必 RMW） | `ceil(size/32K)·32K` |
//! | `inline` | 内联进 16 KiB 索引节点 | 读 1 个 16 KiB 节点 | **RMW** 16 KiB | `size`（挤占节点字节） |
//! | `pack` | 多个小文件共享一个 32 KiB 单元 | 读 1 个 32 KiB 单元 | **RMW** 32 KiB | `size` |
//!
//! ## 失败条款（跑前写死，跑完不许改）
//!
//! 1. **阳性对照，逐臂跑**：`/proc/self/io` 的字节增量与各臂声称的设备字节相符（±2%）。
//! 2. **判别力对照**：`extent` 臂在 512 B 与 16 KiB 两档上**空间占用必须完全相同**
//!    （都占一个单元）。不同 ⇒ 补齐没被建模，整轮作废。
//! 3. **读不到 ≠ 读到 0**：计时为 0、计数取不到，一律整轮作废。
//! 4. N=5 轮，判「通过」要 5 轮全通过。
//! 5. **反过来的结果接不接受**：若 `inline` 在所有档上都不优于 `extent`，
//!    就写「补齐没有改变 E8 的结论」。**不许因为补齐的空间倍数很大就断言内联更好**——
//!    空间不是本工程的判据，要赢得赢在 I/O 或延迟上。

use aes_gcm::{aead::{AeadInPlace, KeyInit}, Aes256Gcm, Nonce};
use e7_index_bench::Emitter;
use std::alloc::{alloc, dealloc, Layout};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::time::Instant;

const O_DIRECT: i32 = 0o40000;
const ALIGN: usize = 4096;
/// D4 已定项 1／已定项 2：数据单元 32 KiB，短 extent 补齐到整单元。
const UNIT: usize = 32768;
/// D8 已定项 2：索引节点 16 KiB。
const NODE: usize = 16384;
/// 节点头，与 E20 同口径。内联的净荷上限 = NODE − NODE_HDR。
const NODE_HDR: usize = 64;
const SIZES: [usize; 8] = [512, 1024, 2048, 4096, 8192, 16384, 32768, 65536];
const SEQ_IO: usize = 1024 * 1024;
/// 丢弃的预热操作数。首批设备操作实测偏高 45–71%，不丢弃就得事后剔一轮，
/// 而「看了数再决定剔哪轮」正是 test-discipline.md 要拦的形态。
const WARMUP: u64 = 64;

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
impl Drop for Aligned { fn drop(&mut self) { unsafe { dealloc(self.ptr, self.layout) } } }

fn next_rand(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x >> 12; x ^= x << 25; x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn proc_io(key: &str) -> Option<u64> {
    let t = std::fs::read_to_string("/proc/self/io").ok()?;
    t.lines().find_map(|l| l.strip_prefix(key).and_then(|v| v.trim().parse().ok()))
}

/// 内联装得下吗：净荷上限是节点减去节点头。
fn inline_fits(size: usize) -> bool { size <= NODE - NODE_HDR }

/// 各臂占多少空间。`extent` 补齐到整单元，另两条紧凑。
fn space_bytes(arm: &str, size: usize) -> u64 {
    match arm {
        "extent" => (size.div_ceil(UNIT) * UNIT) as u64,
        _ => size as u64,
    }
}

/// 一次读要碰多少设备字节。
fn read_bytes_of(arm: &str, size: usize) -> u64 {
    match arm {
        "inline" => NODE as u64,
        _ => (size.div_ceil(UNIT).max(1) * UNIT) as u64,
    }
}

/// 一次写要碰多少设备字节（写侧；`extent` 不必 RMW，另两条要）。
fn write_bytes_of(arm: &str, size: usize) -> u64 { read_bytes_of(arm, size) }
fn needs_rmw(arm: &str) -> bool { arm != "extent" }

/// ⚠️ `verify_ns` 必须单独计：E65 首轮没拆，结果 `full` 臂 25.92% 里有 11 个百分点
/// 是 AES 而不是设备（2026-08-31 三方论证反推腿打中）。同一个坑不踩第二次。
struct Arm { elapsed_ns: u64, verify_ns: u64, ops: u64, sink: u64 }

fn open(path: &str) -> std::fs::File {
    OpenOptions::new().read(true).write(true).custom_flags(O_DIRECT).open(path)
        .unwrap_or_else(|e| { eprintln!("打不开 {path}：{e}"); std::process::exit(3) })
}

fn read_arm(path: &str, chunk: usize, slots: u64, ops: u64, seed: u64) -> Arm {
    let f = open(path);
    let c = Aes256Gcm::new_from_slice(&[0x42u8; 32]).unwrap();
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut buf = Aligned::new(chunk);
    let (mut st, mut sink, mut verify_ns) = (seed | 1, 0u64, 0u64);
    let t0 = Instant::now();
    for _ in 0..ops {
        let s = next_rand(&mut st) % slots;
        f.read_exact_at(buf.as_mut_slice(), s * chunk as u64).expect("读失败");
        let tv = Instant::now();
        sink = sink.wrapping_add(
            c.encrypt_in_place_detached(nonce, b"", buf.as_mut_slice()).expect("验失败")[0] as u64);
        verify_ns += tv.elapsed().as_nanos() as u64;
    }
    Arm { elapsed_ns: t0.elapsed().as_nanos() as u64, verify_ns, ops, sink }
}

fn write_one_arm(path: &str, chunk: usize, slots: u64, ops: u64, seed: u64, rmw: bool) -> Arm {
    let f = open(path);
    let c = Aes256Gcm::new_from_slice(&[0x42u8; 32]).unwrap();
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut buf = Aligned::new(chunk);
    let (mut st, mut sink, mut cursor, mut verify_ns) = (seed | 1, 0u64, 0u64, 0u64);
    let t0 = Instant::now();
    for _ in 0..ops {
        if rmw {
            let s = next_rand(&mut st) % slots;
            f.read_exact_at(buf.as_mut_slice(), s * chunk as u64).expect("RMW 读失败");
            let tv = Instant::now();
            sink = sink.wrapping_add(
                c.encrypt_in_place_detached(nonce, b"", buf.as_mut_slice()).expect("验失败")[0] as u64);
            verify_ns += tv.elapsed().as_nanos() as u64;
        }
        let w = next_rand(&mut st);
        buf.as_mut_slice()[0] = w as u8;
        let tv = Instant::now();
        sink = sink.wrapping_add(
            c.encrypt_in_place_detached(nonce, b"", buf.as_mut_slice()).expect("算 MAC 失败")[0] as u64);
        verify_ns += tv.elapsed().as_nanos() as u64;
        f.write_all_at(buf.as_slice(), cursor * chunk as u64).expect("写失败");
        cursor = (cursor + 1) % slots;
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
    for (i, b) in buf.as_mut_slice().iter_mut().enumerate() { *b = (i as u8).wrapping_mul(31).wrapping_add(7); }
    f.seek(SeekFrom::Start(0)).expect("seek 失败");
    for _ in 0..(region / SEQ_IO as u64) { f.write_all(buf.as_slice()).expect("填充失败"); }
    f.sync_all().expect("sync 失败");
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e66-small-files <块设备或文件> [种子] [ops] [区域 MiB]");
        std::process::exit(2)
    });
    let seed: u64 = std::env::args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(0x6666_1234);
    let ops: u64 = std::env::args().nth(3).and_then(|x| x.parse().ok()).unwrap_or(1024);
    let region_mb: u64 = std::env::args().nth(4).and_then(|x| x.parse().ok()).unwrap_or(8192);
    let mut region = region_mb * 1024 * 1024;

    if std::fs::metadata(&path).map(|m| m.is_file()).unwrap_or(true) { fill(&path, region); }
    let size = open(&path).seek(SeekFrom::End(0)).expect("取不到大小");
    if size == 0 { eprintln!("大小为 0 —— 判定不明，整轮作废"); std::process::exit(5); }
    region = region.min(size);

    // 预热必须在任何一个被计时的臂之前，且它的结果一律丢弃。
    {
        let f = open(&path);
        let mut buf = Aligned::new(UNIT);
        let slots = region / UNIT as u64;
        for i in 0..WARMUP {
            f.read_exact_at(buf.as_mut_slice(), (i % slots) * UNIT as u64).expect("预热读失败");
            f.write_all_at(buf.as_slice(), (i % slots) * UNIT as u64).expect("预热写失败");
        }
    }

    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config dev={path} size={size} region={region} ops={ops} unit={UNIT} node={NODE} \
         node_hdr={NODE_HDR} warmup={WARMUP} seed={seed} sizes={SIZES:?}")));

    for &fs in SIZES.iter() {
        for arm in ["extent", "inline", "pack"] {
            if arm == "inline" && !inline_fits(fs) {
                println!("{}", em.emit_raw(&format!(
                    "name=skip_{arm}_s{fs} file_size={fs} reason=inline_does_not_fit \
                     payload_cap={}", NODE - NODE_HDR)));
                continue;
            }
            if arm == "pack" && fs > UNIT {
                println!("{}", em.emit_raw(&format!(
                    "name=skip_{arm}_s{fs} file_size={fs} reason=file_larger_than_unit unit={UNIT}")));
                continue;
            }
            let rb = read_bytes_of(arm, fs);
            let wb = write_bytes_of(arm, fs);
            let (rslots, wslots) = (region / rb, region / wb);
            let p0 = proc_io("read_bytes:");
            let r = read_arm(&path, rb as usize, rslots, ops, seed);
            let p1 = proc_io("read_bytes:");
            let q0 = proc_io("write_bytes:");
            let w = write_one_arm(&path, wb as usize, wslots, ops, seed, needs_rmw(arm));
            let q1 = proc_io("write_bytes:");
            let ratio = |a: Option<u64>, b: Option<u64>, want: u64| match (a, b) {
                (Some(x), Some(y)) if want > 0 => format!("{:.4}", y.saturating_sub(x) as f64 / want as f64),
                _ => "NA".into(),
            };
            println!("{}", em.emit_raw(&format!(
                "name={arm}_s{fs} arm={arm} file_size={fs} space_bytes={} space_amp={:.4} \
                 read_dev_bytes={rb} write_dev_bytes={wb} rmw={} ops={ops} \
                 read_ns_per_op={:.1} read_verify_ns_per_op={:.1} read_dev_ns_per_op={:.1} \
                 write_ns_per_op={:.1} write_verify_ns_per_op={:.1} write_dev_ns_per_op={:.1} \
                 pr_over_want={} pw_over_want={} sink={}",
                space_bytes(arm, fs),
                space_bytes(arm, fs) as f64 / fs as f64,
                needs_rmw(arm),
                r.elapsed_ns as f64 / r.ops as f64,
                r.verify_ns as f64 / r.ops as f64,
                (r.elapsed_ns - r.verify_ns) as f64 / r.ops as f64,
                w.elapsed_ns as f64 / w.ops as f64,
                w.verify_ns as f64 / w.ops as f64,
                (w.elapsed_ns - w.verify_ns) as f64 / w.ops as f64,
                ratio(p0, p1, ops * rb),
                ratio(q0, q1, ops * wb),
                r.sink.wrapping_add(w.sink))));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言 1：三个格式常量与档位。**
    #[test]
    fn constants_are_pinned() {
        assert_eq!(UNIT, 32768, "D4 已定项 1");
        assert_eq!(NODE, 16384, "D8 已定项 2");
        assert_eq!(NODE_HDR, 64);
        assert_eq!(SIZES, [512, 1024, 2048, 4096, 8192, 16384, 32768, 65536]);
        assert_eq!(WARMUP, 64, "预热要丢弃，否则第一批测量偏高 45–71%");
    }

    /// **绝对值断言 2：补齐让 extent 的空间占用与文件大小无关（判别力对照 2 的算术形式）。**
    #[test]
    fn padding_makes_extent_space_independent_of_file_size() {
        for s in [512, 1024, 2048, 4096, 8192, 16384, 32768] {
            assert_eq!(space_bytes("extent", s), 32768, "size {s}");
        }
        assert_eq!(space_bytes("extent", 32769), 65536);
        assert_eq!(space_bytes("extent", 65536), 65536);
    }

    /// **绝对值断言 3：空间放大逐档。** 512 B 文件占 64 倍，是最坏档。
    #[test]
    fn space_amplification_per_size() {
        assert_eq!(space_bytes("extent", 512) as f64 / 512.0, 64.0);
        assert_eq!(space_bytes("extent", 1024) as f64 / 1024.0, 32.0);
        assert_eq!(space_bytes("extent", 4096) as f64 / 4096.0, 8.0);
        assert_eq!(space_bytes("pack", 512), 512, "打包紧凑，不补齐");
        assert_eq!(space_bytes("inline", 512), 512);
    }

    /// **绝对值断言 4：内联的净荷上限恰是 16320，16 KiB 档落不下。**
    #[test]
    fn inline_capacity_is_node_minus_header() {
        assert_eq!(NODE - NODE_HDR, 16320);
        assert!(inline_fits(16320));
        assert!(!inline_fits(16321));
        assert!(!inline_fits(16384), "16 KiB 档内联落不下");
        assert!(inline_fits(8192));
    }

    /// **读一次碰多少字节：inline 读 16 KiB、另两条读 32 KiB，与文件大小无关。**
    #[test]
    fn read_footprint_is_the_container_not_the_file() {
        for s in [512, 4096, 16384] {
            assert_eq!(read_bytes_of("inline", s), 16384);
            assert_eq!(read_bytes_of("extent", s), 32768);
            assert_eq!(read_bytes_of("pack", s), 32768);
        }
        assert_eq!(read_bytes_of("extent", 65536), 65536, "两个单元");
    }

    /// **只有 extent 不必 RMW**——这是它唯一的写侧优势。
    #[test]
    fn only_extent_skips_the_read_modify_write() {
        assert!(!needs_rmw("extent"));
        assert!(needs_rmw("inline"));
        assert!(needs_rmw("pack"));
    }

    #[test]
    fn prng_is_deterministic() {
        let (mut a, mut b) = (7u64, 7u64);
        let xs: Vec<u64> = (0..8).map(|_| next_rand(&mut a)).collect();
        let ys: Vec<u64> = (0..8).map(|_| next_rand(&mut b)).collect();
        assert_eq!(xs, ys);
        assert_eq!(xs.len(), 8);
        assert!(xs.windows(2).all(|w| w[0] != w[1]));
    }
}
