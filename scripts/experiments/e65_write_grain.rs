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

const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名，取自 <fcntl.h>，名字不归我们定
const ALIGNMENT_BYTES: usize = 4096;
/// 用户一次写多少。与 E58 的读侧同口径，两个实验的倍数才可比。
const USER_WRITE: u64 = 4096;
const GRAINS: [usize; 6] = [4096, 8192, 16384, 32768, 65536, 131072];
/// D4 已定项 1 定的数据单元。`pad` 那一臂拿它当默认档。
const DATA_UNIT_BYTES: usize = 32768;
const SEQUENTIAL_FILL_CHUNK_BYTES: usize = 1024 * 1024;

struct Aligned { pointer: *mut u8, length_in_bytes: usize, layout: Layout }
impl Aligned {
    fn new(length_in_bytes: usize) -> Self {
        let layout = Layout::from_size_align(length_in_bytes, ALIGNMENT_BYTES).expect("对齐参数非法");
        let pointer = unsafe { alloc(layout) };
        assert!(!pointer.is_null(), "分配失败");
        unsafe { std::ptr::write_bytes(pointer, 0xA5, length_in_bytes) };
        Self { pointer, length_in_bytes, layout }
    }
    fn as_slice(&self) -> &[u8] { unsafe { std::slice::from_raw_parts(self.pointer, self.length_in_bytes) } }
    fn as_mut_slice(&mut self) -> &mut [u8] { unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length_in_bytes) } }
}
impl Drop for Aligned {
    fn drop(&mut self) { unsafe { dealloc(self.pointer, self.layout) } }
}

fn next_random_word(state: &mut u64) -> u64 {
    let mut xorshift_state = *state;
    xorshift_state ^= xorshift_state >> 12; xorshift_state ^= xorshift_state << 25; xorshift_state ^= xorshift_state >> 27;
    *state = xorshift_state;
    xorshift_state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn read_process_input_output_counter(key: &str) -> Option<u64> {
    let counter_file_text = std::fs::read_to_string("/proc/self/io").ok()?;
    for line in counter_file_text.lines() {
        if let Some(value_text) = line.strip_prefix(key) {
            return value_text.trim().parse().ok();
        }
    }
    None
}

/// 写放大：写 4 KiB 用户数据实际要写的倍数。补齐之后它是恒等式。
fn write_amplification(grain_bytes: usize) -> f64 { grain_bytes as f64 / USER_WRITE as f64 }

/// D4 已定项 2 的空间放大：一个 `size` 字节的 extent 补齐到 `grain_bytes` 之后占多少倍。
fn padding_amplification(size: u64, grain_bytes: usize) -> f64 {
    let padded_bytes = size.div_ceil(grain_bytes as u64) * grain_bytes as u64;
    padded_bytes as f64 / size.max(1) as f64
}

struct Arm { elapsed_nanoseconds: u64, verify_nanoseconds: u64, operation_count: u64, sink: u64 }

fn open(path: &str) -> std::fs::File {
    OpenOptions::new().read(true).write(true).custom_flags(O_DIRECT).open(path)
        .unwrap_or_else(|error| { eprintln!("打不开 {path}：{error}"); std::process::exit(3) })
}

/// `rmw` / `full` / `rmwsync` 三条臂共用一段：`read_old` 决定读不读、`sync` 决定同不同步。
fn write_arm(path: &str, grain_bytes: usize, units: u64, operation_count: u64, seed: u64, read_old: bool, sync: bool) -> Arm {
    let file = open(path);
    let cipher = Aes256Gcm::new_from_slice(&[0x42u8; 32]).unwrap();
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut unit_buffer = Aligned::new(grain_bytes);
    let mut random_state = seed | 1;
    let (mut sink, mut verify_nanoseconds) = (0u64, 0u64);
    // COW 游标：新单元往前推，不原地覆盖。
    let mut cursor = 0u64;
    let start_instant = Instant::now();
    for _ in 0..operation_count {
        if read_old {
            let old_unit_index = next_random_word(&mut random_state) % units;
            file.read_exact_at(&mut unit_buffer.as_mut_slice()[..grain_bytes], old_unit_index * grain_bytes as u64).expect("RMW 读失败");
            let verify_start = Instant::now();
            // 读回来的旧单元先验一遍：D4 的 Merkle 要求改之前先确认它没坏
            sink = sink.wrapping_add(
                cipher.encrypt_in_place_detached(nonce, b"", &mut unit_buffer.as_mut_slice()[..grain_bytes])
                    .expect("验旧单元失败")[0] as u64);
            verify_nanoseconds += verify_start.elapsed().as_nanos() as u64;
        }
        // 改掉其中 4 KiB
        let random_word = next_random_word(&mut random_state);
        for (byte_index, byte) in unit_buffer.as_mut_slice()[..USER_WRITE as usize].iter_mut().enumerate() {
            *byte = (random_word >> (byte_index % 8 * 8)) as u8;
        }
        let verify_start = Instant::now();
        sink = sink.wrapping_add(
            cipher.encrypt_in_place_detached(nonce, b"", &mut unit_buffer.as_mut_slice()[..grain_bytes])
                .expect("算新 MAC 失败")[0] as u64);
        verify_nanoseconds += verify_start.elapsed().as_nanos() as u64;
        file.write_all_at(unit_buffer.as_slice(), cursor * grain_bytes as u64).expect("写失败");
        if sync { file.sync_data().expect("fdatasync 失败"); }
        cursor = (cursor + 1) % units;
    }
    Arm { elapsed_nanoseconds: start_instant.elapsed().as_nanos() as u64, verify_nanoseconds, operation_count, sink }
}

fn fill(path: &str, region: u64) {
    let mut file = OpenOptions::new().read(true).write(true).create(true).truncate(false)
        .custom_flags(O_DIRECT).open(path)
        .unwrap_or_else(|error| { eprintln!("建不了测试区 {path}：{error}"); std::process::exit(3) });
    if file.seek(SeekFrom::End(0)).expect("取不到大小") >= region { return; }
    eprintln!("填充测试区 {} MiB …", region / (1024 * 1024));
    let mut fill_buffer = Aligned::new(SEQUENTIAL_FILL_CHUNK_BYTES);
    for (byte_index, byte) in fill_buffer.as_mut_slice().iter_mut().enumerate() {
        *byte = (byte_index as u8).wrapping_mul(31).wrapping_add(7);
    }
    file.seek(SeekFrom::Start(0)).expect("seek 失败");
    for _ in 0..(region / SEQUENTIAL_FILL_CHUNK_BYTES as u64) { file.write_all(fill_buffer.as_slice()).expect("填充失败"); }
    file.sync_all().expect("sync 失败");
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法：e65-write-grain <块设备或文件> [种子] [ops] [区域 MiB]");
        std::process::exit(2)
    });
    let seed: u64 = std::env::args().nth(2).and_then(|argument_text| argument_text.parse().ok()).unwrap_or(0x6161_1234);
    let operation_count: u64 = std::env::args().nth(3).and_then(|argument_text| argument_text.parse().ok()).unwrap_or(2048);
    let region_mebibytes: u64 = std::env::args().nth(4).and_then(|argument_text| argument_text.parse().ok()).unwrap_or(8192);
    let mut region = region_mebibytes * 1024 * 1024;

    if std::fs::metadata(&path).map(|file_metadata| file_metadata.is_file()).unwrap_or(true) { fill(&path, region); }
    let size = open(&path).seek(SeekFrom::End(0)).expect("取不到大小");
    if size == 0 { eprintln!("大小为 0 —— 判定不明，整轮作废"); std::process::exit(5); }
    region = region.min(size);

    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config dev={path} size={size} region={region} ops={operation_count} user_write={USER_WRITE} \
         data_unit={DATA_UNIT_BYTES} seed={seed} grains={GRAINS:?}")));

    for &grain_bytes in GRAINS.iter() {
        let units = region / grain_bytes as u64;
        for (arm, read_old, sync) in [("rmw", true, false), ("full", false, false), ("rmwsync", true, true)] {
            let (read_bytes_before, write_bytes_before) = (read_process_input_output_counter("read_bytes:"), read_process_input_output_counter("write_bytes:"));
            let arm_result = write_arm(&path, grain_bytes, units, operation_count, seed, read_old, sync);
            let (read_bytes_after, write_bytes_after) = (read_process_input_output_counter("read_bytes:"), read_process_input_output_counter("write_bytes:"));
            let counter_delta = |before: Option<u64>, after: Option<u64>| match (before, after) {
                (Some(before), Some(after)) => format!("{}", after.saturating_sub(before)),
                _ => "NA".into(),
            };
            let expected_device_write_bytes = arm_result.operation_count * grain_bytes as u64;
            let write_ratio = match (write_bytes_before, write_bytes_after) {
                (Some(before), Some(after)) => format!("{:.4}", after.saturating_sub(before) as f64 / expected_device_write_bytes as f64),
                _ => "NA".into(),
            };
            println!("{}", emitter.emit_raw(&format!(
                "name={arm}_g{grain_bytes} grain={grain_bytes} ops={} user_bytes={} dev_write_bytes={expected_device_write_bytes} \
                 elapsed_ns={} verify_ns={} ns_per_op={:.1} write_amp={:.4} \
                 proc_read_bytes={} proc_write_bytes={} pw_over_want={write_ratio} sink={}",
                arm_result.operation_count, arm_result.operation_count * USER_WRITE, arm_result.elapsed_nanoseconds, arm_result.verify_nanoseconds,
                arm_result.elapsed_nanoseconds as f64 / arm_result.operation_count.max(1) as f64, write_amplification(grain_bytes),
                counter_delta(read_bytes_before, read_bytes_after), counter_delta(write_bytes_before, write_bytes_after), arm_result.sink)));
        }
        println!("{}", emitter.emit_raw(&format!(
            "name=pad_g{grain_bytes} grain={grain_bytes} write_amp={:.4} pad_amp_1k={:.4} pad_amp_4k={:.4} \
             pad_amp_g_minus_one={:.6}",
            write_amplification(grain_bytes), padding_amplification(1024, grain_bytes), padding_amplification(4096, grain_bytes), padding_amplification(grain_bytes as u64 - 1, grain_bytes))));
    }
    println!("{}", emitter.finish());
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
        assert_eq!(SEQUENTIAL_FILL_CHUNK_BYTES, 1048576);
    }

    /// **绝对值断言 2：写放大恰为 G/4096，逐档钉死。**
    #[test]
    fn write_amplification_is_exactly_grain_over_four_kibibytes() {
        assert_eq!(write_amplification(4096), 1.0);
        assert_eq!(write_amplification(16384), 4.0);
        assert_eq!(write_amplification(32768), 8.0);
        assert_eq!(write_amplification(131072), 32.0);
    }

    /// **绝对值断言 3：D4 已定项 2 的空间放大。**
    /// 32 KiB 单元下：1 KiB 文件占 32 倍、4 KiB 占 8 倍；恰好 1 字节不足时是最坏点。
    #[test]
    fn padding_amplification_at_the_settled_unit() {
        assert_eq!(padding_amplification(1024, 32768), 32.0);
        assert_eq!(padding_amplification(4096, 32768), 8.0);
        assert_eq!(padding_amplification(32768, 32768), 1.0);
        // 32769 字节要占 2 个单元 = 65536，倍数 65536/32769 = 1.99994（不是恰好 2）
        assert!((padding_amplification(32769, 32768) - 1.9999390).abs() < 1e-6, "{}", padding_amplification(32769, 32768));
        // 最坏浪费：单元大小减一
        let worst_case_amplification = padding_amplification(32767, 32768);
        assert!((worst_case_amplification - 1.0000305).abs() < 1e-5, "{worst_case_amplification}");
    }

    /// **16 KiB 与 32 KiB 在小文件上的差，绝对值。**
    /// 一个 1 KiB 文件：16 KiB 单元占 16 倍、32 KiB 占 32 倍 —— 差恰好一倍。
    #[test]
    fn thirty_two_doubles_the_small_file_waste_versus_sixteen() {
        assert_eq!(padding_amplification(1024, 16384), 16.0);
        assert_eq!(padding_amplification(1024, 32768), 32.0);
        assert_eq!(padding_amplification(1024, 32768) / padding_amplification(1024, 16384), 2.0);
    }

    #[test]
    fn pseudo_random_generator_is_deterministic() {
        let (mut first_state, mut second_state) = (999u64, 999u64);
        let first_sequence: Vec<u64> = (0..8).map(|_| next_random_word(&mut first_state)).collect();
        let second_sequence: Vec<u64> = (0..8).map(|_| next_random_word(&mut second_state)).collect();
        assert_eq!(first_sequence, second_sequence);
        assert_eq!(first_sequence.len(), 8);
        assert!(first_sequence.windows(2).all(|adjacent_pair| adjacent_pair[0] != adjacent_pair[1]));
    }
}
