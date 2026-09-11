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
const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名

/// O_DIRECT 要求缓冲区、偏移、长度都按块对齐。取 4096 覆盖常见的 512/4096 两种。
const ALIGNMENT_BYTES: usize = 4096;

/// 对齐缓冲区。O_DIRECT 下用普通 Vec 会被内核以 EINVAL 拒掉。
struct AlignedBuffer {
    pointer: *mut u8,
    length_in_bytes: usize,
    layout: Layout,
}

impl AlignedBuffer {
    fn new(length_in_bytes: usize) -> Self {
        let layout = Layout::from_size_align(length_in_bytes, ALIGNMENT_BYTES).expect("对齐参数非法");
        let allocated_pointer = unsafe { alloc(layout) };
        assert!(!allocated_pointer.is_null(), "分配失败");
        unsafe { std::ptr::write_bytes(allocated_pointer, 0xA5, length_in_bytes) };
        Self { pointer: allocated_pointer, length_in_bytes, layout }
    }
    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.pointer, self.length_in_bytes) }
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length_in_bytes) }
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.pointer, self.layout) }
    }
}

/// 确定性伪随机：同一个种子必须给出同一串偏移，否则实验不可复现
/// （rules/test-discipline.md：测试可复现是任何判断的前提）。
fn next_random(state: &mut u64) -> u64 {
    // xorshift64*，够用且无依赖
    let mut xorshift_state = *state;
    xorshift_state ^= xorshift_state >> 12;
    xorshift_state ^= xorshift_state << 25;
    xorshift_state ^= xorshift_state >> 27;
    *state = xorshift_state;
    xorshift_state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn main() {
    let device_path = match std::env::args().nth(1) {
        Some(device_path) => device_path,
        None => {
            eprintln!("用法：e7-index-bench <块设备>");
            std::process::exit(2);
        }
    };

    let mut device_file = match OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(O_DIRECT)
        .open(&device_path)
    {
        Ok(device_file) => device_file,
        Err(error) => {
            eprintln!("打不开 {device_path}（O_DIRECT）：{error}");
            std::process::exit(3);
        }
    };

    let device_size_bytes = match device_file.seek(SeekFrom::End(0)) {
        Ok(device_size_bytes) => device_size_bytes,
        Err(error) => {
            eprintln!("取不到设备大小：{error}");
            std::process::exit(4);
        }
    };
    // 读不到 ≠ 读到 0：大小为 0 说明拿到的不是想要的那块盘，整轮作废。
    if device_size_bytes == 0 {
        eprintln!("设备大小为 0 —— 判定不明，整轮作废");
        std::process::exit(5);
    }
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!("name=device_size bytes={device_size_bytes} path={device_path}"))
    );

    // ── 顺序写 ──────────────────────────────────────────────
    let sequential_chunk_bytes = 1024 * 1024;
    let sequential_write_count: u64 = 256; // 256 MiB，小于虚机内存但走 O_DIRECT 不进页缓存
    let sequential_buffer = AlignedBuffer::new(sequential_chunk_bytes);
    device_file.seek(SeekFrom::Start(0)).expect("seek 失败");
    let sequential_write_start = Instant::now();
    for _ in 0..sequential_write_count {
        device_file.write_all(sequential_buffer.as_slice()).expect("顺序写失败");
    }
    device_file.sync_all().expect("sync 失败");
    let sequential_write_sample = Sample {
        operation_count: sequential_write_count,
        bytes_per_operation: sequential_chunk_bytes as u64,
        elapsed_nanoseconds: sequential_write_start.elapsed().as_nanos() as u64,
    };
    println!("{}", emitter.emit("seq_write_1m", &sequential_write_sample));

    // ── 随机 4K 写 ──────────────────────────────────────────
    let random_block_bytes = 4096usize;
    let random_operation_count: u64 = 4096;
    let device_block_count = device_size_bytes / random_block_bytes as u64;
    let mut random_buffer = AlignedBuffer::new(random_block_bytes);
    let mut random_write_state: u64 = 0x5156_1234_ABCD_0001; // 固定种子 —— 可复现是硬要求
    let random_write_start = Instant::now();
    for _ in 0..random_operation_count {
        let offset_bytes = (next_random(&mut random_write_state) % device_block_count) * random_block_bytes as u64;
        device_file.seek(SeekFrom::Start(offset_bytes)).expect("seek 失败");
        device_file.write_all(random_buffer.as_slice()).expect("随机写失败");
    }
    device_file.sync_all().expect("sync 失败");
    let random_write_sample = Sample {
        operation_count: random_operation_count,
        bytes_per_operation: random_block_bytes as u64,
        elapsed_nanoseconds: random_write_start.elapsed().as_nanos() as u64,
    };
    println!("{}", emitter.emit("rand_write_4k", &random_write_sample));

    // ── 随机 4K 读 ──────────────────────────────────────────
    let mut random_read_state: u64 = 0x5156_1234_ABCD_0001;
    let random_read_start = Instant::now();
    let mut read_byte_accumulator: u64 = 0;
    for _ in 0..random_operation_count {
        let offset_bytes = (next_random(&mut random_read_state) % device_block_count) * random_block_bytes as u64;
        device_file.seek(SeekFrom::Start(offset_bytes)).expect("seek 失败");
        device_file.read_exact(random_buffer.as_mut_slice()).expect("随机读失败");
        read_byte_accumulator = read_byte_accumulator.wrapping_add(random_buffer.as_slice()[0] as u64); // 防止读被优化掉
    }
    let random_read_sample = Sample {
        operation_count: random_operation_count,
        bytes_per_operation: random_block_bytes as u64,
        elapsed_nanoseconds: random_read_start.elapsed().as_nanos() as u64,
    };
    println!("{}", emitter.emit("rand_read_4k", &random_read_sample));
    println!("{}", emitter.emit_raw(&format!("name=checksum_guard acc={read_byte_accumulator}")));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`O_DIRECT` 要求缓冲按 `ALIGNMENT_BYTES` 对齐。** 不对齐会 `EINVAL`，
    /// 而那表现为「I/O 计数为 0」——看起来像很省，实际是根本没跑。
    /// 这是本 harness 里唯一一个「出错时看起来像好消息」的地方。
    #[test]
    fn aligned_buffer_is_actually_aligned() {
        for length_in_bytes in [ALIGNMENT_BYTES, ALIGNMENT_BYTES * 4, ALIGNMENT_BYTES * 256] {
            let buffer = AlignedBuffer::new(length_in_bytes);
            assert_eq!(buffer.as_slice().as_ptr() as usize % ALIGNMENT_BYTES, 0, "长度 {length_in_bytes} 的缓冲没有按 {ALIGNMENT_BYTES} 对齐");
        }
    }

    /// 随机数必须散开：它决定随机读写的落点，退化成常数会让「随机」这一档
    /// 实际变成「反复读同一个块」，而那会被缓存吃掉、量出一个虚高的吞吐。
    #[test]
    fn next_random_spreads_out() {
        let mut random_state = 7u64;
        let distinct_values: std::collections::HashSet<u64> = (0..1000).map(|_| next_random(&mut random_state) % 100_000).collect();
        assert!(distinct_values.len() > 900, "1000 次只取到 {} 个不同值，没散开", distinct_values.len());
    }

    /// 同一个种子必须给出同一串随机数——实验的可复现性靠这一条。
    #[test]
    fn next_random_is_reproducible_from_the_seed() {
        let (mut first_state, mut second_state) = (11u64, 11u64);
        let first_sequence: Vec<u64> = (0..50).map(|_| next_random(&mut first_state)).collect();
        let second_sequence: Vec<u64> = (0..50).map(|_| next_random(&mut second_state)).collect();
        assert_eq!(first_sequence, second_sequence, "同种子给出了不同的序列，实验不可复现");
    }
}
