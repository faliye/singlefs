//! E162（崩溃放量判定块存储选型）第四段：S4 掉电装置——三个候选各按自己声明的持久化路径，断电之后已确认的块丢不丢、坏不坏、库打不打得开。
//!
// admission: always 每跑一次都是新的虚机开机与新的设备侧日志，同一份代码两次跑的断电点落在的条目不同
// run-condition: command python3 bash ldd sha256sum strip
//!
//! 跑前登记：`research/prompts/e162-preregistration.md` 补 1–补 14（S4 的定义在补 5.2、补 6、补 7、补 8、补 9、补 10、补 11；
//! 这个文件是补 12 分段表「第四段」那一行）。独立手写模型，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）：
//! 设备侧日志的解析自己写一份，读法照 dm-log-writes 的格式定义（与 `crates/singlefs-checker-tier/src/device_log.rs` 文件头读同一份定义，
//! 不引它的代码；两处读法的差别写在补 5.2「设备侧日志的读法」）。候选的读写路径照登记 5.2 逐字实现，拷自 S1 那份源文件
//! `research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs`（出处行号在登记第十二节修订）。
//!
//! 角色（`--role <名>`；来宾里的角色前面还有 `vm-bench.sh` 塞进来的设备路径 `/dev/vda`，不用它）：
//!   selftest          锚点自检（登记 7.2 的 A1–A6 与补 7 的 B 系列），对不上退 1（V10）
//!   s4-stage          搭写盘附加根：`--destination <空目录>`；搭完用搭好的加载器实跑一次搭好的二进制 `--role selftest`
//!   s4-drive          宿主上的总驱动：`--arms <R1,R0,K,F 的子集> --kernel <内核> --out <产物文件> [--dry-run]`
//!   s4-write-guest    来宾：写盘开机（分区、mkfs、挂载、建库、标记、逐块写）
//!   s4-verify-guest   来宾：验盘开机（按计划重放盘面、loop、挂载、起核对子进程）
//!   s4-check-point    来宾里验盘角色起的核对子进程（开库、PC4-S/P/O 的改动、逐块核）
//!
//! **s4-drive 起虚机，是重型测试**：环境变量 `SINGLEFS_HEAVY_TESTS` 不是 `user-request` 或 `commit` 时拒跑，退出码 64，
//! stderr 写「s4-drive 起 QEMU 虚机，是重型测试；只在用户要求或提交时带 SINGLEFS_HEAVY_TESTS=user-request（或 commit）跑」。
//! 带 `--dry-run` 时不查这个变量：只搭附加根、打印将要起的每条 vm-bench 命令与变量，不起虚机。单测不调 s4-drive。
//! s4-drive 的退出码：0 全部开机做完、判定行都打了（判「翻」也是 0）；1 锚点对不上（V10）；2 用法不对；3 停机条款打中（H7–H10、H14）
//! 或开跑前提不满足（可用盘 < 40 GiB、可用内存 < 16 GiB、机器上有别的虚机或性能测量）；64 见上一段。
//!
//! 计时：只有 Q4h（重开用时）一个量，由核对子进程自己用 `Instant` 计、写进它的结果行；父进程读子进程输出不打时间戳
//! （`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）。

use e7_index_bench::Emitter;
use redb::ReadableDatabase;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::os::unix::fs::{FileExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// ============================== 登记写死的常量 ==============================

/// seed_S1（登记 5.1）：S4 的块值（补 5.2「每条主臂写 N_W = 6 000 块（2¹⁶ 状态、P-rand、seed_S1……）」）。
const SEED_CRASH_BLOCKS: u64 = 0xE162_0000_0000_0001;
/// seed_S2：只给 7.2 的 A3、A4 锚点用。
const SEED_THROUGHPUT_BLOCKS: u64 = 0xE162_0000_0000_0002;
/// seed_FP：输入指纹。
const SEED_FINGERPRINT: u64 = 0xE162_0000_0000_0007;
/// seed_CUT：分桶取断电点（补 7 B2）。
const SEED_CUT: u64 = 0xE162_0000_0000_000A;
/// seed_SUB：U-rand / U-tear 的子集与撕法（补 7 B3）。
const SEED_SUBSET: u64 = 0xE162_0000_0000_000B;
/// seed_PICK：C-commit 挑块（补 7 B5）。
const SEED_PICK: u64 = 0xE162_0000_0000_000C;

const SPLITMIX_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;
const SPLITMIX_FIRST_MULTIPLIER: u64 = 0xBF58_476D_1CE4_E5B9;
const SPLITMIX_SECOND_MULTIPLIER: u64 = 0x94D0_49BB_1331_11EB;
/// 奇数混淆乘子：块值起点（登记 5.1）与各生成器的状态（补 7 B2、B3、B5）都乘它。
const MIXING_MULTIPLIER: u64 = 0xD1B5_4A32_D192_ED03;

const PRIMARY_STATES_PER_BLOCK: u64 = 65_536;
/// S4-bs10：每块 2¹⁰ 个状态。
const SMALL_STATES_PER_BLOCK: u64 = 1_024;
const BLOCK_KEY_BYTES: usize = 32;
const FINGERPRINT_BYTES: usize = 16;
const INTERLEAVED_STREAM_COUNT: u64 = 64;
const SEGMENTS_PER_NODE: u64 = 8;

/// N_W：每条主臂（与 S4-bs10）写盘开机写的块数（补 5.2）。
const MAIN_BLOCK_COUNT: u64 = 6_000;
/// PC4-L、PC4-NB、PC4-FD 各写 1 000 块（补 5.2 阳性对照表）。
const CONTROL_BLOCK_COUNT: u64 = 1_000;
/// V25：K 的写盘开机按 N_W 翻倍重做，至多到这么多块。
const ROCKSDB_REDO_BLOCK_LIMIT: u64 = 24_000;
/// V25：K 主格写盘开机报的 L0 落盘不到 4 次、或合并 0 次，就翻倍重做。
const ROCKSDB_FLUSH_MINIMUM: u64 = 4;
const ROCKSDB_COMPACTION_MINIMUM: u64 = 1;

const SECTOR_BYTES: usize = 512;
const MEBIBYTE: u64 = 1 << 20;
/// 分区（补 7 B9c）：vda1 自第 2 048 扇区起 3 072 MiB，vda2 紧随 64 MiB。
const FIRST_PARTITION_START_SECTOR: u64 = 2_048;
const FIRST_PARTITION_MEBIBYTES: u64 = 3_072;
const MARKER_PARTITION_MEBIBYTES: u64 = 64;
/// vm-bench 的变量（补 5.2 两种开机那张表）。
const DISK_MEBIBYTES: u64 = 3_200;
const LOG_DISK_MEBIBYTES: u64 = 4_096;
const WRITE_BOOT_MEMORY_MEBIBYTES: u64 = 4_096;
const WRITE_BOOT_PROCESSORS: u64 = 4;
const WRITE_BOOT_TIMEOUT_SECONDS: u64 = 3_600;
const VERIFY_BOOT_MEMORY_MEBIBYTES: u64 = 12_288;
const VERIFY_BOOT_TIMEOUT_SECONDS: u64 = 14_400;
/// 验盘每批不超过 200 个「断电点 × 取法」。
const VERIFY_BATCH_ITEM_LIMIT: usize = 200;

/// PC4-C：在 /dev/vda 第 8 192 扇区起逐个写 100 个扇区，fdatasync，再写 100 个（补 5.2）。
const WRITE_CACHE_MODEL_FIRST_SECTOR: u64 = 8_192;
const WRITE_CACHE_MODEL_BATCH_WRITES: u64 = 100;
/// PC4-C 的 U-rand 取 a = 9、i = 0，登记的丢块数是 47（补 7 B4）。
const WRITE_CACHE_MODEL_RANDOM_SUBSET_LOST: u64 = 47;

/// C-strat：200 点，桶 B 60、桶 C 40、其余归桶 A；S4-bs10：100 点，桶 B 30、桶 C 20。
const STRATIFIED_POINT_TOTAL: usize = 200;
const LONG_COMMIT_BUCKET_QUOTA: usize = 60;
const BETWEEN_COMMITS_BUCKET_QUOTA: usize = 40;
const SMALL_BLOCK_POINT_TOTAL: usize = 100;
const SMALL_BLOCK_LONG_COMMIT_QUOTA: usize = 30;
const SMALL_BLOCK_BETWEEN_COMMITS_QUOTA: usize = 20;
/// L_j 大于全部 L 的中位数 4 倍的算「长窗」。
const LONG_WINDOW_MEDIAN_FACTOR: u64 = 4;
/// C-commit：普通窗挑 6 块、长窗挑 2 块（长窗不足 2 个就全从普通窗补），一块多于 64 个边界时取 64 个等距点。
const PICKED_COMMIT_TOTAL: usize = 8;
const PICKED_LONG_COMMIT_TARGET: usize = 2;
const COMMIT_BOUNDARY_LIMIT: usize = 64;
/// C-commit 各点的 U-rand 生成器下标 = 10 000 + 100 × m + b。
const COMMIT_SUBSET_INDEX_BASE: u64 = 10_000;
const COMMIT_SUBSET_INDEX_STRIDE: u64 = 100;
/// PC4-L、PC4-NB、PC4-FD：20 点里 ≥ 10 点看到丢块信号。
const CONTROL_POINT_TOTAL: usize = 20;
const CONTROL_SIGNAL_POINT_MINIMUM: usize = 10;
/// V24：一条臂 C-strat 200 点里 U(k) 非空的不到 100 点，U-drop 的「不翻」不算不翻。
const UNFLUSHED_POINT_MINIMUM: usize = 100;
/// 分桶生成器的桶号（补 7 B2、B5）。
const BUCKET_NORMAL_COMMITS: u64 = 0;
const BUCKET_LONG_COMMITS: u64 = 1;
const BUCKET_BETWEEN_COMMITS: u64 = 2;
const BUCKET_PICK_NORMAL: u64 = 3;
const BUCKET_PICK_LONG: u64 = 4;

/// 核对子进程 3 600 秒没交回就杀、记 Q4f 超时；Q4h 另报 > 60 秒的次数。
const CHECK_TIMEOUT_SECONDS: u64 = 3_600;
const SLOW_OPEN_MICROSECONDS: u64 = 60_000_000;
const CHECK_POLL_MILLISECONDS: u64 = 100;

/// PC4-S、PC4-P、PC4-O（照登记 5.3 PC-S、PC-P、PC-O）。
const SILENT_CORRUPTION_BLOCK: u64 = 7;
const SILENT_CORRUPTION_BYTE_OFFSET: usize = 40_000;
const SILENT_CORRUPTION_MASK: u8 = 0x5A;
const PHANTOM_KEY_NODE: u32 = 0xFFFF_FFFF;
const REDB_BROKEN_PREFIX_BYTES: usize = 4_096;
const ROCKSDB_BROKEN_CURRENT: &[u8] = b"MANIFEST-999999\n";

/// 候选 F 的文件格式（登记 5.2）。
const FILE_MAGIC: &[u8; 4] = b"SFCB";
const FILE_FORMAT_VERSION: u32 = 1;
const FILE_HEADER_BYTES: usize = 48;
const FILE_HEADER_CHECKSUMMED_BYTES: usize = 44;
const FORMAT_MAGIC: &[u8; 8] = b"SFE162FB";
const FORMAT_FILE_BYTES: usize = 16;
const FORMAT_FILE_NAME: &str = "FORMAT";
const REDB_TABLE: redb::TableDefinition<&[u8], &[u8]> = redb::TableDefinition::new("blocks");
const REDB_FILE_NAME: &str = "blocks.redb";

/// 标记扇区（补 5.2「标记」）：`SFMK`｜种类｜三个 0｜c｜j｜臂号｜前 28 字节的 CRC-32C｜480 个 0。
const MARKER_MAGIC: &[u8; 4] = b"SFMK";
const MARKER_CHECKSUMMED_BYTES: usize = 28;
const MARKER_WITHOUT_BLOCK: u64 = u64::MAX;

/// dm-log-writes 的系统配置与条目头（小端）。
const WRITE_LOG_MAGIC: u64 = 0x006a_7366_7773_6872;
const WRITE_LOG_VERSION: u64 = 1;
const LOG_FLUSH_FLAG: u64 = 1;
const LOG_FUA_FLAG: u64 = 1 << 1;
const LOG_DISCARD_FLAG: u64 = 1 << 2;
const LOG_MARK_FLAG: u64 = 1 << 3;
const LOG_METADATA_FLAG: u64 = 1 << 4;
const LOG_ENTRY_HEADER_BYTES: usize = 32;
const LOG_SUPER_BYTES: usize = 28;

/// Linux x86-64 的 O_DIRECT（`<asm-generic/fcntl.h>` 的 00040000）。标记的 pwrite 要它：返回时请求已经到盘。
const OPEN_DIRECT_FLAG: i32 = 0o40_000;
const DIRECT_WRITE_ALIGNMENT_BYTES: usize = 4_096;

/// 来宾里的路径。
const GUEST_WHOLE_DISK: &str = "/dev/vda";
const GUEST_FIRST_PARTITION: &str = "/dev/vda1";
const GUEST_MARKER_PARTITION: &str = "/dev/vda2";
const GUEST_MOUNT_POINT: &str = "/mnt/s4";
const GUEST_LIBRARY_NAME: &str = "library";
const GUEST_BUSYBOX: &str = "/bin/busybox";
const GUEST_PARTITIONER: &str = "/usr/sbin/sfdisk";
const GUEST_FILESYSTEM_FORMATTER: &str = "/usr/sbin/mkfs.ext4";
const GUEST_IMAGE_DIRECTORY: &str = "/s4-images";
const GUEST_IMAGE_FILESYSTEM_OPTIONS: &str = "size=8g";
const GUEST_LOOP_DEVICE: &str = "/dev/loop0";
const GUEST_LOG_PATH: &str = "/s4/device.log";
const GUEST_PLAN_PATH: &str = "/s4/plan.txt";
const STAGED_BINARY_GUEST_PATH: &str = "/usr/bin/e162-verdict-store-power-cut";
const STAGED_LOADER_GUEST_PATH: &str = "/lib64/ld-linux-x86-64.so.2";

const HEAVY_TEST_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_HEAVY_TESTS";
const HEAVY_TEST_PERMITTED_VALUES: [&str; 2] = ["user-request", "commit"];
const HEAVY_TEST_REFUSAL_EXIT_CODE: i32 = 64;
const HALT_EXIT_CODE: i32 = 3;
const USAGE_EXIT_CODE: i32 = 2;
/// 开跑前提（补 5.3「S4：宿主资源」）：可用内存 ≥ 16 GiB，库所在文件系统可用 ≥ 40 GiB。
const MINIMUM_AVAILABLE_MEMORY_BYTES: u64 = 16 << 30;
const MINIMUM_AVAILABLE_DISK_BYTES: u64 = 40 << 30;

fn sector_bytes() -> u64 {
    u64::try_from(SECTOR_BYTES).expect("512 装得进 u64")
}

fn first_partition_sectors() -> u64 {
    FIRST_PARTITION_MEBIBYTES * MEBIBYTE / sector_bytes()
}

fn marker_partition_start_sector() -> u64 {
    FIRST_PARTITION_START_SECTOR + first_partition_sectors()
}

fn marker_partition_sectors() -> u64 {
    MARKER_PARTITION_MEBIBYTES * MEBIBYTE / sector_bytes()
}

// ============================== 随机源、校验和、SHA-256 ==============================

/// SplitMix64（登记 5.1）：全部 64 位回绕。
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(state: u64) -> Self {
        Self { state }
    }

    fn next_output(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX_INCREMENT);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(SPLITMIX_FIRST_MULTIPLIER);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(SPLITMIX_SECOND_MULTIPLIER);
        mixed ^ (mixed >> 31)
    }
}

/// CRC-32C（Castagnoli，反射多项式 0x82F63B78），查表法。
struct Crc32cTable {
    entries: [u32; 256],
}

impl Crc32cTable {
    fn new() -> Self {
        let mut entries = [0u32; 256];
        for (byte_value, entry) in entries.iter_mut().enumerate() {
            let mut remainder = u32::try_from(byte_value).expect("表下标小于 256");
            for _bit in 0..8 {
                remainder = if remainder & 1 == 1 {
                    (remainder >> 1) ^ 0x82F6_3B78
                } else {
                    remainder >> 1
                };
            }
            *entry = remainder;
        }
        Self { entries }
    }

    fn checksum(&self, parts: &[&[u8]]) -> u32 {
        let mut running = 0xFFFF_FFFFu32;
        for part in parts {
            for &byte in part.iter() {
                let table_index = usize::from(running.to_le_bytes()[0] ^ byte);
                running = (running >> 8) ^ self.entries[table_index];
            }
        }
        running ^ 0xFFFF_FFFF
    }
}

fn crc32c(parts: &[&[u8]]) -> u32 {
    thread_local! {
        static TABLE: Crc32cTable = Crc32cTable::new();
    }
    TABLE.with(|table| table.checksum(parts))
}

/// SHA-256（FIPS 180-4）。只给锚点（补 7 B7、B8 的扇区与镜像摘要）用；宿主上整盘的摘要交给 `sha256sum`。
const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a_2f98, 0x7137_4491, 0xb5c0_fbcf, 0xe9b5_dba5, 0x3956_c25b, 0x59f1_11f1, 0x923f_82a4, 0xab1c_5ed5,
    0xd807_aa98, 0x1283_5b01, 0x2431_85be, 0x550c_7dc3, 0x72be_5d74, 0x80de_b1fe, 0x9bdc_06a7, 0xc19b_f174,
    0xe49b_69c1, 0xefbe_4786, 0x0fc1_9dc6, 0x240c_a1cc, 0x2de9_2c6f, 0x4a74_84aa, 0x5cb0_a9dc, 0x76f9_88da,
    0x983e_5152, 0xa831_c66d, 0xb003_27c8, 0xbf59_7fc7, 0xc6e0_0bf3, 0xd5a7_9147, 0x06ca_6351, 0x1429_2967,
    0x27b7_0a85, 0x2e1b_2138, 0x4d2c_6dfc, 0x5338_0d13, 0x650a_7354, 0x766a_0abb, 0x81c2_c92e, 0x9272_2c85,
    0xa2bf_e8a1, 0xa81a_664b, 0xc24b_8b70, 0xc76c_51a3, 0xd192_e819, 0xd699_0624, 0xf40e_3585, 0x106a_a070,
    0x19a4_c116, 0x1e37_6c08, 0x2748_774c, 0x34b0_bcb5, 0x391c_0cb3, 0x4ed8_aa4a, 0x5b9c_ca4f, 0x682e_6ff3,
    0x748f_82ee, 0x78a5_636f, 0x84c8_7814, 0x8cc7_0208, 0x90be_fffa, 0xa450_6ceb, 0xbef9_a3f7, 0xc671_78f2,
];
const SHA256_INITIAL_STATE: [u32; 8] =
    [0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372, 0xa54f_f53a, 0x510e_527f, 0x9b05_688c, 0x1f83_d9ab, 0x5be0_cd19];

fn sha256_hexadecimal(message: &[u8]) -> String {
    let message_bits = u64::try_from(message.len()).expect("消息长度装得进 u64") * 8;
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&message_bits.to_be_bytes());
    let mut state = SHA256_INITIAL_STATE;
    let (chunks, _padded_to_whole_chunks) = padded.as_chunks::<64>();
    for chunk in chunks {
        let mut schedule = [0u32; 64];
        for (word_index, word) in chunk.as_chunks::<4>().0.iter().enumerate() {
            schedule[word_index] = u32::from_be_bytes(*word);
        }
        for round in 16..64 {
            let earlier = schedule[round - 15];
            let recent = schedule[round - 2];
            let small_sigma_zero = earlier.rotate_right(7) ^ earlier.rotate_right(18) ^ (earlier >> 3);
            let small_sigma_one = recent.rotate_right(17) ^ recent.rotate_right(19) ^ (recent >> 10);
            schedule[round] = schedule[round - 16]
                .wrapping_add(small_sigma_zero)
                .wrapping_add(schedule[round - 7])
                .wrapping_add(small_sigma_one);
        }
        let mut working = state;
        for (round, schedule_word) in schedule.iter().enumerate() {
            let big_sigma_one = working[4].rotate_right(6) ^ working[4].rotate_right(11) ^ working[4].rotate_right(25);
            let choice = (working[4] & working[5]) ^ (!working[4] & working[6]);
            let first_addend = working[7]
                .wrapping_add(big_sigma_one)
                .wrapping_add(choice)
                .wrapping_add(SHA256_ROUND_CONSTANTS[round])
                .wrapping_add(*schedule_word);
            let big_sigma_zero = working[0].rotate_right(2) ^ working[0].rotate_right(13) ^ working[0].rotate_right(22);
            let majority = (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let second_addend = big_sigma_zero.wrapping_add(majority);
            working[7] = working[6];
            working[6] = working[5];
            working[5] = working[4];
            working[4] = working[3].wrapping_add(first_addend);
            working[3] = working[2];
            working[2] = working[1];
            working[1] = working[0];
            working[0] = first_addend.wrapping_add(second_addend);
        }
        for (slot, added) in state.iter_mut().zip(working) {
            *slot = slot.wrapping_add(added);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

// ============================== 块值与键（登记 5.1，照 S1 那份源文件） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValuePattern {
    /// P-rand：从起始状态连取 V/8 个输出，逐个小端拼成 V 字节。
    Random,
    /// P-sparse：每状态取一个输出 r，r >> 56 == 0 时是 1 + (r & 0xFF) mod 255，否则 0（只给 A4 锚点用）。
    Sparse,
}

fn block_start_state(seed: u64, block_index: u64) -> u64 {
    seed ^ block_index.wrapping_mul(MIXING_MULTIPLIER)
}

fn block_value(pattern: ValuePattern, seed: u64, block_index: u64, states_per_block: u64) -> Vec<u8> {
    let value_bytes = usize::try_from(states_per_block).expect("每块状态数装得进 usize");
    let mut generator = SplitMix64::new(block_start_state(seed, block_index));
    match pattern {
        ValuePattern::Random => {
            let mut value = Vec::with_capacity(value_bytes);
            for _word in 0..value_bytes / 8 {
                value.extend_from_slice(&generator.next_output().to_le_bytes());
            }
            value
        }
        ValuePattern::Sparse => {
            let mut value = vec![0u8; value_bytes];
            for state_verdict in value.iter_mut() {
                let output = generator.next_output();
                if output >> 56 == 0 {
                    *state_verdict = 1 + u8::try_from((output & 0xFF) % 255).expect("小于 255");
                }
            }
            value
        }
    }
}

fn input_fingerprint() -> [u8; FINGERPRINT_BYTES] {
    let mut generator = SplitMix64::new(SEED_FINGERPRINT);
    let mut fingerprint = [0u8; FINGERPRINT_BYTES];
    fingerprint[..8].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint[8..].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint
}

type BlockKey = [u8; BLOCK_KEY_BYTES];

fn compose_block_key(node: u32, segment: u32, first_state: u64, fingerprint: &[u8; FINGERPRINT_BYTES]) -> BlockKey {
    let mut key = [0u8; BLOCK_KEY_BYTES];
    key[0..4].copy_from_slice(&node.to_be_bytes());
    key[4..8].copy_from_slice(&segment.to_be_bytes());
    key[8..16].copy_from_slice(&first_state.to_be_bytes());
    key[16..32].copy_from_slice(fingerprint);
    key
}

fn interleaved_block_key(block_index: u64, states_per_block: u64, fingerprint: &[u8; FINGERPRINT_BYTES]) -> BlockKey {
    let stream = block_index % INTERLEAVED_STREAM_COUNT;
    let block_in_stream = block_index / INTERLEAVED_STREAM_COUNT;
    let node = u32::try_from(stream / SEGMENTS_PER_NODE).expect("节点号小于 8");
    let segment = u32::try_from(stream % SEGMENTS_PER_NODE).expect("段号小于 8");
    compose_block_key(node, segment, block_in_stream * states_per_block, fingerprint)
}

fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ============================== 臂（补 7 B1）与写入设定 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Candidate {
    /// R1：redb，每个写事务 Durability::Immediate + quick-repair。
    RedbQuickRepair,
    /// R0：redb，Durability::Immediate，quick-repair 关。
    RedbDefault,
    /// K：RocksDB，LZ4，同步写、WAL 开。
    RocksDatabase,
    /// F：加固的文件。
    HardenedFile,
}

impl Candidate {
    fn label(self) -> &'static str {
        match self {
            Candidate::RedbQuickRepair => "R1",
            Candidate::RedbDefault => "R0",
            Candidate::RocksDatabase => "K",
            Candidate::HardenedFile => "F",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RedbCommitDurability {
    Immediate,
    NotPersisted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RedbRepairMode {
    QuickRepair,
    FullRepair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RocksDatabaseWriteSync {
    SyncEachWrite,
    NoSync,
}

/// F 的持久步骤：全做；临时文件与目录都不 fsync（F-nofsync）；只拿掉改名之后的父目录 fsync（F-nodirsync）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileSyncSteps {
    TemporaryFileAndDirectory,
    NeitherTemporaryFileNorDirectory,
    TemporaryFileOnly,
}

/// ext4 的挂载选项：默认（barrier 开）或 `barrier=0`（PC4-NB）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilesystemBarrier {
    Enabled,
    Disabled,
}

/// 一条臂的全部写入选项；对照形态与正确形态逐项比，必须恰好差登记的那一处（M26）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WriteSettings {
    redb_commit_durability: RedbCommitDurability,
    redb_repair_mode: RedbRepairMode,
    rocksdb_write_sync: RocksDatabaseWriteSync,
    file_sync_steps: FileSyncSteps,
    filesystem_barrier: FilesystemBarrier,
}

impl WriteSettings {
    fn registered_for(candidate: Candidate) -> WriteSettings {
        WriteSettings {
            redb_commit_durability: RedbCommitDurability::Immediate,
            redb_repair_mode: match candidate {
                Candidate::RedbQuickRepair => RedbRepairMode::QuickRepair,
                Candidate::RedbDefault | Candidate::RocksDatabase | Candidate::HardenedFile => RedbRepairMode::FullRepair,
            },
            rocksdb_write_sync: RocksDatabaseWriteSync::SyncEachWrite,
            file_sync_steps: FileSyncSteps::TemporaryFileAndDirectory,
            filesystem_barrier: FilesystemBarrier::Enabled,
        }
    }

    fn differing_setting_names(self, other: WriteSettings) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.redb_commit_durability != other.redb_commit_durability {
            names.push("redb_commit_durability");
        }
        if self.redb_repair_mode != other.redb_repair_mode {
            names.push("redb_repair_mode");
        }
        if self.rocksdb_write_sync != other.rocksdb_write_sync {
            names.push("rocksdb_write_sync");
        }
        if self.file_sync_steps != other.file_sync_steps {
            names.push("file_sync_steps");
        }
        if self.filesystem_barrier != other.filesystem_barrier {
            names.push("filesystem_barrier");
        }
        names
    }
}

/// 补 7 B1 的 18 个臂号。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PowerCutArm {
    RedbQuickRepair,
    RedbDefault,
    RocksDatabase,
    HardenedFile,
    RedbQuickRepairDurabilityNone,
    RedbDefaultDurabilityNone,
    RocksDatabaseWithoutSync,
    HardenedFileWithoutFsync,
    HardenedFileWithoutDirectorySync,
    WriteCacheModel,
    RedbQuickRepairWithoutBarrier,
    RedbDefaultWithoutBarrier,
    RocksDatabaseWithoutBarrier,
    HardenedFileWithoutBarrier,
    RedbQuickRepairSmallBlocks,
    RedbDefaultSmallBlocks,
    RocksDatabaseSmallBlocks,
    HardenedFileSmallBlocks,
}

const ALL_POWER_CUT_ARMS: [PowerCutArm; 18] = [
    PowerCutArm::RedbQuickRepair,
    PowerCutArm::RedbDefault,
    PowerCutArm::RocksDatabase,
    PowerCutArm::HardenedFile,
    PowerCutArm::RedbQuickRepairDurabilityNone,
    PowerCutArm::RedbDefaultDurabilityNone,
    PowerCutArm::RocksDatabaseWithoutSync,
    PowerCutArm::HardenedFileWithoutFsync,
    PowerCutArm::HardenedFileWithoutDirectorySync,
    PowerCutArm::WriteCacheModel,
    PowerCutArm::RedbQuickRepairWithoutBarrier,
    PowerCutArm::RedbDefaultWithoutBarrier,
    PowerCutArm::RocksDatabaseWithoutBarrier,
    PowerCutArm::HardenedFileWithoutBarrier,
    PowerCutArm::RedbQuickRepairSmallBlocks,
    PowerCutArm::RedbDefaultSmallBlocks,
    PowerCutArm::RocksDatabaseSmallBlocks,
    PowerCutArm::HardenedFileSmallBlocks,
];

/// 一次写盘开机在判定里管哪一格。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArmPurpose {
    /// 主格（判定在 U-drop 上）。
    Main,
    /// PC4-L：丢页缓存查得出。
    LostPageCacheControl,
    /// PC4-NB：丢写缓存查得出。
    LostWriteCacheControl,
    /// PC4-FD：少一步查得出（只有 F）。
    MissingDirectorySyncControl,
    /// PC4-C：断电模型分得出写缓存（模型级，一次）。
    WriteCacheModelControl,
    /// S4-bs10 取样点。
    SmallBlockSamplingPoint,
}

impl PowerCutArm {
    fn index(self) -> u32 {
        match self {
            PowerCutArm::RedbQuickRepair => 0,
            PowerCutArm::RedbDefault => 1,
            PowerCutArm::RocksDatabase => 2,
            PowerCutArm::HardenedFile => 3,
            PowerCutArm::RedbQuickRepairDurabilityNone => 4,
            PowerCutArm::RedbDefaultDurabilityNone => 5,
            PowerCutArm::RocksDatabaseWithoutSync => 6,
            PowerCutArm::HardenedFileWithoutFsync => 7,
            PowerCutArm::HardenedFileWithoutDirectorySync => 8,
            PowerCutArm::WriteCacheModel => 9,
            PowerCutArm::RedbQuickRepairWithoutBarrier => 10,
            PowerCutArm::RedbDefaultWithoutBarrier => 11,
            PowerCutArm::RocksDatabaseWithoutBarrier => 12,
            PowerCutArm::HardenedFileWithoutBarrier => 13,
            PowerCutArm::RedbQuickRepairSmallBlocks => 14,
            PowerCutArm::RedbDefaultSmallBlocks => 15,
            PowerCutArm::RocksDatabaseSmallBlocks => 16,
            PowerCutArm::HardenedFileSmallBlocks => 17,
        }
    }

    fn label(self) -> &'static str {
        match self {
            PowerCutArm::RedbQuickRepair => "R1",
            PowerCutArm::RedbDefault => "R0",
            PowerCutArm::RocksDatabase => "K",
            PowerCutArm::HardenedFile => "F",
            PowerCutArm::RedbQuickRepairDurabilityNone => "R1-None",
            PowerCutArm::RedbDefaultDurabilityNone => "R0-None",
            PowerCutArm::RocksDatabaseWithoutSync => "K-nosync",
            PowerCutArm::HardenedFileWithoutFsync => "F-nofsync",
            PowerCutArm::HardenedFileWithoutDirectorySync => "F-nodirsync",
            PowerCutArm::WriteCacheModel => "PC4-C",
            PowerCutArm::RedbQuickRepairWithoutBarrier => "R1-NB",
            PowerCutArm::RedbDefaultWithoutBarrier => "R0-NB",
            PowerCutArm::RocksDatabaseWithoutBarrier => "K-NB",
            PowerCutArm::HardenedFileWithoutBarrier => "F-NB",
            PowerCutArm::RedbQuickRepairSmallBlocks => "R1-bs10",
            PowerCutArm::RedbDefaultSmallBlocks => "R0-bs10",
            PowerCutArm::RocksDatabaseSmallBlocks => "K-bs10",
            PowerCutArm::HardenedFileSmallBlocks => "F-bs10",
        }
    }

    fn from_index(index: u32) -> Option<PowerCutArm> {
        ALL_POWER_CUT_ARMS.iter().copied().find(|arm| arm.index() == index)
    }

    fn main_arm_from_label(label: &str) -> Option<PowerCutArm> {
        [PowerCutArm::RedbQuickRepair, PowerCutArm::RedbDefault, PowerCutArm::RocksDatabase, PowerCutArm::HardenedFile]
            .into_iter()
            .find(|arm| arm.label() == label)
    }

    fn candidate(self) -> Option<Candidate> {
        match self {
            PowerCutArm::RedbQuickRepair
            | PowerCutArm::RedbQuickRepairDurabilityNone
            | PowerCutArm::RedbQuickRepairWithoutBarrier
            | PowerCutArm::RedbQuickRepairSmallBlocks => Some(Candidate::RedbQuickRepair),
            PowerCutArm::RedbDefault
            | PowerCutArm::RedbDefaultDurabilityNone
            | PowerCutArm::RedbDefaultWithoutBarrier
            | PowerCutArm::RedbDefaultSmallBlocks => Some(Candidate::RedbDefault),
            PowerCutArm::RocksDatabase
            | PowerCutArm::RocksDatabaseWithoutSync
            | PowerCutArm::RocksDatabaseWithoutBarrier
            | PowerCutArm::RocksDatabaseSmallBlocks => Some(Candidate::RocksDatabase),
            PowerCutArm::HardenedFile
            | PowerCutArm::HardenedFileWithoutFsync
            | PowerCutArm::HardenedFileWithoutDirectorySync
            | PowerCutArm::HardenedFileWithoutBarrier
            | PowerCutArm::HardenedFileSmallBlocks => Some(Candidate::HardenedFile),
            PowerCutArm::WriteCacheModel => None,
        }
    }

    fn purpose(self) -> ArmPurpose {
        match self {
            PowerCutArm::RedbQuickRepair | PowerCutArm::RedbDefault | PowerCutArm::RocksDatabase | PowerCutArm::HardenedFile => {
                ArmPurpose::Main
            }
            PowerCutArm::RedbQuickRepairDurabilityNone
            | PowerCutArm::RedbDefaultDurabilityNone
            | PowerCutArm::RocksDatabaseWithoutSync
            | PowerCutArm::HardenedFileWithoutFsync => ArmPurpose::LostPageCacheControl,
            PowerCutArm::HardenedFileWithoutDirectorySync => ArmPurpose::MissingDirectorySyncControl,
            PowerCutArm::WriteCacheModel => ArmPurpose::WriteCacheModelControl,
            PowerCutArm::RedbQuickRepairWithoutBarrier
            | PowerCutArm::RedbDefaultWithoutBarrier
            | PowerCutArm::RocksDatabaseWithoutBarrier
            | PowerCutArm::HardenedFileWithoutBarrier => ArmPurpose::LostWriteCacheControl,
            PowerCutArm::RedbQuickRepairSmallBlocks
            | PowerCutArm::RedbDefaultSmallBlocks
            | PowerCutArm::RocksDatabaseSmallBlocks
            | PowerCutArm::HardenedFileSmallBlocks => ArmPurpose::SmallBlockSamplingPoint,
        }
    }

    /// 对照形态的正确形态（主臂、取样点与 PC4-C 是它自己）。
    fn correct_form(self) -> PowerCutArm {
        match self.candidate() {
            None => self,
            Some(Candidate::RedbQuickRepair) => PowerCutArm::RedbQuickRepair,
            Some(Candidate::RedbDefault) => PowerCutArm::RedbDefault,
            Some(Candidate::RocksDatabase) => PowerCutArm::RocksDatabase,
            Some(Candidate::HardenedFile) => PowerCutArm::HardenedFile,
        }
    }

    /// 登记里这个对照形态与正确形态差的那一处（补 5.2 阳性对照表）；主臂与取样点是 None。
    fn registered_control_difference(self) -> Option<&'static str> {
        match self.purpose() {
            ArmPurpose::LostPageCacheControl => match self.candidate() {
                Some(Candidate::RedbQuickRepair | Candidate::RedbDefault) => Some("redb_commit_durability"),
                Some(Candidate::RocksDatabase) => Some("rocksdb_write_sync"),
                Some(Candidate::HardenedFile) => Some("file_sync_steps"),
                None => None,
            },
            ArmPurpose::MissingDirectorySyncControl => Some("file_sync_steps"),
            ArmPurpose::LostWriteCacheControl => Some("filesystem_barrier"),
            ArmPurpose::Main | ArmPurpose::WriteCacheModelControl | ArmPurpose::SmallBlockSamplingPoint => None,
        }
    }

    fn write_settings(self) -> Option<WriteSettings> {
        let candidate = self.candidate()?;
        let registered = WriteSettings::registered_for(candidate);
        let settings = match self {
            PowerCutArm::RedbQuickRepairDurabilityNone | PowerCutArm::RedbDefaultDurabilityNone => WriteSettings {
                redb_commit_durability: RedbCommitDurability::NotPersisted,
                ..registered
            },
            PowerCutArm::RocksDatabaseWithoutSync => WriteSettings { rocksdb_write_sync: RocksDatabaseWriteSync::NoSync, ..registered },
            PowerCutArm::HardenedFileWithoutFsync => WriteSettings {
                file_sync_steps: FileSyncSteps::NeitherTemporaryFileNorDirectory,
                ..registered
            },
            PowerCutArm::HardenedFileWithoutDirectorySync => {
                WriteSettings { file_sync_steps: FileSyncSteps::TemporaryFileOnly, ..registered }
            }
            PowerCutArm::RedbQuickRepairWithoutBarrier
            | PowerCutArm::RedbDefaultWithoutBarrier
            | PowerCutArm::RocksDatabaseWithoutBarrier
            | PowerCutArm::HardenedFileWithoutBarrier => WriteSettings { filesystem_barrier: FilesystemBarrier::Disabled, ..registered },
            PowerCutArm::RedbQuickRepair
            | PowerCutArm::RedbDefault
            | PowerCutArm::RocksDatabase
            | PowerCutArm::HardenedFile
            | PowerCutArm::RedbQuickRepairSmallBlocks
            | PowerCutArm::RedbDefaultSmallBlocks
            | PowerCutArm::RocksDatabaseSmallBlocks
            | PowerCutArm::HardenedFileSmallBlocks
            | PowerCutArm::WriteCacheModel => registered,
        };
        Some(settings)
    }

    fn states_per_block(self) -> u64 {
        match self.purpose() {
            ArmPurpose::SmallBlockSamplingPoint => SMALL_STATES_PER_BLOCK,
            ArmPurpose::Main
            | ArmPurpose::LostPageCacheControl
            | ArmPurpose::LostWriteCacheControl
            | ArmPurpose::MissingDirectorySyncControl
            | ArmPurpose::WriteCacheModelControl => PRIMARY_STATES_PER_BLOCK,
        }
    }

    fn registered_block_count(self) -> u64 {
        match self.purpose() {
            ArmPurpose::Main | ArmPurpose::SmallBlockSamplingPoint => MAIN_BLOCK_COUNT,
            ArmPurpose::LostPageCacheControl | ArmPurpose::LostWriteCacheControl | ArmPurpose::MissingDirectorySyncControl => {
                CONTROL_BLOCK_COUNT
            }
            ArmPurpose::WriteCacheModelControl => 2 * WRITE_CACHE_MODEL_BATCH_WRITES,
        }
    }

    fn lost_page_cache_control(self) -> PowerCutArm {
        match self.correct_form() {
            PowerCutArm::RedbQuickRepair => PowerCutArm::RedbQuickRepairDurabilityNone,
            PowerCutArm::RedbDefault => PowerCutArm::RedbDefaultDurabilityNone,
            PowerCutArm::RocksDatabase => PowerCutArm::RocksDatabaseWithoutSync,
            other => {
                assert_eq!(other, PowerCutArm::HardenedFile, "PC4-L 只给四条主臂");
                PowerCutArm::HardenedFileWithoutFsync
            }
        }
    }

    fn lost_write_cache_control(self) -> PowerCutArm {
        match self.correct_form() {
            PowerCutArm::RedbQuickRepair => PowerCutArm::RedbQuickRepairWithoutBarrier,
            PowerCutArm::RedbDefault => PowerCutArm::RedbDefaultWithoutBarrier,
            PowerCutArm::RocksDatabase => PowerCutArm::RocksDatabaseWithoutBarrier,
            other => {
                assert_eq!(other, PowerCutArm::HardenedFile, "PC4-NB 只给四条主臂");
                PowerCutArm::HardenedFileWithoutBarrier
            }
        }
    }

    fn small_block_sampling_point(self) -> PowerCutArm {
        match self.correct_form() {
            PowerCutArm::RedbQuickRepair => PowerCutArm::RedbQuickRepairSmallBlocks,
            PowerCutArm::RedbDefault => PowerCutArm::RedbDefaultSmallBlocks,
            PowerCutArm::RocksDatabase => PowerCutArm::RocksDatabaseSmallBlocks,
            other => {
                assert_eq!(other, PowerCutArm::HardenedFile, "S4-bs10 只给四条主臂");
                PowerCutArm::HardenedFileSmallBlocks
            }
        }
    }
}

// ============================== 候选的存储（登记 5.2，照 S1 那份源文件） ==============================

#[derive(Debug)]
enum StoreError {
    Redb(String),
    RocksDatabase(String),
    File(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Redb(message) => write!(formatter, "redb: {message}"),
            StoreError::RocksDatabase(message) => write!(formatter, "rocksdb: {message}"),
            StoreError::File(message) => write!(formatter, "file: {message}"),
        }
    }
}

fn redb_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::Redb(error.to_string())
}

fn rocksdb_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::RocksDatabase(error.to_string())
}

fn file_error(context: &str, error: impl std::fmt::Display) -> StoreError {
    StoreError::File(format!("{context}: {error}"))
}

#[derive(Debug, PartialEq, Eq)]
enum ReadOutcome {
    Found(Vec<u8>),
    NotFound,
    ReadError(String),
}

enum StoreBackend {
    Redb(redb::Database),
    RocksDatabase(rocksdb::DB),
    File(FileLibrary),
}

struct BlockStore {
    settings: WriteSettings,
    backend: StoreBackend,
}

fn rocksdb_options() -> rocksdb::Options {
    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);
    options.set_compression_type(rocksdb::DBCompressionType::Lz4);
    options
}

/// 写入用的打开：库不在就建。
fn open_store_for_writing(candidate: Candidate, settings: WriteSettings, library: &Path) -> Result<BlockStore, StoreError> {
    let backend = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            fs::create_dir_all(library).map_err(|error| file_error("建 redb 库目录", error))?;
            StoreBackend::Redb(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?)
        }
        Candidate::RocksDatabase => StoreBackend::RocksDatabase(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?),
        Candidate::HardenedFile => StoreBackend::File(FileLibrary::open(library, settings.file_sync_steps, true)?),
    };
    Ok(BlockStore { settings, backend })
}

/// 核对用的打开：同一个打开调用（redb 的 `Database::create`、RocksDB 的 `DB::open`），F 不建新库。
fn open_existing_store(candidate: Candidate, library: &Path) -> Result<BlockStore, StoreError> {
    let settings = WriteSettings::registered_for(candidate);
    let backend = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            StoreBackend::Redb(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?)
        }
        Candidate::RocksDatabase => StoreBackend::RocksDatabase(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?),
        Candidate::HardenedFile => StoreBackend::File(FileLibrary::open(library, settings.file_sync_steps, false)?),
    };
    Ok(BlockStore { settings, backend })
}

impl BlockStore {
    /// 一次持久提交写一块；返回 Ok 就算这一块已确认。
    fn put_block(&mut self, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        match &mut self.backend {
            StoreBackend::Redb(database) => {
                let mut transaction = database.begin_write().map_err(redb_error)?;
                let durability = match self.settings.redb_commit_durability {
                    RedbCommitDurability::Immediate => redb::Durability::Immediate,
                    RedbCommitDurability::NotPersisted => redb::Durability::None,
                };
                transaction.set_durability(durability).map_err(redb_error)?;
                transaction.set_quick_repair(self.settings.redb_repair_mode == RedbRepairMode::QuickRepair);
                {
                    let mut table = transaction.open_table(REDB_TABLE).map_err(redb_error)?;
                    table.insert(&key[..], value).map_err(redb_error)?;
                }
                transaction.commit().map_err(redb_error)
            }
            StoreBackend::RocksDatabase(database) => {
                let mut write_options = rocksdb::WriteOptions::default();
                write_options.disable_wal(false);
                write_options.set_sync(self.settings.rocksdb_write_sync == RocksDatabaseWriteSync::SyncEachWrite);
                database.put_opt(key, value, &write_options).map_err(rocksdb_error)
            }
            StoreBackend::File(library) => library.put_block(key, value),
        }
    }

    fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match &self.backend {
            StoreBackend::Redb(database) => {
                let transaction = match database.begin_read() {
                    Ok(transaction) => transaction,
                    Err(error) => return ReadOutcome::ReadError(error.to_string()),
                };
                let table = match transaction.open_table(REDB_TABLE) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => return ReadOutcome::NotFound,
                    Err(error) => return ReadOutcome::ReadError(error.to_string()),
                };
                let outcome = match redb::ReadableTable::get(&table, &key[..]) {
                    Ok(Some(guard)) => ReadOutcome::Found(guard.value().to_vec()),
                    Ok(None) => ReadOutcome::NotFound,
                    Err(error) => ReadOutcome::ReadError(error.to_string()),
                };
                outcome
            }
            StoreBackend::RocksDatabase(database) => match database.get(key) {
                Ok(Some(value)) => ReadOutcome::Found(value),
                Ok(None) => ReadOutcome::NotFound,
                Err(error) => ReadOutcome::ReadError(error.to_string()),
            },
            StoreBackend::File(library) => library.read_block(key),
        }
    }

    fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        match &self.backend {
            StoreBackend::Redb(database) => {
                let transaction = database.begin_read().map_err(redb_error)?;
                let table = match transaction.open_table(REDB_TABLE) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
                    Err(error) => return Err(redb_error(error)),
                };
                let mut entries = Vec::new();
                for item in redb::ReadableTable::iter(&table).map_err(redb_error)? {
                    let (key, value) = item.map_err(redb_error)?;
                    entries.push((key.value().to_vec(), u64::try_from(value.value().len()).expect("块值长度装得进 u64")));
                }
                Ok(entries)
            }
            StoreBackend::RocksDatabase(database) => {
                let mut entries = Vec::new();
                for item in database.iterator(rocksdb::IteratorMode::Start) {
                    let (key, value) = item.map_err(rocksdb_error)?;
                    entries.push((key.to_vec(), u64::try_from(value.len()).expect("块值长度装得进 u64")));
                }
                Ok(entries)
            }
            StoreBackend::File(library) => library.enumerate_keys_with_value_lengths(),
        }
    }
}

/// 一块一个文件：`<根>/<节点 8 位十六进制>/<段号 8 位十六进制>/<起点 16 位十六进制>-<指纹 32 位十六进制>.blk`。
struct FileLibrary {
    root: PathBuf,
    file_sync_steps: FileSyncSteps,
    known_directories: HashSet<PathBuf>,
}

fn sync_directory(directory: &Path) -> Result<(), StoreError> {
    File::open(directory)
        .and_then(|handle| handle.sync_all())
        .map_err(|error| file_error(&format!("目录 fsync {}", directory.display()), error))
}

fn format_file_contents() -> [u8; FORMAT_FILE_BYTES] {
    let mut contents = [0u8; FORMAT_FILE_BYTES];
    contents[0..8].copy_from_slice(FORMAT_MAGIC);
    contents[8..12].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    let checksum = crc32c(&[&contents[0..12]]);
    contents[12..16].copy_from_slice(&checksum.to_le_bytes());
    contents
}

fn block_file_relative_path(key: &[u8]) -> Option<PathBuf> {
    if key.len() != BLOCK_KEY_BYTES {
        return None;
    }
    let node = u32::from_be_bytes(key[0..4].try_into().expect("4 字节"));
    let segment = u32::from_be_bytes(key[4..8].try_into().expect("4 字节"));
    let first_state = u64::from_be_bytes(key[8..16].try_into().expect("8 字节"));
    Some(
        PathBuf::from(format!("{node:08x}"))
            .join(format!("{segment:08x}"))
            .join(format!("{first_state:016x}-{}.blk", hexadecimal(&key[16..32]))),
    )
}

fn temporary_path_of(final_path: &Path) -> PathBuf {
    let mut name = final_path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

fn block_file_header(key: &BlockKey, value: &[u8]) -> [u8; FILE_HEADER_BYTES] {
    let mut header = [0u8; FILE_HEADER_BYTES];
    header[0..4].copy_from_slice(FILE_MAGIC);
    header[4..8].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    header[8..40].copy_from_slice(key);
    let value_length = u32::try_from(value.len()).expect("块值装得进 u32");
    header[40..44].copy_from_slice(&value_length.to_le_bytes());
    let checksum = crc32c(&[&header[..FILE_HEADER_CHECKSUMMED_BYTES], value]);
    header[44..48].copy_from_slice(&checksum.to_le_bytes());
    header
}

fn validate_block_file(bytes: &[u8], key: &BlockKey) -> Result<Vec<u8>, String> {
    if bytes.len() < FILE_HEADER_BYTES {
        return Err(format!("文件只有 {} 字节，短于文件头", bytes.len()));
    }
    if &bytes[0..4] != FILE_MAGIC {
        return Err("魔数不对".to_string());
    }
    if bytes[4..8] != FILE_FORMAT_VERSION.to_le_bytes() {
        return Err("版本不对".to_string());
    }
    if &bytes[8..40] != key {
        return Err("头里的键与文件名不符".to_string());
    }
    let value_length = u32::from_le_bytes(bytes[40..44].try_into().expect("4 字节"));
    let value = &bytes[FILE_HEADER_BYTES..];
    if u64::from(value_length) != u64::try_from(value.len()).expect("装得进 u64") {
        return Err(format!("长度字段 {value_length} 与文件里的 {} 字节块值不符", value.len()));
    }
    let stored_checksum = u32::from_le_bytes(bytes[44..48].try_into().expect("4 字节"));
    if crc32c(&[&bytes[..FILE_HEADER_CHECKSUMMED_BYTES], value]) != stored_checksum {
        return Err("CRC-32C 不对".to_string());
    }
    Ok(value.to_vec())
}

fn hexadecimal_name_as_u32(path: &Path) -> Option<u32> {
    if !path.is_dir() {
        return None;
    }
    let name = path.file_name()?.to_str()?;
    if name.len() != 8 {
        return None;
    }
    u32::from_str_radix(name, 16).ok()
}

fn parse_hexadecimal(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).ok())
        .collect()
}

impl FileLibrary {
    /// 打开：根目录在、FORMAT 在且对，否则打开失败；打开时删掉残留的 `*.tmp`。
    fn open(root: &Path, file_sync_steps: FileSyncSteps, allow_create: bool) -> Result<FileLibrary, StoreError> {
        if !root.exists() {
            if !allow_create {
                return Err(StoreError::File(format!("根目录 {} 不在", root.display())));
            }
            Self::create_root_atomically(root)?;
        }
        let format_bytes = fs::read(root.join(FORMAT_FILE_NAME)).map_err(|error| file_error("读 FORMAT", error))?;
        if format_bytes[..] != format_file_contents()[..] {
            return Err(StoreError::File("FORMAT 不对".to_string()));
        }
        let mut known_directories = HashSet::new();
        for node_entry in fs::read_dir(root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            if !node_path.is_dir() {
                continue;
            }
            known_directories.insert(node_path.clone());
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                if !segment_path.is_dir() {
                    continue;
                }
                known_directories.insert(segment_path.clone());
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_path = block_entry.map_err(|error| file_error("列段目录", error))?.path();
                    if block_path.extension().is_some_and(|extension| extension == "tmp") {
                        fs::remove_file(&block_path).map_err(|error| file_error("删残留的临时文件", error))?;
                    }
                }
            }
        }
        Ok(FileLibrary { root: root.to_path_buf(), file_sync_steps, known_directories })
    }

    fn create_root_atomically(root: &Path) -> Result<(), StoreError> {
        let mut creating_name = root.as_os_str().to_owned();
        creating_name.push(".creating");
        let creating = PathBuf::from(creating_name);
        if creating.exists() {
            fs::remove_dir_all(&creating).map_err(|error| file_error("删上一次没建完的根目录", error))?;
        }
        fs::create_dir_all(&creating).map_err(|error| file_error("建根目录", error))?;
        let mut format_file = File::create(creating.join(FORMAT_FILE_NAME)).map_err(|error| file_error("建 FORMAT", error))?;
        format_file.write_all(&format_file_contents()).map_err(|error| file_error("写 FORMAT", error))?;
        format_file.sync_all().map_err(|error| file_error("fsync FORMAT", error))?;
        sync_directory(&creating)?;
        fs::rename(&creating, root).map_err(|error| file_error("根目录改名", error))?;
        let parent = root.parent().expect("根目录有父目录");
        sync_directory(parent)
    }

    /// 节点与段目录不在就建；除 F-nofsync 外，建完对新目录与它的父目录各 fsync。
    fn ensure_directory(&mut self, directory: &Path) -> Result<(), StoreError> {
        if self.known_directories.contains(directory) {
            return Ok(());
        }
        let parent = directory.parent().expect("目录有父目录").to_path_buf();
        if parent != self.root {
            self.ensure_directory(&parent)?;
        }
        if !directory.exists() {
            fs::create_dir(directory).map_err(|error| file_error("建目录", error))?;
            if self.file_sync_steps != FileSyncSteps::NeitherTemporaryFileNorDirectory {
                sync_directory(directory)?;
                sync_directory(&parent)?;
            }
        }
        self.known_directories.insert(directory.to_path_buf());
        Ok(())
    }

    fn final_path_of(&self, key: &BlockKey) -> PathBuf {
        self.root.join(block_file_relative_path(key).expect("块键 32 字节"))
    }

    /// 登记 5.2：`<终名>.tmp` 一次 `write_all(头 + 块值)` → `sync_all` → `rename` 到终名 → 打开父目录 `sync_all`。
    fn put_block(&mut self, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        let final_path = self.final_path_of(key);
        let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
        self.ensure_directory(&directory)?;
        let temporary_path = temporary_path_of(&final_path);
        let mut contents = Vec::with_capacity(FILE_HEADER_BYTES + value.len());
        contents.extend_from_slice(&block_file_header(key, value));
        contents.extend_from_slice(value);
        let mut handle = File::create(&temporary_path).map_err(|error| file_error("建临时文件", error))?;
        handle.write_all(&contents).map_err(|error| file_error("写临时文件", error))?;
        if self.file_sync_steps != FileSyncSteps::NeitherTemporaryFileNorDirectory {
            handle.sync_all().map_err(|error| file_error("fsync 临时文件", error))?;
        }
        drop(handle);
        fs::rename(&temporary_path, &final_path).map_err(|error| file_error("改名", error))?;
        if self.file_sync_steps == FileSyncSteps::TemporaryFileAndDirectory {
            sync_directory(&directory)?;
        }
        Ok(())
    }

    fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match fs::read(self.final_path_of(key)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::NotFound,
            Err(error) => ReadOutcome::ReadError(error.to_string()),
            Ok(bytes) => match validate_block_file(&bytes, key) {
                Ok(value) => ReadOutcome::Found(value),
                Err(reason) => ReadOutcome::ReadError(reason),
            },
        }
    }

    fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        let mut entries = Vec::new();
        for node_entry in fs::read_dir(&self.root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            let Some(node) = hexadecimal_name_as_u32(&node_path) else { continue };
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                let Some(segment) = hexadecimal_name_as_u32(&segment_path) else { continue };
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_entry = block_entry.map_err(|error| file_error("列段目录", error))?;
                    let name = block_entry.file_name().to_string_lossy().to_string();
                    let Some(stem) = name.strip_suffix(".blk") else { continue };
                    let Some((first_state_text, fingerprint_text)) = stem.split_once('-') else { continue };
                    let (Ok(first_state), Some(fingerprint)) =
                        (u64::from_str_radix(first_state_text, 16), parse_hexadecimal(fingerprint_text))
                    else {
                        continue;
                    };
                    let mut key = Vec::with_capacity(BLOCK_KEY_BYTES);
                    key.extend_from_slice(&node.to_be_bytes());
                    key.extend_from_slice(&segment.to_be_bytes());
                    key.extend_from_slice(&first_state.to_be_bytes());
                    key.extend_from_slice(&fingerprint);
                    let file_length = block_entry.metadata().map_err(|error| file_error("读文件长度", error))?.len();
                    entries.push((key, file_length.saturating_sub(u64::try_from(FILE_HEADER_BYTES).expect("48"))));
                }
            }
        }
        Ok(entries)
    }
}

/// RocksDB 的 `LOG` 里数 L0 落盘与合并各完成了几次（V25 / Q4q）。
fn rocksdb_background_counts(library: &Path) -> (u64, u64) {
    let log_text = fs::read_to_string(library.join("LOG")).unwrap_or_default();
    let flushes = u64::try_from(log_text.matches("\"event\": \"flush_finished\"").count()).expect("装得进 u64");
    let compactions = u64::try_from(log_text.matches("\"event\": \"compaction_finished\"").count()).expect("装得进 u64");
    (flushes, compactions)
}

// ============================== 逐块核（登记 5.1 的表；补 6 Q4a–Q4f） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockClass {
    /// C 标记下标 < k。
    Confirmed,
    /// B 标记下标 < k 而 C 不 < k。
    InFlight,
    /// B 不 < k。
    NotStarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadKind {
    MatchingBytes,
    DifferentBytes,
    NotFound,
    ReadError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockVerdict {
    Correct,
    /// Q4a
    LostConfirmed,
    /// Q4b
    SilentlyCorrupted,
    /// Q4c
    ConfirmedReadError,
    /// Q4d
    UnconfirmedReadError,
    /// Q4e
    Phantom,
}

fn classify_block(class: BlockClass, read: ReadKind) -> BlockVerdict {
    match (class, read) {
        (BlockClass::Confirmed, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::Confirmed, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::Confirmed, ReadKind::NotFound) => BlockVerdict::LostConfirmed,
        (BlockClass::Confirmed, ReadKind::ReadError) => BlockVerdict::ConfirmedReadError,
        (BlockClass::InFlight, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::InFlight, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
        (BlockClass::NotStarted, ReadKind::MatchingBytes | ReadKind::DifferentBytes) => BlockVerdict::Phantom,
        (BlockClass::NotStarted, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::NotStarted, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
    }
}

fn values_match(found: &[u8], expected: &[u8]) -> bool {
    found.len() == expected.len() && found == expected
}

/// 一次核的计划：块按 j 升序逐个写，所以已确认的是前缀 [0, confirmed_end)，在途的是 [confirmed_end, started_end)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VerificationPlan {
    states_per_block: u64,
    block_count: u64,
    confirmed_end: u64,
    started_end: u64,
}

impl VerificationPlan {
    fn class_of(&self, block_index: u64) -> BlockClass {
        if block_index < self.confirmed_end {
            BlockClass::Confirmed
        } else if block_index < self.started_end {
            BlockClass::InFlight
        } else {
            BlockClass::NotStarted
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct VerificationReport {
    open_failed: bool,
    open_error: String,
    /// 只有 K：第一次打开失败之后调 DB::repair 再开，能开就是 Some(true)。
    opened_after_repair: Option<bool>,
    open_microseconds: u64,
    lost_confirmed: Vec<u64>,
    silently_corrupted: Vec<u64>,
    confirmed_read_errors: Vec<u64>,
    unconfirmed_read_errors: Vec<u64>,
    phantom_keys: Vec<String>,
    enumerated_key_count: u64,
    enumeration_error: String,
    first_read_error: String,
}

fn elapsed_microseconds(since: Instant) -> u64 {
    u64::try_from(since.elapsed().as_micros()).expect("微秒数装得进 u64")
}

fn record_open_failure_then_try_repair(
    candidate: Candidate,
    library: &Path,
    open_error: StoreError,
    report: &mut VerificationReport,
) -> Option<BlockStore> {
    report.open_failed = true;
    report.open_error = open_error.to_string();
    match candidate {
        Candidate::RocksDatabase => {
            let repaired = rocksdb::DB::repair(&rocksdb_options(), library)
                .map_err(rocksdb_error)
                .and_then(|()| open_existing_store(candidate, library));
            report.opened_after_repair = Some(repaired.is_ok());
            repaired.ok()
        }
        Candidate::RedbQuickRepair | Candidate::RedbDefault | Candidate::HardenedFile => None,
    }
}

fn verify_library(candidate: Candidate, library: &Path, plan: &VerificationPlan) -> VerificationReport {
    let mut report = VerificationReport::default();
    let open_started = Instant::now();
    let opened = open_existing_store(candidate, library);
    report.open_microseconds = elapsed_microseconds(open_started);
    let store = match opened {
        Ok(store) => store,
        Err(open_error) => match record_open_failure_then_try_repair(candidate, library, open_error, &mut report) {
            Some(repaired) => repaired,
            None => return report,
        },
    };
    let fingerprint = input_fingerprint();
    let mut phantom_keys = std::collections::BTreeSet::new();
    for block_index in 0..plan.block_count {
        let class = plan.class_of(block_index);
        let key = interleaved_block_key(block_index, plan.states_per_block, &fingerprint);
        let read = match store.read_block(&key) {
            ReadOutcome::Found(found) => {
                let expected = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, block_index, plan.states_per_block);
                if values_match(&found, &expected) {
                    ReadKind::MatchingBytes
                } else {
                    ReadKind::DifferentBytes
                }
            }
            ReadOutcome::NotFound => ReadKind::NotFound,
            ReadOutcome::ReadError(reason) => {
                if report.first_read_error.is_empty() {
                    report.first_read_error = format!("block={block_index}:{reason}");
                }
                ReadKind::ReadError
            }
        };
        match classify_block(class, read) {
            BlockVerdict::Correct => {}
            BlockVerdict::LostConfirmed => report.lost_confirmed.push(block_index),
            BlockVerdict::SilentlyCorrupted => report.silently_corrupted.push(block_index),
            BlockVerdict::ConfirmedReadError => report.confirmed_read_errors.push(block_index),
            BlockVerdict::UnconfirmedReadError => report.unconfirmed_read_errors.push(block_index),
            BlockVerdict::Phantom => {
                phantom_keys.insert(hexadecimal(&key));
            }
        }
    }
    let started_keys: HashSet<BlockKey> = (0..plan.started_end)
        .map(|block_index| interleaved_block_key(block_index, plan.states_per_block, &fingerprint))
        .collect();
    match store.enumerate_keys_with_value_lengths() {
        Ok(entries) => {
            for (key, _value_length) in entries {
                report.enumerated_key_count += 1;
                let is_started = <[u8; BLOCK_KEY_BYTES]>::try_from(key.as_slice())
                    .is_ok_and(|fixed_key| started_keys.contains(&fixed_key));
                if !is_started {
                    phantom_keys.insert(hexadecimal(&key));
                }
            }
        }
        Err(error) => report.enumeration_error = error.to_string(),
    }
    report.phantom_keys = phantom_keys.into_iter().collect();
    report
}

/// PC4-S、PC4-P、PC4-O 在「全部条目」重放出的盘面上动的那一处。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryDamage {
    Untouched,
    /// PC4-S：经写入接口把第 7 块改写成「第 40 000 字节异或 0x5A」的值。
    SilentCorruption,
    /// PC4-P：经写入接口加一个计划外的键。
    PhantomKey,
    /// PC4-O：毁掉打开要用的那一处。
    OpenBreaking,
}

impl LibraryDamage {
    fn label(self) -> &'static str {
        match self {
            LibraryDamage::Untouched => "untouched",
            LibraryDamage::SilentCorruption => "silent",
            LibraryDamage::PhantomKey => "phantom",
            LibraryDamage::OpenBreaking => "open-breaking",
        }
    }

    fn parse(text: &str) -> Option<LibraryDamage> {
        [LibraryDamage::Untouched, LibraryDamage::SilentCorruption, LibraryDamage::PhantomKey, LibraryDamage::OpenBreaking]
            .into_iter()
            .find(|damage| damage.label() == text)
    }
}

/// R1、R0 把库文件前 4096 字节写成 0xFF；K 把 CURRENT 换成 `MANIFEST-999999\n`；F 把 FORMAT 的 16 字节写成 0xFF。
fn break_library_opening(candidate: Candidate, library: &Path) -> Result<(), String> {
    let (path, bytes): (PathBuf, Vec<u8>) = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => (library.join(REDB_FILE_NAME), vec![0xFF; REDB_BROKEN_PREFIX_BYTES]),
        Candidate::RocksDatabase => (library.join("CURRENT"), ROCKSDB_BROKEN_CURRENT.to_vec()),
        Candidate::HardenedFile => (library.join(FORMAT_FILE_NAME), vec![0xFF; FORMAT_FILE_BYTES]),
    };
    let truncate = candidate == Candidate::RocksDatabase;
    let mut handle = OpenOptions::new()
        .write(true)
        .truncate(truncate)
        .open(&path)
        .map_err(|error| format!("打不开要毁的 {}：{error}", path.display()))?;
    handle.write_all(&bytes).map_err(|error| format!("写不进要毁的 {}：{error}", path.display()))?;
    handle.sync_all().map_err(|error| format!("fsync 不了要毁的 {}：{error}", path.display()))
}

fn apply_library_damage(candidate: Candidate, library: &Path, damage: LibraryDamage, states_per_block: u64) -> Result<(), String> {
    let fingerprint = input_fingerprint();
    match damage {
        LibraryDamage::Untouched => Ok(()),
        LibraryDamage::SilentCorruption => {
            let mut store = open_existing_store(candidate, library).map_err(|error| format!("PC4-S 开不了库：{error}"))?;
            let mut value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, SILENT_CORRUPTION_BLOCK, states_per_block);
            value[SILENT_CORRUPTION_BYTE_OFFSET] ^= SILENT_CORRUPTION_MASK;
            let key = interleaved_block_key(SILENT_CORRUPTION_BLOCK, states_per_block, &fingerprint);
            store.put_block(&key, &value).map_err(|error| format!("PC4-S 写不进：{error}"))
        }
        LibraryDamage::PhantomKey => {
            let mut store = open_existing_store(candidate, library).map_err(|error| format!("PC4-P 开不了库：{error}"))?;
            let phantom_key = compose_block_key(PHANTOM_KEY_NODE, 0, 0, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 0, states_per_block);
            store.put_block(&phantom_key, &value).map_err(|error| format!("PC4-P 写不进：{error}"))
        }
        LibraryDamage::OpenBreaking => break_library_opening(candidate, library),
    }
}

// ============================== 标记（补 5.2「标记」、补 7 B7） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkerKind {
    Start,
    Begin,
    Confirm,
    End,
}

impl MarkerKind {
    fn byte(self) -> u8 {
        match self {
            MarkerKind::Start => b'S',
            MarkerKind::Begin => b'B',
            MarkerKind::Confirm => b'C',
            MarkerKind::End => b'E',
        }
    }

    fn from_byte(byte: u8) -> Option<MarkerKind> {
        [MarkerKind::Start, MarkerKind::Begin, MarkerKind::Confirm, MarkerKind::End]
            .into_iter()
            .find(|kind| kind.byte() == byte)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Marker {
    kind: MarkerKind,
    sequence: u64,
    /// S、E 写全 1（None）。
    block_index: Option<u64>,
    arm_index: u32,
}

fn encode_marker(marker: &Marker) -> [u8; SECTOR_BYTES] {
    let mut sector = [0u8; SECTOR_BYTES];
    sector[0..4].copy_from_slice(MARKER_MAGIC);
    sector[4] = marker.kind.byte();
    sector[8..16].copy_from_slice(&marker.sequence.to_le_bytes());
    sector[16..24].copy_from_slice(&marker.block_index.unwrap_or(MARKER_WITHOUT_BLOCK).to_le_bytes());
    sector[24..28].copy_from_slice(&marker.arm_index.to_le_bytes());
    let checksum = crc32c(&[&sector[..MARKER_CHECKSUMMED_BYTES]]);
    sector[28..32].copy_from_slice(&checksum.to_le_bytes());
    sector
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MarkerDecoding {
    Valid(Marker),
    NotAMarker,
    ChecksumMismatch,
    Malformed(String),
}

fn decode_marker(sector: &[u8]) -> MarkerDecoding {
    if sector.len() != SECTOR_BYTES || &sector[0..4] != MARKER_MAGIC {
        return MarkerDecoding::NotAMarker;
    }
    let stored_checksum = u32::from_le_bytes(sector[28..32].try_into().expect("4 字节"));
    if crc32c(&[&sector[..MARKER_CHECKSUMMED_BYTES]]) != stored_checksum {
        return MarkerDecoding::ChecksumMismatch;
    }
    let Some(kind) = MarkerKind::from_byte(sector[4]) else {
        return MarkerDecoding::Malformed(format!("种类字节 {:#04x}", sector[4]));
    };
    if sector[5..8].iter().any(|&byte| byte != 0) || sector[32..].iter().any(|&byte| byte != 0) {
        return MarkerDecoding::Malformed("该是 0 的字节不是 0".to_string());
    }
    let sequence = u64::from_le_bytes(sector[8..16].try_into().expect("8 字节"));
    let raw_block = u64::from_le_bytes(sector[16..24].try_into().expect("8 字节"));
    let arm_index = u32::from_le_bytes(sector[24..28].try_into().expect("4 字节"));
    let block_index = match (kind, raw_block == MARKER_WITHOUT_BLOCK) {
        (MarkerKind::Start | MarkerKind::End, true) => None,
        (MarkerKind::Begin | MarkerKind::Confirm, false) => Some(raw_block),
        (MarkerKind::Start | MarkerKind::End, false) | (MarkerKind::Begin | MarkerKind::Confirm, true) => {
            return MarkerDecoding::Malformed(format!("种类 {:?} 配块号 {raw_block}", kind));
        }
    };
    MarkerDecoding::Valid(Marker { kind, sequence, block_index, arm_index })
}

/// O_DIRECT 要按 4 KiB 对齐的缓冲；从一段稍大的 Vec 里切出对齐的 512 字节。
struct AlignedSector {
    storage: Vec<u8>,
    aligned_offset: usize,
}

impl AlignedSector {
    fn new() -> AlignedSector {
        let storage = vec![0u8; SECTOR_BYTES + DIRECT_WRITE_ALIGNMENT_BYTES];
        let aligned_offset = storage.as_ptr().align_offset(DIRECT_WRITE_ALIGNMENT_BYTES);
        assert!(aligned_offset + SECTOR_BYTES <= storage.len(), "对齐之后还放得下一个扇区");
        AlignedSector { storage, aligned_offset }
    }

    fn fill(&mut self, sector: &[u8; SECTOR_BYTES]) -> &[u8] {
        let aligned = &mut self.storage[self.aligned_offset..self.aligned_offset + SECTOR_BYTES];
        aligned.copy_from_slice(sector);
        aligned
    }
}

/// 单测里记下的写入循环事件次序（补 9 M21）。
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteLoopEvent {
    Marker(Marker),
    WriteCallStarted { block_index: u64 },
    WriteCallReturned { block_index: u64 },
}

/// 标记发到哪里：来宾里写进 vda2 的第 c 个扇区（O_DIRECT）；单测里记下事件次序（补 9 M21，只在单测里有）。
enum MarkerSink {
    MarkerPartition { device: File, aligned: AlignedSector, next_sequence: u64, arm_index: u32 },
    #[cfg(test)]
    Recording { events: Vec<WriteLoopEvent>, next_sequence: u64, arm_index: u32 },
}

impl MarkerSink {
    fn open_marker_partition(path: &str, arm_index: u32) -> Result<MarkerSink, String> {
        let device = OpenOptions::new()
            .write(true)
            .custom_flags(OPEN_DIRECT_FLAG)
            .open(path)
            .map_err(|error| format!("以 O_DIRECT 打不开 {path}：{error}"))?;
        Ok(MarkerSink::MarkerPartition { device, aligned: AlignedSector::new(), next_sequence: 0, arm_index })
    }

    #[cfg(test)]
    fn recording(arm_index: u32) -> MarkerSink {
        MarkerSink::Recording { events: Vec::new(), next_sequence: 0, arm_index }
    }

    fn emit(&mut self, kind: MarkerKind, block_index: Option<u64>) -> Result<(), String> {
        match self {
            MarkerSink::MarkerPartition { device, aligned, next_sequence, arm_index } => {
                let marker = Marker { kind, sequence: *next_sequence, block_index, arm_index: *arm_index };
                let sector = encode_marker(&marker);
                let offset = *next_sequence * sector_bytes();
                device
                    .write_all_at(aligned.fill(&sector), offset)
                    .map_err(|error| format!("第 {} 个标记写不进：{error}", *next_sequence))?;
                *next_sequence += 1;
                Ok(())
            }
            #[cfg(test)]
            MarkerSink::Recording { events, next_sequence, arm_index } => {
                events.push(WriteLoopEvent::Marker(Marker { kind, sequence: *next_sequence, block_index, arm_index: *arm_index }));
                *next_sequence += 1;
                Ok(())
            }
        }
    }

    fn note_write_call_started(&mut self, block_index: u64) {
        match self {
            MarkerSink::MarkerPartition { .. } => {
                let _marker_partition_does_not_record = block_index;
            }
            #[cfg(test)]
            MarkerSink::Recording { events, .. } => events.push(WriteLoopEvent::WriteCallStarted { block_index }),
        }
    }

    fn note_write_call_returned(&mut self, block_index: u64) {
        match self {
            MarkerSink::MarkerPartition { .. } => {
                let _marker_partition_does_not_record = block_index;
            }
            #[cfg(test)]
            MarkerSink::Recording { events, .. } => events.push(WriteLoopEvent::WriteCallReturned { block_index }),
        }
    }

    fn emitted_marker_count(&self) -> u64 {
        match self {
            MarkerSink::MarkerPartition { next_sequence, .. } => *next_sequence,
            #[cfg(test)]
            MarkerSink::Recording { next_sequence, .. } => *next_sequence,
        }
    }
}

/// 写块的那一个线程：开始写之前一个 S；每块写入调用之前一个 B，返回 Ok 之后一个 C；全部写完一个 E。返回已确认的块数。
fn write_blocks_with_markers(store: &mut BlockStore, sink: &mut MarkerSink, block_count: u64, states_per_block: u64) -> Result<u64, String> {
    let fingerprint = input_fingerprint();
    sink.emit(MarkerKind::Start, None)?;
    let mut confirmed_blocks = 0u64;
    for block_index in 0..block_count {
        let key = interleaved_block_key(block_index, states_per_block, &fingerprint);
        let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, block_index, states_per_block);
        sink.emit(MarkerKind::Begin, Some(block_index))?;
        sink.note_write_call_started(block_index);
        let write_outcome = store.put_block(&key, &value);
        sink.note_write_call_returned(block_index);
        write_outcome.map_err(|error| format!("第 {block_index} 块写入失败：{error}"))?;
        sink.emit(MarkerKind::Confirm, Some(block_index))?;
        confirmed_blocks += 1;
    }
    sink.emit(MarkerKind::End, None)?;
    Ok(confirmed_blocks)
}

// ============================== 设备侧日志（dm-log-writes 格式，补 5.2「设备侧日志的读法」） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogEntry {
    Write { first_sector: u64, sector_count: u64, is_forced_unit_access: bool, data_offset_in_log: u64 },
    Flush,
    Discard { first_sector: u64, sector_count: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeviceLog {
    /// 系统配置声明的条目数；系统配置全 0 时是 None。
    declared_entry_count: Option<u64>,
    entries: Vec<LogEntry>,
    /// 最后一个条目（连同数据）之后的字节偏移：「截到最后一个条目」截到这里。
    end_offset_in_log: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeviceLogError {
    BadMagic { found: u64 },
    UnknownVersion { found: u64 },
    UnsupportedSectorSize { found: u64 },
    TruncatedEntry { index: usize },
    UnknownFlags { index: usize, flags: u64 },
    UnsupportedFlagCombination { index: usize, flags: u64, sector_count: u64 },
    ReadFailed(String),
}

impl std::fmt::Display for DeviceLogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// 条目数怎么数：照系统配置声明的封顶（判定用），或读到全 0 的条目头为止（只给 H9 的诊断行用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclaredCountHandling {
    StopAtDeclared,
    ReadUntilZeroHeader,
}

/// 日志字节在哪：单测与锚点在内存里，宿主与来宾在文件里（按偏移读，不整份读进内存）。
enum LogBytes {
    InMemory(Vec<u8>),
    OnDisk { file: File, length: u64 },
}

impl LogBytes {
    fn open(path: &Path) -> Result<LogBytes, String> {
        let file = File::open(path).map_err(|error| format!("打不开日志 {}：{error}", path.display()))?;
        let length = file.metadata().map_err(|error| format!("读不到日志长度：{error}"))?.len();
        Ok(LogBytes::OnDisk { file, length })
    }

    fn length(&self) -> u64 {
        match self {
            LogBytes::InMemory(bytes) => u64::try_from(bytes.len()).expect("装得进 u64"),
            LogBytes::OnDisk { length, .. } => *length,
        }
    }

    fn read_exact_at(&self, buffer: &mut [u8], offset: u64) -> Result<(), String> {
        match self {
            LogBytes::InMemory(bytes) => {
                let start = usize::try_from(offset).expect("偏移装得进 usize");
                let end = start + buffer.len();
                if end > bytes.len() {
                    return Err(format!("读 [{start}, {end}) 越过日志末尾 {}", bytes.len()));
                }
                buffer.copy_from_slice(&bytes[start..end]);
                Ok(())
            }
            LogBytes::OnDisk { file, .. } => file.read_exact_at(buffer, offset).map_err(|error| format!("读日志偏移 {offset} 失败：{error}")),
        }
    }
}

fn parse_device_log(bytes: &LogBytes, handling: DeclaredCountHandling) -> Result<DeviceLog, DeviceLogError> {
    let length = bytes.length();
    let mut super_bytes = [0u8; LOG_SUPER_BYTES];
    let declared_entry_count = if length >= u64::try_from(LOG_SUPER_BYTES).expect("28") {
        bytes.read_exact_at(&mut super_bytes, 0).map_err(DeviceLogError::ReadFailed)?;
        if super_bytes.iter().all(|&byte| byte == 0) {
            None
        } else {
            let magic = u64::from_le_bytes(super_bytes[0..8].try_into().expect("8 字节"));
            if magic != WRITE_LOG_MAGIC {
                return Err(DeviceLogError::BadMagic { found: magic });
            }
            let version = u64::from_le_bytes(super_bytes[8..16].try_into().expect("8 字节"));
            if version != WRITE_LOG_VERSION {
                return Err(DeviceLogError::UnknownVersion { found: version });
            }
            let declared = u64::from_le_bytes(super_bytes[16..24].try_into().expect("8 字节"));
            let sector_size = u64::from(u32::from_le_bytes(super_bytes[24..28].try_into().expect("4 字节")));
            if sector_size != sector_bytes() {
                return Err(DeviceLogError::UnsupportedSectorSize { found: sector_size });
            }
            Some(declared)
        }
    } else {
        None
    };
    let header_bytes = u64::try_from(LOG_ENTRY_HEADER_BYTES).expect("32");
    let known_flags = LOG_FLUSH_FLAG | LOG_FUA_FLAG | LOG_DISCARD_FLAG | LOG_MARK_FLAG | LOG_METADATA_FLAG;
    let mut entries = Vec::new();
    let mut offset = sector_bytes();
    loop {
        if offset + header_bytes > length {
            break;
        }
        let mut header = [0u8; LOG_ENTRY_HEADER_BYTES];
        bytes.read_exact_at(&mut header, offset).map_err(DeviceLogError::ReadFailed)?;
        if header.iter().all(|&byte| byte == 0) {
            break;
        }
        let parsed_count = u64::try_from(entries.len()).expect("条目数装得进 u64");
        if handling == DeclaredCountHandling::StopAtDeclared && declared_entry_count.is_some_and(|declared| parsed_count >= declared) {
            break;
        }
        let index = entries.len();
        let first_sector = u64::from_le_bytes(header[0..8].try_into().expect("8 字节"));
        let sector_count = u64::from_le_bytes(header[8..16].try_into().expect("8 字节"));
        let flags = u64::from_le_bytes(header[16..24].try_into().expect("8 字节"));
        if flags & !known_flags != 0 || flags & (LOG_MARK_FLAG | LOG_METADATA_FLAG) != 0 {
            return Err(DeviceLogError::UnknownFlags { index, flags });
        }
        offset += sector_bytes();
        if flags & LOG_DISCARD_FLAG != 0 {
            if flags != LOG_DISCARD_FLAG {
                return Err(DeviceLogError::UnsupportedFlagCombination { index, flags, sector_count });
            }
            entries.push(LogEntry::Discard { first_sector, sector_count });
            continue;
        }
        if flags & LOG_FLUSH_FLAG != 0 {
            if sector_count != 0 || flags != LOG_FLUSH_FLAG {
                return Err(DeviceLogError::UnsupportedFlagCombination { index, flags, sector_count });
            }
            entries.push(LogEntry::Flush);
            continue;
        }
        if sector_count == 0 {
            return Err(DeviceLogError::UnsupportedFlagCombination { index, flags, sector_count });
        }
        let data_bytes = sector_count * sector_bytes();
        if offset + data_bytes > length {
            return Err(DeviceLogError::TruncatedEntry { index });
        }
        entries.push(LogEntry::Write {
            first_sector,
            sector_count,
            is_forced_unit_access: flags & LOG_FUA_FLAG != 0,
            data_offset_in_log: offset,
        });
        offset += data_bytes;
    }
    Ok(DeviceLog { declared_entry_count, entries, end_offset_in_log: offset })
}

/// 合成一份日志（锚点 B8 与单测用）：系统配置声明 declared 个条目，条目依次是（起始扇区，扇区数，标志，数据）。
fn synthetic_log_bytes(entries: &[(u64, u64, u64, Vec<u8>)], declared: u64) -> Vec<u8> {
    let mut log = Vec::new();
    log.extend_from_slice(&WRITE_LOG_MAGIC.to_le_bytes());
    log.extend_from_slice(&WRITE_LOG_VERSION.to_le_bytes());
    log.extend_from_slice(&declared.to_le_bytes());
    log.extend_from_slice(&u32::try_from(SECTOR_BYTES).expect("512").to_le_bytes());
    log.resize(SECTOR_BYTES, 0);
    for (first_sector, sector_count, flags, data) in entries {
        let header_start = log.len();
        log.extend_from_slice(&first_sector.to_le_bytes());
        log.extend_from_slice(&sector_count.to_le_bytes());
        log.extend_from_slice(&flags.to_le_bytes());
        log.extend_from_slice(&0u64.to_le_bytes());
        log.resize(header_start + SECTOR_BYTES, 0);
        log.extend_from_slice(data);
    }
    log
}

// ============================== 断电盘面的四种取法（补 5.2、补 7 B3、B8） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplayVariant {
    /// U-drop（判定）：最后一个 FLUSH 之前的写全落，U(k) 全丢。
    DropUnflushed,
    /// U-keep：前 k 个条目的写全落。
    KeepUnflushed,
    /// U-rand：U(k) 里每个写按 B3 的生成器各自留或丢。
    RandomSubset,
    /// U-tear：同 U-rand，另把留下的某一个多扇区写截成前 t 个扇区。
    RandomSubsetWithTear,
}

const ALL_REPLAY_VARIANTS: [ReplayVariant; 4] =
    [ReplayVariant::DropUnflushed, ReplayVariant::KeepUnflushed, ReplayVariant::RandomSubset, ReplayVariant::RandomSubsetWithTear];

impl ReplayVariant {
    fn label(self) -> &'static str {
        match self {
            ReplayVariant::DropUnflushed => "U-drop",
            ReplayVariant::KeepUnflushed => "U-keep",
            ReplayVariant::RandomSubset => "U-rand",
            ReplayVariant::RandomSubsetWithTear => "U-tear",
        }
    }

    fn parse(text: &str) -> Option<ReplayVariant> {
        ALL_REPLAY_VARIANTS.into_iter().find(|variant| variant.label() == text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TornWrite {
    position_in_unflushed: usize,
    sectors_kept: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnflushedSubset {
    kept: Vec<bool>,
    torn: Option<TornWrite>,
}

/// 补 7 B3：状态 = seed_SUB XOR (((a << 32) | i) × 乘子)；U(k) 里每个写按日志次序取一个输出、最高位为 1 留；
/// 再在留下的多扇区写里取第 `输出 mod 个数` 个，截成前 `1 + 输出 mod (扇区数 − 1)` 个扇区。
fn unflushed_subset(arm_index: u64, subset_index: u64, unflushed_sector_counts: &[u64]) -> UnflushedSubset {
    let mut generator = SplitMix64::new(SEED_SUBSET ^ ((arm_index << 32) | subset_index).wrapping_mul(MIXING_MULTIPLIER));
    let kept: Vec<bool> = unflushed_sector_counts.iter().map(|_sector_count| generator.next_output() >> 63 == 1).collect();
    let multi_sector_kept: Vec<usize> = kept
        .iter()
        .zip(unflushed_sector_counts)
        .enumerate()
        .filter(|(_position, (is_kept, sector_count))| **is_kept && **sector_count >= 2)
        .map(|(position, _pair)| position)
        .collect();
    let torn = if multi_sector_kept.is_empty() {
        None
    } else {
        let choices = u64::try_from(multi_sector_kept.len()).expect("装得进 u64");
        let chosen = multi_sector_kept[usize::try_from(generator.next_output() % choices).expect("小于个数")];
        let sectors_kept = 1 + generator.next_output() % (unflushed_sector_counts[chosen] - 1);
        Some(TornWrite { position_in_unflushed: chosen, sectors_kept })
    };
    UnflushedSubset { kept, torn }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReplayedWrite {
    first_sector: u64,
    sectors_applied: u64,
    data_offset_in_log: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplaySelection {
    last_flush_entry: Option<usize>,
    /// U(k)：最后一个 FLUSH 之后、k 之前、不带 FUA 的写条目下标。
    unflushed_entries: Vec<usize>,
    subset: UnflushedSubset,
    writes: Vec<ReplayedWrite>,
}

/// 断电点 point（日志前 point 个条目已送到虚拟盘）上按 variant 取的写，按日志次序（后写的盖前写的）。
/// 带 FUA 的写只算它自己收到即持久，不进 U(k)。
fn select_replayed_writes(entries: &[LogEntry], point: usize, variant: ReplayVariant, arm_index: u64, subset_index: u64) -> ReplaySelection {
    assert!(point <= entries.len(), "断电点 {point} 越过条目数 {}", entries.len());
    let last_flush_entry = entries[..point].iter().rposition(|entry| *entry == LogEntry::Flush);
    let mut unflushed_entries = Vec::new();
    let mut unflushed_sector_counts = Vec::new();
    for (entry_index, entry) in entries[..point].iter().enumerate() {
        if let LogEntry::Write { sector_count, is_forced_unit_access: false, .. } = *entry {
            if last_flush_entry.is_none_or(|flush| entry_index > flush) {
                unflushed_entries.push(entry_index);
                unflushed_sector_counts.push(sector_count);
            }
        }
    }
    let subset = unflushed_subset(arm_index, subset_index, &unflushed_sector_counts);
    let mut writes = Vec::new();
    let mut unflushed_position = 0usize;
    for (entry_index, entry) in entries[..point].iter().enumerate() {
        let LogEntry::Write { first_sector, sector_count, is_forced_unit_access, data_offset_in_log } = *entry else {
            continue;
        };
        let is_unflushed = last_flush_entry.is_none_or(|flush| entry_index > flush) && !is_forced_unit_access;
        let sectors_applied = if is_unflushed {
            let position = unflushed_position;
            unflushed_position += 1;
            let is_kept = match variant {
                ReplayVariant::DropUnflushed => false,
                ReplayVariant::KeepUnflushed => true,
                ReplayVariant::RandomSubset | ReplayVariant::RandomSubsetWithTear => subset.kept[position],
            };
            if !is_kept {
                continue;
            }
            let torn_here = variant == ReplayVariant::RandomSubsetWithTear
                && subset.torn.is_some_and(|torn| torn.position_in_unflushed == position);
            if torn_here {
                subset.torn.map_or(sector_count, |torn| torn.sectors_kept)
            } else {
                sector_count
            }
        } else {
            sector_count
        };
        writes.push(ReplayedWrite { first_sector, sectors_applied, data_offset_in_log });
    }
    ReplaySelection { last_flush_entry, unflushed_entries, subset, writes }
}

/// U(k) 里有几个写（Q4u 用）：从 point 往回数到最后一个 FLUSH。
fn unflushed_write_count(entries: &[LogEntry], point: usize) -> usize {
    let mut count = 0usize;
    for entry in entries[..point].iter().rev() {
        match entry {
            LogEntry::Flush => break,
            LogEntry::Write { is_forced_unit_access: false, .. } => count += 1,
            LogEntry::Write { is_forced_unit_access: true, .. } | LogEntry::Discard { .. } => {}
        }
    }
    count
}

/// 盘面写到哪：锚点与单测在内存里，宿主与来宾在（稀疏）文件里。
enum ReplayImage {
    InMemory(Vec<u8>),
    OnDisk(File),
}

fn apply_replayed_writes(log: &LogBytes, writes: &[ReplayedWrite], image: &mut ReplayImage) -> Result<(), String> {
    let mut buffer = Vec::new();
    for write in writes {
        let byte_count = usize::try_from(write.sectors_applied * sector_bytes()).expect("装得进 usize");
        buffer.resize(byte_count, 0);
        log.read_exact_at(&mut buffer, write.data_offset_in_log)?;
        let image_offset = write.first_sector * sector_bytes();
        match image {
            ReplayImage::InMemory(bytes) => {
                let start = usize::try_from(image_offset).expect("装得进 usize");
                let end = start + byte_count;
                if end > bytes.len() {
                    return Err(format!("写 [{start}, {end}) 越过镜像末尾 {}", bytes.len()));
                }
                bytes[start..end].copy_from_slice(&buffer);
            }
            ReplayImage::OnDisk(file) => file
                .write_all_at(&buffer, image_offset)
                .map_err(|error| format!("镜像偏移 {image_offset} 写不进：{error}"))?,
        }
    }
    Ok(())
}

/// 只取镜像里 [first_sector, first_sector + sector_count) 这一截（PC4-C 与标记核对用，不造整盘）。
fn replayed_sector_range(log: &LogBytes, writes: &[ReplayedWrite], first_sector: u64, sector_count: u64) -> Result<Vec<u8>, String> {
    let range_end = first_sector + sector_count;
    let mut window = vec![0u8; usize::try_from(sector_count * sector_bytes()).expect("装得进 usize")];
    let mut sector = [0u8; SECTOR_BYTES];
    for write in writes {
        let write_end = write.first_sector + write.sectors_applied;
        let overlap_start = write.first_sector.max(first_sector);
        let overlap_end = write_end.min(range_end);
        for disk_sector in overlap_start..overlap_end {
            let offset_in_write = (disk_sector - write.first_sector) * sector_bytes();
            log.read_exact_at(&mut sector, write.data_offset_in_log + offset_in_write)?;
            let window_start = usize::try_from((disk_sector - first_sector) * sector_bytes()).expect("装得进 usize");
            window[window_start..window_start + SECTOR_BYTES].copy_from_slice(&sector);
        }
    }
    Ok(window)
}

// ============================== PC4-C（补 5.2、补 7 B4；V17） ==============================

#[derive(Debug, Clone, PartialEq, Eq)]
struct WriteCacheModelJudgement {
    point: usize,
    entries_between_batches: usize,
    flushes_between_batches: usize,
    /// 每种取法：（前 100 个丢几个，后 100 个丢几个）。
    lost_by_variant: Vec<(ReplayVariant, u64, u64)>,
    registered_counts_hold: bool,
}

fn judge_write_cache_model(log: &LogBytes, device_log: &DeviceLog) -> Result<WriteCacheModelJudgement, String> {
    let marker_count = 2 * WRITE_CACHE_MODEL_BATCH_WRITES;
    let range_end = WRITE_CACHE_MODEL_FIRST_SECTOR + marker_count;
    let mut marker_entries: Vec<Option<usize>> = vec![None; usize::try_from(marker_count).expect("200")];
    for (entry_index, entry) in device_log.entries.iter().enumerate() {
        let LogEntry::Write { first_sector, sector_count, .. } = *entry else { continue };
        for disk_sector in first_sector.max(WRITE_CACHE_MODEL_FIRST_SECTOR)..(first_sector + sector_count).min(range_end) {
            let slot = usize::try_from(disk_sector - WRITE_CACHE_MODEL_FIRST_SECTOR).expect("小于 200");
            if marker_entries[slot].is_none() {
                marker_entries[slot] = Some(entry_index);
            }
        }
    }
    let Some(entries_of_markers) = marker_entries.into_iter().collect::<Option<Vec<usize>>>() else {
        return Err("日志里找不齐 PC4-C 的 200 个扇区写".to_string());
    };
    let batch = usize::try_from(WRITE_CACHE_MODEL_BATCH_WRITES).expect("100");
    let last_of_first_batch = entries_of_markers[..batch].iter().copied().max().expect("100 个");
    let first_of_second_batch = entries_of_markers[batch..].iter().copied().min().expect("100 个");
    let point = entries_of_markers.iter().copied().max().expect("200 个") + 1;
    let (entries_between_batches, flushes_between_batches) = if first_of_second_batch > last_of_first_batch {
        let between = &device_log.entries[last_of_first_batch + 1..first_of_second_batch];
        (between.len(), between.iter().filter(|entry| **entry == LogEntry::Flush).count())
    } else {
        (0, 0)
    };
    let arm_index = u64::from(PowerCutArm::WriteCacheModel.index());
    let mut lost_by_variant = Vec::new();
    for variant in ALL_REPLAY_VARIANTS {
        let selection = select_replayed_writes(&device_log.entries, point, variant, arm_index, 0);
        let window = replayed_sector_range(log, &selection.writes, WRITE_CACHE_MODEL_FIRST_SECTOR, marker_count)?;
        let mut first_lost = 0u64;
        let mut second_lost = 0u64;
        for position in 0..marker_count {
            let start = usize::try_from(position * sector_bytes()).expect("装得进 usize");
            let survived = matches!(
                decode_marker(&window[start..start + SECTOR_BYTES]),
                MarkerDecoding::Valid(Marker { kind: MarkerKind::Confirm, sequence, .. }) if sequence == position
            );
            if !survived {
                if position < WRITE_CACHE_MODEL_BATCH_WRITES {
                    first_lost += 1;
                } else {
                    second_lost += 1;
                }
            }
        }
        lost_by_variant.push((variant, first_lost, second_lost));
    }
    let registered_counts_hold = entries_between_batches == 1
        && flushes_between_batches == 1
        && lost_by_variant.iter().all(|(variant, first_lost, second_lost)| {
            *first_lost == 0
                && match variant {
                    ReplayVariant::DropUnflushed => *second_lost == WRITE_CACHE_MODEL_BATCH_WRITES,
                    ReplayVariant::KeepUnflushed => *second_lost == 0,
                    ReplayVariant::RandomSubset => *second_lost == WRITE_CACHE_MODEL_RANDOM_SUBSET_LOST,
                    ReplayVariant::RandomSubsetWithTear => true,
                }
        });
    Ok(WriteCacheModelJudgement { point, entries_between_batches, flushes_between_batches, lost_by_variant, registered_counts_hold })
}

/// 照 PC4-C 的写流合成日志：前 100 个 C 标记、一个 FLUSH、后 100 个 C 标记（单测与补 9 M16、M17 用）。
fn synthetic_write_cache_model_log() -> Vec<u8> {
    let mut entries = Vec::new();
    for position in 0..2 * WRITE_CACHE_MODEL_BATCH_WRITES {
        if position == WRITE_CACHE_MODEL_BATCH_WRITES {
            entries.push((0, 0, LOG_FLUSH_FLAG, Vec::new()));
        }
        let marker = Marker {
            kind: MarkerKind::Confirm,
            sequence: position,
            block_index: Some(position),
            arm_index: PowerCutArm::WriteCacheModel.index(),
        };
        entries.push((WRITE_CACHE_MODEL_FIRST_SECTOR + position, 1, 0, encode_marker(&marker).to_vec()));
    }
    let declared = u64::try_from(entries.len()).expect("装得进 u64");
    synthetic_log_bytes(&entries, declared)
}

// ============================== 标记核对（V22）与分桶（补 5.2、补 7 B2、B5） ==============================

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkerScan {
    /// （条目下标，标记），按日志次序。
    markers: Vec<(usize, Marker)>,
    checksum_failures: u64,
    malformed: u64,
    misplaced: u64,
}

fn scan_markers(log: &LogBytes, device_log: &DeviceLog) -> Result<MarkerScan, String> {
    let partition_start = marker_partition_start_sector();
    let partition_end = partition_start + marker_partition_sectors();
    let mut scan = MarkerScan { markers: Vec::new(), checksum_failures: 0, malformed: 0, misplaced: 0 };
    let mut sector = [0u8; SECTOR_BYTES];
    for (entry_index, entry) in device_log.entries.iter().enumerate() {
        let LogEntry::Write { first_sector, sector_count, data_offset_in_log, .. } = *entry else { continue };
        for disk_sector in first_sector.max(partition_start)..(first_sector + sector_count).min(partition_end) {
            log.read_exact_at(&mut sector, data_offset_in_log + (disk_sector - first_sector) * sector_bytes())?;
            match decode_marker(&sector) {
                MarkerDecoding::Valid(marker) => {
                    if marker.sequence != disk_sector - partition_start {
                        scan.misplaced += 1;
                    }
                    scan.markers.push((entry_index, marker));
                }
                MarkerDecoding::ChecksumMismatch => scan.checksum_failures += 1,
                MarkerDecoding::NotAMarker | MarkerDecoding::Malformed(_) => scan.malformed += 1,
            }
        }
    }
    Ok(scan)
}

/// 标记在日志上的位置：S、E 与每块的 B_j、C_j 的条目下标。
#[derive(Debug, Clone, PartialEq, Eq)]
struct CommitWindows {
    start_entry: usize,
    end_entry: usize,
    begin_entries: Vec<usize>,
    confirm_entries: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkerAudit {
    problems: Vec<String>,
    confirm_count: u64,
    windows: Option<CommitWindows>,
}

/// V22：c 在日志里从 0 起逐个加 1；次序是 S、（B_j、C_j）…、E；C 标记个数与来宾报的已确认块数相等；CRC 都对。
fn audit_markers(scan: &MarkerScan, arm_index: u32, guest_confirmed_blocks: u64) -> MarkerAudit {
    let mut problems = Vec::new();
    if scan.checksum_failures > 0 {
        problems.push(format!("checksum_failures={}", scan.checksum_failures));
    }
    if scan.malformed > 0 {
        problems.push(format!("malformed={}", scan.malformed));
    }
    if scan.misplaced > 0 {
        problems.push(format!("misplaced={}", scan.misplaced));
    }
    let mut start_entry = None;
    let mut end_entry = None;
    let mut begin_entries = Vec::new();
    let mut confirm_entries = Vec::new();
    for (position, (entry_index, marker)) in scan.markers.iter().enumerate() {
        if marker.sequence != u64::try_from(position).expect("装得进 u64") {
            problems.push(format!("sequence_gap_at={position}"));
            break;
        }
        if marker.arm_index != arm_index {
            problems.push(format!("arm_mismatch_at={position}"));
            break;
        }
        let expected_block = u64::try_from(confirm_entries.len()).expect("装得进 u64");
        let in_order = match marker.kind {
            MarkerKind::Start => position == 0,
            MarkerKind::Begin => {
                start_entry.is_some() && end_entry.is_none() && begin_entries.len() == confirm_entries.len() && marker.block_index == Some(expected_block)
            }
            MarkerKind::Confirm => {
                begin_entries.len() == confirm_entries.len() + 1 && marker.block_index == Some(expected_block)
            }
            MarkerKind::End => start_entry.is_some() && end_entry.is_none() && begin_entries.len() == confirm_entries.len(),
        };
        if !in_order {
            problems.push(format!("out_of_order_at={position}:{:?}", marker.kind));
            break;
        }
        match marker.kind {
            MarkerKind::Start => start_entry = Some(*entry_index),
            MarkerKind::Begin => begin_entries.push(*entry_index),
            MarkerKind::Confirm => confirm_entries.push(*entry_index),
            MarkerKind::End => end_entry = Some(*entry_index),
        }
    }
    let confirm_count = u64::try_from(confirm_entries.len()).expect("装得进 u64");
    if start_entry.is_none() {
        problems.push("missing_start".to_string());
    }
    if end_entry.is_none() {
        problems.push("missing_end".to_string());
    }
    if confirm_count != guest_confirmed_blocks {
        problems.push(format!("confirm_markers={confirm_count}_guest_confirmed={guest_confirmed_blocks}"));
    }
    let windows = match (start_entry, end_entry, problems.is_empty()) {
        (Some(start), Some(end), true) => Some(CommitWindows { start_entry: start, end_entry: end, begin_entries, confirm_entries }),
        (Some(_) | None, Some(_) | None, true | false) => None,
    };
    MarkerAudit { problems, confirm_count, windows }
}

/// 断电点 point 上：已确认 = C 标记下标 < point 的块数；已开始 = B 标记下标 < point 的块数。
fn block_classes_at(windows: &CommitWindows, point: usize) -> (u64, u64) {
    let confirmed_end = windows.confirm_entries.iter().filter(|&&entry| entry < point).count();
    let started_end = windows.begin_entries.iter().filter(|&&entry| entry < point).count();
    (u64::try_from(confirmed_end).expect("装得进 u64"), u64::try_from(started_end).expect("装得进 u64"))
}

fn sorted_percentile(sorted: &[u64], numerator: usize, denominator: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (sorted.len() * numerator).div_ceil(denominator).max(1);
    sorted[rank - 1]
}

fn median_of(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted_percentile(&sorted, 1, 2)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BucketPools {
    /// 桶 A：普通窗的 (B_j, C_j] 里全部条目边界。
    normal_commit_points: Vec<usize>,
    /// 桶 B：长窗的 (B_j, C_j]。
    long_commit_points: Vec<usize>,
    /// 桶 C：(S, B_0]、(C_j, B_{j+1}]、(C_末, E]。
    between_commit_points: Vec<usize>,
    median_window_entries: u64,
    normal_blocks: Vec<u64>,
    long_blocks: Vec<u64>,
}

fn bucket_pools(windows: &CommitWindows) -> BucketPools {
    let window_lengths: Vec<u64> = windows
        .begin_entries
        .iter()
        .zip(&windows.confirm_entries)
        .map(|(begin, confirm)| u64::try_from(confirm - begin).expect("装得进 u64"))
        .collect();
    let median_window_entries = median_of(&window_lengths);
    let mut pools = BucketPools {
        normal_commit_points: Vec::new(),
        long_commit_points: Vec::new(),
        between_commit_points: Vec::new(),
        median_window_entries,
        normal_blocks: Vec::new(),
        long_blocks: Vec::new(),
    };
    for (block_position, (&begin, &confirm)) in windows.begin_entries.iter().zip(&windows.confirm_entries).enumerate() {
        let block_index = u64::try_from(block_position).expect("装得进 u64");
        if window_lengths[block_position] > LONG_WINDOW_MEDIAN_FACTOR * median_window_entries {
            pools.long_blocks.push(block_index);
            pools.long_commit_points.extend(begin + 1..=confirm);
        } else {
            pools.normal_blocks.push(block_index);
            pools.normal_commit_points.extend(begin + 1..=confirm);
        }
    }
    let mut previous_boundary = windows.start_entry;
    for (&begin, &confirm) in windows.begin_entries.iter().zip(&windows.confirm_entries) {
        pools.between_commit_points.extend(previous_boundary + 1..=begin);
        previous_boundary = confirm;
    }
    pools.between_commit_points.extend(previous_boundary + 1..=windows.end_entry);
    pools
}

fn bucket_generator(seed: u64, arm_index: u64, bucket: u64) -> SplitMix64 {
    SplitMix64::new(seed ^ ((arm_index << 8) | bucket).wrapping_mul(MIXING_MULTIPLIER))
}

/// `池[输出 mod 池长]`、跳过重复；池不比要的多就全取。
fn draw_distinct(generator: &mut SplitMix64, pool: &[usize], wanted: usize) -> Vec<usize> {
    if pool.len() <= wanted {
        return pool.to_vec();
    }
    let pool_length = u64::try_from(pool.len()).expect("装得进 u64");
    let mut drawn = Vec::with_capacity(wanted);
    let mut seen = HashSet::new();
    while drawn.len() < wanted {
        let candidate = pool[usize::try_from(generator.next_output() % pool_length).expect("小于池长")];
        if seen.insert(candidate) {
            drawn.push(candidate);
        }
    }
    drawn
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StratifiedDraw {
    bucket_normal: Vec<usize>,
    bucket_long: Vec<usize>,
    bucket_between: Vec<usize>,
}

/// 补 7 B2：先取桶 B、再取桶 C，桶 A 取 total 减去 B、C 两桶实取的数。
fn stratified_draw(arm_index: u64, pools: &BucketPools, long_quota: usize, between_quota: usize, total: usize) -> StratifiedDraw {
    let mut long_generator = bucket_generator(SEED_CUT, arm_index, BUCKET_LONG_COMMITS);
    let bucket_long = draw_distinct(&mut long_generator, &pools.long_commit_points, long_quota);
    let mut between_generator = bucket_generator(SEED_CUT, arm_index, BUCKET_BETWEEN_COMMITS);
    let bucket_between = draw_distinct(&mut between_generator, &pools.between_commit_points, between_quota);
    let mut normal_generator = bucket_generator(SEED_CUT, arm_index, BUCKET_NORMAL_COMMITS);
    let bucket_normal =
        draw_distinct(&mut normal_generator, &pools.normal_commit_points, total - bucket_long.len() - bucket_between.len());
    StratifiedDraw { bucket_normal, bucket_long, bucket_between }
}

/// 补 7 B5：长窗挑 2 个（不足就全挑），普通窗补足 8 个；下标是各自按 j 升序排好之后的位置。
fn picked_commit_positions(arm_index: u64, normal_count: usize, long_count: usize) -> (Vec<usize>, Vec<usize>) {
    let mut normal_generator = bucket_generator(SEED_PICK, arm_index, BUCKET_PICK_NORMAL);
    let mut long_generator = bucket_generator(SEED_PICK, arm_index, BUCKET_PICK_LONG);
    let long_positions = draw_distinct(&mut long_generator, &(0..long_count).collect::<Vec<usize>>(), PICKED_LONG_COMMIT_TARGET);
    let normal_positions =
        draw_distinct(&mut normal_generator, &(0..normal_count).collect::<Vec<usize>>(), PICKED_COMMIT_TOTAL - long_positions.len());
    (normal_positions, long_positions)
}

/// 一块的 (B_j, C_j] 里逐个条目边界；多于 64 个时取包含两端的 64 个等距点（第 s 个取 floor(s × (n − 1) ÷ 63)）。
fn commit_window_boundaries(begin_entry: usize, confirm_entry: usize) -> Vec<usize> {
    let all: Vec<usize> = (begin_entry + 1..=confirm_entry).collect();
    if all.len() <= COMMIT_BOUNDARY_LIMIT {
        return all;
    }
    let last = all.len() - 1;
    (0..COMMIT_BOUNDARY_LIMIT).map(|step| all[step * last / (COMMIT_BOUNDARY_LIMIT - 1)]).collect()
}

/// PC4-L、PC4-NB、PC4-FD 的 20 点：(S, E] 全部边界里单桶取（补 7 B2c 那种取法）。
fn control_points(arm_index: u64, windows: &CommitWindows) -> Vec<usize> {
    let pools = BucketPools {
        normal_commit_points: (windows.start_entry + 1..=windows.end_entry).collect(),
        long_commit_points: Vec::new(),
        between_commit_points: Vec::new(),
        median_window_entries: 0,
        normal_blocks: Vec::new(),
        long_blocks: Vec::new(),
    };
    let mut points = stratified_draw(arm_index, &pools, 0, 0, CONTROL_POINT_TOTAL).bucket_normal;
    points.sort_unstable();
    points
}

// ============================== 验盘计划 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointGroup {
    /// C-strat 桶 A「普通提交里」。
    StratifiedNormalCommits,
    /// C-strat 桶 B「长窗提交里」。
    StratifiedLongCommits,
    /// C-strat 桶 C「两次提交之间」。
    StratifiedBetweenCommits,
    /// C-commit：挑出的第 picked 块（先普通后长窗）窗口里第 boundary 个边界。
    CommitSweep { picked: u64, boundary: u64, block_index: u64 },
    /// PC4-L、PC4-NB、PC4-FD 的单桶 20 点。
    ControlUniform,
    /// PC4-S、PC4-P、PC4-O：「全部条目」重放出的盘面。
    FullReplayDamage,
}

impl PointGroup {
    fn labels(self) -> (&'static str, &'static str) {
        match self {
            PointGroup::StratifiedNormalCommits => ("C-strat", "A"),
            PointGroup::StratifiedLongCommits => ("C-strat", "B"),
            PointGroup::StratifiedBetweenCommits => ("C-strat", "C"),
            PointGroup::CommitSweep { .. } => ("C-commit", "-"),
            PointGroup::ControlUniform => ("control", "-"),
            PointGroup::FullReplayDamage => ("full-replay", "-"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifyItem {
    item_number: usize,
    group: PointGroup,
    point: usize,
    variant: ReplayVariant,
    subset_index: u64,
    confirmed_end: u64,
    started_end: u64,
    damage: LibraryDamage,
}

impl VerifyItem {
    fn plan_line(&self) -> String {
        let (group_label, bucket_label) = self.group.labels();
        let (picked, boundary, block) = match self.group {
            PointGroup::CommitSweep { picked, boundary, block_index } => (picked.to_string(), boundary.to_string(), block_index.to_string()),
            PointGroup::StratifiedNormalCommits
            | PointGroup::StratifiedLongCommits
            | PointGroup::StratifiedBetweenCommits
            | PointGroup::ControlUniform
            | PointGroup::FullReplayDamage => ("-".to_string(), "-".to_string(), "-".to_string()),
        };
        format!(
            "item number={} group={group_label} bucket={bucket_label} picked={picked} boundary={boundary} block={block} point={} variant={} subset={} confirmed_end={} started_end={} damage={}",
            self.item_number,
            self.point,
            self.variant.label(),
            self.subset_index,
            self.confirmed_end,
            self.started_end,
            self.damage.label()
        )
    }
}

/// 一次写盘开机之后要验的全部「断电点 × 取法」（补 5.2 断电点表；对照 20 点；PC4-S/P/O）。
/// full_replay_point 是这次开机的条目总数：PC4-S/P/O 在「全部条目」重放出的盘面上做。
fn verification_items(arm: PowerCutArm, windows: &CommitWindows, pools: &BucketPools, full_replay_point: usize) -> Vec<VerifyItem> {
    let arm_index = u64::from(arm.index());
    let mut items = Vec::new();
    let push = |items: &mut Vec<VerifyItem>, group: PointGroup, point: usize, variant: ReplayVariant, subset_index: u64, damage: LibraryDamage| {
        let (confirmed_end, started_end) = block_classes_at(windows, point);
        let item_number = items.len();
        items.push(VerifyItem { item_number, group, point, variant, subset_index, confirmed_end, started_end, damage });
    };
    match arm.purpose() {
        ArmPurpose::Main | ArmPurpose::SmallBlockSamplingPoint => {
            let (total, long_quota, between_quota, variants): (usize, usize, usize, &[ReplayVariant]) = match arm.purpose() {
                ArmPurpose::SmallBlockSamplingPoint => (
                    SMALL_BLOCK_POINT_TOTAL,
                    SMALL_BLOCK_LONG_COMMIT_QUOTA,
                    SMALL_BLOCK_BETWEEN_COMMITS_QUOTA,
                    &[ReplayVariant::DropUnflushed, ReplayVariant::RandomSubset],
                ),
                ArmPurpose::Main
                | ArmPurpose::LostPageCacheControl
                | ArmPurpose::LostWriteCacheControl
                | ArmPurpose::MissingDirectorySyncControl
                | ArmPurpose::WriteCacheModelControl => {
                    (STRATIFIED_POINT_TOTAL, LONG_COMMIT_BUCKET_QUOTA, BETWEEN_COMMITS_BUCKET_QUOTA, &ALL_REPLAY_VARIANTS)
                }
            };
            let draw = stratified_draw(arm_index, pools, long_quota, between_quota, total);
            let mut stratified: Vec<(usize, PointGroup)> = draw
                .bucket_normal
                .iter()
                .map(|&point| (point, PointGroup::StratifiedNormalCommits))
                .chain(draw.bucket_long.iter().map(|&point| (point, PointGroup::StratifiedLongCommits)))
                .chain(draw.bucket_between.iter().map(|&point| (point, PointGroup::StratifiedBetweenCommits)))
                .collect();
            stratified.sort_unstable_by_key(|(point, _group)| *point);
            for (subset_index, (point, group)) in stratified.into_iter().enumerate() {
                for &variant in variants {
                    push(&mut items, group, point, variant, u64::try_from(subset_index).expect("装得进 u64"), LibraryDamage::Untouched);
                }
            }
            if arm.purpose() == ArmPurpose::Main {
                let (normal_positions, long_positions) = picked_commit_positions(arm_index, pools.normal_blocks.len(), pools.long_blocks.len());
                let picked_blocks: Vec<u64> = normal_positions
                    .iter()
                    .map(|&position| pools.normal_blocks[position])
                    .chain(long_positions.iter().map(|&position| pools.long_blocks[position]))
                    .collect();
                for (picked, block_index) in picked_blocks.into_iter().enumerate() {
                    let block_position = usize::try_from(block_index).expect("装得进 usize");
                    let boundaries = commit_window_boundaries(windows.begin_entries[block_position], windows.confirm_entries[block_position]);
                    for (boundary, point) in boundaries.into_iter().enumerate() {
                        let picked_number = u64::try_from(picked).expect("装得进 u64");
                        let boundary_number = u64::try_from(boundary).expect("装得进 u64");
                        let subset_index = COMMIT_SUBSET_INDEX_BASE + COMMIT_SUBSET_INDEX_STRIDE * picked_number + boundary_number;
                        let group = PointGroup::CommitSweep { picked: picked_number, boundary: boundary_number, block_index };
                        for variant in [ReplayVariant::DropUnflushed, ReplayVariant::RandomSubset] {
                            push(&mut items, group, point, variant, subset_index, LibraryDamage::Untouched);
                        }
                    }
                }
                assert!(full_replay_point > windows.end_entry, "全部条目的断电点在 E 标记之后");
                for damage in [LibraryDamage::SilentCorruption, LibraryDamage::PhantomKey, LibraryDamage::OpenBreaking] {
                    push(&mut items, PointGroup::FullReplayDamage, full_replay_point, ReplayVariant::KeepUnflushed, 0, damage);
                }
            }
        }
        ArmPurpose::LostPageCacheControl | ArmPurpose::LostWriteCacheControl | ArmPurpose::MissingDirectorySyncControl => {
            let variant = match arm.purpose() {
                ArmPurpose::LostPageCacheControl => ReplayVariant::KeepUnflushed,
                ArmPurpose::LostWriteCacheControl
                | ArmPurpose::MissingDirectorySyncControl
                | ArmPurpose::Main
                | ArmPurpose::SmallBlockSamplingPoint
                | ArmPurpose::WriteCacheModelControl => ReplayVariant::DropUnflushed,
            };
            for (subset_index, point) in control_points(arm_index, windows).into_iter().enumerate() {
                push(&mut items, PointGroup::ControlUniform, point, variant, u64::try_from(subset_index).expect("装得进 u64"), LibraryDamage::Untouched);
            }
        }
        ArmPurpose::WriteCacheModelControl => {}
    }
    items
}

// ============================== 产物行与参数 ==============================

/// 行打到哪：标准输出（给了文件就同时写进那份文件），或全扔掉（来宾开头的锚点自检只要对不上的个数）。
enum LineDestination {
    StandardOutput { copy_to_file: Option<File> },
    Discarded,
}

/// 产物行：每行 `E7RESULT ` 起头，收尾 `name=done emitted=N`。
struct ProductLines {
    emitter: Emitter,
    destination: LineDestination,
}

impl ProductLines {
    fn to_standard_output() -> ProductLines {
        ProductLines { emitter: Emitter::new(), destination: LineDestination::StandardOutput { copy_to_file: None } }
    }

    fn discarding() -> ProductLines {
        ProductLines { emitter: Emitter::new(), destination: LineDestination::Discarded }
    }

    fn with_file_copy(path: &Path) -> Result<ProductLines, String> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("建不出产物文件 {}（已存在就换一个新文件名）：{error}", path.display()))?;
        Ok(ProductLines { emitter: Emitter::new(), destination: LineDestination::StandardOutput { copy_to_file: Some(file) } })
    }

    fn line(&mut self, body: impl AsRef<str>) {
        let text = self.emitter.emit_raw(body.as_ref());
        self.write_through(&text);
    }

    fn finish(&mut self) {
        let text = self.emitter.finish();
        self.write_through(&text);
    }

    fn write_through(&mut self, text: &str) {
        match &mut self.destination {
            LineDestination::Discarded => {}
            LineDestination::StandardOutput { copy_to_file } => {
                let mut standard_output = std::io::stdout().lock();
                writeln!(standard_output, "{text}").expect("写得进标准输出");
                standard_output.flush().expect("刷得出标准输出");
                if let Some(file) = copy_to_file {
                    writeln!(file, "{text}").expect("写得进产物文件");
                }
            }
        }
    }
}

fn field_value<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    line.split(' ').find_map(|field| field.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
}

fn field_number(line: &str, key: &str) -> Option<u64> {
    field_value(line, key).and_then(|text| text.parse().ok())
}

/// 行里的自由文本（错误信息、dmesg）：空白换成下划线、截到 240 个字符，免得撑破「空格分字段」的产物行。
fn sanitized(text: &str) -> String {
    let flattened: String = text.chars().map(|character| if character.is_whitespace() { '_' } else { character }).collect();
    let shortened: String = flattened.chars().take(240).collect();
    if shortened.is_empty() {
        "-".to_string()
    } else {
        shortened
    }
}

/// `--键 值` 与 `--开关`；不以 `--` 起头又没被当成值的，算位置参数（来宾里 vm-bench 塞进来的 /dev/vda）。
struct RoleArguments {
    values: BTreeMap<String, String>,
}

impl RoleArguments {
    fn parse(arguments: &[String]) -> RoleArguments {
        let mut values = BTreeMap::new();
        let mut position = 0usize;
        while position < arguments.len() {
            let token = &arguments[position];
            if let Some(key) = token.strip_prefix("--") {
                let next_is_value = arguments.get(position + 1).is_some_and(|next| !next.starts_with("--"));
                if next_is_value {
                    values.insert(key.to_string(), arguments[position + 1].clone());
                    position += 2;
                } else {
                    values.insert(key.to_string(), "true".to_string());
                    position += 1;
                }
            } else {
                position += 1;
            }
        }
        RoleArguments { values }
    }

    fn text(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    fn number(&self, key: &str) -> Option<u64> {
        self.text(key).and_then(|text| text.parse().ok())
    }

    fn flag(&self, key: &str) -> bool {
        self.text(key) == Some("true")
    }
}

// ============================== 锚点自检（登记 7.2 A1–A6、补 7 B 系列） ==============================

fn synthetic_anchor_log_entries() -> Vec<(u64, u64, u64, Vec<u8>)> {
    vec![
        (0, 1, 0, vec![0x11; 512]),
        (8, 2, 0, vec![0x22; 1024]),
        (0, 0, LOG_FLUSH_FLAG, Vec::new()),
        (0, 1, 0, vec![0x33; 512]),
        (16, 1, 0, vec![0x44; 512]),
        (4, 4, 0, vec![0x55; 2048]),
    ]
}

const ANCHOR_IMAGE_SECTORS: usize = 24;
const ANCHOR_REPLAY_ARM_INDEX: u64 = 0;
const ANCHOR_REPLAY_SUBSET_INDEX: u64 = 15;

/// B8：合成日志在断电点 point 上按 variant 重放出的 24 扇区镜像的 sha256。
fn synthetic_anchor_image_digest(log_bytes: Vec<u8>, point: usize, variant: ReplayVariant) -> Result<String, String> {
    let log = LogBytes::InMemory(log_bytes);
    let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).map_err(|error| error.to_string())?;
    let selection = select_replayed_writes(&device_log.entries, point, variant, ANCHOR_REPLAY_ARM_INDEX, ANCHOR_REPLAY_SUBSET_INDEX);
    let mut image = ReplayImage::InMemory(vec![0u8; ANCHOR_IMAGE_SECTORS * SECTOR_BYTES]);
    apply_replayed_writes(&log, &selection.writes, &mut image)?;
    match image {
        ReplayImage::InMemory(bytes) => Ok(sha256_hexadecimal(&bytes)),
        ReplayImage::OnDisk(_) => Err("B8 的镜像在内存里".to_string()),
    }
}

fn bit_string(bits: &[bool]) -> String {
    bits.iter().map(|&bit| if bit { '1' } else { '0' }).collect()
}

fn join_usize(values: &[usize]) -> String {
    values.iter().map(usize::to_string).collect::<Vec<String>>().join(",")
}

/// 跑全部锚点，每个一行；返回对不上的个数。
fn run_anchors(output: &mut ProductLines) -> usize {
    let mut mismatches = 0usize;
    let mut record = |output: &mut ProductLines, name: &str, got: String, expected: &str| {
        let matches = got == expected;
        if !matches {
            mismatches += 1;
        }
        output.line(format!("name=anchor id={name} value={got} matches_registration={matches}"));
    };
    let mut zero_generator = SplitMix64::new(0);
    let first_three: Vec<String> = (0..3).map(|_draw| format!("{:#018x}", zero_generator.next_output())).collect();
    record(output, "A1", first_three.join(","), "0xe220a8397b1dcdaf,0x6e789e6aa1b965f4,0x06c45d188009454f");
    record(output, "A2", format!("{:#010x}", crc32c(&[b"123456789"])), "0xe3069283");
    for (block_index, expected) in [
        (0u64, "76020f1b59444d587a6607195c0f8fed/8351809/0x08eefd5c"),
        (12_345, "b3c1f0df0c77c9e98c9df5b21d0395bf/8321879/0x565d4801"),
    ] {
        let value = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        let byte_sum: u64 = value.iter().map(|&byte| u64::from(byte)).sum();
        record(output, &format!("A3_j{block_index}"), format!("{}/{byte_sum}/{:#010x}", hexadecimal(&value[..16]), crc32c(&[&value])), expected);
    }
    let small = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 0, SMALL_STATES_PER_BLOCK);
    let small_sum: u64 = small.iter().map(|&byte| u64::from(byte)).sum();
    record(output, "A3b", format!("{}/{small_sum}", hexadecimal(&small[..16])), "76020f1b59444d587a6607195c0f8fed/131533");
    for (block_index, expected) in [(0u64, "267/32629"), (12_345, "245/28761")] {
        let value = block_value(ValuePattern::Sparse, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        let nonzero = value.iter().filter(|&&byte| byte != 0).count();
        let byte_sum: u64 = value.iter().map(|&byte| u64::from(byte)).sum();
        record(output, &format!("A4_j{block_index}"), format!("{nonzero}/{byte_sum}"), expected);
    }
    let fingerprint = input_fingerprint();
    record(output, "A5", hexadecimal(&fingerprint), "7299025377b7594e92ceeb8a1c3cbeac");
    for (block_index, expected) in [
        (0u64, "000000000000000000000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (1, "000000000000000100000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (63, "000000070000000700000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (64, "000000000000000000000000000100007299025377b7594e92ceeb8a1c3cbeac"),
        (12_345, "00000007000000010000000000c000007299025377b7594e92ceeb8a1c3cbeac"),
        (99_999, "000000030000000700000000061a00007299025377b7594e92ceeb8a1c3cbeac"),
    ] {
        let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        record(output, &format!("A6_j{block_index}"), hexadecimal(&key), expected);
    }
    let small_key = interleaved_block_key(12_345, SMALL_STATES_PER_BLOCK, &fingerprint);
    record(output, "A6b", hexadecimal(&small_key), "000000070000000100000000000300007299025377b7594e92ceeb8a1c3cbeac");
    let arm_names: Vec<String> = ALL_POWER_CUT_ARMS.iter().map(|arm| format!("{}={}", arm.index(), arm.label())).collect();
    record(
        output,
        "B1",
        arm_names.join(","),
        "0=R1,1=R0,2=K,3=F,4=R1-None,5=R0-None,6=K-nosync,7=F-nofsync,8=F-nodirsync,9=PC4-C,10=R1-NB,11=R0-NB,12=K-NB,13=F-NB,14=R1-bs10,15=R0-bs10,16=K-bs10,17=F-bs10",
    );
    let anchor_pools = |long_pool: Vec<usize>| BucketPools {
        normal_commit_points: (1_000..41_000).collect(),
        long_commit_points: long_pool,
        between_commit_points: (60_000..66_000).collect(),
        median_window_entries: 0,
        normal_blocks: Vec::new(),
        long_blocks: Vec::new(),
    };
    let draw = stratified_draw(0, &anchor_pools((50_000..50_500).collect()), LONG_COMMIT_BUCKET_QUOTA, BETWEEN_COMMITS_BUCKET_QUOTA, STRATIFIED_POINT_TOTAL);
    let draw_sum: usize = draw.bucket_normal.iter().chain(&draw.bucket_long).chain(&draw.bucket_between).sum();
    record(
        output,
        "B2",
        format!(
            "{}/{}/{};{};{};{};{draw_sum}",
            draw.bucket_normal.len(),
            draw.bucket_long.len(),
            draw.bucket_between.len(),
            join_usize(&draw.bucket_normal[..5]),
            join_usize(&draw.bucket_long[..5]),
            join_usize(&draw.bucket_between[..5])
        ),
        "100/60/40;6895,5605,40371,9726,18098;50396,50274,50461,50044,50216;64169,60139,63547,63950,62245;7787893",
    );
    let short_draw = stratified_draw(3, &anchor_pools((50_000..50_030).collect()), LONG_COMMIT_BUCKET_QUOTA, BETWEEN_COMMITS_BUCKET_QUOTA, STRATIFIED_POINT_TOTAL);
    record(
        output,
        "B2b",
        format!("{}/{}/{};{}", short_draw.bucket_normal.len(), short_draw.bucket_long.len(), short_draw.bucket_between.len(), join_usize(&short_draw.bucket_normal[..3])),
        "130/30/40;26612,4589,2232",
    );
    let control_windows = CommitWindows { start_entry: 999, end_entry: 40_999, begin_entries: Vec::new(), confirm_entries: Vec::new() };
    record(
        output,
        "B2c",
        join_usize(&control_points(3, &control_windows)),
        "2232,2241,2612,4589,9184,14497,14598,15089,16593,18900,21649,26612,27155,28849,31573,32287,32340,34273,37366,38401",
    );
    let subset_anchor_sectors = [8u64, 1, 16, 8, 1, 1, 128, 8, 2, 1];
    let first_subset = unflushed_subset(0, 0, &subset_anchor_sectors);
    record(
        output,
        "B3",
        format!("{}/{:?}", bit_string(&first_subset.kept), first_subset.torn.map(|torn| (torn.position_in_unflushed, torn.sectors_kept))),
        "1000000100/Some((7, 1))",
    );
    let second_subset = unflushed_subset(2, 7, &subset_anchor_sectors);
    record(
        output,
        "B3b",
        format!("{}/{:?}", bit_string(&second_subset.kept), second_subset.torn.map(|torn| (torn.position_in_unflushed, torn.sectors_kept))),
        "0110000011/Some((2, 5))",
    );
    let write_cache_model_subset = unflushed_subset(u64::from(PowerCutArm::WriteCacheModel.index()), 0, &[1u64; 100]);
    let write_cache_model_kept = write_cache_model_subset.kept.iter().filter(|&&is_kept| is_kept).count();
    record(output, "B4", format!("{write_cache_model_kept}/{}", 100 - write_cache_model_kept), "53/47");
    for (arm_index, expected) in [
        (0u64, "1380,2481,2596,4737,5356,3368/86,69"),
        (1, "4969,4100,4435,810,3663,762/7,31"),
        (2, "5473,3722,537,2873,1713,5287/78,42"),
        (3, "4929,1614,967,5654,1819,4038/57,29"),
    ] {
        let (normal_positions, long_positions) = picked_commit_positions(arm_index, 5_900, 100);
        record(output, &format!("B5_a{arm_index}"), format!("{}/{}", join_usize(&normal_positions), join_usize(&long_positions)), expected);
    }
    let (normal_positions_without_long, long_positions_when_none) = picked_commit_positions(0, 6_000, 0);
    record(
        output,
        "B5b",
        format!("{}/{}", join_usize(&normal_positions_without_long), join_usize(&long_positions_when_none)),
        "5280,2981,296,1337,4056,4668,2830,1561/",
    );
    let marker_anchor = encode_marker(&Marker { kind: MarkerKind::Confirm, sequence: 5, block_index: Some(2), arm_index: 0 });
    record(
        output,
        "B7",
        format!("{}/{}", hexadecimal(&marker_anchor[..32]), sha256_hexadecimal(&marker_anchor)),
        "53464d4b43000000050000000000000002000000000000000000000070e31542/845bcdb2f8e741bbb81e81f3f40032f0e4bd948f1032c56d171d388abea214ee",
    );
    let synthetic_anchor_log = synthetic_log_bytes(&synthetic_anchor_log_entries(), 6);
    record(output, "B8_log", format!("{}/{}", synthetic_anchor_log.len(), sha256_hexadecimal(&synthetic_anchor_log)), "8192/078793918714a9ff52eb78e50f4871973e9073eaba9c4fcaf61b53463c098897");
    for (point, variant, expected) in [
        (6usize, ReplayVariant::DropUnflushed, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (6, ReplayVariant::KeepUnflushed, "6e6d2a487aa2864b75b62637bd06aa2983e5fd5a5a8344debcdee784e774e510"),
        (6, ReplayVariant::RandomSubset, "df9020850109e36d3b6d34da043be5a0fb016ffd4004372c52dac115009c1d65"),
        (6, ReplayVariant::RandomSubsetWithTear, "6467f85ca3d8eefcb7eac7ea3cd4978d4cbb92d6b66b842b68a19adb6d46a763"),
        (5, ReplayVariant::DropUnflushed, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (5, ReplayVariant::KeepUnflushed, "4d2a669e7156f1c1ec6cc051809c0a67c191b9b5c65df784ad0c717eec5a4608"),
        (5, ReplayVariant::RandomSubset, "0783c6aa6d776285d9fd1100701a4d57a6256e54332bafea82a53af6b1bacb55"),
        (5, ReplayVariant::RandomSubsetWithTear, "0783c6aa6d776285d9fd1100701a4d57a6256e54332bafea82a53af6b1bacb55"),
        (3, ReplayVariant::DropUnflushed, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (3, ReplayVariant::KeepUnflushed, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (3, ReplayVariant::RandomSubset, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (3, ReplayVariant::RandomSubsetWithTear, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
    ] {
        let digest = synthetic_anchor_image_digest(synthetic_anchor_log.clone(), point, variant).unwrap_or_else(|error| format!("error:{}", sanitized(&error)));
        record(output, &format!("B8_k{point}_{}", variant.label()), digest, expected);
    }
    let mut stale_entries = synthetic_anchor_log_entries();
    stale_entries.push((20, 1, 0, vec![0x66; 512]));
    let stale_log = synthetic_log_bytes(&stale_entries, 6);
    let stale_parsed = parse_device_log(&LogBytes::InMemory(stale_log.clone()), DeclaredCountHandling::StopAtDeclared)
        .map(|device_log| device_log.entries.len())
        .unwrap_or(0);
    let stale_digest = synthetic_anchor_image_digest(stale_log.clone(), stale_parsed, ReplayVariant::KeepUnflushed).unwrap_or_else(|error| format!("error:{}", sanitized(&error)));
    record(
        output,
        "B8b",
        format!("{}/{}/{stale_parsed}/{stale_digest}", stale_log.len(), sha256_hexadecimal(&stale_log)),
        "9216/9956234bb62be39429e2086b19e37c511266fa9e56c16d3429579ba9a9803bd1/6/6e6d2a487aa2864b75b62637bd06aa2983e5fd5a5a8344debcdee784e774e510",
    );
    let synthetic_model_log = LogBytes::InMemory(synthetic_write_cache_model_log());
    let synthetic_model = parse_device_log(&synthetic_model_log, DeclaredCountHandling::StopAtDeclared)
        .map_err(|error| error.to_string())
        .and_then(|device_log| judge_write_cache_model(&synthetic_model_log, &device_log));
    let synthetic_model_text = match synthetic_model {
        Ok(judgement) => {
            let lost: Vec<String> = judgement
                .lost_by_variant
                .iter()
                .map(|(variant, first_lost, second_lost)| format!("{}:{first_lost},{second_lost}", variant.label()))
                .collect();
            format!(
                "{}/{}/{}/{}/{}",
                judgement.point,
                judgement.entries_between_batches,
                judgement.flushes_between_batches,
                lost.join(";"),
                judgement.registered_counts_hold
            )
        }
        Err(error) => format!("error:{}", sanitized(&error)),
    };
    record(output, "PC4-C_synthetic", synthetic_model_text, "201/1/1/U-drop:0,100;U-keep:0,0;U-rand:0,47;U-tear:0,47/true");
    let control_differences: Vec<String> = ALL_POWER_CUT_ARMS
        .iter()
        .filter_map(|arm| {
            let settings = arm.write_settings()?;
            let correct = arm.correct_form().write_settings()?;
            let differing = settings.differing_setting_names(correct);
            let registered: Vec<&str> = arm.registered_control_difference().into_iter().collect();
            (differing != registered).then(|| format!("{}:{}", arm.label(), differing.join("+")))
        })
        .collect();
    record(output, "M26_control_forms", if control_differences.is_empty() { "exactly_registered".to_string() } else { control_differences.join(",") }, "exactly_registered");
    let marker_capacity_needed = 2 * ROCKSDB_REDO_BLOCK_LIMIT + 2;
    record(output, "B9b", format!("{}/{}", 2 * MAIN_BLOCK_COUNT + 2, marker_partition_sectors()), "12002/131072");
    record(
        output,
        "B9c",
        format!(
            "{FIRST_PARTITION_START_SECTOR}/{}/{}/{}/{}",
            first_partition_sectors(),
            marker_partition_start_sector(),
            marker_partition_sectors(),
            (marker_partition_start_sector() + marker_partition_sectors()) * sector_bytes() / MEBIBYTE
        ),
        "2048/6291456/6293504/131072/3137",
    );
    record(
        output,
        "B9d",
        format!("{}", marker_capacity_needed <= marker_partition_sectors() && (marker_partition_start_sector() + marker_partition_sectors()) * sector_bytes() <= DISK_MEBIBYTES * MEBIBYTE),
        "true",
    );
    mismatches
}

// ============================== 入口 ==============================

/// 来宾里跑的角色与核对子进程：来宾里没有 python3，不判准入（宿主上的 s4-drive 已经判过）。
const GUEST_ROLES: [&str; 3] = ["s4-write-guest", "s4-verify-guest", "s4-check-point"];

fn role_of(arguments: &[String]) -> Option<String> {
    arguments.iter().position(|argument| argument == "--role").and_then(|position| arguments.get(position + 1)).cloned()
}

fn preflight() -> Option<String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if role_of(&arguments).is_some_and(|role| GUEST_ROLES.contains(&role.as_str())) {
        return None;
    }
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e162_verdict_store_power_cut.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
    let first_line = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").to_string();
    first_line.strip_prefix("forced").map(|summary| summary.trim().replace(' ', "_"))
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    let parsed = RoleArguments::parse(&arguments);
    let exit_code = match parsed.text("role").unwrap_or("") {
        "selftest" => {
            let mut output = ProductLines::to_standard_output();
            if let Some(summary) = &forced_summary {
                output.line(format!("name=preflight forced={summary}"));
            }
            let mismatches = run_anchors(&mut output);
            output.line(format!("name=verdict part=anchors anchor_mismatches={mismatches}"));
            output.finish();
            i32::from(mismatches != 0)
        }
        "s4-stage" => run_stage_role(&parsed),
        "s4-drive" => run_drive_role(&parsed, forced_summary.as_deref()),
        "s4-write-guest" => run_write_guest_role(&parsed),
        "s4-verify-guest" => run_verify_guest_role(&parsed),
        "s4-check-point" => run_check_point_role(&parsed),
        _ => {
            eprintln!("  ✗ 用法：e162-verdict-store-power-cut --role selftest | s4-stage --destination <空目录> | s4-drive --arms <R1,R0,K,F> --kernel <内核> --out <产物> [--dry-run]");
            eprintln!("  → 怎么办：s4-write-guest、s4-verify-guest、s4-check-point 是来宾里的角色，由 s4-drive 经 vm-bench.sh 起，不手敲");
            USAGE_EXIT_CODE
        }
    };
    std::process::exit(exit_code);
}

// ============================== 来宾：公共 ==============================

fn run_tool(program: &str, arguments: &[&str], standard_input: Option<&str>) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(arguments).stdout(Stdio::piped()).stderr(Stdio::piped());
    command.stdin(if standard_input.is_some() { Stdio::piped() } else { Stdio::null() });
    let mut child = command.spawn().map_err(|error| format!("起不了 {program}：{error}"))?;
    if let Some(text) = standard_input {
        let mut child_input = child.stdin.take().expect("标准输入接在管道上");
        child_input.write_all(text.as_bytes()).map_err(|error| format!("写不进 {program} 的标准输入：{error}"))?;
    }
    let output = child.wait_with_output().map_err(|error| format!("等不到 {program}：{error}"))?;
    let combined = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    if output.status.success() {
        Ok(combined)
    } else {
        Err(format!("{program} {} 退出 {:?}：{combined}", arguments.join(" "), output.status.code()))
    }
}

fn read_trimmed(path: &str) -> String {
    sanitized(fs::read_to_string(path).unwrap_or_else(|_error| "unreadable".to_string()).trim())
}

fn guest_environment_line(role: &str, arm: PowerCutArm) -> String {
    let dirty_fields: Vec<String> = [
        "dirty_background_bytes",
        "dirty_background_ratio",
        "dirty_bytes",
        "dirty_ratio",
        "dirty_expire_centisecs",
        "dirty_writeback_centisecs",
    ]
    .iter()
    .map(|name| format!("{name}={}", read_trimmed(&format!("/proc/sys/vm/{name}"))))
    .collect();
    format!(
        "name=guest_environment role={role} arm={} kernel_release={} write_cache={} fua={} {}",
        arm.label(),
        read_trimmed("/proc/sys/kernel/osrelease"),
        read_trimmed("/sys/block/vda/queue/write_cache"),
        read_trimmed("/sys/block/vda/queue/fua"),
        dirty_fields.join(" ")
    )
}

/// 来宾开头的锚点自检：锚点照跑一遍、行不打出去，只报对不上的个数一行。
fn emit_guest_anchor_check(output: &mut ProductLines, role: &str) -> bool {
    let mut discarded = ProductLines::discarding();
    let mismatches = run_anchors(&mut discarded);
    output.line(format!("name=anchor_self_check role={role} anchor_mismatches={mismatches}"));
    mismatches == 0
}

fn halt_line(output: &mut ProductLines, clause: &str, step: &str, detail: &str) -> i32 {
    output.line(format!("name=halt clause={clause} step={step} detail={}", sanitized(detail)));
    HALT_EXIT_CODE
}

// ============================== 来宾：写盘开机 ==============================

fn run_write_guest_role(arguments: &RoleArguments) -> i32 {
    let mut output = ProductLines::to_standard_output();
    let exit_code = write_guest_body(&mut output, arguments);
    output.finish();
    exit_code
}

fn write_guest_body(output: &mut ProductLines, arguments: &RoleArguments) -> i32 {
    let Some(arm) = arguments.number("arm").and_then(|index| u32::try_from(index).ok()).and_then(PowerCutArm::from_index) else {
        return halt_line(output, "usage", "arguments", "少了 --arm 或臂号认不出");
    };
    if !emit_guest_anchor_check(output, "s4-write-guest") {
        return halt_line(output, "V10", "anchors", "来宾里锚点对不上");
    }
    output.line(guest_environment_line("write", arm));
    match arm.candidate() {
        None => write_cache_model_guest(output),
        Some(candidate) => {
            let block_count = arguments.number("blocks").unwrap_or_else(|| arm.registered_block_count());
            store_write_guest(output, arm, candidate, block_count)
        }
    }
}

/// PC4-C：/dev/vda 第 8 192 扇区起 O_DIRECT 逐个写 100 个 C 标记，fdatasync，再写 100 个；
/// 末尾再补一次 fdatasync，让设备侧日志的系统配置把 200 个写都数进去（登记第十二节修订：不改断电点 k 与判据）。
fn write_cache_model_guest(output: &mut ProductLines) -> i32 {
    let device = match OpenOptions::new().write(true).custom_flags(OPEN_DIRECT_FLAG).open(GUEST_WHOLE_DISK) {
        Ok(device) => device,
        Err(error) => return halt_line(output, "H8", "open-whole-disk", &error.to_string()),
    };
    let mut aligned = AlignedSector::new();
    let arm_index = PowerCutArm::WriteCacheModel.index();
    for position in 0..2 * WRITE_CACHE_MODEL_BATCH_WRITES {
        if position == WRITE_CACHE_MODEL_BATCH_WRITES {
            if let Err(error) = device.sync_data() {
                return halt_line(output, "H8", "fdatasync-between-batches", &error.to_string());
            }
        }
        let marker = Marker { kind: MarkerKind::Confirm, sequence: position, block_index: Some(position), arm_index };
        let offset = (WRITE_CACHE_MODEL_FIRST_SECTOR + position) * sector_bytes();
        if let Err(error) = device.write_all_at(aligned.fill(&encode_marker(&marker)), offset) {
            return halt_line(output, "H8", "write-marker", &error.to_string());
        }
    }
    if let Err(error) = device.sync_data() {
        return halt_line(output, "H8", "fdatasync-at-end", &error.to_string());
    }
    output.line(format!("name=write_cache_model_guest writes={}", 2 * WRITE_CACHE_MODEL_BATCH_WRITES));
    0
}

fn partition_whole_disk() -> Result<(), String> {
    let script = format!(
        "label: dos\nstart={FIRST_PARTITION_START_SECTOR}, size={}, type=83\nstart={}, size={}, type=83\n",
        first_partition_sectors(),
        marker_partition_start_sector(),
        marker_partition_sectors()
    );
    run_tool(GUEST_PARTITIONER, &[GUEST_WHOLE_DISK], Some(&script))?;
    let _reread_by_sfdisk_already = run_tool(GUEST_BUSYBOX, &["blockdev", "--rereadpt", GUEST_WHOLE_DISK], None);
    run_tool(GUEST_BUSYBOX, &["mdev", "-s"], None)?;
    for partition in [GUEST_FIRST_PARTITION, GUEST_MARKER_PARTITION] {
        if !Path::new(partition).exists() {
            return Err(format!("分区之后没有 {partition}"));
        }
    }
    Ok(())
}

fn mounted_line_of(device: &str) -> String {
    fs::read_to_string("/proc/mounts")
        .unwrap_or_default()
        .lines()
        .find(|line| line.starts_with(device))
        .map(sanitized)
        .unwrap_or_else(|| "absent".to_string())
}

fn store_write_guest(output: &mut ProductLines, arm: PowerCutArm, candidate: Candidate, block_count: u64) -> i32 {
    let settings = arm.write_settings().expect("带候选的臂都有写入设定");
    if let Err(detail) = partition_whole_disk() {
        return halt_line(output, "H8", "partition", &detail);
    }
    if let Err(detail) = run_tool(
        GUEST_FILESYSTEM_FORMATTER,
        &["-F", "-q", "-E", "nodiscard,lazy_itable_init=0,lazy_journal_init=0", GUEST_FIRST_PARTITION],
        None,
    ) {
        return halt_line(output, "H8", "mkfs", &detail);
    }
    if let Err(error) = fs::create_dir_all(GUEST_MOUNT_POINT) {
        return halt_line(output, "H8", "mount-point", &error.to_string());
    }
    let mount_outcome = match settings.filesystem_barrier {
        FilesystemBarrier::Enabled => run_tool(GUEST_BUSYBOX, &["mount", "-t", "ext4", GUEST_FIRST_PARTITION, GUEST_MOUNT_POINT], None),
        FilesystemBarrier::Disabled => {
            run_tool(GUEST_BUSYBOX, &["mount", "-t", "ext4", "-o", "barrier=0", GUEST_FIRST_PARTITION, GUEST_MOUNT_POINT], None)
        }
    };
    if let Err(detail) = mount_outcome {
        return halt_line(output, "H8", "mount", &detail);
    }
    output.line(format!("name=guest_mount role=write arm={} mounts_line={}", arm.label(), mounted_line_of(GUEST_FIRST_PARTITION)));
    let mut sink = match MarkerSink::open_marker_partition(GUEST_MARKER_PARTITION, arm.index()) {
        Ok(sink) => sink,
        Err(detail) => return halt_line(output, "H8", "open-marker-partition", &detail),
    };
    let library = Path::new(GUEST_MOUNT_POINT).join(GUEST_LIBRARY_NAME);
    let mut store = match open_store_for_writing(candidate, settings, &library) {
        Ok(store) => store,
        Err(error) => return halt_line(output, "store-create", "open-for-writing", &error.to_string()),
    };
    let write_outcome = write_blocks_with_markers(&mut store, &mut sink, block_count, arm.states_per_block());
    drop(store);
    let (flushes, compactions) = match candidate {
        Candidate::RocksDatabase => rocksdb_background_counts(&library),
        Candidate::RedbQuickRepair | Candidate::RedbDefault | Candidate::HardenedFile => (0, 0),
    };
    if candidate == Candidate::RocksDatabase {
        output.line(format!("name=rocksdb_background arm={} flushes={flushes} compactions={compactions}", arm.label()));
    }
    let unmount_outcome = run_tool(GUEST_BUSYBOX, &["umount", GUEST_MOUNT_POINT], None);
    let final_flush = OpenOptions::new().write(true).open(GUEST_WHOLE_DISK).and_then(|whole_disk| whole_disk.sync_all());
    let confirmed = match write_outcome {
        Ok(confirmed) => confirmed,
        Err(detail) => return halt_line(output, "write-failed", "write-blocks", &detail),
    };
    output.line(format!(
        "name=write_guest arm={} candidate={} blocks={block_count} states_per_block={} confirmed={confirmed} markers={} unmounted={} final_flush={}",
        arm.label(),
        candidate.label(),
        arm.states_per_block(),
        sink.emitted_marker_count(),
        unmount_outcome.is_ok(),
        final_flush.is_ok()
    ));
    0
}

// ============================== 来宾：验盘开机 ==============================

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanHeader {
    arm: PowerCutArm,
    block_count: u64,
    states_per_block: u64,
    disk_bytes: u64,
    start_entry: usize,
    end_entry: usize,
}

fn plan_header_line(header: &PlanHeader) -> String {
    format!(
        "header arm={} blocks={} states_per_block={} disk_bytes={} start_entry={} end_entry={}",
        header.arm.index(),
        header.block_count,
        header.states_per_block,
        header.disk_bytes,
        header.start_entry,
        header.end_entry
    )
}

fn parse_plan_header(line: &str) -> Option<PlanHeader> {
    Some(PlanHeader {
        arm: PowerCutArm::from_index(u32::try_from(field_number(line, "arm")?).ok()?)?,
        block_count: field_number(line, "blocks")?,
        states_per_block: field_number(line, "states_per_block")?,
        disk_bytes: field_number(line, "disk_bytes")?,
        start_entry: usize::try_from(field_number(line, "start_entry")?).ok()?,
        end_entry: usize::try_from(field_number(line, "end_entry")?).ok()?,
    })
}

fn parse_plan_item(line: &str) -> Option<VerifyItem> {
    let group = match (field_value(line, "group")?, field_value(line, "bucket")?) {
        ("C-strat", "A") => PointGroup::StratifiedNormalCommits,
        ("C-strat", "B") => PointGroup::StratifiedLongCommits,
        ("C-strat", "C") => PointGroup::StratifiedBetweenCommits,
        ("C-commit", _bucket) => PointGroup::CommitSweep {
            picked: field_number(line, "picked")?,
            boundary: field_number(line, "boundary")?,
            block_index: field_number(line, "block")?,
        },
        ("control", _bucket) => PointGroup::ControlUniform,
        ("full-replay", _bucket) => PointGroup::FullReplayDamage,
        (_group, _bucket) => return None,
    };
    Some(VerifyItem {
        item_number: usize::try_from(field_number(line, "number")?).ok()?,
        group,
        point: usize::try_from(field_number(line, "point")?).ok()?,
        variant: ReplayVariant::parse(field_value(line, "variant")?)?,
        subset_index: field_number(line, "subset")?,
        confirmed_end: field_number(line, "confirmed_end")?,
        started_end: field_number(line, "started_end")?,
        damage: LibraryDamage::parse(field_value(line, "damage")?)?,
    })
}

fn run_verify_guest_role(arguments: &RoleArguments) -> i32 {
    let mut output = ProductLines::to_standard_output();
    let exit_code = verify_guest_body(&mut output, arguments);
    output.finish();
    exit_code
}

fn verify_guest_body(output: &mut ProductLines, arguments: &RoleArguments) -> i32 {
    if !emit_guest_anchor_check(output, "s4-verify-guest") {
        return halt_line(output, "V10", "anchors", "来宾里锚点对不上");
    }
    let plan_text = match fs::read_to_string(arguments.text("plan").unwrap_or(GUEST_PLAN_PATH)) {
        Ok(text) => text,
        Err(error) => return halt_line(output, "H10", "read-plan", &error.to_string()),
    };
    let Some(header) = plan_text.lines().find(|line| line.starts_with("header ")).and_then(parse_plan_header) else {
        return halt_line(output, "H10", "plan-header", "计划里没有能读的 header 行");
    };
    let items: Vec<VerifyItem> = plan_text.lines().filter(|line| line.starts_with("item ")).filter_map(parse_plan_item).collect();
    output.line(guest_environment_line("verify", header.arm));
    output.line(format!("name=verify_guest_plan arm={} items={}", header.arm.label(), items.len()));
    let log = match LogBytes::open(Path::new(arguments.text("log").unwrap_or(GUEST_LOG_PATH))) {
        Ok(log) => log,
        Err(detail) => return halt_line(output, "H10", "open-log", &detail),
    };
    let device_log = match parse_device_log(&log, DeclaredCountHandling::StopAtDeclared) {
        Ok(device_log) => device_log,
        Err(error) => return halt_line(output, "H9", "parse-log", &error.to_string()),
    };
    if let Err(detail) = fs::create_dir_all(GUEST_IMAGE_DIRECTORY)
        .map_err(|error| error.to_string())
        .and_then(|()| run_tool(GUEST_BUSYBOX, &["mount", "-t", "tmpfs", "-o", GUEST_IMAGE_FILESYSTEM_OPTIONS, "tmpfs", GUEST_IMAGE_DIRECTORY], None).map(|_text| ()))
        .and_then(|()| fs::create_dir_all(GUEST_MOUNT_POINT).map_err(|error| error.to_string()))
    {
        return halt_line(output, "H8", "image-tmpfs", &detail);
    }
    let mut mount_failures = 0u64;
    for item in &items {
        let point_line = verify_one_item(&header, &log, &device_log, item);
        if field_value(&point_line, "mount") == Some("failed") {
            mount_failures += 1;
        }
        output.line(point_line);
    }
    output.line(format!("name=verify_guest_summary arm={} items={} mount_failures={mount_failures}", header.arm.label(), items.len()));
    0
}

enum CheckChildOutcome {
    Finished { report_line: Option<String>, exit_code: Option<i32> },
    TimedOut,
    SpawnFailed(String),
}

/// 子进程的标准输出另起一个线程读到 EOF；这里不计时、不转打。
fn collect_output_in_background(standard_output: std::process::ChildStdout) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut text = String::new();
        let _read_outcome = BufReader::new(standard_output).read_to_string(&mut text);
        text
    })
}

/// 起核对子进程；3 600 秒没交回就 SIGKILL（`Child::kill`）。等的循环只看它退没退，不读它的输出。
fn run_check_child_with_timeout(role_arguments: &[String], timeout: Duration) -> CheckChildOutcome {
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => return CheckChildOutcome::SpawnFailed(error.to_string()),
    };
    let mut child = match Command::new(executable).args(role_arguments).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn() {
        Ok(child) => child,
        Err(error) => return CheckChildOutcome::SpawnFailed(error.to_string()),
    };
    let reader = collect_output_in_background(child.stdout.take().expect("标准输出接在管道上"));
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _already_exited = child.kill();
                    let _reaped = child.wait();
                    let _discarded = reader.join();
                    return CheckChildOutcome::TimedOut;
                }
                std::thread::sleep(Duration::from_millis(CHECK_POLL_MILLISECONDS));
            }
            Err(_error) => break None,
        }
    };
    let text = reader.join().unwrap_or_default();
    let report_line = text.lines().find(|line| line.starts_with("S4CHECK ")).map(str::to_string);
    CheckChildOutcome::Finished { report_line, exit_code: status.and_then(|finished| finished.code()) }
}

fn dmesg_tail() -> String {
    let text = run_tool(GUEST_BUSYBOX, &["dmesg"], None).unwrap_or_else(|error| error);
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(12);
    sanitized(&lines[start..].join(" | "))
}

fn verify_one_item(header: &PlanHeader, log: &LogBytes, device_log: &DeviceLog, item: &VerifyItem) -> String {
    let (group_label, bucket_label) = item.group.labels();
    let span = header.end_entry.saturating_sub(header.start_entry).max(1);
    let fraction = (item.point.saturating_sub(header.start_entry)) as f64 / span as f64;
    let selection = select_replayed_writes(&device_log.entries, item.point, item.variant, u64::from(header.arm.index()), item.subset_index);
    let block_field = match item.group {
        PointGroup::CommitSweep { picked, boundary, block_index } => format!("picked={picked} boundary={boundary} block={block_index}"),
        PointGroup::StratifiedNormalCommits
        | PointGroup::StratifiedLongCommits
        | PointGroup::StratifiedBetweenCommits
        | PointGroup::ControlUniform
        | PointGroup::FullReplayDamage => "picked=- boundary=- block=-".to_string(),
    };
    let prefix = format!(
        "name=s4_point arm={} item={} group={group_label} bucket={bucket_label} {block_field} point={} fraction={fraction:.6} variant={} subset={} unflushed_writes={} confirmed_end={} started_end={} damage={}",
        header.arm.label(),
        item.item_number,
        item.point,
        item.variant.label(),
        item.subset_index,
        selection.unflushed_entries.len(),
        item.confirmed_end,
        item.started_end,
        item.damage.label()
    );
    let image_path = Path::new(GUEST_IMAGE_DIRECTORY).join("disk.img");
    let image_built = File::create(&image_path)
        .and_then(|file| file.set_len(header.disk_bytes).map(|()| file))
        .map_err(|error| error.to_string())
        .and_then(|file| {
            let mut image = ReplayImage::OnDisk(file);
            apply_replayed_writes(log, &selection.writes, &mut image)
        });
    if let Err(detail) = image_built {
        let _cleanup = fs::remove_file(&image_path);
        return format!("{prefix} mount=not_attempted check=image_failed detail={}", sanitized(&detail));
    }
    let offset_text = (FIRST_PARTITION_START_SECTOR * sector_bytes()).to_string();
    let image_text = image_path.to_string_lossy().to_string();
    if let Err(detail) = run_tool(GUEST_BUSYBOX, &["losetup", "-o", &offset_text, GUEST_LOOP_DEVICE, &image_text], None) {
        let _cleanup = fs::remove_file(&image_path);
        return format!("{prefix} mount=not_attempted check=loop_failed detail={}", sanitized(&detail));
    }
    let line = match run_tool(GUEST_BUSYBOX, &["mount", "-t", "ext4", GUEST_LOOP_DEVICE, GUEST_MOUNT_POINT], None) {
        Err(detail) => format!("{prefix} mount=failed check=not_run detail={} dmesg_tail={}", sanitized(&detail), dmesg_tail()),
        Ok(_text) => {
            let library = Path::new(GUEST_MOUNT_POINT).join(GUEST_LIBRARY_NAME);
            let child_arguments: Vec<String> = [
                "--role",
                "s4-check-point",
                "--arm",
                &header.arm.index().to_string(),
                "--library",
                &library.to_string_lossy(),
                "--blocks",
                &header.block_count.to_string(),
                "--states-per-block",
                &header.states_per_block.to_string(),
                "--confirmed-end",
                &item.confirmed_end.to_string(),
                "--started-end",
                &item.started_end.to_string(),
                "--damage",
                item.damage.label(),
            ]
            .iter()
            .map(|text| text.to_string())
            .collect();
            let check_fields = match run_check_child_with_timeout(&child_arguments, Duration::from_secs(CHECK_TIMEOUT_SECONDS)) {
                CheckChildOutcome::Finished { report_line: Some(report_line), .. } => {
                    format!("check=done {}", report_line.trim_start_matches("S4CHECK "))
                }
                CheckChildOutcome::Finished { report_line: None, exit_code } => format!("check=crashed exit_code={exit_code:?}"),
                CheckChildOutcome::TimedOut => "check=timeout q4f=1".to_string(),
                CheckChildOutcome::SpawnFailed(detail) => format!("check=spawn_failed detail={}", sanitized(&detail)),
            };
            let unmounted = run_tool(GUEST_BUSYBOX, &["umount", GUEST_MOUNT_POINT], None).is_ok();
            format!("{prefix} mount=ok {check_fields} unmounted={unmounted}")
        }
    };
    let _detached = run_tool(GUEST_BUSYBOX, &["losetup", "-d", GUEST_LOOP_DEVICE], None);
    let _removed = fs::remove_file(&image_path);
    line
}

// ============================== 来宾：核对子进程 ==============================

fn check_report_fields(report: &VerificationReport) -> String {
    let first_blocks = |blocks: &[u64]| -> String {
        if blocks.is_empty() {
            "-".to_string()
        } else {
            blocks.iter().take(8).map(u64::to_string).collect::<Vec<String>>().join(",")
        }
    };
    format!(
        "q4a={} q4b={} q4c={} q4d={} q4e={} q4f={} opened_after_repair={} open_microseconds={} q4a_blocks={} q4b_blocks={} q4c_blocks={} q4d_blocks={} enumerated_keys={} enumeration_error={} first_read_error={} open_error={}",
        report.lost_confirmed.len(),
        report.silently_corrupted.len(),
        report.confirmed_read_errors.len(),
        report.unconfirmed_read_errors.len(),
        report.phantom_keys.len(),
        u8::from(report.open_failed),
        match report.opened_after_repair {
            None => "not_applicable",
            Some(true) => "yes",
            Some(false) => "no",
        },
        report.open_microseconds,
        first_blocks(&report.lost_confirmed),
        first_blocks(&report.silently_corrupted),
        first_blocks(&report.confirmed_read_errors),
        first_blocks(&report.unconfirmed_read_errors),
        report.enumerated_key_count,
        sanitized(&report.enumeration_error),
        sanitized(&report.first_read_error),
        sanitized(&report.open_error)
    )
}

fn run_check_point_role(arguments: &RoleArguments) -> i32 {
    let parsed = (|| {
        let arm = PowerCutArm::from_index(u32::try_from(arguments.number("arm")?).ok()?)?;
        let candidate = arm.candidate()?;
        let plan = VerificationPlan {
            states_per_block: arguments.number("states-per-block")?,
            block_count: arguments.number("blocks")?,
            confirmed_end: arguments.number("confirmed-end")?,
            started_end: arguments.number("started-end")?,
        };
        Some((candidate, PathBuf::from(arguments.text("library")?), plan, LibraryDamage::parse(arguments.text("damage")?)?))
    })();
    let Some((candidate, library, plan, damage)) = parsed else {
        eprintln!("  ✗ s4-check-point 的参数不齐");
        return USAGE_EXIT_CODE;
    };
    let damage_outcome = apply_library_damage(candidate, &library, damage, plan.states_per_block);
    let report = verify_library(candidate, &library, &plan);
    println!(
        "S4CHECK {} damage_applied={} damage_error={}",
        check_report_fields(&report),
        damage_outcome.is_ok(),
        sanitized(&damage_outcome.err().unwrap_or_default())
    );
    0
}

// ============================== 宿主：附加根（照 research/scripts/e152-stage-root.sh:59-75、:88、:133-137 的办法） ==============================

fn find_on_path(program: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")
        .map(|path_value| std::env::split_paths(&path_value).map(|directory| directory.join(program)).collect::<Vec<PathBuf>>())
        .unwrap_or_default()
        .into_iter()
        .find(|candidate_path| candidate_path.is_file())
}

fn install_file(source: &Path, destination_root: &Path, guest_path: &str, mode: u32) -> Result<(), String> {
    let target = destination_root.join(guest_path.trim_start_matches('/'));
    if target.exists() {
        return Ok(());
    }
    let parent = target.parent().expect("来宾路径有父目录");
    fs::create_dir_all(parent).map_err(|error| format!("建不了 {}：{error}", parent.display()))?;
    let resolved = fs::canonicalize(source).map_err(|error| format!("解不开 {}：{error}", source.display()))?;
    fs::copy(&resolved, &target).map_err(|error| format!("拷不了 {} → {}：{error}", resolved.display(), target.display()))?;
    fs::set_permissions(&target, fs::Permissions::from_mode(mode)).map_err(|error| format!("设不了 {} 的权限：{error}", target.display()))
}

/// `ldd` 解出的库与加载器，按宿主上的路径装进来宾的同名路径，0755（库与加载器都要执行位，`.claude/kb/vm-harness.md`）。
fn install_binary_with_libraries(source: &Path, destination_root: &Path, guest_path: &str) -> Result<usize, String> {
    let resolution = run_tool("ldd", &[&source.to_string_lossy()], None)?;
    if resolution.contains("not found") {
        return Err(format!("{} 有库解析不到：{}", source.display(), sanitized(&resolution)));
    }
    install_file(source, destination_root, guest_path, 0o755)?;
    let mut installed_libraries = 0usize;
    for line in resolution.lines() {
        let trimmed = line.trim();
        let library_path = if let Some((_name, rest)) = trimmed.split_once("=> ") {
            rest.split(" (").next().unwrap_or("").to_string()
        } else if trimmed.starts_with('/') {
            trimmed.split(" (").next().unwrap_or("").to_string()
        } else {
            continue;
        };
        if !library_path.starts_with('/') {
            continue;
        }
        install_file(Path::new(&library_path), destination_root, &library_path, 0o755)?;
        installed_libraries += 1;
    }
    Ok(installed_libraries)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StageSummary {
    file_count: usize,
    total_bytes: u64,
    staged_selftest_passed: bool,
    staged_formatter_version: String,
    staged_partitioner_version: String,
}

fn directory_file_totals(directory: &Path) -> (usize, u64) {
    let Ok(entries) = fs::read_dir(directory) else { return (0, 0) };
    let mut totals = (0usize, 0u64);
    for entry in entries.flatten() {
        let Ok(metadata) = fs::symlink_metadata(entry.path()) else { continue };
        if metadata.is_dir() {
            let (files, bytes) = directory_file_totals(&entry.path());
            totals.0 += files;
            totals.1 += bytes;
        } else if metadata.is_file() {
            totals.0 += 1;
            totals.1 += metadata.len();
        }
    }
    totals
}

/// 写盘附加根：S4 二进制、mkfs.ext4、sfdisk 连同库与加载器，/etc/mke2fs.conf，/etc/mtab；搭完用搭好的加载器实跑三样。
fn stage_write_root(destination: &Path) -> Result<StageSummary, String> {
    fs::create_dir_all(destination).map_err(|error| format!("建不了 {}：{error}", destination.display()))?;
    if fs::read_dir(destination).map_err(|error| error.to_string())?.next().is_some() {
        return Err(format!("{} 不是空目录", destination.display()));
    }
    let executable = std::env::current_exe().map_err(|error| format!("找不到自己的可执行文件：{error}"))?;
    install_binary_with_libraries(&executable, destination, STAGED_BINARY_GUEST_PATH)?;
    // release 配置带 debug = true，二进制约 450 MB；附加根整份进 initramfs，验盘开机还要再带设备侧日志，
    // 所以只拿掉调试信息（代码段不动，搭完用搭好的加载器实跑 selftest 核过）。
    let staged_binary = destination.join(STAGED_BINARY_GUEST_PATH.trim_start_matches('/')).to_string_lossy().to_string();
    run_tool("strip", &["--strip-debug", &staged_binary], None)?;
    let formatter = find_on_path("mkfs.ext4").ok_or("宿主上没有 mkfs.ext4（装 e2fsprogs）")?;
    install_binary_with_libraries(&formatter, destination, GUEST_FILESYSTEM_FORMATTER)?;
    let partitioner = find_on_path("sfdisk").ok_or("宿主上没有 sfdisk（装 fdisk / util-linux）")?;
    install_binary_with_libraries(&partitioner, destination, GUEST_PARTITIONER)?;
    install_file(Path::new("/etc/mke2fs.conf"), destination, "/etc/mke2fs.conf", 0o644)?;
    std::os::unix::fs::symlink("/proc/self/mounts", destination.join("etc/mtab")).map_err(|error| format!("建不了 etc/mtab：{error}"))?;
    let loader = destination.join(STAGED_LOADER_GUEST_PATH.trim_start_matches('/'));
    let library_path = format!("{}:{}", destination.join("lib/x86_64-linux-gnu").display(), destination.join("usr/lib/x86_64-linux-gnu").display());
    let staged = |guest_path: &str| destination.join(guest_path.trim_start_matches('/')).to_string_lossy().to_string();
    let loader_text = loader.to_string_lossy().to_string();
    let selftest = run_tool(&loader_text, &["--library-path", &library_path, &staged(STAGED_BINARY_GUEST_PATH), "--role", "selftest"], None);
    let staged_selftest_passed = selftest.as_ref().is_ok_and(|text| text.contains("anchor_mismatches=0") && text.contains("name=done"));
    let staged_formatter_version = run_tool(&loader_text, &["--library-path", &library_path, &staged(GUEST_FILESYSTEM_FORMATTER), "-V"], None)
        .map(|text| sanitized(text.lines().next().unwrap_or("")))
        .unwrap_or_else(|error| format!("failed:{}", sanitized(&error)));
    let staged_partitioner_version = run_tool(&loader_text, &["--library-path", &library_path, &staged(GUEST_PARTITIONER), "--version"], None)
        .map(|text| sanitized(text.lines().next().unwrap_or("")))
        .unwrap_or_else(|error| format!("failed:{}", sanitized(&error)));
    let (file_count, total_bytes) = directory_file_totals(destination);
    Ok(StageSummary { file_count, total_bytes, staged_selftest_passed, staged_formatter_version, staged_partitioner_version })
}

fn stage_summary_line(destination: &Path, summary: &StageSummary) -> String {
    format!(
        "name=stage_root destination={} files={} bytes={} staged_selftest_passed={} staged_formatter={} staged_partitioner={}",
        destination.display(),
        summary.file_count,
        summary.total_bytes,
        summary.staged_selftest_passed,
        summary.staged_formatter_version,
        summary.staged_partitioner_version
    )
}

fn run_stage_role(arguments: &RoleArguments) -> i32 {
    let Some(destination) = arguments.text("destination") else {
        eprintln!("  ✗ s4-stage 要 --destination <空目录>");
        return USAGE_EXIT_CODE;
    };
    let mut output = ProductLines::to_standard_output();
    let exit_code = match stage_write_root(Path::new(destination)) {
        Ok(summary) => {
            output.line(stage_summary_line(Path::new(destination), &summary));
            if summary.staged_selftest_passed {
                0
            } else {
                1
            }
        }
        Err(detail) => halt_line(&mut output, "H10", "stage", &detail),
    };
    output.finish();
    exit_code
}

/// 验盘附加根：写盘附加根逐个硬链接过来（vm-bench 会整份拷进 initramfs，硬链接只省宿主的盘），再放日志与计划。
fn link_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| format!("建不了 {}：{error}", destination.display()))?;
    for entry in fs::read_dir(source).map_err(|error| format!("列不了 {}：{error}", source.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let target = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.is_dir() {
            link_tree(&entry.path(), &target)?;
        } else if metadata.file_type().is_symlink() {
            let pointee = fs::read_link(entry.path()).map_err(|error| error.to_string())?;
            std::os::unix::fs::symlink(pointee, &target).map_err(|error| error.to_string())?;
        } else {
            fs::hard_link(entry.path(), &target).map_err(|error| format!("硬链接不了 {}：{error}", target.display()))?;
        }
    }
    Ok(())
}

fn copy_log_prefix(log_path: &Path, byte_count: u64, destination: &Path) -> Result<(), String> {
    let source = File::open(log_path).map_err(|error| format!("打不开 {}：{error}", log_path.display()))?;
    let mut target = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| format!("建不了 {}：{error}", destination.display()))?;
    let copied = std::io::copy(&mut source.take(byte_count), &mut target).map_err(|error| error.to_string())?;
    if copied != byte_count {
        return Err(format!("日志只拷了 {copied} 字节，要 {byte_count}"));
    }
    Ok(())
}

// ============================== 宿主：驱动 ==============================

fn heavy_tests_permitted(value: Option<&str>) -> bool {
    value.is_some_and(|value_text| HEAVY_TEST_PERMITTED_VALUES.contains(&value_text))
}

/// 写盘开机一次：臂、块数、在开机次序里的号。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlannedWriteBoot {
    order: usize,
    arm: PowerCutArm,
    block_count: u64,
}

/// 补 12 第四段的开机次序：PC4-C → 各臂 PC4-L、PC4-NB、PC4-FD → 各臂主格 → 各臂 S4-bs10。
fn planned_write_boots(main_arms: &[PowerCutArm]) -> Vec<PlannedWriteBoot> {
    let mut arms = vec![PowerCutArm::WriteCacheModel];
    for &arm in main_arms {
        arms.push(arm.lost_page_cache_control());
        arms.push(arm.lost_write_cache_control());
        if arm == PowerCutArm::HardenedFile {
            arms.push(PowerCutArm::HardenedFileWithoutDirectorySync);
        }
    }
    arms.extend(main_arms.iter().copied());
    arms.extend(main_arms.iter().map(|arm| arm.small_block_sampling_point()));
    arms.into_iter()
        .enumerate()
        .map(|(order, arm)| PlannedWriteBoot { order, arm, block_count: arm.registered_block_count() })
        .collect()
}

/// 一次 vm-bench 调用：环境变量与参数（参数跟在 launcher 之后，来宾里再跟在 /dev/vda 之后）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct VirtualMachineRun {
    environment: Vec<(String, String)>,
    arguments: Vec<String>,
}

fn write_boot_run(boot: &PlannedWriteBoot, log_directory: &Path, write_root: &Path, kernel: &str) -> VirtualMachineRun {
    VirtualMachineRun {
        environment: vec![
            ("VM_BLKLOGWRITES_DIR".to_string(), log_directory.display().to_string()),
            ("VM_DISKS".to_string(), "1".to_string()),
            ("VM_DISK_MB".to_string(), DISK_MEBIBYTES.to_string()),
            ("VM_LOG_MB".to_string(), LOG_DISK_MEBIBYTES.to_string()),
            ("VM_MEM".to_string(), WRITE_BOOT_MEMORY_MEBIBYTES.to_string()),
            ("VM_CPUS".to_string(), WRITE_BOOT_PROCESSORS.to_string()),
            ("VM_TIMEOUT".to_string(), WRITE_BOOT_TIMEOUT_SECONDS.to_string()),
            ("VM_EXTRA_ROOT".to_string(), write_root.display().to_string()),
            ("SINGLEFS_KERNEL".to_string(), kernel.to_string()),
        ],
        arguments: vec![
            "--role".to_string(),
            "s4-write-guest".to_string(),
            "--arm".to_string(),
            boot.arm.index().to_string(),
            "--blocks".to_string(),
            boot.block_count.to_string(),
        ],
    }
}

fn verify_boot_run(verify_root: &Path, kernel: &str) -> VirtualMachineRun {
    VirtualMachineRun {
        environment: vec![
            ("VM_DISKS".to_string(), "1".to_string()),
            ("VM_MEM".to_string(), VERIFY_BOOT_MEMORY_MEBIBYTES.to_string()),
            ("VM_TIMEOUT".to_string(), VERIFY_BOOT_TIMEOUT_SECONDS.to_string()),
            ("VM_EXTRA_ROOT".to_string(), verify_root.display().to_string()),
            ("SINGLEFS_KERNEL".to_string(), kernel.to_string()),
        ],
        arguments: vec![
            "--role".to_string(),
            "s4-verify-guest".to_string(),
            "--log".to_string(),
            GUEST_LOG_PATH.to_string(),
            "--plan".to_string(),
            GUEST_PLAN_PATH.to_string(),
        ],
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn vm_bench_path() -> PathBuf {
    repository_root().join("research").join("scripts").join("vm-bench.sh")
}

fn run_description(run: &VirtualMachineRun, launcher: &Path) -> String {
    let environment: Vec<String> = run.environment.iter().map(|(key, value)| format!("{key}={value}")).collect();
    format!(
        "environment={} command=bash,{},{},{}",
        environment.join(";"),
        vm_bench_path().display(),
        launcher.display(),
        run.arguments.join(",")
    )
}

/// 起 vm-bench，把它的标准输出整份读到 EOF 再解析（不在读的循环里打时间戳）；返回来宾的 E7RESULT 行（去掉前缀）与退出码。
fn execute_virtual_machine(run: &VirtualMachineRun, launcher: &Path) -> (Vec<String>, Option<i32>) {
    let mut command = Command::new("bash");
    command.arg(vm_bench_path()).arg(launcher).args(&run.arguments).stdout(Stdio::piped()).stderr(Stdio::inherit());
    for (key, value) in &run.environment {
        command.env(key, value);
    }
    let Ok(output) = command.output() else { return (Vec::new(), None) };
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let guest_lines = text
        .lines()
        .filter_map(|line| line.find("E7RESULT ").map(|position| line[position + "E7RESULT ".len()..].trim_end().to_string()))
        .collect();
    (guest_lines, output.status.code())
}

/// 来宾的行转进产物：在 `name=…` 之后插上 `boot=<号>`。
fn tagged_guest_line(boot_label: &str, guest_line: &str) -> String {
    match guest_line.split_once(' ') {
        Some((name_field, rest)) => format!("{name_field} boot={boot_label} {rest}"),
        None => format!("{guest_line} boot={boot_label}"),
    }
}

fn available_memory_bytes() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .unwrap_or_default()
        .lines()
        .find_map(|line| line.strip_prefix("MemAvailable:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|kibibytes| kibibytes.parse::<u64>().ok())
        .map_or(0, |kibibytes| kibibytes * 1024)
}

fn available_disk_bytes(directory: &Path) -> u64 {
    let directory_text = directory.to_string_lossy().to_string();
    run_tool("df", &["-B1", "--output=avail", &directory_text], None)
        .ok()
        .and_then(|text| text.lines().last().and_then(|line| line.trim().parse().ok()))
        .unwrap_or(0)
}

/// 补 5.3「宿主资源」：机器上有没有别的虚机或性能测量（按 /proc/*/cmdline 的第一个参数认，不按模式杀任何东西）。
fn interfering_processes() -> Vec<String> {
    let own = std::process::id().to_string();
    let Ok(entries) = fs::read_dir("/proc") else { return Vec::new() };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.chars().all(|character| character.is_ascii_digit()) || name == own {
            continue;
        }
        let Ok(command_line) = fs::read(entry.path().join("cmdline")) else { continue };
        let words: Vec<String> = command_line.split(|&byte| byte == 0).map(|word| String::from_utf8_lossy(word).to_string()).collect();
        let program = words.first().map(|word| word.rsplit('/').next().unwrap_or("").to_string()).unwrap_or_default();
        let script = words.get(1).map(|word| word.rsplit('/').next().unwrap_or("").to_string()).unwrap_or_default();
        let interferes = program.starts_with("qemu-system")
            || program == "fio"
            || program.starts_with("e152-file-system-benchmark")
            || script == "vm-bench.sh";
        if interferes {
            found.push(format!("{name}:{program}"));
        }
    }
    found
}

/// `${TMPDIR:-/tmp}/e162-s4-<pid>/`，Drop 时整目录删掉；E162_KEEP_SCRATCH=1 时留下并打印路径。
struct ScratchDirectory {
    path: PathBuf,
}

impl ScratchDirectory {
    fn create() -> Result<ScratchDirectory, String> {
        let base = std::env::var_os("TMPDIR").map_or_else(|| PathBuf::from("/tmp"), PathBuf::from);
        let path = base.join(format!("e162-s4-{}", std::process::id()));
        fs::create_dir(&path).map_err(|error| format!("建不了 {}：{error}", path.display()))?;
        Ok(ScratchDirectory { path })
    }
}

impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        if std::env::var("E162_KEEP_SCRATCH").is_ok_and(|value| value == "1") {
            eprintln!("E162_KEEP_SCRATCH=1：现场留在 {}", self.path.display());
        } else {
            let _cleanup_outcome = fs::remove_dir_all(&self.path);
        }
    }
}

/// 验盘开机里每个「断电点 × 取法」的结果（从来宾的 s4_point 行读回来）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct PointResult {
    item: VerifyItem,
    mount_ok: bool,
    check_label: String,
    lost_confirmed: u64,
    silently_corrupted: u64,
    confirmed_read_errors: u64,
    unconfirmed_read_errors: u64,
    phantom: u64,
    open_failed: bool,
    open_microseconds: u64,
    silently_corrupted_blocks: String,
}

impl PointResult {
    fn from_line(item: &VerifyItem, line: &str) -> PointResult {
        let number = |key: &str| field_number(line, key).unwrap_or(0);
        PointResult {
            item: item.clone(),
            mount_ok: field_value(line, "mount") == Some("ok"),
            check_label: field_value(line, "check").unwrap_or("missing").to_string(),
            lost_confirmed: number("q4a"),
            silently_corrupted: number("q4b"),
            confirmed_read_errors: number("q4c"),
            unconfirmed_read_errors: number("q4d"),
            phantom: number("q4e"),
            open_failed: number("q4f") == 1,
            open_microseconds: number("open_microseconds"),
            silently_corrupted_blocks: field_value(line, "q4b_blocks").unwrap_or("-").to_string(),
        }
    }

    fn lost_signal(&self) -> bool {
        self.lost_confirmed + self.confirmed_read_errors > 0 || self.open_failed
    }

    fn any_positive(&self) -> bool {
        self.lost_signal() || self.silently_corrupted + self.unconfirmed_read_errors + self.phantom > 0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PointSummary {
    points: usize,
    judged_points: usize,
    lost_confirmed_total: u64,
    corrupted_total: u64,
    unopenable_points: usize,
    positive_points: usize,
    lost_signal_points: usize,
    lost_confirmed_points: usize,
    first_positive_point: Option<usize>,
    peak_single_point: u64,
    last_point_value: u64,
    open_peak_microseconds: u64,
    open_median_microseconds: u64,
    open_last_microseconds: u64,
    slow_opens: usize,
    unjudged: usize,
    mount_failures: usize,
}

fn summarize_points(results: &[&PointResult]) -> PointSummary {
    let mut sorted: Vec<&PointResult> = results.to_vec();
    sorted.sort_by_key(|result| (result.item.point, result.item.item_number));
    let mut summary = PointSummary { points: sorted.len(), ..PointSummary::default() };
    let mut open_times = Vec::new();
    for result in &sorted {
        if !result.mount_ok {
            summary.mount_failures += 1;
        }
        if result.check_label != "done" && result.check_label != "timeout" {
            summary.unjudged += 1;
            continue;
        }
        summary.judged_points += 1;
        summary.lost_confirmed_total += result.lost_confirmed + result.confirmed_read_errors;
        summary.corrupted_total += result.silently_corrupted + result.unconfirmed_read_errors + result.phantom;
        if result.open_failed {
            summary.unopenable_points += 1;
        }
        if result.lost_signal() {
            summary.lost_signal_points += 1;
        }
        if result.lost_confirmed > 0 {
            summary.lost_confirmed_points += 1;
        }
        let point_value = result.lost_confirmed
            + result.confirmed_read_errors
            + result.silently_corrupted
            + result.unconfirmed_read_errors
            + result.phantom
            + u64::from(result.open_failed);
        if point_value > 0 {
            summary.positive_points += 1;
            summary.first_positive_point.get_or_insert(result.item.point);
        }
        summary.peak_single_point = summary.peak_single_point.max(point_value);
        summary.last_point_value = point_value;
        open_times.push(result.open_microseconds);
        summary.open_last_microseconds = result.open_microseconds;
        if result.open_microseconds > SLOW_OPEN_MICROSECONDS {
            summary.slow_opens += 1;
        }
    }
    summary.open_peak_microseconds = open_times.iter().copied().max().unwrap_or(0);
    summary.open_median_microseconds = median_of(&open_times);
    summary
}

fn summary_fields(summary: &PointSummary) -> String {
    format!(
        "points={} judged_points={} unjudged_points={} mount_failures={} lost_confirmed={} corrupted={} unopenable={} positive_points={} first_positive_point={} peak_single_point={} last_point_value={} open_peak_microseconds={} open_median_microseconds={} open_last_microseconds={} opens_over_60_seconds={}",
        summary.points,
        summary.judged_points,
        summary.unjudged,
        summary.mount_failures,
        summary.lost_confirmed_total,
        summary.corrupted_total,
        summary.unopenable_points,
        summary.positive_points,
        summary.first_positive_point.map_or_else(|| "none".to_string(), |point| point.to_string()),
        summary.peak_single_point,
        summary.last_point_value,
        summary.open_peak_microseconds,
        summary.open_median_microseconds,
        summary.open_last_microseconds,
        summary.slow_opens
    )
}

/// 一次写盘开机连同它的验盘之后留下的、判定要用的东西。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BootOutcome {
    replay_fidelity_holds: Option<bool>,
    markers_hold: Option<bool>,
    write_cache_is_write_back: Option<bool>,
    barrier_flushes_between_markers: Option<usize>,
    unflushed_stratified_points: Option<usize>,
    rocksdb_background_holds: Option<bool>,
    points: Vec<PointResult>,
}

/// 驱动的停机：条款与说明。
struct DriveHalt {
    clause: String,
    detail: String,
}

struct DriveContext {
    kernel: String,
    launcher: PathBuf,
    write_root: PathBuf,
    scratch: PathBuf,
}

fn sha256_of_file(path: &Path) -> Result<String, String> {
    let text = run_tool("sha256sum", &[&path.to_string_lossy()], None)?;
    text.split_whitespace().next().map(str::to_string).ok_or_else(|| "sha256sum 没有输出".to_string())
}

/// PC4-R：把这次开机的全部条目重放到全 0 的镜像上，与它的 disk0.img 整盘 sha256 比。
fn replay_fidelity(output: &mut ProductLines, boot_label: &str, boot_directory: &Path, log: &LogBytes, device_log: &DeviceLog) -> Result<bool, String> {
    let disk_path = boot_directory.join("disk0.img");
    let disk_length = fs::metadata(&disk_path).map_err(|error| format!("读不到 disk0.img：{error}"))?.len();
    let replay_path = boot_directory.join("replay-all.img");
    let replay_file = OpenOptions::new().write(true).create_new(true).open(&replay_path).map_err(|error| error.to_string())?;
    replay_file.set_len(disk_length).map_err(|error| error.to_string())?;
    let selection = select_replayed_writes(&device_log.entries, device_log.entries.len(), ReplayVariant::KeepUnflushed, 0, 0);
    let mut image = ReplayImage::OnDisk(replay_file);
    apply_replayed_writes(log, &selection.writes, &mut image)?;
    drop(image);
    let replay_digest = sha256_of_file(&replay_path)?;
    let disk_digest = sha256_of_file(&disk_path)?;
    let _removed = fs::remove_file(&replay_path);
    let identical = replay_digest == disk_digest;
    output.line(format!(
        "name=pc4_replay_fidelity boot={boot_label} entries={} replay_sha256={replay_digest} disk_sha256={disk_digest} identical={identical}",
        device_log.entries.len()
    ));
    Ok(identical)
}

fn log_summary_line(boot_label: &str, device_log: &DeviceLog, entries_until_zero_header: usize) -> String {
    let mut writes = 0usize;
    let mut forced_writes = 0usize;
    let mut flushes = 0usize;
    let mut discards = 0usize;
    for entry in &device_log.entries {
        match entry {
            LogEntry::Write { is_forced_unit_access, .. } => {
                writes += 1;
                if *is_forced_unit_access {
                    forced_writes += 1;
                }
            }
            LogEntry::Flush => flushes += 1,
            LogEntry::Discard { .. } => discards += 1,
        }
    }
    format!(
        "name=log_summary boot={boot_label} declared_entries={} parsed_entries={} entries_until_zero_header={entries_until_zero_header} writes={writes} fua_writes={forced_writes} flushes={flushes} discards={discards} end_offset_in_log={}",
        device_log.declared_entry_count.map_or_else(|| "none".to_string(), |declared| declared.to_string()),
        device_log.entries.len(),
        device_log.end_offset_in_log
    )
}

/// 一批验盘开机：搭验盘附加根、起虚机、收 s4_point 行；缺行的「断电点 × 取法」重做一次（V30），还缺就停机 H10。
fn run_verify_batches(
    output: &mut ProductLines,
    context: &DriveContext,
    boot: &PlannedWriteBoot,
    boot_directory: &Path,
    device_log: &DeviceLog,
    header: &PlanHeader,
    items: &[VerifyItem],
) -> Result<Vec<PointResult>, DriveHalt> {
    let mut results = Vec::new();
    for (batch_number, batch) in items.chunks(VERIFY_BATCH_ITEM_LIMIT).enumerate() {
        let mut attempt = 0usize;
        loop {
            let verify_root = context.scratch.join(format!("verify-root-{}-{batch_number}-{attempt}", boot.order));
            let staged = link_tree(&context.write_root, &verify_root)
                .and_then(|()| fs::create_dir_all(verify_root.join("s4")).map_err(|error| error.to_string()))
                .and_then(|()| copy_log_prefix(&boot_directory.join("log0.img"), device_log.end_offset_in_log, &verify_root.join(GUEST_LOG_PATH.trim_start_matches('/'))))
                .and_then(|()| {
                    let mut plan_text = plan_header_line(header);
                    plan_text.push('\n');
                    for item in batch {
                        plan_text.push_str(&item.plan_line());
                        plan_text.push('\n');
                    }
                    fs::write(verify_root.join(GUEST_PLAN_PATH.trim_start_matches('/')), plan_text).map_err(|error| error.to_string())
                });
            if let Err(detail) = staged {
                let _cleanup = fs::remove_dir_all(&verify_root);
                return Err(DriveHalt { clause: "H10".to_string(), detail: format!("搭不出验盘附加根：{detail}") });
            }
            let run = verify_boot_run(&verify_root, &context.kernel);
            let boot_label = format!("{}-{}-verify{batch_number}-attempt{attempt}", boot.order, boot.arm.label());
            output.line(format!("name=boot_started boot={boot_label} kind=verify items={} {}", batch.len(), run_description(&run, &context.launcher)));
            let (guest_lines, exit_code) = execute_virtual_machine(&run, &context.launcher);
            let _cleanup = fs::remove_dir_all(&verify_root);
            let mut lines_by_item: BTreeMap<usize, String> = BTreeMap::new();
            for guest_line in &guest_lines {
                output.line(tagged_guest_line(&boot_label, guest_line));
                if guest_line.starts_with("name=s4_point ") {
                    if let Some(item_number) = field_number(guest_line, "item").and_then(|number| usize::try_from(number).ok()) {
                        lines_by_item.insert(item_number, guest_line.clone());
                    }
                }
            }
            let has_done = guest_lines.iter().any(|line| line.starts_with("name=done "));
            let missing: Vec<usize> = batch.iter().map(|item| item.item_number).filter(|number| !lines_by_item.contains_key(number)).collect();
            output.line(format!(
                "name=boot_finished boot={boot_label} exit_code={exit_code:?} guest_lines={} done_line={has_done} missing_items={}",
                guest_lines.len(),
                missing.len()
            ));
            if exit_code == Some(0) && has_done && missing.is_empty() {
                results.extend(batch.iter().map(|item| PointResult::from_line(item, &lines_by_item[&item.item_number])));
                if results.iter().any(|result| result.check_label == "loop_failed" || result.check_label == "image_failed") {
                    return Err(DriveHalt { clause: "H8".to_string(), detail: format!("{boot_label} 里有断电点 losetup 失败或 tmpfs 上造不出镜像") });
                }
                if results.iter().any(|result| !result.mount_ok && result.check_label == "not_run") {
                    return Err(DriveHalt { clause: "H7".to_string(), detail: format!("{} 的某个断电点重放出的盘面挂不上 ext4（Q4m > 0）", boot_label) });
                }
                break;
            }
            if attempt >= 1 {
                return Err(DriveHalt { clause: "H10".to_string(), detail: format!("{boot_label} 重做一次仍缺收尾行或缺 {} 个断电点（V30）", missing.len()) });
            }
            output.line(format!("name=v30_redo boot={boot_label} missing_items={}", missing.len()));
            attempt += 1;
        }
    }
    Ok(results)
}

fn guest_confirmed_blocks(guest_lines: &[String]) -> Option<u64> {
    guest_lines.iter().find(|line| line.starts_with("name=write_guest ")).and_then(|line| field_number(line, "confirmed"))
}

/// 写盘开机一次，连同 PC4-R、标记核对、分桶与它的全部验盘开机；K 的主格照 V25 翻倍重做。
fn run_one_write_boot(output: &mut ProductLines, context: &DriveContext, planned: &PlannedWriteBoot) -> Result<BootOutcome, DriveHalt> {
    let mut boot = *planned;
    let mut redo_count = 0usize;
    loop {
        let boot_label = format!("{}-{}-write{redo_count}", boot.order, boot.arm.label());
        let boot_directory = context.scratch.join(format!("boot-{boot_label}"));
        fs::create_dir(&boot_directory).map_err(|error| DriveHalt { clause: "H10".to_string(), detail: error.to_string() })?;
        let run = write_boot_run(&boot, &boot_directory, &context.write_root, &context.kernel);
        output.line(format!("name=boot_started boot={boot_label} kind=write arm={} arm_index={} blocks={} {}", boot.arm.label(), boot.arm.index(), boot.block_count, run_description(&run, &context.launcher)));
        let (guest_lines, exit_code) = execute_virtual_machine(&run, &context.launcher);
        for guest_line in &guest_lines {
            output.line(tagged_guest_line(&boot_label, guest_line));
        }
        output.line(format!("name=boot_finished boot={boot_label} exit_code={exit_code:?} guest_lines={}", guest_lines.len()));
        if exit_code != Some(0) {
            let clause = guest_lines.iter().find(|line| line.starts_with("name=halt ")).and_then(|line| field_value(line, "clause")).unwrap_or("H10").to_string();
            let _cleanup = fs::remove_dir_all(&boot_directory);
            return Err(DriveHalt { clause, detail: format!("写盘开机 {boot_label} 退出码 {exit_code:?}") });
        }
        let outcome = judge_write_boot(output, context, &boot, &boot_label, &boot_directory, &guest_lines);
        let redo_for_background = match &outcome {
            Ok(result) => {
                result.rocksdb_background_holds == Some(false) && boot.arm == PowerCutArm::RocksDatabase && boot.block_count * 2 <= ROCKSDB_REDO_BLOCK_LIMIT
            }
            Err(_halt) => false,
        };
        let _cleanup = fs::remove_dir_all(&boot_directory);
        if !redo_for_background {
            return outcome;
        }
        output.line(format!("name=v25_redo boot={boot_label} blocks_before={} blocks_after={}", boot.block_count, boot.block_count * 2));
        boot.block_count *= 2;
        redo_count += 1;
    }
}

fn judge_write_boot(
    output: &mut ProductLines,
    context: &DriveContext,
    boot: &PlannedWriteBoot,
    boot_label: &str,
    boot_directory: &Path,
    guest_lines: &[String],
) -> Result<BootOutcome, DriveHalt> {
    let halt = |clause: &str, detail: String| DriveHalt { clause: clause.to_string(), detail };
    let log = LogBytes::open(&boot_directory.join("log0.img")).map_err(|detail| halt("H9", detail))?;
    let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).map_err(|error| halt("H9", error.to_string()))?;
    let entries_until_zero_header = parse_device_log(&log, DeclaredCountHandling::ReadUntilZeroHeader).map_or(0, |until_zero| until_zero.entries.len());
    output.line(log_summary_line(boot_label, &device_log, entries_until_zero_header));
    let mut outcome = BootOutcome::default();
    let fidelity = replay_fidelity(output, boot_label, boot_directory, &log, &device_log).map_err(|detail| halt("H10", detail))?;
    outcome.replay_fidelity_holds = Some(fidelity);
    if !fidelity && entries_until_zero_header != device_log.entries.len() {
        return Err(halt("H9", format!("{boot_label}：声明条目数 {} 与读到全 0 为止的 {entries_until_zero_header} 不同，PC4-R 又过不了", device_log.entries.len())));
    }
    let environment_line = guest_lines.iter().find(|line| line.starts_with("name=guest_environment "));
    outcome.write_cache_is_write_back = environment_line.map(|line| field_value(line, "write_cache") == Some("write_back"));
    if boot.arm == PowerCutArm::WriteCacheModel {
        let judgement = judge_write_cache_model(&log, &device_log).map_err(|detail| halt("H10", detail))?;
        let lost_fields: Vec<String> = judgement
            .lost_by_variant
            .iter()
            .map(|(variant, first_lost, second_lost)| format!("{}_first_lost={first_lost} {}_second_lost={second_lost}", variant.label(), variant.label()))
            .collect();
        output.line(format!(
            "name=verdict part=pc4_c boot={boot_label} point={} entries_between_batches={} flushes_between_batches={} {} registered_counts_hold={}",
            judgement.point,
            judgement.entries_between_batches,
            judgement.flushes_between_batches,
            lost_fields.join(" "),
            judgement.registered_counts_hold
        ));
        outcome.markers_hold = Some(judgement.registered_counts_hold);
        return Ok(outcome);
    }
    let scan = scan_markers(&log, &device_log).map_err(|detail| halt("H10", detail))?;
    let guest_confirmed = guest_confirmed_blocks(guest_lines).unwrap_or(u64::MAX);
    let audit = audit_markers(&scan, boot.arm.index(), guest_confirmed);
    output.line(format!(
        "name=marker_check boot={boot_label} markers={} confirm_markers={} guest_confirmed={guest_confirmed} problems={} markers_hold={}",
        scan.markers.len(),
        audit.confirm_count,
        if audit.problems.is_empty() { "none".to_string() } else { audit.problems.join(",") },
        audit.windows.is_some()
    ));
    outcome.markers_hold = Some(audit.windows.is_some());
    let Some(windows) = audit.windows else {
        return Ok(outcome);
    };
    let discards_between = device_log.entries[windows.start_entry..windows.end_entry].iter().filter(|entry| matches!(entry, LogEntry::Discard { .. })).count();
    let flushes_between = device_log.entries[windows.start_entry..windows.end_entry].iter().filter(|entry| **entry == LogEntry::Flush).count();
    outcome.barrier_flushes_between_markers = Some(flushes_between);
    if discards_between > 0 {
        return Err(halt("H14", format!("{boot_label}：S 与 E 之间有 {discards_between} 个 DISCARD 条目")));
    }
    if boot.arm == PowerCutArm::RocksDatabase {
        let background = guest_lines.iter().find(|line| line.starts_with("name=rocksdb_background "));
        let flushes = background.and_then(|line| field_number(line, "flushes")).unwrap_or(0);
        let compactions = background.and_then(|line| field_number(line, "compactions")).unwrap_or(0);
        let holds = flushes >= ROCKSDB_FLUSH_MINIMUM && compactions >= ROCKSDB_COMPACTION_MINIMUM;
        outcome.rocksdb_background_holds = Some(holds);
        output.line(format!("name=q4q boot={boot_label} flushes={flushes} compactions={compactions} background_writes_in_sampled_range={holds}"));
        if !holds && boot.block_count * 2 <= ROCKSDB_REDO_BLOCK_LIMIT {
            return Ok(outcome);
        }
    }
    let pools = bucket_pools(&windows);
    let items = verification_items(boot.arm, &windows, &pools, device_log.entries.len());
    let stratified_points: Vec<usize> = items
        .iter()
        .filter(|item| matches!(item.group, PointGroup::StratifiedNormalCommits | PointGroup::StratifiedLongCommits | PointGroup::StratifiedBetweenCommits))
        .map(|item| item.point)
        .collect::<std::collections::BTreeSet<usize>>()
        .into_iter()
        .collect();
    let unflushed_points = stratified_points.iter().filter(|&&point| unflushed_write_count(&device_log.entries, point) > 0).count();
    let inside_commit_points = items
        .iter()
        .filter(|item| matches!(item.group, PointGroup::StratifiedNormalCommits | PointGroup::StratifiedLongCommits) && item.variant == ReplayVariant::DropUnflushed)
        .count();
    outcome.unflushed_stratified_points = Some(unflushed_points);
    output.line(format!(
        "name=cut_plan boot={boot_label} start_entry={} end_entry={} median_window_entries={} normal_windows={} long_windows={} pool_normal={} pool_long={} pool_between={} stratified_points={} q4u_unflushed_points={unflushed_points} q4g_inside_commit_points={inside_commit_points} items={}",
        windows.start_entry,
        windows.end_entry,
        pools.median_window_entries,
        pools.normal_blocks.len(),
        pools.long_blocks.len(),
        pools.normal_commit_points.len(),
        pools.long_commit_points.len(),
        pools.between_commit_points.len(),
        stratified_points.len(),
        items.len()
    ));
    let disk_bytes = fs::metadata(boot_directory.join("disk0.img")).map_err(|error| halt("H10", error.to_string()))?.len();
    let header = PlanHeader {
        arm: boot.arm,
        block_count: boot.block_count,
        states_per_block: boot.arm.states_per_block(),
        disk_bytes,
        start_entry: windows.start_entry,
        end_entry: windows.end_entry,
    };
    outcome.points = run_verify_batches(output, context, boot, boot_directory, &device_log, &header, &items)?;
    Ok(outcome)
}

fn damage_verdict_line(arm: PowerCutArm, damage: LibraryDamage, result: Option<&PointResult>) -> String {
    let matches = result.is_some_and(|result| match damage {
        LibraryDamage::SilentCorruption => {
            result.silently_corrupted == 1
                && result.silently_corrupted_blocks == SILENT_CORRUPTION_BLOCK.to_string()
                && result.lost_confirmed + result.confirmed_read_errors + result.unconfirmed_read_errors + result.phantom == 0
                && !result.open_failed
        }
        LibraryDamage::PhantomKey => {
            result.phantom == 1 && result.lost_confirmed + result.silently_corrupted + result.confirmed_read_errors + result.unconfirmed_read_errors == 0 && !result.open_failed
        }
        LibraryDamage::OpenBreaking => result.open_failed,
        LibraryDamage::Untouched => false,
    });
    let part = match damage {
        LibraryDamage::SilentCorruption => "pc4_s",
        LibraryDamage::PhantomKey => "pc4_p",
        LibraryDamage::OpenBreaking => "pc4_o",
        LibraryDamage::Untouched => "pc4_untouched",
    };
    format!(
        "name=verdict part={part} arm={} q4a={} q4b={} q4b_blocks={} q4c={} q4d={} q4e={} q4f={} matches_registration={matches}",
        arm.label(),
        result.map_or(0, |result| result.lost_confirmed),
        result.map_or(0, |result| result.silently_corrupted),
        result.map_or("-", |result| result.silently_corrupted_blocks.as_str()),
        result.map_or(0, |result| result.confirmed_read_errors),
        result.map_or(0, |result| result.unconfirmed_read_errors),
        result.map_or(0, |result| result.phantom),
        result.map_or(0, |result| u8::from(result.open_failed))
    )
}

/// 补 6、补 10、补 11：每条主臂的判定行与各阳性对照、作废条款的判定行。
fn emit_verdicts(output: &mut ProductLines, main_arms: &[PowerCutArm], outcomes: &BTreeMap<u32, BootOutcome>) {
    let model_holds = outcomes.get(&PowerCutArm::WriteCacheModel.index()).and_then(|outcome| outcome.markers_hold).unwrap_or(false);
    output.line(format!("name=verdict part=v17_write_cache_model write_cache_model_holds={model_holds}"));
    let mut decision_flips = Vec::new();
    for &arm in main_arms {
        let empty = BootOutcome::default();
        let main = outcomes.get(&arm.index()).unwrap_or(&empty);
        let page_cache_control = outcomes.get(&arm.lost_page_cache_control().index()).unwrap_or(&empty);
        let write_cache_control = outcomes.get(&arm.lost_write_cache_control().index()).unwrap_or(&empty);
        let page_cache_signal = page_cache_control.points.iter().filter(|result| result.lost_signal()).count();
        let write_cache_signal = write_cache_control.points.iter().filter(|result| result.lost_signal()).count();
        let write_cache_control_holds = write_cache_control.barrier_flushes_between_markers == Some(0);
        output.line(format!(
            "name=verdict part=pc4_l arm={} control={} points={} signal_points={page_cache_signal} required={CONTROL_SIGNAL_POINT_MINIMUM} sees_lost_page_cache={}",
            arm.label(),
            arm.lost_page_cache_control().label(),
            page_cache_control.points.len(),
            page_cache_signal >= CONTROL_SIGNAL_POINT_MINIMUM
        ));
        output.line(format!(
            "name=verdict part=pc4_nb arm={} control={} points={} signal_points={write_cache_signal} required={CONTROL_SIGNAL_POINT_MINIMUM} sees_lost_write_cache={} barrier_flushes_between_markers={} barrier_off_holds={write_cache_control_holds}",
            arm.label(),
            arm.lost_write_cache_control().label(),
            write_cache_control.points.len(),
            write_cache_signal >= CONTROL_SIGNAL_POINT_MINIMUM,
            write_cache_control.barrier_flushes_between_markers.map_or_else(|| "unknown".to_string(), |count| count.to_string())
        ));
        if arm == PowerCutArm::HardenedFile {
            let directory_control = outcomes.get(&PowerCutArm::HardenedFileWithoutDirectorySync.index()).unwrap_or(&empty);
            let directory_signal = directory_control.points.iter().filter(|result| result.lost_confirmed > 0).count();
            output.line(format!(
                "name=verdict part=pc4_fd arm=F control=F-nodirsync points={} q4a_points={directory_signal} required={CONTROL_SIGNAL_POINT_MINIMUM} sees_missing_directory_sync={}",
                directory_control.points.len(),
                directory_signal >= CONTROL_SIGNAL_POINT_MINIMUM
            ));
        }
        for damage in [LibraryDamage::SilentCorruption, LibraryDamage::PhantomKey, LibraryDamage::OpenBreaking] {
            let result = main.points.iter().find(|result| result.item.damage == damage);
            output.line(damage_verdict_line(arm, damage, result));
        }
        let validity = format!(
            "replay_fidelity_holds={} markers_hold={} write_cache_is_write_back={} q4u_unflushed_points={} unflushed_points_enough={}{}",
            main.replay_fidelity_holds.unwrap_or(false),
            main.markers_hold.unwrap_or(false),
            main.write_cache_is_write_back.unwrap_or(false),
            main.unflushed_stratified_points.map_or_else(|| "unknown".to_string(), |count| count.to_string()),
            main.unflushed_stratified_points.is_some_and(|count| count >= UNFLUSHED_POINT_MINIMUM),
            if arm == PowerCutArm::RocksDatabase {
                format!(" background_writes_in_sampled_range={}", main.rocksdb_background_holds.unwrap_or(false))
            } else {
                String::new()
            }
        );
        let decision_points: Vec<&PointResult> = main
            .points
            .iter()
            .filter(|result| result.item.damage == LibraryDamage::Untouched && result.item.variant == ReplayVariant::DropUnflushed)
            .collect();
        let decision = summarize_points(&decision_points);
        let flips = decision_points.iter().any(|result| result.any_positive());
        decision_flips.push((arm, flips));
        output.line(format!("name=verdict part=s4_decision arm={} reading=U-drop groups=C-strat,C-commit {} {validity} flips={flips}", arm.label(), summary_fields(&decision)));
        for variant in [ReplayVariant::KeepUnflushed, ReplayVariant::RandomSubset, ReplayVariant::RandomSubsetWithTear] {
            let sampled: Vec<&PointResult> = main
                .points
                .iter()
                .filter(|result| result.item.damage == LibraryDamage::Untouched && result.item.variant == variant)
                .collect();
            let sampled_flips = sampled.iter().any(|result| result.any_positive());
            output.line(format!("name=verdict part=s4_sampling arm={} reading={} {} flips={sampled_flips}", arm.label(), variant.label(), summary_fields(&summarize_points(&sampled))));
        }
        let small = outcomes.get(&arm.small_block_sampling_point().index()).unwrap_or(&empty);
        for variant in [ReplayVariant::DropUnflushed, ReplayVariant::RandomSubset] {
            let sampled: Vec<&PointResult> = small.points.iter().filter(|result| result.item.variant == variant).collect();
            let sampled_flips = sampled.iter().any(|result| result.any_positive());
            output.line(format!(
                "name=verdict part=s4_sampling arm={} reading=bs10-{} {} replay_fidelity_holds={} markers_hold={} flips={sampled_flips}",
                arm.label(),
                variant.label(),
                summary_fields(&summarize_points(&sampled)),
                small.replay_fidelity_holds.unwrap_or(false),
                small.markers_hold.unwrap_or(false)
            ));
        }
    }
    let every_arm_flips = !decision_flips.is_empty() && decision_flips.iter().all(|(_arm, flips)| *flips);
    output.line(format!("name=verdict part=f13_all_arms_flip reading=U-drop arms={} every_arm_flips={every_arm_flips}", decision_flips.len()));
}

fn parse_main_arms(text: &str) -> Option<Vec<PowerCutArm>> {
    let arms: Option<Vec<PowerCutArm>> = text.split(',').map(PowerCutArm::main_arm_from_label).collect();
    arms.filter(|parsed| !parsed.is_empty())
}

fn write_launcher(scratch: &Path) -> Result<PathBuf, String> {
    let launcher = scratch.join("launcher.sh");
    fs::write(&launcher, format!("#!/bin/sh\nexec {STAGED_BINARY_GUEST_PATH} \"$@\"\n")).map_err(|error| error.to_string())?;
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).map_err(|error| error.to_string())?;
    Ok(launcher)
}

/// 验盘开机的「断电点 × 取法」数的上界（干跑时打印用，真数要等写盘开机的日志）。
fn verify_items_upper_bound(arm: PowerCutArm) -> usize {
    match arm.purpose() {
        ArmPurpose::Main => STRATIFIED_POINT_TOTAL * ALL_REPLAY_VARIANTS.len() + PICKED_COMMIT_TOTAL * COMMIT_BOUNDARY_LIMIT * 2 + 3,
        ArmPurpose::SmallBlockSamplingPoint => SMALL_BLOCK_POINT_TOTAL * 2,
        ArmPurpose::LostPageCacheControl | ArmPurpose::LostWriteCacheControl | ArmPurpose::MissingDirectorySyncControl => CONTROL_POINT_TOTAL,
        ArmPurpose::WriteCacheModelControl => 0,
    }
}

fn run_drive_role(arguments: &RoleArguments, forced_summary: Option<&str>) -> i32 {
    let is_dry_run = arguments.flag("dry-run");
    if !is_dry_run && !heavy_tests_permitted(std::env::var(HEAVY_TEST_ENVIRONMENT_VARIABLE).ok().as_deref()) {
        eprintln!("  ✗ s4-drive 起 QEMU 虚机，是重型测试；只在用户要求或提交时带 SINGLEFS_HEAVY_TESTS=user-request（或 commit）跑");
        eprintln!("  → 怎么办：要先看它会起什么，加 --dry-run（只搭附加根、打印每条 vm-bench 命令，不起虚机）");
        return HEAVY_TEST_REFUSAL_EXIT_CODE;
    }
    let Some(main_arms) = arguments.text("arms").and_then(parse_main_arms) else {
        eprintln!("  ✗ --arms 要写成 R1,R0,K,F 的子集（逗号分隔）");
        return USAGE_EXIT_CODE;
    };
    let kernel = arguments.text("kernel").map(str::to_string);
    if !is_dry_run && kernel.as_deref().is_none_or(|path| !Path::new(path).is_file()) {
        eprintln!("  ✗ --kernel 要给一个可读的内核镜像（补 5.2「内核」）");
        return USAGE_EXIT_CODE;
    }
    let kernel_text = kernel.unwrap_or_else(|| "<kernel>".to_string());
    let mut output = match arguments.text("out") {
        Some(path) => match ProductLines::with_file_copy(Path::new(path)) {
            Ok(output) => output,
            Err(detail) => {
                eprintln!("  ✗ {detail}");
                return USAGE_EXIT_CODE;
            }
        },
        None => ProductLines::to_standard_output(),
    };
    if let Some(summary) = forced_summary {
        output.line(format!("name=preflight forced={summary}"));
    }
    let exit_code = drive_body(&mut output, &main_arms, &kernel_text, is_dry_run);
    output.finish();
    exit_code
}

fn drive_body(output: &mut ProductLines, main_arms: &[PowerCutArm], kernel: &str, is_dry_run: bool) -> i32 {
    let arm_labels: Vec<&str> = main_arms.iter().map(|arm| arm.label()).collect();
    output.line(format!(
        "name=s4_drive_config dry_run={is_dry_run} arms={} kernel={kernel} main_blocks={MAIN_BLOCK_COUNT} control_blocks={CONTROL_BLOCK_COUNT} states_per_block={PRIMARY_STATES_PER_BLOCK} small_states_per_block={SMALL_STATES_PER_BLOCK}",
        arm_labels.join(",")
    ));
    let mismatches = run_anchors(output);
    output.line(format!("name=verdict part=anchors anchor_mismatches={mismatches}"));
    if mismatches != 0 {
        return 1;
    }
    let scratch = match ScratchDirectory::create() {
        Ok(scratch) => scratch,
        Err(detail) => return halt_line(output, "H10", "scratch", &detail),
    };
    let memory = available_memory_bytes();
    let disk = available_disk_bytes(&scratch.path);
    let interfering = interfering_processes();
    let resources_ok = memory >= MINIMUM_AVAILABLE_MEMORY_BYTES && disk >= MINIMUM_AVAILABLE_DISK_BYTES && interfering.is_empty();
    output.line(format!(
        "name=host_resources scratch={} available_memory_bytes={memory} available_disk_bytes={disk} interfering_processes={} resources_ok={resources_ok}",
        scratch.path.display(),
        if interfering.is_empty() { "none".to_string() } else { interfering.join(",") }
    ));
    let write_root = scratch.path.join("write-root");
    match stage_write_root(&write_root) {
        Ok(summary) => {
            output.line(stage_summary_line(&write_root, &summary));
            if !summary.staged_selftest_passed {
                return halt_line(output, "V10", "staged-selftest", "搭好的加载器跑搭好的二进制 --role selftest 没过");
            }
        }
        Err(detail) => return halt_line(output, "H10", "stage", &detail),
    }
    let launcher = match write_launcher(&scratch.path) {
        Ok(launcher) => launcher,
        Err(detail) => return halt_line(output, "H10", "launcher", &detail),
    };
    let context = DriveContext { kernel: kernel.to_string(), launcher, write_root, scratch: scratch.path.clone() };
    let boots = planned_write_boots(main_arms);
    if is_dry_run {
        for boot in &boots {
            let boot_label = format!("{}-{}-write0", boot.order, boot.arm.label());
            let run = write_boot_run(boot, &context.scratch.join(format!("boot-{boot_label}")), &context.write_root, kernel);
            output.line(format!(
                "name=planned_boot order={} kind=write arm={} arm_index={} blocks={} states_per_block={} {}",
                boot.order,
                boot.arm.label(),
                boot.arm.index(),
                boot.block_count,
                boot.arm.states_per_block(),
                run_description(&run, &context.launcher)
            ));
            let items_upper_bound = verify_items_upper_bound(boot.arm);
            if items_upper_bound > 0 {
                let batches = items_upper_bound.div_ceil(VERIFY_BATCH_ITEM_LIMIT);
                let run = verify_boot_run(&context.scratch.join(format!("verify-root-{}-<batch>-<attempt>", boot.order)), kernel);
                output.line(format!(
                    "name=planned_boot order={} kind=verify after_write_boot={boot_label} items_at_most={items_upper_bound} batches_at_most={batches} {}",
                    boot.order,
                    run_description(&run, &context.launcher)
                ));
            }
        }
        output.line(format!("name=dry_run_summary write_boots={} note=not_started", boots.len()));
        return 0;
    }
    if !resources_ok {
        return halt_line(output, "precondition", "host-resources", "可用内存 < 16 GiB、可用盘 < 40 GiB，或机器上有别的虚机与性能测量（补 5.3）");
    }
    let mut outcomes: BTreeMap<u32, BootOutcome> = BTreeMap::new();
    for boot in &boots {
        match run_one_write_boot(output, &context, boot) {
            Ok(outcome) => {
                outcomes.insert(boot.arm.index(), outcome);
            }
            Err(halt) => {
                emit_verdicts(output, main_arms, &outcomes);
                return halt_line(output, &halt.clause, &format!("boot-{}-{}", boot.order, boot.arm.label()), &halt.detail);
            }
        }
    }
    emit_verdicts(output, main_arms, &outcomes);
    0
}

// ============================== 单测 ==============================

#[cfg(test)]
mod tests {
    use super::*;

    /// 单测自己的临时目录：`${TMPDIR:-/tmp}/e162-s4-test-<pid>-<名>`，Drop 时删掉。
    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new(name: &str) -> TestDirectory {
            let path = std::env::temp_dir().join(format!("e162-s4-test-{}-{name}", std::process::id()));
            let _stale = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("建得了测试目录");
            TestDirectory { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _cleanup = fs::remove_dir_all(&self.path);
        }
    }

    const MAIN_CANDIDATES: [Candidate; 4] = [Candidate::RedbQuickRepair, Candidate::RedbDefault, Candidate::RocksDatabase, Candidate::HardenedFile];

    fn quiet_output() -> ProductLines {
        ProductLines::discarding()
    }

    /// 在宿主目录上建一个库、写前 written 块（seed_S1、2¹⁶ 状态）。
    fn host_library(directory: &Path, candidate: Candidate, written: u64) -> PathBuf {
        let library = directory.join(format!("library-{}", candidate.label()));
        let mut store = open_store_for_writing(candidate, WriteSettings::registered_for(candidate), &library).expect("宿主目录上打得开新库");
        let fingerprint = input_fingerprint();
        for block_index in 0..written {
            let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
            store.put_block(&key, &value).expect("宿主目录上写得进");
        }
        library
    }

    fn full_plan(block_count: u64) -> VerificationPlan {
        VerificationPlan { states_per_block: PRIMARY_STATES_PER_BLOCK, block_count, confirmed_end: block_count, started_end: block_count }
    }

    #[test]
    fn every_registered_anchor_matches_the_independent_script() {
        let mut output = quiet_output();
        assert_eq!(run_anchors(&mut output), 0, "登记 7.2 与补 7 的锚点应当全对（补 13 锚点脚本的原样输出）");
    }

    #[test]
    fn sha256_matches_the_standard_test_vectors() {
        assert_eq!(sha256_hexadecimal(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(sha256_hexadecimal(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let fifty_six_bytes = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(sha256_hexadecimal(fifty_six_bytes), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
    }

    #[test]
    fn marker_round_trips_and_rejects_a_flipped_byte() {
        let marker = Marker { kind: MarkerKind::Begin, sequence: 12_001, block_index: Some(23_999), arm_index: 17 };
        let sector = encode_marker(&marker);
        assert_eq!(decode_marker(&sector), MarkerDecoding::Valid(marker));
        let start = Marker { kind: MarkerKind::Start, sequence: 0, block_index: None, arm_index: 0 };
        assert_eq!(decode_marker(&encode_marker(&start)), MarkerDecoding::Valid(start));
        let mut flipped = sector;
        flipped[9] ^= 1;
        assert_eq!(decode_marker(&flipped), MarkerDecoding::ChecksumMismatch);
        assert_eq!(decode_marker(&[0u8; SECTOR_BYTES]), MarkerDecoding::NotAMarker);
    }

    #[test]
    fn parser_stops_at_the_declared_entry_count_when_a_stale_entry_follows() {
        let mut entries = synthetic_anchor_log_entries();
        entries.push((20, 1, 0, vec![0x66; 512]));
        let log = LogBytes::InMemory(synthetic_log_bytes(&entries, 6));
        let declared = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).expect("解析得了");
        assert_eq!(declared.entries.len(), 6, "系统配置声明 6 个条目，第 7 个不许读进来（B8b）");
        assert_eq!(declared.end_offset_in_log, 8_192, "截到第 6 个条目的末尾");
        let until_zero = parse_device_log(&log, DeclaredCountHandling::ReadUntilZeroHeader).expect("解析得了");
        assert_eq!(until_zero.entries.len(), 7);
    }

    #[test]
    fn parser_rejects_flush_with_data_and_unknown_flags() {
        let with_data = LogBytes::InMemory(synthetic_log_bytes(&[(0, 1, LOG_FLUSH_FLAG, vec![0x11; 512])], 1));
        assert!(matches!(parse_device_log(&with_data, DeclaredCountHandling::StopAtDeclared), Err(DeviceLogError::UnsupportedFlagCombination { index: 0, .. })));
        let marked = LogBytes::InMemory(synthetic_log_bytes(&[(0, 0, LOG_MARK_FLAG, Vec::new())], 1));
        assert!(matches!(parse_device_log(&marked, DeclaredCountHandling::StopAtDeclared), Err(DeviceLogError::UnknownFlags { index: 0, .. })));
    }

    #[test]
    fn replay_images_match_the_registered_synthetic_log_digests() {
        let log = synthetic_log_bytes(&synthetic_anchor_log_entries(), 6);
        assert_eq!(synthetic_anchor_image_digest(log.clone(), 6, ReplayVariant::DropUnflushed).expect("重放得了"), "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0");
        assert_eq!(synthetic_anchor_image_digest(log.clone(), 6, ReplayVariant::KeepUnflushed).expect("重放得了"), "6e6d2a487aa2864b75b62637bd06aa2983e5fd5a5a8344debcdee784e774e510");
        assert_eq!(synthetic_anchor_image_digest(log.clone(), 6, ReplayVariant::RandomSubset).expect("重放得了"), "df9020850109e36d3b6d34da043be5a0fb016ffd4004372c52dac115009c1d65");
        assert_eq!(
            synthetic_anchor_image_digest(log, 6, ReplayVariant::RandomSubsetWithTear).expect("重放得了"),
            "6467f85ca3d8eefcb7eac7ea3cd4978d4cbb92d6b66b842b68a19adb6d46a763",
            "U-tear 要把留下的那个多扇区写截短（补 9 M19）"
        );
    }

    #[test]
    fn subset_generator_matches_the_registered_keep_patterns() {
        let first_subset = unflushed_subset(0, 0, &[8, 1, 16, 8, 1, 1, 128, 8, 2, 1]);
        assert_eq!(bit_string(&first_subset.kept), "1000000100", "补 7 B3");
        assert_eq!(first_subset.torn, Some(TornWrite { position_in_unflushed: 7, sectors_kept: 1 }));
        let write_cache_model_subset = unflushed_subset(9, 0, &[1u64; 100]);
        assert_eq!(write_cache_model_subset.kept.iter().filter(|&&is_kept| !is_kept).count(), 47, "PC4-C 的 U-rand 丢恰好 47 个（补 7 B4）");
        assert_eq!(write_cache_model_subset.torn, None, "单扇区写不撕");
    }

    #[test]
    fn write_cache_model_on_the_synthetic_log_gives_the_registered_counts() {
        let log = LogBytes::InMemory(synthetic_write_cache_model_log());
        let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).expect("解析得了");
        let judgement = judge_write_cache_model(&log, &device_log).expect("判得了");
        assert_eq!(judgement.point, 201, "k = 第 200 个写条目下标 + 1（中间夹一个 FLUSH）");
        assert_eq!((judgement.entries_between_batches, judgement.flushes_between_batches), (1, 1));
        assert_eq!(
            judgement.lost_by_variant,
            vec![
                (ReplayVariant::DropUnflushed, 0, 100),
                (ReplayVariant::KeepUnflushed, 0, 0),
                (ReplayVariant::RandomSubset, 0, 47),
                (ReplayVariant::RandomSubsetWithTear, 0, 47),
            ],
            "前 100 个四种取法都不丢；后 100 个 U-drop 丢 100（M16）、U-keep 丢 0、U-rand 丢 47；FLUSH 之前的写不丢（M17）"
        );
        assert!(judgement.registered_counts_hold);
    }

    #[test]
    fn forced_unit_access_write_after_the_last_flush_survives_drop_unflushed() {
        let entries = vec![
            LogEntry::Write { first_sector: 0, sector_count: 1, is_forced_unit_access: false, data_offset_in_log: 1_024 },
            LogEntry::Flush,
            LogEntry::Write { first_sector: 1, sector_count: 1, is_forced_unit_access: true, data_offset_in_log: 2_048 },
            LogEntry::Write { first_sector: 2, sector_count: 1, is_forced_unit_access: false, data_offset_in_log: 3_072 },
        ];
        let selection = select_replayed_writes(&entries, 4, ReplayVariant::DropUnflushed, 0, 0);
        let sectors: Vec<u64> = selection.writes.iter().map(|write| write.first_sector).collect();
        assert_eq!(sectors, vec![0, 1], "FUA 写收到即持久，只丢不带 FUA 的那一个");
        assert_eq!(selection.unflushed_entries, vec![3]);
        assert_eq!(unflushed_write_count(&entries, 4), 1);
        assert_eq!(unflushed_write_count(&entries, 2), 0);
    }

    /// 合成一份写盘开机的设备侧日志：S、B0、C0、B1、C1、B2、（断电点落在这里）、C2、E，全部落在 vda2 的第 c 个扇区。
    fn synthetic_marker_log(arm_index: u32, block_count: u64) -> Vec<u8> {
        let mut entries = Vec::new();
        let mut sequence = 0u64;
        let mut push = |entries: &mut Vec<(u64, u64, u64, Vec<u8>)>, kind: MarkerKind, block_index: Option<u64>| {
            let marker = Marker { kind, sequence, block_index, arm_index };
            entries.push((marker_partition_start_sector() + sequence, 1, 0, encode_marker(&marker).to_vec()));
            sequence += 1;
        };
        push(&mut entries, MarkerKind::Start, None);
        for block_index in 0..block_count {
            push(&mut entries, MarkerKind::Begin, Some(block_index));
            entries.push((FIRST_PARTITION_START_SECTOR + 8 * block_index, 8, 0, vec![0xAB; 8 * SECTOR_BYTES]));
            entries.push((0, 0, LOG_FLUSH_FLAG, Vec::new()));
            push(&mut entries, MarkerKind::Confirm, Some(block_index));
        }
        push(&mut entries, MarkerKind::End, None);
        let declared = u64::try_from(entries.len()).expect("装得进 u64");
        synthetic_log_bytes(&entries, declared)
    }

    #[test]
    fn in_flight_block_between_its_markers_is_not_counted_as_confirmed() {
        let log = LogBytes::InMemory(synthetic_marker_log(3, 3));
        let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).expect("解析得了");
        let scan = scan_markers(&log, &device_log).expect("扫得了");
        let audit = audit_markers(&scan, 3, 3);
        assert!(audit.problems.is_empty(), "合成的标记应当过 V22：{:?}", audit.problems);
        let windows = audit.windows.expect("有窗口");
        assert_eq!(windows.begin_entries, vec![1, 5, 9]);
        assert_eq!(windows.confirm_entries, vec![4, 8, 12]);
        let point = windows.begin_entries[2] + 1;
        assert_eq!(block_classes_at(&windows, point), (2, 3), "第 2 块在 (B_2, C_2] 里：已确认 2 块、已开始 3 块（M18）");
        let directory = TestDirectory::new("in-flight");
        let library = host_library(&directory.path, Candidate::HardenedFile, 2);
        let (confirmed_end, started_end) = block_classes_at(&windows, point);
        let plan = VerificationPlan { states_per_block: PRIMARY_STATES_PER_BLOCK, block_count: 3, confirmed_end, started_end };
        let report = verify_library(Candidate::HardenedFile, &library, &plan);
        assert_eq!(report.lost_confirmed.len(), 0, "在途而读不到不算丢块");
        assert!(report.phantom_keys.is_empty() && report.silently_corrupted.is_empty());
    }

    #[test]
    fn marker_audit_reports_a_missing_begin_and_a_count_mismatch() {
        let log_bytes = synthetic_marker_log(0, 2);
        let log = LogBytes::InMemory(log_bytes);
        let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).expect("解析得了");
        let mut scan = scan_markers(&log, &device_log).expect("扫得了");
        let wrong_count = audit_markers(&scan, 0, 5);
        assert!(wrong_count.windows.is_none());
        assert!(wrong_count.problems.iter().any(|problem| problem.starts_with("confirm_markers=2")));
        scan.markers.remove(1);
        let renumbered: Vec<(usize, Marker)> = scan
            .markers
            .iter()
            .enumerate()
            .map(|(position, (entry, marker))| (*entry, Marker { sequence: u64::try_from(position).expect("小"), ..*marker }))
            .collect();
        scan.markers = renumbered;
        let missing_begin = audit_markers(&scan, 0, 2);
        assert!(missing_begin.windows.is_none(), "C_0 前面没有 B_0 要判 V22");
    }

    #[test]
    fn write_loop_emits_the_confirm_marker_only_after_the_write_call_returns() {
        let directory = TestDirectory::new("event-order");
        let library = directory.path.join("library");
        let mut store = open_store_for_writing(Candidate::HardenedFile, WriteSettings::registered_for(Candidate::HardenedFile), &library).expect("打得开");
        let mut sink = MarkerSink::recording(3);
        let confirmed = write_blocks_with_markers(&mut store, &mut sink, 3, SMALL_STATES_PER_BLOCK).expect("写得进");
        assert_eq!(confirmed, 3);
        let MarkerSink::Recording { events, .. } = sink else { panic!("是记录用的发射器") };
        let described: Vec<String> = events
            .iter()
            .map(|event| match event {
                WriteLoopEvent::Marker(marker) => format!("{:?}{}", marker.kind, marker.block_index.map_or_else(String::new, |block| block.to_string())),
                WriteLoopEvent::WriteCallStarted { block_index } => format!("start{block_index}"),
                WriteLoopEvent::WriteCallReturned { block_index } => format!("return{block_index}"),
            })
            .collect();
        assert_eq!(
            described.join(" "),
            "Start Begin0 start0 return0 Confirm0 Begin1 start1 return1 Confirm1 Begin2 start2 return2 Confirm2 End",
            "C 标记在写入调用返回之后（补 9 M21）"
        );
        let sequences: Vec<u64> = events
            .iter()
            .filter_map(|event| match event {
                WriteLoopEvent::Marker(marker) => Some(marker.sequence),
                WriteLoopEvent::WriteCallStarted { .. } | WriteLoopEvent::WriteCallReturned { .. } => None,
            })
            .collect();
        assert_eq!(sequences, (0..8).collect::<Vec<u64>>(), "c 从 0 起逐个加 1");
    }

    #[test]
    fn phantom_key_positive_control_finds_exactly_one_phantom_on_every_candidate() {
        let directory = TestDirectory::new("phantom");
        for candidate in MAIN_CANDIDATES {
            let library = host_library(&directory.path, candidate, 16);
            apply_library_damage(candidate, &library, LibraryDamage::PhantomKey, PRIMARY_STATES_PER_BLOCK).expect("加得进计划外的键");
            let report = verify_library(candidate, &library, &full_plan(16));
            assert_eq!(report.phantom_keys.len(), 1, "{} 上 PC4-P 的 Q4e 恰好 1（补 9 M22）", candidate.label());
            assert_eq!(report.lost_confirmed.len() + report.silently_corrupted.len() + report.confirmed_read_errors.len(), 0);
            assert!(!report.open_failed);
        }
    }

    #[test]
    fn silent_corruption_positive_control_finds_block_seven_on_every_candidate() {
        let directory = TestDirectory::new("silent");
        for candidate in MAIN_CANDIDATES {
            let library = host_library(&directory.path, candidate, 16);
            apply_library_damage(candidate, &library, LibraryDamage::SilentCorruption, PRIMARY_STATES_PER_BLOCK).expect("改得了第 7 块");
            let report = verify_library(candidate, &library, &full_plan(16));
            assert_eq!(report.silently_corrupted, vec![7], "{} 上 PC4-S 的 Q4b 恰好 1、在第 7 块（补 9 M23）", candidate.label());
            assert_eq!(report.lost_confirmed.len() + report.phantom_keys.len() + report.confirmed_read_errors.len(), 0);
        }
    }

    #[test]
    fn open_breaking_positive_control_fails_to_open_on_every_candidate() {
        let directory = TestDirectory::new("open-breaking");
        for candidate in MAIN_CANDIDATES {
            let library = host_library(&directory.path, candidate, 16);
            apply_library_damage(candidate, &library, LibraryDamage::OpenBreaking, PRIMARY_STATES_PER_BLOCK).expect("毁得了打开要用的那一处");
            let report = verify_library(candidate, &library, &full_plan(16));
            assert!(report.open_failed, "{} 上 PC4-O 的 Q4f 恰好 1（补 9 M24）", candidate.label());
            let repair_expectation = if candidate == Candidate::RocksDatabase { report.opened_after_repair.is_some() } else { report.opened_after_repair.is_none() };
            assert!(repair_expectation, "只有 K 另报修复后能不能开");
        }
    }

    #[test]
    fn untouched_library_verifies_clean_on_every_candidate() {
        let directory = TestDirectory::new("clean");
        for candidate in MAIN_CANDIDATES {
            let library = host_library(&directory.path, candidate, 16);
            let report = verify_library(candidate, &library, &full_plan(16));
            assert_eq!(
                (report.lost_confirmed.len(), report.silently_corrupted.len(), report.phantom_keys.len(), report.open_failed, report.enumerated_key_count),
                (0, 0, 0, false, 16),
                "{} 没动过的库逐块核全对、枚举出恰好 16 个键",
                candidate.label()
            );
        }
    }

    #[test]
    fn every_control_form_differs_from_its_correct_form_in_exactly_the_registered_setting() {
        for arm in ALL_POWER_CUT_ARMS {
            let Some(settings) = arm.write_settings() else {
                assert_eq!(arm, PowerCutArm::WriteCacheModel, "只有 PC4-C 没有候选");
                continue;
            };
            let correct = arm.correct_form().write_settings().expect("正确形态有写入设定");
            let differing = settings.differing_setting_names(correct);
            match arm.registered_control_difference() {
                Some(registered) => assert_eq!(differing, vec![registered], "{} 与 {} 应当恰好差「{registered}」（补 9 M26）", arm.label(), arm.correct_form().label()),
                None => assert!(differing.is_empty(), "{} 不是对照形态，写入设定应当与正确形态相同", arm.label()),
            }
        }
        assert_eq!(PowerCutArm::RocksDatabaseWithoutSync.write_settings().map(|settings| settings.rocksdb_write_sync), Some(RocksDatabaseWriteSync::NoSync));
    }

    #[test]
    fn bucket_pools_split_windows_and_gaps_on_a_synthetic_marker_layout() {
        let windows = CommitWindows { start_entry: 10, end_entry: 100, begin_entries: vec![12, 20, 60], confirm_entries: vec![15, 23, 90] };
        let pools = bucket_pools(&windows);
        assert_eq!(pools.median_window_entries, 3);
        assert_eq!(pools.long_blocks, vec![2], "30 > 4 × 3 算长窗");
        assert_eq!(pools.normal_commit_points, vec![13, 14, 15, 21, 22, 23]);
        assert_eq!(pools.long_commit_points, (61..=90).collect::<Vec<usize>>());
        let expected_between: Vec<usize> = (11..=12).chain(16..=20).chain(24..=60).chain(91..=100).collect();
        assert_eq!(pools.between_commit_points, expected_between);
        let all_points = pools.normal_commit_points.len() + pools.long_commit_points.len() + pools.between_commit_points.len();
        assert_eq!(all_points, 100 - 10, "三个桶合起来恰好是 (S, E] 的全部边界");
    }

    #[test]
    fn commit_window_boundaries_keep_both_ends_and_cap_at_sixty_four() {
        assert_eq!(commit_window_boundaries(5, 9), vec![6, 7, 8, 9]);
        assert_eq!(commit_window_boundaries(0, 64).len(), 64);
        let sixty_five = commit_window_boundaries(0, 65);
        assert_eq!((sixty_five.len(), sixty_five[0], sixty_five[63]), (64, 1, 65));
        let thousand = commit_window_boundaries(100, 1_100);
        assert_eq!((thousand.len(), thousand[0], thousand[63]), (64, 101, 1_100));
        assert!(thousand.windows(2).all(|pair| pair[0] < pair[1]), "等距点严格递增、不重复");
        let largest_subset_index = COMMIT_SUBSET_INDEX_BASE + COMMIT_SUBSET_INDEX_STRIDE * 7 + 63;
        assert_eq!(largest_subset_index, 10_763, "C-commit 的生成器下标装得进 (a << 32) | i 的低 32 位");
    }

    #[test]
    fn main_arm_plan_has_the_registered_item_counts() {
        let begin_entries: Vec<usize> = (0..6_000).map(|block| 100 + 10 * block).collect();
        let confirm_entries: Vec<usize> = (0..6_000).map(|block| 100 + 10 * block + if block % 100 == 7 { 60 } else { 4 }).collect();
        let windows = CommitWindows { start_entry: 90, end_entry: 60_200, begin_entries, confirm_entries };
        let pools = bucket_pools(&windows);
        assert_eq!(pools.long_blocks.len(), 60);
        let full_replay_point = 60_250;
        let items = verification_items(PowerCutArm::RedbQuickRepair, &windows, &pools, full_replay_point);
        let stratified = items.iter().filter(|item| item.group.labels().0 == "C-strat").count();
        let commit_sweep = items.iter().filter(|item| item.group.labels().0 == "C-commit").count();
        let damage: Vec<(usize, u64, u64)> = items
            .iter()
            .filter(|item| item.damage != LibraryDamage::Untouched)
            .map(|item| (item.point, item.confirmed_end, item.started_end))
            .collect();
        assert_eq!(stratified, 200 * 4, "C-strat 200 点 × 四种取法");
        assert_eq!(commit_sweep, (6 * 4 + 2 * 60) * 2, "6 个普通窗各 4 个边界、2 个长窗各 60 个边界，各核 U-drop 与 U-rand");
        assert_eq!(damage, vec![(60_250, 6_000, 6_000); 3], "PC4-S/P/O 在全部条目重放出的盘面上做，块全都已确认");
        let small_items = verification_items(PowerCutArm::RedbQuickRepairSmallBlocks, &windows, &pools, full_replay_point);
        assert_eq!(small_items.len(), 100 * 2, "S4-bs10：100 点 × U-drop、U-rand");
        let control_items = verification_items(PowerCutArm::RocksDatabaseWithoutSync, &windows, &pools, full_replay_point);
        assert_eq!(control_items.len(), 20);
        assert!(control_items.iter().all(|item| item.variant == ReplayVariant::KeepUnflushed), "PC4-L 核 U-keep");
        let barrier_items = verification_items(PowerCutArm::HardenedFileWithoutBarrier, &windows, &pools, full_replay_point);
        assert!(barrier_items.iter().all(|item| item.variant == ReplayVariant::DropUnflushed), "PC4-NB 核 U-drop");
        let parsed: Vec<VerifyItem> = items.iter().map(|item| parse_plan_item(&item.plan_line()).expect("计划行读得回来")).collect();
        assert_eq!(parsed, items, "计划行写出去再读回来逐项相同");
    }

    #[test]
    fn planned_boots_follow_the_registered_order() {
        let boots = planned_write_boots(&[PowerCutArm::RedbQuickRepair, PowerCutArm::HardenedFile]);
        let labels: Vec<&str> = boots.iter().map(|boot| boot.arm.label()).collect();
        assert_eq!(labels, vec!["PC4-C", "R1-None", "R1-NB", "F-nofsync", "F-NB", "F-nodirsync", "R1", "F", "R1-bs10", "F-bs10"]);
        let blocks: Vec<u64> = boots.iter().map(|boot| boot.block_count).collect();
        assert_eq!(blocks, vec![200, 1_000, 1_000, 1_000, 1_000, 1_000, 6_000, 6_000, 6_000, 6_000]);
    }

    #[test]
    fn heavy_test_gate_only_admits_user_request_or_commit() {
        assert!(heavy_tests_permitted(Some("user-request")));
        assert!(heavy_tests_permitted(Some("commit")));
        assert!(!heavy_tests_permitted(Some("yes")));
        assert!(!heavy_tests_permitted(None));
    }

    #[test]
    fn role_arguments_skip_the_device_path_that_vm_bench_prepends() {
        let arguments: Vec<String> = ["/dev/vda", "--role", "s4-write-guest", "--arm", "12", "--dry-run"].iter().map(|text| text.to_string()).collect();
        let parsed = RoleArguments::parse(&arguments);
        assert_eq!(parsed.text("role"), Some("s4-write-guest"));
        assert_eq!(parsed.number("arm"), Some(12));
        assert!(parsed.flag("dry-run"));
        assert_eq!(tagged_guest_line("3-K-write0", "name=write_guest arm=K confirmed=6000"), "name=write_guest boot=3-K-write0 arm=K confirmed=6000");
    }
}
