//! E7 的最小端到端验证：在虚机里对一块真实的块设备做 O_DIRECT 读写。
//!
//! 现在**只验证管道**——静态二进制能在 busybox initramfs 里跑、能拿到 /dev/vda、
//! 结果行能被宿主抓到。索引候选的实现还没有，见 kb/experiments.md E7。

use e7_index_bench::{Emitter, Sample};
use std::alloc::{alloc, dealloc, Layout};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::time::Instant;

/// x86_64 Linux 的 O_DIRECT。写成字面量是为了不引入 libc 依赖；
/// 换架构要重新核对（powerpc 与 alpha 上不是这个值）。
const O_DIRECT: i32 = 0o40000;

/// O_DIRECT 要求缓冲区、偏移、长度都按块对齐。取 4096 覆盖常见的 512/4096 两种。
const ALIGN: usize = 4096;

/// 对齐缓冲区。O_DIRECT 下用普通 Vec 会被内核以 EINVAL 拒掉。
struct Aligned {
    ptr: *mut u8,
    len: usize,
    layout: Layout,
}

impl Aligned {
    fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len, ALIGN).expect("对齐参数非法");
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null(), "分配失败");
        unsafe { std::ptr::write_bytes(ptr, 0xA5, len) };
        Self { ptr, len, layout }
    }
    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for Aligned {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.layout) }
    }
}

/// 确定性伪随机：同一个种子必须给出同一串偏移，否则实验不可复现
/// （rules/test-discipline.md：测试可复现是任何判断的前提）。
fn next_rand(state: &mut u64) -> u64 {
    // xorshift64*，够用且无依赖
    let mut x = *state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn main() {
    let dev = match std::env::args().nth(1) {
        Some(d) => d,
        None => {
            eprintln!("用法：e7-index-bench <块设备>");
            std::process::exit(2);
        }
    };

    let mut f = match OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(O_DIRECT)
        .open(&dev)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("打不开 {dev}（O_DIRECT）：{e}");
            std::process::exit(3);
        }
    };

    let size = match f.seek(SeekFrom::End(0)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("取不到设备大小：{e}");
            std::process::exit(4);
        }
    };
    // 读不到 ≠ 读到 0：大小为 0 说明拿到的不是想要的那块盘，整轮作废。
    if size == 0 {
        eprintln!("设备大小为 0 —— 判定不明，整轮作废");
        std::process::exit(5);
    }
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!("name=device_size bytes={size} path={dev}"))
    );

    // ── 顺序写 ──────────────────────────────────────────────
    let chunk = 1024 * 1024;
    let seq_ops: u64 = 256; // 256 MiB，小于虚机内存但走 O_DIRECT 不进页缓存
    let buf = Aligned::new(chunk);
    f.seek(SeekFrom::Start(0)).expect("seek 失败");
    let t0 = Instant::now();
    for _ in 0..seq_ops {
        f.write_all(buf.as_slice()).expect("顺序写失败");
    }
    f.sync_all().expect("sync 失败");
    let seq = Sample {
        ops: seq_ops,
        bytes_per_op: chunk as u64,
        elapsed_ns: t0.elapsed().as_nanos() as u64,
    };
    println!("{}", em.emit("seq_write_1m", &seq));

    // ── 随机 4K 写 ──────────────────────────────────────────
    let blk = 4096usize;
    let rnd_ops: u64 = 4096;
    let span = size / blk as u64;
    let mut rbuf = Aligned::new(blk);
    let mut state: u64 = 0x5156_1234_ABCD_0001; // 固定种子 —— 可复现是硬要求
    let t1 = Instant::now();
    for _ in 0..rnd_ops {
        let off = (next_rand(&mut state) % span) * blk as u64;
        f.seek(SeekFrom::Start(off)).expect("seek 失败");
        f.write_all(rbuf.as_slice()).expect("随机写失败");
    }
    f.sync_all().expect("sync 失败");
    let rnd = Sample {
        ops: rnd_ops,
        bytes_per_op: blk as u64,
        elapsed_ns: t1.elapsed().as_nanos() as u64,
    };
    println!("{}", em.emit("rand_write_4k", &rnd));

    // ── 随机 4K 读 ──────────────────────────────────────────
    let mut state2: u64 = 0x5156_1234_ABCD_0001;
    let t2 = Instant::now();
    let mut acc: u64 = 0;
    for _ in 0..rnd_ops {
        let off = (next_rand(&mut state2) % span) * blk as u64;
        f.seek(SeekFrom::Start(off)).expect("seek 失败");
        f.read_exact(rbuf.as_mut_slice()).expect("随机读失败");
        acc = acc.wrapping_add(rbuf.as_slice()[0] as u64); // 防止读被优化掉
    }
    let rrd = Sample {
        ops: rnd_ops,
        bytes_per_op: blk as u64,
        elapsed_ns: t2.elapsed().as_nanos() as u64,
    };
    println!("{}", em.emit("rand_read_4k", &rrd));
    println!("{}", em.emit_raw(&format!("name=checksum_guard acc={acc}")));
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`O_DIRECT` 要求缓冲按 `ALIGN` 对齐。** 不对齐会 `EINVAL`，
    /// 而那表现为「I/O 计数为 0」——看起来像很省，实际是根本没跑。
    /// 这是本 harness 里唯一一个「出错时看起来像好消息」的地方。
    #[test]
    fn aligned_buffer_is_actually_aligned() {
        for len in [ALIGN, ALIGN * 4, ALIGN * 256] {
            let a = Aligned::new(len);
            assert_eq!(a.as_slice().as_ptr() as usize % ALIGN, 0, "长度 {len} 的缓冲没有按 {ALIGN} 对齐");
        }
    }

    /// 随机数必须散开：它决定随机读写的落点，退化成常数会让「随机」这一档
    /// 实际变成「反复读同一个块」，而那会被缓存吃掉、量出一个虚高的吞吐。
    #[test]
    fn next_rand_spreads_out() {
        let mut st = 7u64;
        let v: std::collections::HashSet<u64> = (0..1000).map(|_| next_rand(&mut st) % 100_000).collect();
        assert!(v.len() > 900, "1000 次只取到 {} 个不同值，没散开", v.len());
    }

    /// 同一个种子必须给出同一串随机数——实验的可复现性靠这一条。
    #[test]
    fn next_rand_is_reproducible_from_the_seed() {
        let (mut a, mut b) = (11u64, 11u64);
        let xs: Vec<u64> = (0..50).map(|_| next_rand(&mut a)).collect();
        let ys: Vec<u64> = (0..50).map(|_| next_rand(&mut b)).collect();
        assert_eq!(xs, ys, "同种子给出了不同的序列，实验不可复现");
    }
}
