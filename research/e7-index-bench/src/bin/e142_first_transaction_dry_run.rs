//! E142：第一个事务的干跑——把 `.claude/kb/first-txn-layout.md` 那张字节表原样写成字节，
//! mkfs + 一个小文件 + 一次发布，两块 512 槽的盘；冷启动读回；按 D13（验证路线） 已定项 4 的层 0
//! 崩溃状态集合逐个恢复。目的是让代码把「字节表里哪几格写不出来、哪几条已定条款在字节层面互相顶着」逼出来：
//! 每一处装置不得不自己补的取法都报成一行 `gap`。跑前登记 `research/prompts/e142-preregistration.md`。
//!
//! 判 oracle 用 E77（发布的持久顺序） 判据 1：根槽已持久而恢复 ≠ 新态或走读失败即违例；根槽未持久时新旧两态都合法。
//! 纯确定性：fsid、时间戳都是固定参数，跑 N 遍逐字节一样。

use e7_index_bench::Emitter;
use std::collections::BTreeMap;

// ───────────────────────── 格式常量（每条带分项号；标「预想」的是仓里没定、装置自取的） ─────────────────────────

/// D20（承重面：单元的原子性与自包含）：判定宽度是运行时探测值，本机 nvme0n1 是 512（字节表零那一节）。装置按参数给。
const PHYSICAL_BLOCK_BYTES: u64 = 512;
/// D3（空间分配） 已定项 7：落点粒度 16 KiB 槽。
const SLOT_BYTES: u64 = 16384;
/// D8（核心索引结构） 已定项 2。
const NODE_BYTES: u64 = 16384;
/// D4（校验和位置） 已定项 5 / 已定项 7。
const DATA_UNIT_BYTES: u64 = 32768;
/// D18（块里携带什么信息） 已定项 7：共同前缀 42 + 数据单元类身份段 63。
const DATA_UNIT_HEADER_BYTES: u64 = 105;
/// D18（块里携带什么信息） 已定项 11：共同前缀 42 + 码 3 类身份段 65。
const PACKED_UNIT_HEADER_BYTES: u64 = 107;
/// D18（块里携带什么信息） 已定项 7 的共同明文前缀。
const COMMON_PREFIX_BYTES: u64 = 42;
/// D18（块里携带什么信息） 已定项 14：nonce 12 + MAC 16 预留位，紧接类身份段之后。
const NONCE_MAC_RESERVED_BYTES: u64 = 28;
/// D22（单元原子性怎么合成） 已定项 7（2026-09-13 起含回退下界 F 8 字节）。
const ROOT_RECORD_BYTES: u64 = 250;
/// D19（块指针的结构与宽度预算） 已定项 4：设备 4 + 16 KiB 槽号 6 + 密文校验和 4。
const LOC_ENTRY: u64 = 14; // naming-lint:external 名字由 kb 的 format-const 登记位定（D19 已定项 4），门禁按这个名字绑值
/// D19（块指针的结构与宽度预算） 已定项 7：MAC 16 + nonce 12 + 算法 1 + extent 偏移 2 + 出生树 8 + 出生 txg 8。
const POINTER_HEAD_BYTES: u64 = 47;
/// D19（块指针的结构与宽度预算） 已定项 8：指向码 2 / 码 3 的指针 = 头部 + 位置条目 × 2 + 实例代号 4 + 出生序号 4。
const NODE_POINTER_BYTES: u64 = 83;
/// D19（块指针的结构与宽度预算） 已定项 8：指向码 1 的指针 = 头部 + 位置条目 × 2 + 写序 10。
const DATA_POINTER_BYTES: u64 = 85;
/// D8（核心索引结构） 已定项 8。
const TREE_TABLE_ENTRY_BYTES: u64 = 145;
/// D8（核心索引结构） 已定项 6。
const INODE_RECORD_BYTES: u64 = 140;
/// D8（核心索引结构） 已定项 6：分隔 key 8 + 身份引用 26 + 子指针 83。
const INODE_INTERNAL_ENTRY: u64 = 117;
/// D8（核心索引结构） 已定项 3 + D19 已定项 8：key 24 + 指向码 1 的指针 85。
const EXTENT_LEAF_RECORD_BYTES: u64 = 109;
/// D3（空间分配） 已定项 7：key 12（设备 4 + 槽号 6 + 跨度 2）+ value 8。
const ALLOCATION_RECORD_BYTES: u64 = 20;
/// D5（快照 / 空间记账机制） 已定项 5 + D8 已定项 7：key 22 + value 8 + seq 4。
const ACCOUNTING_ENTRY_BYTES: u64 = 34;
/// D19（块指针的结构与宽度预算） 已定项 6：码 1 的映射 key 27；码 2 / 码 3 的 25——装置补 2 字节零到 27（预想，gap G3）。
const MAPPING_KEY_BYTES: u64 = 27;
/// 映射条目 = key 27 + value（位置条目 × 2）。
const MAPPING_ENTRY_BYTES: u64 = 55;
/// D18（块里携带什么信息） 已定项 11：实例表一片记录宽 88（2026-09-13 随 C304 从 64 改成 88：kind 1 + 有无下一片 1 + 位置指针 83 + 预留 3）。
const INSTANCE_ROW_BYTES: u64 = 88;
/// D23（journal 的角色与格式） 已定项 12。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// 记录头：登记值 78（`JOURNAL_HEADER_BYTES`）加三笔已定增量（事务号 8 + 提交标记 1、反向链 4、载荷校验和 4）= 95，
/// 差记在 C94（登记的格式常量与后来的定案对不上）；字节表六那一节的预想表就是这个数。
const JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES: u64 = 95;
/// D23（journal 的角色与格式） 已定项 4 口径的点名项宽度；构成无落点，装置按字节表六的预想构成写。
const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
/// 字节表一的超级块预想字段表合计（D22（单元原子性怎么合成） 未定项 9 未定）。
const SUPERBLOCK_BYTES: u64 = 413;
/// 头校验和 / 自证校验和的宽度（D18 已定项 7、D22 已定项 7、D23 已定项 4 同口径）；算法全仓未定（gap G5），装置取 SHA-256。
const WIDE_CHECKSUM_BYTES: u64 = 32;

/// 码 2 节点头里 key 区间之外的部分：共同前缀 42 + 树 ID 8 + 层级 1 + 诞生代号 8 + fsid 8 + 写序 4 + 出生序号 4 + 载荷 CRC 4 + 预留 2 = 81。
/// key 区间是 min + max 各一个 key 宽，随树走——这就是 gap G1 / G2：字节表写 84，按 D18 已定项 7 的字段集合加出来是 81 + 2 × key 宽。
const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: u64 = 81;

/// 字节表零：超级块槽 0 / 1 的设备内偏移（预想）。
const SUPERBLOCK_SLOT_OFFSETS: [u64; 2] = [0, 4096];
/// 字节表零：根环起点 1 MiB（槽 64）、P = 3、chunk = 1 MiB、每区 8 槽、槽距 4096（预想）。
const RING_START_OFFSET: u64 = 1 << 20;
const RING_PRIME_STEP: u64 = 3;
const RING_CHUNK_BYTES: u64 = 1 << 20;
const RING_REGIONS: u64 = 3;
const RING_SLOTS_PER_REGION: u64 = 8;
const RING_SLOT_SPACING: u64 = 4096;
/// 字节表零：journal 环 16 MiB 起、64 MiB 长，两盘互为镜像（预想）。
const JOURNAL_START_SLOT: u64 = 1024;
const JOURNAL_RING_BYTES: u64 = 64 << 20;
/// 字节表零：单元区从槽 5120 起，分配顺序 m1 m2 t2 t1 t3 t4 t5 t6 t7 t8。
const SLOT_INSTANCE_TABLE: u64 = 5120;
const SLOT_TREE_TABLE_GENESIS: u64 = 5122;
const SLOT_EXTENT_ROOT: u64 = 5123;
const SLOT_DATA_UNIT: u64 = 5124;
const SLOT_INODE_LEAF: u64 = 5126;
const SLOT_INODE_ROOT: u64 = 5128;
const SLOT_ALLOCATION_ROOT: u64 = 5129;
const SLOT_ACCOUNTING_ROOT: u64 = 5130;
const SLOT_MAPPING_ROOT: u64 = 5131;
const SLOT_TREE_TABLE_FIRST_PUBLISH: u64 = 5132;

/// 字节表零：树 ID 的分配（预想）。树表单元与实例表单元的树 ID 段写 0（无归属）。
const TREE_IDENTIFIER_NONE: u64 = 0;
const TREE_IDENTIFIER_EXTENT: u64 = 1;
const TREE_IDENTIFIER_INODE: u64 = 2;
const TREE_IDENTIFIER_ALLOCATION: u64 = 3;
const TREE_IDENTIFIER_ACCOUNTING: u64 = 4;
const TREE_IDENTIFIER_MAPPING: u64 = 5;
/// 树的种类的码（字节表七：预想 1..5，与树 ID 同号）。
const TREE_KIND_EXTENT: u16 = 1;
const TREE_KIND_INODE: u16 = 2;
const TREE_KIND_ALLOCATION: u16 = 3;
const TREE_KIND_ACCOUNTING: u16 = 4;
const TREE_KIND_MAPPING: u16 = 5;

/// D18（块里携带什么信息） 已定项 11 登记表的码。
const UNIT_CLASS_DATA: u8 = 1;
const UNIT_CLASS_INDEX_NODE: u8 = 2;
const UNIT_CLASS_PACKED: u8 = 3;
/// D18 已定项 11 第二级登记表：打包记录类型 2 = inode 记录，4 = 实例表。
const PACKED_TYPE_INODE: u16 = 2;
const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;

/// 记账统计量标签（C71 未定，字节表五预想：已分配字节 1、inode 号水位 12）。
const STATISTIC_ALLOCATED_BYTES: u16 = 1;
const STATISTIC_INODE_WATERMARK: u16 = 12;

const UNIT_MAGIC: [u8; 4] = *b"SFSU";
const ROOT_MAGIC: [u8; 4] = *b"SFSR";
const SUPERBLOCK_MAGIC: [u8; 4] = *b"SFSB";
const JOURNAL_MAGIC: [u8; 4] = *b"SFSJ";
const FORMAT_VERSION: u16 = 1;

/// 装置固定的 mkfs 参数：fsid 与写入时刻都是参数，产物才能逐字节复跑（gap G11：里程碑步 1 的「同参数两次逐字节相同」要求 fsid 是参数而不是随机）。
const FIXED_FSID: [u8; 16] = [0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const FIRST_INODE_NUMBER: u64 = 1;
const FIRST_INSTANCE_GENERATION: u32 = 1;

// ───────────────────────── 地址空间的 newtype：混用编译不过 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct DeviceIdentity(u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct SlotNumber(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct DeviceOffset(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct CheckpointTxg(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct InstanceGeneration(u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct TransactionNumber(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct TreeIdentifier(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct BirthSequence(u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct JournalCounter(u64);

impl SlotNumber {
    fn device_offset(self) -> DeviceOffset {
        DeviceOffset(self.0 * SLOT_BYTES)
    }
}

/// C113 定案 P1 的写序：实例代号 4 + 事务号低 48 位。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct WriteOrder {
    instance: InstanceGeneration,
    transaction: TransactionNumber,
}

// ───────────────────────── 字节读写 ─────────────────────────

struct ByteWriter {
    bytes: Vec<u8>,
    cursor: usize,
}

impl ByteWriter {
    fn new(total_bytes: usize) -> Self {
        Self { bytes: vec![0u8; total_bytes], cursor: 0 }
    }
    fn put(&mut self, slice: &[u8]) {
        self.bytes[self.cursor..self.cursor + slice.len()].copy_from_slice(slice);
        self.cursor += slice.len();
    }
    fn put_u8(&mut self, value: u8) {
        self.put(&[value]);
    }
    fn put_u16(&mut self, value: u16) {
        self.put(&value.to_le_bytes());
    }
    fn put_u32(&mut self, value: u32) {
        self.put(&value.to_le_bytes());
    }
    fn put_six_byte_unsigned(&mut self, value: u64) {
        assert!(value < (1u64 << 48), "48 位字段装不下 {value}");
        self.put(&value.to_le_bytes()[..6]);
    }
    fn put_u64(&mut self, value: u64) {
        self.put(&value.to_le_bytes());
    }
    fn skip(&mut self, byte_count: usize) {
        self.cursor += byte_count;
    }
    fn position(&self) -> usize {
        self.cursor
    }
    fn assert_position(&self, expected: u64, what: &str) {
        assert_eq!(self.cursor as u64, expected, "{what} 的偏移与字段表不符");
    }
}

struct ByteReader<'bytes> {
    bytes: &'bytes [u8],
    cursor: usize,
}

impl<'bytes> ByteReader<'bytes> {
    fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }
    fn at(bytes: &'bytes [u8], offset: usize) -> Self {
        Self { bytes, cursor: offset }
    }
    fn take(&mut self, byte_count: usize) -> &'bytes [u8] {
        let slice = &self.bytes[self.cursor..self.cursor + byte_count];
        self.cursor += byte_count;
        slice
    }
    fn get_u8(&mut self) -> u8 {
        self.take(1)[0]
    }
    fn get_u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().expect("切了 2 字节"))
    }
    fn get_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().expect("切了 4 字节"))
    }
    fn get_six_byte_unsigned(&mut self) -> u64 {
        let mut eight = [0u8; 8];
        eight[..6].copy_from_slice(self.take(6));
        u64::from_le_bytes(eight)
    }
    fn get_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().expect("切了 8 字节"))
    }
    fn skip(&mut self, byte_count: usize) {
        self.cursor += byte_count;
    }
}

// ───────────────────────── 校验和：CRC32C（D19 已定项 2 / D23 已定项 11）与 32 字节摘要（算法未定，装置取 SHA-256） ─────────────────────────

const CASTAGNOLI_POLYNOMIAL_REFLECTED: u32 = 0x82F6_3B78;

fn castagnoli_tables() -> &'static [[u32; 256]; 8] {
    static TABLES: std::sync::OnceLock<[[u32; 256]; 8]> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let mut tables = [[0u32; 256]; 8];
        for byte_value in 0..256u32 {
            let mut remainder = byte_value;
            for _ in 0..8 {
                remainder = if remainder & 1 == 1 { (remainder >> 1) ^ CASTAGNOLI_POLYNOMIAL_REFLECTED } else { remainder >> 1 };
            }
            tables[0][byte_value as usize] = remainder;
        }
        for byte_value in 0..256usize {
            let mut remainder = tables[0][byte_value];
            for table_index in 1..8 {
                remainder = tables[0][(remainder & 0xff) as usize] ^ (remainder >> 8);
                tables[table_index][byte_value] = remainder;
            }
        }
        tables
    })
}

/// 完整 32 位 CRC32C（Castagnoli，反射，初值与输出异或都是全 1），slicing-by-8。
fn castagnoli_crc32(bytes: &[u8]) -> u32 {
    let tables = castagnoli_tables();
    let mut remainder = !0u32;
    let mut chunks = bytes.chunks_exact(8);
    for chunk in &mut chunks {
        let low = u32::from_le_bytes(chunk[..4].try_into().expect("切了 4 字节")) ^ remainder;
        let high = u32::from_le_bytes(chunk[4..].try_into().expect("切了 4 字节"));
        remainder = tables[7][(low & 0xff) as usize]
            ^ tables[6][((low >> 8) & 0xff) as usize]
            ^ tables[5][((low >> 16) & 0xff) as usize]
            ^ tables[4][(low >> 24) as usize]
            ^ tables[3][(high & 0xff) as usize]
            ^ tables[2][((high >> 8) & 0xff) as usize]
            ^ tables[1][((high >> 16) & 0xff) as usize]
            ^ tables[0][(high >> 24) as usize];
    }
    for &byte in chunks.remainder() {
        remainder = tables[0][((remainder ^ u32::from(byte)) & 0xff) as usize] ^ (remainder >> 8);
    }
    !remainder
}

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// 32 字节摘要。头校验和 / 自证校验和的算法全仓没定（E76 与 D23 已定项 13 都自陈只定了宽度与覆盖范围），装置取标准 SHA-256。
fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut state: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut padded = bytes.to_vec();
    let bit_length = (bytes.len() as u64) * 8;
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_length.to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut schedule = [0u32; 64];
        for (word_index, word) in block.chunks_exact(4).enumerate() {
            schedule[word_index] = u32::from_be_bytes(word.try_into().expect("切了 4 字节"));
        }
        for word_index in 16..64 {
            let sigma0 = schedule[word_index - 15].rotate_right(7) ^ schedule[word_index - 15].rotate_right(18) ^ (schedule[word_index - 15] >> 3);
            let sigma1 = schedule[word_index - 2].rotate_right(17) ^ schedule[word_index - 2].rotate_right(19) ^ (schedule[word_index - 2] >> 10);
            schedule[word_index] = schedule[word_index - 16].wrapping_add(sigma0).wrapping_add(schedule[word_index - 7]).wrapping_add(sigma1);
        }
        let mut working = state;
        for round in 0..64 {
            let big_sigma1 = working[4].rotate_right(6) ^ working[4].rotate_right(11) ^ working[4].rotate_right(25);
            let choose = (working[4] & working[5]) ^ (!working[4] & working[6]);
            let sum_one = working[7].wrapping_add(big_sigma1).wrapping_add(choose).wrapping_add(SHA256_ROUND_CONSTANTS[round]).wrapping_add(schedule[round]);
            let big_sigma0 = working[0].rotate_right(2) ^ working[0].rotate_right(13) ^ working[0].rotate_right(22);
            let majority = (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let sum_two = big_sigma0.wrapping_add(majority);
            working.copy_within(0..7, 1);
            working[4] = working[4].wrapping_add(sum_one);
            working[0] = sum_one.wrapping_add(sum_two);
        }
        for (slot, value) in state.iter_mut().zip(working) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut digest = [0u8; 32];
    for (word_index, word) in state.iter().enumerate() {
        digest[word_index * 4..word_index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

/// 「校验和字段自身按 0 参与」（I-2.4）：把 `[field_offset, field_offset + 32)` 清零后对 `[0, cover_end)` 求摘要。
fn wide_checksum_with_field_zeroed(bytes: &[u8], cover_end: usize, field_offset: usize) -> [u8; 32] {
    let mut covered = bytes[..cover_end].to_vec();
    covered[field_offset..field_offset + WIDE_CHECKSUM_BYTES as usize].fill(0);
    sha256(&covered)
}

// ───────────────────────── 设备、录制器、崩溃镜像 ─────────────────────────

/// 稀疏镜像：按 512 字节扇区存，没写过的扇区读出来恒为 0（mkfs「整环写 0」在这个模型里等价于不写）。
#[derive(Clone, Default)]
struct SparseDevice {
    sectors: BTreeMap<u64, [u8; PHYSICAL_BLOCK_BYTES as usize]>,
}

impl SparseDevice {
    fn read(&self, offset: DeviceOffset, length: usize) -> Vec<u8> {
        assert_eq!(offset.0 % PHYSICAL_BLOCK_BYTES, 0, "读要按扇区对齐");
        assert_eq!(length as u64 % PHYSICAL_BLOCK_BYTES, 0, "读长度要是整扇区");
        let mut out = vec![0u8; length];
        let first_sector = offset.0 / PHYSICAL_BLOCK_BYTES;
        for sector_index in 0..(length as u64 / PHYSICAL_BLOCK_BYTES) {
            if let Some(sector) = self.sectors.get(&(first_sector + sector_index)) {
                let start = (sector_index * PHYSICAL_BLOCK_BYTES) as usize;
                out[start..start + PHYSICAL_BLOCK_BYTES as usize].copy_from_slice(sector);
            }
        }
        out
    }
    fn write(&mut self, offset: DeviceOffset, bytes: &[u8]) {
        assert_eq!(offset.0 % PHYSICAL_BLOCK_BYTES, 0, "写要按扇区对齐");
        assert_eq!(bytes.len() as u64 % PHYSICAL_BLOCK_BYTES, 0, "写长度要是整扇区");
        let first_sector = offset.0 / PHYSICAL_BLOCK_BYTES;
        for (sector_index, chunk) in bytes.chunks_exact(PHYSICAL_BLOCK_BYTES as usize).enumerate() {
            self.sectors.insert(first_sector + sector_index as u64, chunk.try_into().expect("整扇区"));
        }
    }
    fn written_sectors_in(&self, offset: DeviceOffset, length: u64) -> Vec<u64> {
        let first_sector = offset.0 / PHYSICAL_BLOCK_BYTES;
        let end_sector = (offset.0 + length) / PHYSICAL_BLOCK_BYTES;
        self.sectors.range(first_sector..end_sector).map(|(sector, _)| *sector).collect()
    }
}

/// 恢复路径只通过这个 trait 读盘。两个实现：完整的池，与层 0 枚举出的崩溃镜像。
trait BlockReader {
    fn device_count(&self) -> usize;
    fn read(&self, device: DeviceIdentity, offset: DeviceOffset, length: usize) -> Vec<u8>;
    /// 扫描环时的优化：范围内有过写入的扇区号。没写过的扇区读出来恒为 0，一条全 0 的记录 magic 必不匹配，
    /// 所以跳过它们与逐扇区读零判 magic 等价。
    fn written_sectors(&self, device: DeviceIdentity, offset: DeviceOffset, length: u64) -> Vec<u64>;
}

#[derive(Clone)]
struct Pool {
    devices: Vec<SparseDevice>,
}

impl BlockReader for Pool {
    fn device_count(&self) -> usize {
        self.devices.len()
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffset, length: usize) -> Vec<u8> {
        self.devices[device.0 as usize].read(offset, length)
    }
    fn written_sectors(&self, device: DeviceIdentity, offset: DeviceOffset, length: u64) -> Vec<u64> {
        self.devices[device.0 as usize].written_sectors_in(offset, length)
    }
}

#[derive(Clone, Debug)]
struct WriteRequest {
    device: DeviceIdentity,
    offset: DeviceOffset,
    bytes: Vec<u8>,
    is_fua: bool,
    label: &'static str,
}

#[derive(Clone, Debug)]
enum RecordedOperation {
    Write(WriteRequest),
    Barrier,
}

/// 录制器包在池外面：每个写请求与屏障记成一条流，给层 0 枚举用。
struct RecordingPool {
    pool: Pool,
    operations: Vec<RecordedOperation>,
}

impl RecordingPool {
    fn write(&mut self, device: DeviceIdentity, offset: DeviceOffset, bytes: &[u8], is_fua: bool, label: &'static str) {
        self.pool.devices[device.0 as usize].write(offset, bytes);
        self.operations.push(RecordedOperation::Write(WriteRequest { device, offset, bytes: bytes.to_vec(), is_fua, label }));
    }
    fn barrier(&mut self) {
        self.operations.push(RecordedOperation::Barrier);
    }
}

/// 一个崩溃状态的镜像：崩溃前的池 + 这次事务的写请求里被判持久的那个子集。没持久的位置读到旧字节（D13 已定项 4）。
struct CrashImage<'base> {
    base: &'base Pool,
    writes: &'base [WriteRequest],
    persisted: Vec<bool>,
}

impl BlockReader for CrashImage<'_> {
    fn device_count(&self) -> usize {
        self.base.devices.len()
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffset, length: usize) -> Vec<u8> {
        let mut out = self.base.read(device, offset, length);
        let read_start = offset.0;
        let read_end = offset.0 + length as u64;
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            let write_start = write.offset.0;
            let write_end = write_start + write.bytes.len() as u64;
            let overlap_start = read_start.max(write_start);
            let overlap_end = read_end.min(write_end);
            if overlap_start >= overlap_end {
                continue;
            }
            let destination = (overlap_start - read_start) as usize;
            let source = (overlap_start - write_start) as usize;
            let overlap_length = (overlap_end - overlap_start) as usize;
            out[destination..destination + overlap_length].copy_from_slice(&write.bytes[source..source + overlap_length]);
        }
        out
    }
    fn written_sectors(&self, device: DeviceIdentity, offset: DeviceOffset, length: u64) -> Vec<u64> {
        let mut sectors = self.base.written_sectors(device, offset, length);
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            let first = write.offset.0 / PHYSICAL_BLOCK_BYTES;
            let count = write.bytes.len() as u64 / PHYSICAL_BLOCK_BYTES;
            for sector in first..first + count {
                if sector * PHYSICAL_BLOCK_BYTES >= offset.0 && sector * PHYSICAL_BLOCK_BYTES < offset.0 + length {
                    sectors.push(sector);
                }
            }
        }
        sectors.sort_unstable();
        sectors.dedup();
        sectors
    }
}

// ───────────────────────── 盘上结构：位置条目、指针、单元头 ─────────────────────────

/// D19（块指针的结构与宽度预算） 已定项 4：设备 4 + 16 KiB 槽号 6 + 密文校验和 4（无加密时是整单元 CRC32C，字节表三预想）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct LocationEntry {
    device: DeviceIdentity,
    slot: SlotNumber,
    unit_checksum: u32,
}

impl LocationEntry {
    fn write_to(&self, writer: &mut ByteWriter) {
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u32(self.unit_checksum);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        Self { device: DeviceIdentity(reader.get_u32()), slot: SlotNumber(reader.get_six_byte_unsigned()), unit_checksum: reader.get_u32() }
    }
}

/// 指针头部 47（D19 已定项 7）：MAC 16 + nonce 12 + 算法类型 1 + extent 距起点偏移 2（第一版全 0）+ 出生树 8 + 出生 txg 8。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PointerHead {
    birth_tree: TreeIdentifier,
    birth_txg: CheckpointTxg,
}

impl PointerHead {
    fn write_to(&self, writer: &mut ByteWriter) {
        writer.skip(16 + 12 + 1 + 2);
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        reader.skip(16 + 12 + 1 + 2);
        Self { birth_tree: TreeIdentifier(reader.get_u64()), birth_txg: CheckpointTxg(reader.get_u64()) }
    }
}

/// 指向码 2 / 码 3 的指针 83（D19 已定项 8）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct NodePointer {
    head: PointerHead,
    locations: [LocationEntry; 2],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
}

impl NodePointer {
    fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        self.head.write_to(writer);
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u32(self.instance.0);
        writer.put_u32(self.birth_sequence.0);
        assert_eq!(writer.position() - start, NODE_POINTER_BYTES as usize);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        let head = PointerHead::read_from(reader);
        let locations = [LocationEntry::read_from(reader), LocationEntry::read_from(reader)];
        Self { head, locations, instance: InstanceGeneration(reader.get_u32()), birth_sequence: BirthSequence(reader.get_u32()) }
    }
}

/// 指向码 1 的指针 85（D19 已定项 8）：头部 + 位置条目 × 2 + 写序 10。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct DataPointer {
    head: PointerHead,
    locations: [LocationEntry; 2],
    write_order: WriteOrder,
}

impl DataPointer {
    fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        self.head.write_to(writer);
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u32(self.write_order.instance.0);
        writer.put_six_byte_unsigned(self.write_order.transaction.0);
        assert_eq!(writer.position() - start, DATA_POINTER_BYTES as usize);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        let head = PointerHead::read_from(reader);
        let locations = [LocationEntry::read_from(reader), LocationEntry::read_from(reader)];
        let instance = InstanceGeneration(reader.get_u32());
        let transaction = TransactionNumber(reader.get_six_byte_unsigned());
        Self { head, locations, write_order: WriteOrder { instance, transaction } }
    }
}

/// D18 已定项 3 的五元组（33 字节）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct DataUnitIdentity {
    tree: TreeIdentifier,
    object: u64,
    object_birth: CheckpointTxg,
    anchor_offset: u64,
}

/// 共同前缀 42：magic 4 + 版本 2 + 类标签 1 + flags 1 + 声明长度 2 + 头校验和 32。头校验和最后填。
fn write_common_prefix(writer: &mut ByteWriter, unit_class: u8, declared_length: u16) {
    writer.put(&UNIT_MAGIC);
    writer.put_u16(FORMAT_VERSION);
    writer.put_u8(unit_class);
    writer.put_u8(0);
    writer.put_u16(declared_length);
    writer.skip(WIDE_CHECKSUM_BYTES as usize);
    writer.assert_position(COMMON_PREFIX_BYTES, "共同前缀");
}

const HEADER_CHECKSUM_OFFSET: usize = 10;

/// 头校验和覆盖偏移 0 到该类明文头末尾（I-2.4），自身按 0 参与。
fn seal_header_checksum(bytes: &mut [u8], header_end: usize) {
    let digest = wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET);
    bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
}

fn header_checksum_holds(bytes: &[u8], header_end: usize) -> bool {
    wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET) == bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32]
}

/// fsid 在单元头里是 8 字节（D18 已定项 7）；字节表二预想取超级块 fsid 的低 8 字节。
fn unit_fsid(fsid: &[u8; 16]) -> u64 {
    u64::from_le_bytes(fsid[..8].try_into().expect("切了 8 字节"))
}

/// 码 1 数据单元 32768：头 105 + 预留 28 + 载荷（声明长度之后补 0，补齐参与 CRC——I-2.3）。
fn build_data_unit(identity: DataUnitIdentity, birth_txg: CheckpointTxg, fsid: &[u8; 16], write_order: WriteOrder, payload: &[u8]) -> Vec<u8> {
    let payload_start = (DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize;
    assert!(payload.len() <= DATA_UNIT_BYTES as usize - payload_start, "载荷装不进一个单元");
    let mut writer = ByteWriter::new(DATA_UNIT_BYTES as usize);
    write_common_prefix(&mut writer, UNIT_CLASS_DATA, u16::try_from(payload.len()).expect("声明长度 2 字节"));
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(identity.tree.0);
    writer.put_u64(identity.object);
    writer.put_u64(identity.object_birth.0);
    writer.put_u64(identity.anchor_offset);
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_fsid(fsid));
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction.0);
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(DATA_UNIT_HEADER_BYTES, "数据单元头");
    writer.skip(NONCE_MAC_RESERVED_BYTES as usize);
    writer.put(payload);
    let mut bytes = writer.bytes;
    // 载荷 CRC 覆盖偏移 105 到单元末尾（字节表二：含预留位与补齐）。
    let payload_crc = castagnoli_crc32(&bytes[DATA_UNIT_HEADER_BYTES as usize..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, DATA_UNIT_HEADER_BYTES as usize);
    bytes
}

/// 码 3 打包记录单元的类身份四元组（D18 已定项 11）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PackedIdentity {
    birth_tree: TreeIdentifier,
    record_type: u16,
    container: u64,
    container_birth: CheckpointTxg,
}

/// 码 3 打包记录单元 32768：头 107（偏移照 D18 已定项 11 的表）+ 预留 28 + 定宽记录区。
fn build_packed_unit(
    identity: PackedIdentity,
    record_width: u16,
    records: &[Vec<u8>],
    birth_txg: CheckpointTxg,
    fsid: &[u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    let record_area_start = (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize;
    let declared_length = u16::try_from(records.len() * record_width as usize).expect("声明长度 2 字节");
    let mut writer = ByteWriter::new(DATA_UNIT_BYTES as usize);
    write_common_prefix(&mut writer, UNIT_CLASS_PACKED, declared_length);
    writer.put_u8(UNIT_CLASS_PACKED);
    writer.put_u64(identity.birth_tree.0);
    writer.put_u16(identity.record_type);
    writer.put_u64(identity.container);
    writer.put_u64(identity.container_birth.0);
    writer.assert_position(69, "码 3 记录数");
    writer.put_u16(u16::try_from(records.len()).expect("记录数 2 字节"));
    writer.put_u16(record_width);
    writer.assert_position(73, "码 3 诞生代号");
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_fsid(fsid));
    writer.assert_position(89, "码 3 载荷校验和");
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(93, "码 3 写序");
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction.0);
    writer.assert_position(103, "码 3 出生序号");
    writer.put_u32(birth_sequence.0);
    writer.assert_position(PACKED_UNIT_HEADER_BYTES, "码 3 头");
    writer.skip(NONCE_MAC_RESERVED_BYTES as usize);
    for record in records {
        assert_eq!(record.len(), record_width as usize, "记录宽与头里写的不一致");
        writer.put(record);
    }
    let _ = record_area_start;
    let mut bytes = writer.bytes;
    // 载荷校验和覆盖偏移 107 到单元末尾（D18 已定项 11 逐字：nonce / MAC 预留位 + 记录区 + 补齐）。
    let payload_crc = castagnoli_crc32(&bytes[PACKED_UNIT_HEADER_BYTES as usize..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, PACKED_UNIT_HEADER_BYTES as usize);
    bytes
}

/// 码 2 索引节点头的宽度：81 + 2 × key 宽（gap G1 / G2）。
fn index_node_header_bytes(key_width: usize) -> usize {
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE as usize + 2 * key_width
}

/// 码 2 索引节点 16384：共同前缀 42 + 树 ID 8 + 层级 1 + key 区间 (min, max) + 诞生代号 8 + fsid 8 + 写序 4（只实例代号）
/// + 出生序号 4 + 载荷 CRC 4 + 预留 2（D18 已定项 7 / 已定项 12 / C288 第 ① 条）+ 预留 28；
/// 载荷内部布局（第 4 层，不冻结，装置预想，gap G13）：条目数 u16 + 条目宽 u16 + 条目区。
#[allow(clippy::too_many_arguments, reason = "字段表就是这么多段，收成结构体只会多一层没人验的名字")]
fn build_index_node(
    tree: TreeIdentifier,
    level: u8,
    key_width: usize,
    smallest_key: &[u8],
    largest_key: &[u8],
    birth_txg: CheckpointTxg,
    fsid: &[u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: &[Vec<u8>],
) -> Vec<u8> {
    assert_eq!(smallest_key.len(), key_width);
    assert_eq!(largest_key.len(), key_width);
    let header_end = index_node_header_bytes(key_width);
    let entries_start = header_end + NONCE_MAC_RESERVED_BYTES as usize + 4;
    let used_payload = 4 + entries.len() * entry_width as usize;
    assert!(entries_start + entries.len() * entry_width as usize <= NODE_BYTES as usize, "条目装不进一个节点");
    let mut writer = ByteWriter::new(NODE_BYTES as usize);
    // 码 2 的「声明长度」语义未定（I-2.3 说码 2 不在射程内），装置写载荷已用字节（gap G15）。
    write_common_prefix(&mut writer, UNIT_CLASS_INDEX_NODE, u16::try_from(used_payload).expect("声明长度 2 字节"));
    writer.put_u64(tree.0);
    writer.put_u8(level);
    writer.put(smallest_key);
    writer.put(largest_key);
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_fsid(fsid));
    writer.put_u32(instance.0);
    writer.put_u32(birth_sequence.0);
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.skip(2);
    writer.assert_position(header_end as u64, "码 2 头");
    writer.skip(NONCE_MAC_RESERVED_BYTES as usize);
    writer.put_u16(u16::try_from(entries.len()).expect("条目数 2 字节"));
    writer.put_u16(entry_width);
    for entry in entries {
        assert_eq!(entry.len(), entry_width as usize, "条目宽与节点里写的不一致");
        writer.put(entry);
    }
    let mut bytes = writer.bytes;
    // 载荷 CRC 覆盖从头末尾到单元末尾（D18 已定项 7 索引节点类：口径照码 1 与码 3）。
    let payload_crc = castagnoli_crc32(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, header_end);
    bytes
}

/// 解出来的码 2 节点头（解析器要先知道 key 宽——它不在头里，gap G2）。
#[derive(Clone, Debug)]
struct IndexNodeHeader {
    tree: TreeIdentifier,
    level: u8,
    smallest_key: Vec<u8>,
    largest_key: Vec<u8>,
    birth_txg: CheckpointTxg,
    fsid: u64,
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entries: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum UnitError {
    BadMagic,
    BadVersion,
    NonZeroFlags,
    WrongClass { expected: u8, found: u8 },
    HeaderChecksum,
    PayloadChecksum,
    Structure(&'static str),
}

fn check_common_prefix(bytes: &[u8], expected_class: u8) -> Result<u16, UnitError> {
    if bytes[..4] != UNIT_MAGIC {
        return Err(UnitError::BadMagic);
    }
    let mut reader = ByteReader::at(bytes, 4);
    if reader.get_u16() != FORMAT_VERSION {
        return Err(UnitError::BadVersion);
    }
    let found_class = reader.get_u8();
    if reader.get_u8() != 0 {
        return Err(UnitError::NonZeroFlags);
    }
    if found_class != expected_class {
        return Err(UnitError::WrongClass { expected: expected_class, found: found_class });
    }
    Ok(reader.get_u16())
}

fn parse_index_node(bytes: &[u8], key_width: usize) -> Result<IndexNodeHeader, UnitError> {
    check_common_prefix(bytes, UNIT_CLASS_INDEX_NODE)?;
    let header_end = index_node_header_bytes(key_width);
    if !header_checksum_holds(bytes, header_end) {
        return Err(UnitError::HeaderChecksum);
    }
    let mut reader = ByteReader::at(bytes, COMMON_PREFIX_BYTES as usize);
    let tree = TreeIdentifier(reader.get_u64());
    let level = reader.get_u8();
    let smallest_key = reader.take(key_width).to_vec();
    let largest_key = reader.take(key_width).to_vec();
    let birth_txg = CheckpointTxg(reader.get_u64());
    let fsid = reader.get_u64();
    let instance = InstanceGeneration(reader.get_u32());
    let birth_sequence = BirthSequence(reader.get_u32());
    let payload_crc = reader.get_u32();
    if castagnoli_crc32(&bytes[header_end..]) != payload_crc {
        return Err(UnitError::PayloadChecksum);
    }
    let mut payload = ByteReader::at(bytes, header_end + NONCE_MAC_RESERVED_BYTES as usize);
    let entry_count = payload.get_u16() as usize;
    let entry_width = payload.get_u16() as usize;
    if entry_width == 0 || payload.cursor + entry_count * entry_width > bytes.len() {
        return Err(UnitError::Structure("条目区越界"));
    }
    let entries = (0..entry_count).map(|_| payload.take(entry_width).to_vec()).collect();
    Ok(IndexNodeHeader { tree, level, smallest_key, largest_key, birth_txg, fsid, instance, birth_sequence, entries })
}

#[derive(Clone, Debug)]
struct PackedUnitHeader {
    identity: PackedIdentity,
    record_width: u16,
    birth_txg: CheckpointTxg,
    fsid: u64,
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
    records: Vec<Vec<u8>>,
}

fn parse_packed_unit(bytes: &[u8]) -> Result<PackedUnitHeader, UnitError> {
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_PACKED)?;
    if !header_checksum_holds(bytes, PACKED_UNIT_HEADER_BYTES as usize) {
        return Err(UnitError::HeaderChecksum);
    }
    let mut reader = ByteReader::at(bytes, COMMON_PREFIX_BYTES as usize);
    if reader.get_u8() != UNIT_CLASS_PACKED {
        return Err(UnitError::Structure("类身份段首字节不等于偏移 6"));
    }
    let identity = PackedIdentity {
        birth_tree: TreeIdentifier(reader.get_u64()),
        record_type: reader.get_u16(),
        container: reader.get_u64(),
        container_birth: CheckpointTxg(reader.get_u64()),
    };
    let record_count = reader.get_u16() as usize;
    let record_width = reader.get_u16();
    let birth_txg = CheckpointTxg(reader.get_u64());
    let fsid = reader.get_u64();
    let payload_crc = reader.get_u32();
    let write_order = WriteOrder { instance: InstanceGeneration(reader.get_u32()), transaction: TransactionNumber(reader.get_six_byte_unsigned()) };
    let birth_sequence = BirthSequence(reader.get_u32());
    // I-1.7 的判定顺序：边界先于载荷校验和。
    if record_count * record_width as usize > declared_length as usize || declared_length as usize > (DATA_UNIT_BYTES - PACKED_UNIT_HEADER_BYTES) as usize {
        return Err(UnitError::Structure("记录数 × 记录宽 > 声明长度"));
    }
    if castagnoli_crc32(&bytes[PACKED_UNIT_HEADER_BYTES as usize..]) != payload_crc {
        return Err(UnitError::PayloadChecksum);
    }
    let mut record_reader = ByteReader::at(bytes, (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize);
    let records = (0..record_count).map(|_| record_reader.take(record_width as usize).to_vec()).collect();
    Ok(PackedUnitHeader { identity, record_width, birth_txg, fsid, write_order, birth_sequence, records })
}

#[derive(Clone, Debug)]
struct DataUnitHeader {
    identity: DataUnitIdentity,
    declared_length: u16,
    birth_txg: CheckpointTxg,
    fsid: u64,
    write_order: WriteOrder,
}

fn parse_data_unit(bytes: &[u8]) -> Result<DataUnitHeader, UnitError> {
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_DATA)?;
    if !header_checksum_holds(bytes, DATA_UNIT_HEADER_BYTES as usize) {
        return Err(UnitError::HeaderChecksum);
    }
    let mut reader = ByteReader::at(bytes, COMMON_PREFIX_BYTES as usize);
    if reader.get_u8() != UNIT_CLASS_DATA {
        return Err(UnitError::Structure("五元组首字节不等于偏移 6"));
    }
    let identity = DataUnitIdentity {
        tree: TreeIdentifier(reader.get_u64()),
        object: reader.get_u64(),
        object_birth: CheckpointTxg(reader.get_u64()),
        anchor_offset: reader.get_u64(),
    };
    let birth_txg = CheckpointTxg(reader.get_u64());
    let fsid = reader.get_u64();
    let write_order = WriteOrder { instance: InstanceGeneration(reader.get_u32()), transaction: TransactionNumber(reader.get_six_byte_unsigned()) };
    let payload_crc = reader.get_u32();
    if castagnoli_crc32(&bytes[DATA_UNIT_HEADER_BYTES as usize..]) != payload_crc {
        return Err(UnitError::PayloadChecksum);
    }
    Ok(DataUnitHeader { identity, declared_length, birth_txg, fsid, write_order })
}

fn data_unit_payload(bytes: &[u8], declared_length: u16) -> &[u8] {
    let start = (DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize;
    &bytes[start..start + declared_length as usize]
}

// ───────────────────────── 根记录、超级块、journal 记录 ─────────────────────────

/// D22（单元原子性怎么合成） 已定项 7 的字段表，250 字节，字段序照那张表（gap G4：字节表七的行序与它不同，两处都没写偏移）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RootRecord {
    fsid: [u8; 16],
    instance: InstanceGeneration,
    checkpoint_txg: CheckpointTxg,
    tree_table: NodePointer,
    tree_identifier_watermark: u64,
    rollback_floor: CheckpointTxg,
    instance_table: NodePointer,
}

const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + NODE_POINTER_BYTES as usize + 8 + 8;

impl RootRecord {
    /// 写成一个判定宽度的槽：记录 250 字节，其余补 0；自证校验和覆盖整条记录、自身按 0 参与（预想）。
    fn to_slot(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(PHYSICAL_BLOCK_BYTES as usize);
        writer.put(&ROOT_MAGIC);
        writer.put(&self.fsid);
        writer.put_u32(0);
        writer.put_u32(self.instance.0);
        writer.put_u64(self.checkpoint_txg.0);
        self.tree_table.write_to(&mut writer);
        writer.put_u64(self.tree_identifier_watermark);
        writer.put_u64(self.rollback_floor.0);
        writer.assert_position(ROOT_CHECKSUM_OFFSET as u64, "根记录自证校验和");
        writer.skip(WIDE_CHECKSUM_BYTES as usize);
        self.instance_table.write_to(&mut writer);
        writer.assert_position(ROOT_RECORD_BYTES, "根记录");
        let mut bytes = writer.bytes;
        let digest = wide_checksum_with_field_zeroed(&bytes, ROOT_RECORD_BYTES as usize, ROOT_CHECKSUM_OFFSET);
        bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    fn parse_slot(bytes: &[u8], expected_fsid: &[u8; 16]) -> Option<Self> {
        if bytes[..4] != ROOT_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, ROOT_RECORD_BYTES as usize, ROOT_CHECKSUM_OFFSET) != bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32] {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4);
        let fsid: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        if &fsid != expected_fsid {
            return None;
        }
        let _flags = reader.get_u32();
        let instance = InstanceGeneration(reader.get_u32());
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        let tree_table = NodePointer::read_from(&mut reader);
        let tree_identifier_watermark = reader.get_u64();
        let rollback_floor = CheckpointTxg(reader.get_u64());
        reader.skip(WIDE_CHECKSUM_BYTES as usize);
        let instance_table = NodePointer::read_from(&mut reader);
        Some(Self { fsid, instance, checkpoint_txg, tree_table, tree_identifier_watermark, rollback_floor, instance_table })
    }
}

/// 字节表一的超级块预想字段表（D22 未定项 9 未定），413 字节，字段序照那张表。
#[derive(Clone, PartialEq, Eq, Debug)]
struct Superblock {
    fsid: [u8; 16],
    this_device: DeviceIdentity,
    device_count: u32,
    slot_generation: u64,
    region_devices: [DeviceIdentity; 3],
    journal_tail: u64,
    journal_instance: InstanceGeneration,
}

const SUPERBLOCK_CHECKSUM_OFFSET: usize = 4 + 2 + 96 + 16 + 4 + 4 + 8;
const SUPERBLOCK_REGION_DEVICES_OFFSET: usize = 268;
const SUPERBLOCK_TAIL_OFFSET: usize = 401;

impl Superblock {
    fn to_slot(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(PHYSICAL_BLOCK_BYTES as usize);
        writer.put(&SUPERBLOCK_MAGIC);
        writer.put_u16(FORMAT_VERSION);
        writer.skip(96); // feature bits 全 0（gap G12：D12 要求每套布局各占一个 incompat 位，第一条线的位没赋值）
        writer.put(&self.fsid);
        writer.put_u32(self.this_device.0);
        writer.put_u32(self.device_count);
        writer.put_u64(self.slot_generation);
        writer.assert_position(SUPERBLOCK_CHECKSUM_OFFSET as u64, "超级块整槽校验和");
        writer.skip(WIDE_CHECKSUM_BYTES as usize);
        writer.skip(16 + 12 + 2); // 超级块 MAC、nonce 水位、KDF 标识
        writer.put_u8(0); // 加密类型：关
        writer.put_u8(16); // MAC 长度声明
        writer.put_u32(NODE_BYTES as u32);
        writer.put_u32(DATA_UNIT_BYTES as u32);
        writer.put_u32(SLOT_BYTES as u32);
        writer.put_u32(LOC_ENTRY as u32);
        writer.put_u64(JOURNAL_START_SLOT);
        writer.put_u64(JOURNAL_RING_BYTES);
        writer.put_u32(JOURNAL_RECORD_BYTES as u32);
        writer.put_u32(1); // 在飞记录数上限：串行提交
        writer.put_u64(JOURNAL_RECORD_BYTES); // 最坏占用
        writer.put_u32(3); // 安全系数 F
        writer.put_u8(RING_REGIONS as u8);
        writer.put_u8(RING_SLOTS_PER_REGION as u8);
        writer.put_u32(RING_PRIME_STEP as u32);
        writer.put_u32(RING_CHUNK_BYTES as u32);
        writer.put_u64(RING_START_OFFSET / SLOT_BYTES);
        writer.assert_position(SUPERBLOCK_REGION_DEVICES_OFFSET as u64, "根环逐区域设备身份");
        for region_device in &self.region_devices {
            writer.put_u32(region_device.0);
        }
        writer.put_u8(2); // w_max
        writer.put_u8(2); // 组大小 g
        writer.skip(59); // 间接目录单元指针：第一版预想省略、全 0
        writer.skip(24); // 映射来源
        writer.put_u32(5); // T_time 秒
        writer.put_u64(2 << 30); // T_dirty
        writer.skip(24); // 整理三条水位：占位
        writer.assert_position(SUPERBLOCK_TAIL_OFFSET as u64, "journal tail");
        writer.put_u64(self.journal_tail);
        writer.put_u32(self.journal_instance.0);
        writer.assert_position(SUPERBLOCK_BYTES, "超级块");
        let mut bytes = writer.bytes;
        // 「整槽校验和」：覆盖整个 512 槽、自身按 0 参与（预想）。
        let digest = wide_checksum_with_field_zeroed(&bytes, PHYSICAL_BLOCK_BYTES as usize, SUPERBLOCK_CHECKSUM_OFFSET);
        bytes[SUPERBLOCK_CHECKSUM_OFFSET..SUPERBLOCK_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    fn parse_slot(bytes: &[u8]) -> Option<Self> {
        if bytes[..4] != SUPERBLOCK_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, PHYSICAL_BLOCK_BYTES as usize, SUPERBLOCK_CHECKSUM_OFFSET) != bytes[SUPERBLOCK_CHECKSUM_OFFSET..SUPERBLOCK_CHECKSUM_OFFSET + 32] {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4 + 2 + 96);
        let fsid: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        let this_device = DeviceIdentity(reader.get_u32());
        let device_count = reader.get_u32();
        let slot_generation = reader.get_u64();
        let mut region_reader = ByteReader::at(bytes, SUPERBLOCK_REGION_DEVICES_OFFSET);
        let region_devices = [DeviceIdentity(region_reader.get_u32()), DeviceIdentity(region_reader.get_u32()), DeviceIdentity(region_reader.get_u32())];
        let mut tail_reader = ByteReader::at(bytes, SUPERBLOCK_TAIL_OFFSET);
        let journal_tail = tail_reader.get_u64();
        let journal_instance = InstanceGeneration(tail_reader.get_u32());
        Some(Self { fsid, this_device, device_count, slot_generation, region_devices, journal_tail, journal_instance })
    }
}

/// 根环区域 r 的起点（设备内偏移）：`1 MiB + r × P × chunk`（字节表零，预想按设备内偏移读）。
fn ring_region_offset(region: u64) -> DeviceOffset {
    DeviceOffset(RING_START_OFFSET + region * RING_PRIME_STEP * RING_CHUNK_BYTES)
}

fn ring_slot_offset(region: u64, slot: u64) -> DeviceOffset {
    DeviceOffset(ring_region_offset(region).0 + slot * RING_SLOT_SPACING)
}

/// 第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`（字节表一，区域内公式是预想）。
fn ring_target_for_publish(checkpoint_txg: CheckpointTxg) -> (u64, u64) {
    (checkpoint_txg.0 % RING_REGIONS, (checkpoint_txg.0 / RING_REGIONS) % RING_SLOTS_PER_REGION)
}

/// 点名项 56（字节表六的预想构成）：位置条目 × 2 + 树 ID 8 + 诞生代号 8 + 类标签 1 + flags 1 + 预留 2 + 单元大小 4 + 载荷 CRC 4。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct NamedUnit {
    locations: [LocationEntry; 2],
    tree: TreeIdentifier,
    birth_txg: CheckpointTxg,
    unit_class: u8,
    unit_bytes: u32,
    payload_crc: u32,
}

impl NamedUnit {
    fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u64(self.tree.0);
        writer.put_u64(self.birth_txg.0);
        writer.put_u8(self.unit_class);
        writer.put_u8(0);
        writer.put_u16(0);
        writer.put_u32(self.unit_bytes);
        writer.put_u32(self.payload_crc);
        assert_eq!(writer.position() - start, JOURNAL_NAMED_ENTRY_BYTES as usize);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        let locations = [LocationEntry::read_from(reader), LocationEntry::read_from(reader)];
        let tree = TreeIdentifier(reader.get_u64());
        let birth_txg = CheckpointTxg(reader.get_u64());
        let unit_class = reader.get_u8();
        reader.skip(3);
        let unit_bytes = reader.get_u32();
        let payload_crc = reader.get_u32();
        Self { locations, tree, birth_txg, unit_class, unit_bytes, payload_crc }
    }
}

/// journal 记录 4096：头 95（字节表六的预想表序）+ 点名项数组。
#[derive(Clone, PartialEq, Eq, Debug)]
struct JournalRecord {
    instance: InstanceGeneration,
    counter: JournalCounter,
    checkpoint_txg: CheckpointTxg,
    transaction: TransactionNumber,
    is_commit: bool,
    back_chain: u32,
    named: Vec<NamedUnit>,
}

const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 4 + 2 + 1 + 1 + 4 + 4 + 10 + 8 + 12;

impl JournalRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(JOURNAL_RECORD_BYTES as usize);
        writer.put(&JOURNAL_MAGIC);
        writer.put_u16(1); // 类型：普通记录
        writer.put_u8(0); // 算法类型
        writer.put_u8(0); // 对齐填充
        writer.put_u32(JOURNAL_RECORD_BYTES as u32);
        writer.put_u32(u32::try_from(self.named.len()).expect("点名项数 4 字节"));
        writer.put_u32(self.instance.0);
        writer.put_six_byte_unsigned(self.counter.0);
        writer.put_u64(self.checkpoint_txg.0);
        writer.skip(12); // nonce
        writer.assert_position(JOURNAL_HEADER_CHECKSUM_OFFSET as u64, "记录头校验和");
        writer.skip(WIDE_CHECKSUM_BYTES as usize);
        writer.put_u64(self.transaction.0);
        writer.put_u8(u8::from(self.is_commit));
        writer.put_u32(self.back_chain);
        let payload_checksum_offset = writer.position();
        writer.skip(4);
        writer.assert_position(JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES, "记录头");
        for named in &self.named {
            named.write_to(&mut writer);
        }
        let payload_end = writer.position();
        let mut bytes = writer.bytes;
        // 载荷校验和覆盖点名项数组（D23 已定项 13 候选乙），它自己住头里、落在头校验和覆盖内。
        let payload_checksum = castagnoli_crc32(&bytes[JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES as usize..payload_end]);
        bytes[payload_checksum_offset..payload_checksum_offset + 4].copy_from_slice(&payload_checksum.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&bytes, JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES as usize, JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes[..4] != JOURNAL_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES as usize, JOURNAL_HEADER_CHECKSUM_OFFSET)
            != bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
        {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4);
        let _record_type = reader.get_u16();
        reader.skip(2);
        let record_length = reader.get_u32();
        let named_count = reader.get_u32() as usize;
        let instance = InstanceGeneration(reader.get_u32());
        let counter = JournalCounter(reader.get_six_byte_unsigned());
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        reader.skip(12 + WIDE_CHECKSUM_BYTES as usize);
        let transaction = TransactionNumber(reader.get_u64());
        let is_commit = reader.get_u8() == 1;
        let back_chain = reader.get_u32();
        let payload_checksum = reader.get_u32();
        let payload_end = JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES as usize + named_count * JOURNAL_NAMED_ENTRY_BYTES as usize;
        if record_length as usize != bytes.len() || payload_end > bytes.len() {
            return None;
        }
        if castagnoli_crc32(&bytes[JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES as usize..payload_end]) != payload_checksum {
            return None;
        }
        let named = (0..named_count).map(|_| NamedUnit::read_from(&mut reader)).collect();
        Some(Self { instance, counter, checkpoint_txg, transaction, is_commit, back_chain, named })
    }
}

// ───────────────────────── 树表条目、inode、extent、分配、记账、映射 ─────────────────────────

/// D8（核心索引结构） 已定项 8：长度 2 + 树的种类 2 + flags 2 + 树 ID 8 + 根指针 83 + previous_snapshot_txg 8 + 诞生 txg 8 + 预留 32 = 145。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct TreeTableEntry {
    kind: u16,
    tree: TreeIdentifier,
    root: NodePointer,
    birth_txg: CheckpointTxg,
}

impl TreeTableEntry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(TREE_TABLE_ENTRY_BYTES as usize);
        writer.put_u16(TREE_TABLE_ENTRY_BYTES as u16);
        writer.put_u16(self.kind);
        writer.put_u16(0);
        writer.put_u64(self.tree.0);
        self.root.write_to(&mut writer);
        writer.put_u64(0); // previous_snapshot_txg
        writer.put_u64(self.birth_txg.0);
        writer.skip(32);
        writer.assert_position(TREE_TABLE_ENTRY_BYTES, "树表条目");
        writer.bytes
    }
    fn parse(bytes: &[u8]) -> Result<Self, UnitError> {
        let mut reader = ByteReader::new(bytes);
        if reader.get_u16() as u64 != TREE_TABLE_ENTRY_BYTES {
            return Err(UnitError::Structure("树表条目长度"));
        }
        let kind = reader.get_u16();
        if reader.get_u16() != 0 {
            return Err(UnitError::Structure("树表条目 flags 未知位")); // D8 已定项 8 ㊁：未知位一律拒收
        }
        let tree = TreeIdentifier(reader.get_u64());
        let root = NodePointer::read_from(&mut reader);
        let _previous_snapshot_txg = reader.get_u64();
        let birth_txg = CheckpointTxg(reader.get_u64());
        Ok(Self { kind, tree, root, birth_txg })
    }
}

/// 每棵树的 key 宽（码 2 头里的 key 区间随它走，gap G2）。
fn key_width_for_kind(kind: u16) -> Option<usize> {
    match kind {
        TREE_KIND_EXTENT => Some(24),
        TREE_KIND_INODE => Some(8),
        TREE_KIND_ALLOCATION => Some(12),
        TREE_KIND_ACCOUNTING => Some(22),
        TREE_KIND_MAPPING => Some(MAPPING_KEY_BYTES as usize),
        _ => None,
    }
}
/// 树表单元自己按 key = 树 ID（8 字节）走（预想）。
const TREE_TABLE_KEY_WIDTH: usize = 8;

/// inode 树内部节点条目 117（D8 已定项 6）：分隔 key 8 + 身份引用 26（出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8）+ 子指针 83。
fn build_inode_internal_entry(separator_key: u64, child_identity: PackedIdentity, child: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(INODE_INTERNAL_ENTRY as usize);
    writer.put_u64(separator_key);
    writer.put_u64(child_identity.birth_tree.0);
    writer.put_u16(child_identity.record_type);
    writer.put_u64(child_identity.container);
    writer.put_u64(child_identity.container_birth.0);
    child.write_to(&mut writer);
    writer.assert_position(INODE_INTERNAL_ENTRY, "inode 内部条目");
    writer.bytes
}

fn parse_inode_internal_entry(bytes: &[u8]) -> (u64, PackedIdentity, NodePointer) {
    let mut reader = ByteReader::new(bytes);
    let separator_key = reader.get_u64();
    let child_identity = PackedIdentity {
        birth_tree: TreeIdentifier(reader.get_u64()),
        record_type: reader.get_u16(),
        container: reader.get_u64(),
        container_birth: CheckpointTxg(reader.get_u64()),
    };
    (separator_key, child_identity, NodePointer::read_from(&mut reader))
}

/// inode 记录 140（D8 已定项 6 的偏移表）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct InodeRecord {
    inode: u64,
    object_birth: CheckpointTxg,
    size: u64,
    change_count: u64,
}

impl InodeRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(INODE_RECORD_BYTES as usize);
        writer.put_u64(self.inode);
        writer.put_u64(self.object_birth.0);
        writer.put_u64(0); // locality_id：第一版无父目录取 0
        writer.put_u32(0o100644); // mode
        writer.put_u32(0); // uid
        writer.put_u32(0); // gid
        writer.put_u32(1); // nlink
        writer.assert_position(40, "size");
        writer.put_u64(self.size);
        writer.put_u64(DATA_UNIT_BYTES / 512); // blocks，按 512 字节块计
        writer.put_u64(0); // rdev
        writer.assert_position(64, "时间秒");
        for _ in 0..3 {
            writer.put_u64(FIXED_WRITE_TIME_SECONDS);
        }
        writer.assert_position(88, "改动计数");
        writer.put_u64(self.change_count);
        writer.assert_position(96, "时间纳秒");
        writer.skip(12);
        writer.assert_position(108, "填充 / flags / 预留");
        writer.skip(4 + 8 + 20);
        writer.assert_position(INODE_RECORD_BYTES, "inode 记录");
        writer.bytes
    }
    fn parse(bytes: &[u8]) -> Result<Self, UnitError> {
        let mut reader = ByteReader::new(bytes);
        let inode = reader.get_u64();
        let object_birth = CheckpointTxg(reader.get_u64());
        reader.skip(8 + 16);
        let size = reader.get_u64();
        reader.skip(16 + 24);
        let change_count = reader.get_u64();
        if bytes[108..].iter().any(|&byte| byte != 0) {
            return Err(UnitError::Structure("inode 记录三段非零")); // I-9.7
        }
        Ok(Self { inode, object_birth, size, change_count })
    }
}

/// extent 叶记录 109：key (locality_id 8, inode 8, offset 8) + 指向码 1 的指针 85。
fn build_extent_record(inode: u64, offset: u64, pointer: DataPointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(EXTENT_LEAF_RECORD_BYTES as usize);
    writer.put_u64(0);
    writer.put_u64(inode);
    writer.put_u64(offset);
    pointer.write_to(&mut writer);
    writer.assert_position(EXTENT_LEAF_RECORD_BYTES, "extent 叶记录");
    writer.bytes
}

fn parse_extent_record(bytes: &[u8]) -> ([u8; 24], DataPointer) {
    let mut reader = ByteReader::new(bytes);
    let key: [u8; 24] = reader.take(24).try_into().expect("切了 24 字节");
    (key, DataPointer::read_from(&mut reader))
}

/// 分配记录 20（D3 已定项 7）：设备 4 + 槽号 6 + 跨度 2（最高位 = 已释放）+ 分配代 8。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AllocationRecord {
    device: DeviceIdentity,
    slot: SlotNumber,
    span_slots: u16,
    generation: CheckpointTxg,
}

impl AllocationRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(ALLOCATION_RECORD_BYTES as usize);
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u16(self.span_slots);
        writer.put_u64(self.generation.0);
        writer.assert_position(ALLOCATION_RECORD_BYTES, "分配记录");
        writer.bytes
    }
    fn parse(bytes: &[u8]) -> Self {
        let mut reader = ByteReader::new(bytes);
        Self { device: DeviceIdentity(reader.get_u32()), slot: SlotNumber(reader.get_six_byte_unsigned()), span_slots: reader.get_u16(), generation: CheckpointTxg(reader.get_u64()) }
    }
    fn key_bytes(&self) -> Vec<u8> {
        self.to_bytes()[..12].to_vec()
    }
}

/// 记账条目 34（D5 已定项 5 + D8 已定项 7）：key (标签 2, 树 ID 8, 设备 4, 代 8) + value 8 + seq 4。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AccountingEntry {
    statistic: u16,
    tree: TreeIdentifier,
    device: DeviceIdentity,
    generation: CheckpointTxg,
    value: u64,
    sequence: u32,
}

impl AccountingEntry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(ACCOUNTING_ENTRY_BYTES as usize);
        writer.put_u16(self.statistic);
        writer.put_u64(self.tree.0);
        writer.put_u32(self.device.0);
        writer.put_u64(self.generation.0);
        writer.put_u64(self.value);
        writer.put_u32(self.sequence);
        writer.assert_position(ACCOUNTING_ENTRY_BYTES, "记账条目");
        writer.bytes
    }
    fn parse(bytes: &[u8]) -> Self {
        let mut reader = ByteReader::new(bytes);
        Self {
            statistic: reader.get_u16(),
            tree: TreeIdentifier(reader.get_u64()),
            device: DeviceIdentity(reader.get_u32()),
            generation: CheckpointTxg(reader.get_u64()),
            value: reader.get_u64(),
            sequence: reader.get_u32(),
        }
    }
    fn key_bytes(&self) -> Vec<u8> {
        self.to_bytes()[..22].to_vec()
    }
}

/// 中央映射 key（D19 已定项 6）：码 1 = 类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10 = 27；
/// 码 2 / 码 3 = 类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 = 25，装置补 2 字节零到 27（gap G3）。
fn mapping_key_for_data(head: PointerHead, write_order: WriteOrder) -> Vec<u8> {
    let mut writer = ByteWriter::new(MAPPING_KEY_BYTES as usize);
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(head.birth_tree.0);
    writer.put_u64(head.birth_txg.0);
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction.0);
    writer.assert_position(MAPPING_KEY_BYTES, "码 1 映射 key");
    writer.bytes
}

fn mapping_key_for_node(unit_class: u8, pointer: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(MAPPING_KEY_BYTES as usize);
    writer.put_u8(unit_class);
    writer.put_u64(pointer.head.birth_tree.0);
    writer.put_u64(pointer.head.birth_txg.0);
    writer.put_u32(pointer.instance.0);
    writer.put_u32(pointer.birth_sequence.0);
    writer.assert_position(25, "码 2 / 码 3 映射 key");
    writer.bytes
}

fn build_mapping_entry(key: &[u8], locations: [LocationEntry; 2]) -> Vec<u8> {
    let mut writer = ByteWriter::new(MAPPING_ENTRY_BYTES as usize);
    writer.put(key);
    for location in &locations {
        location.write_to(&mut writer);
    }
    writer.assert_position(MAPPING_ENTRY_BYTES, "映射条目");
    writer.bytes
}

fn parse_mapping_entry(bytes: &[u8]) -> (Vec<u8>, [LocationEntry; 2]) {
    let mut reader = ByteReader::new(bytes);
    let key = reader.take(MAPPING_KEY_BYTES as usize).to_vec();
    (key, [LocationEntry::read_from(&mut reader), LocationEntry::read_from(&mut reader)])
}

/// 实例表 kind = 1 链指针记录（D18 已定项 11）：kind 1 | 有无下一片 1 | 位置指针 83（无下一片时清零）| 预留 3 = 88。
fn build_instance_table_chain_record() -> Vec<u8> {
    let mut writer = ByteWriter::new(INSTANCE_ROW_BYTES as usize);
    writer.put_u8(1);
    writer.put_u8(0);
    writer.skip(NODE_POINTER_BYTES as usize);
    writer.skip(3);
    writer.assert_position(INSTANCE_ROW_BYTES, "实例表链指针记录");
    writer.bytes
}

// ───────────────────────── mkfs 与第一个事务 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BarrierPolicy {
    /// D16 已定项 7：单元 → 屏障 → 记录 → 屏障 → 根槽 FUA。
    Settled,
    /// 阳性对照：一道屏障都不放。
    None,
}

#[derive(Clone, Debug)]
struct PoolParameters {
    fsid: [u8; 16],
    device_count: usize,
    region_devices: [DeviceIdentity; 3],
    barriers: BarrierPolicy,
}

impl PoolParameters {
    fn settled_two_devices() -> Self {
        Self { fsid: FIXED_FSID, device_count: 2, region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)], barriers: BarrierPolicy::Settled }
    }
    fn control_one_device_no_barriers() -> Self {
        Self { fsid: FIXED_FSID, device_count: 1, region_devices: [DeviceIdentity(0), DeviceIdentity(0), DeviceIdentity(0)], barriers: BarrierPolicy::None }
    }
    /// 一个单元的两条位置条目：两盘各一份（D2 已定项 10）；只有一块盘时两条都指盘 0（对照臂）。
    fn location_entries(&self, slot: SlotNumber, unit: &[u8]) -> [LocationEntry; 2] {
        let unit_checksum = castagnoli_crc32(unit);
        let second_device = DeviceIdentity(u32::try_from(self.device_count - 1).expect("设备数"));
        [LocationEntry { device: DeviceIdentity(0), slot, unit_checksum }, LocationEntry { device: second_device, slot, unit_checksum }]
    }
    fn devices(&self) -> Vec<DeviceIdentity> {
        (0..self.device_count).map(|index| DeviceIdentity(u32::try_from(index).expect("设备数"))).collect()
    }
}

/// 出生序号（C291：分配规则只在实验源码里）——装置预想：按 (树, txg, 实例) 从 0 计，码 2 与码 3 共用一个计数。
#[derive(Default)]
struct BirthSequenceAllocator {
    counters: BTreeMap<(TreeIdentifier, CheckpointTxg, InstanceGeneration), u32>,
}

impl BirthSequenceAllocator {
    fn next(&mut self, tree: TreeIdentifier, txg: CheckpointTxg, instance: InstanceGeneration) -> BirthSequence {
        let counter = self.counters.entry((tree, txg, instance)).or_insert(0);
        let sequence = BirthSequence(*counter);
        *counter += 1;
        sequence
    }
}

fn write_unit_to_every_device(pool: &mut RecordingPool, parameters: &PoolParameters, slot: SlotNumber, unit: &[u8], label: &'static str) {
    for device in parameters.devices() {
        pool.write(device, slot.device_offset(), unit, false, label);
    }
}

#[derive(Clone, Debug)]
struct MkfsOutput {
    root: RootRecord,
    instance_table_unit: Vec<u8>,
    tree_table_genesis_unit: Vec<u8>,
}

fn mkfs(parameters: &PoolParameters) -> (RecordingPool, MkfsOutput) {
    let mut pool = RecordingPool { pool: Pool { devices: vec![SparseDevice::default(); parameters.device_count] }, operations: Vec::new() };
    let instance = InstanceGeneration(FIRST_INSTANCE_GENERATION);
    let genesis = CheckpointTxg(0);
    let genesis_write_order = WriteOrder { instance, transaction: TransactionNumber(0) };
    let mut sequences = BirthSequenceAllocator::default();

    let instance_table_identity = PackedIdentity { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), record_type: PACKED_TYPE_INSTANCE_TABLE, container: 0, container_birth: genesis };
    let instance_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), genesis, instance);
    let instance_table_unit = build_packed_unit(
        instance_table_identity,
        INSTANCE_ROW_BYTES as u16,
        &[build_instance_table_chain_record()],
        genesis,
        &parameters.fsid,
        genesis_write_order,
        instance_table_sequence,
    );
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), genesis, instance);
    let tree_table_genesis_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &[0u8; 8],
        &[0u8; 8],
        genesis,
        &parameters.fsid,
        instance,
        tree_table_sequence,
        TREE_TABLE_ENTRY_BYTES as u16,
        &[],
    );
    write_unit_to_every_device(&mut pool, parameters, SlotNumber(SLOT_INSTANCE_TABLE), &instance_table_unit, "m1 实例表单元");
    write_unit_to_every_device(&mut pool, parameters, SlotNumber(SLOT_TREE_TABLE_GENESIS), &tree_table_genesis_unit, "m2 树表单元第 0 版");
    pool.barrier();

    let root = RootRecord {
        fsid: parameters.fsid,
        instance,
        checkpoint_txg: genesis,
        tree_table: NodePointer {
            head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), birth_txg: genesis },
            locations: parameters.location_entries(SlotNumber(SLOT_TREE_TABLE_GENESIS), &tree_table_genesis_unit),
            instance,
            birth_sequence: tree_table_sequence,
        },
        tree_identifier_watermark: 1,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer {
            head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), birth_txg: genesis },
            locations: parameters.location_entries(SlotNumber(SLOT_INSTANCE_TABLE), &instance_table_unit),
            instance,
            birth_sequence: instance_table_sequence,
        },
    };
    let root_slot = root.to_slot();
    // D22 已定项 8：第 0 代根种进全部区域，各自槽 0，FUA。
    for region in 0..RING_REGIONS {
        pool.write(parameters.region_devices[region as usize], ring_slot_offset(region, 0), &root_slot, true, "m3 第 0 代根");
    }
    for device in parameters.devices() {
        let superblock = Superblock {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation: 1,
            region_devices: parameters.region_devices,
            journal_tail: 0,
            journal_instance: instance,
        };
        pool.write(device, DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[0]), &superblock.to_slot(), false, "m4 超级块槽 0");
    }
    pool.barrier();
    (pool, MkfsOutput { root, instance_table_unit, tree_table_genesis_unit })
}

/// 第一个事务写出的东西，留给探针与断言用。
#[derive(Clone, Debug)]
struct TransactionOutput {
    root: RootRecord,
    record: JournalRecord,
    record_bytes: Vec<u8>,
    units_by_slot: Vec<(SlotNumber, &'static str, Vec<u8>)>,
    data_pointer: DataPointer,
    mapping_keys: Vec<Vec<u8>>,
    allocation_records: Vec<AllocationRecord>,
    accounting_entries: Vec<AccountingEntry>,
    index_node_header_widths: Vec<(&'static str, usize)>,
}

/// 从单元头里读出载荷 CRC（点名项要带它）。
fn payload_crc_of_unit(unit: &[u8], unit_class: u8, key_width: usize) -> u32 {
    let offset = match unit_class {
        UNIT_CLASS_DATA => 101,
        UNIT_CLASS_PACKED => 89,
        UNIT_CLASS_INDEX_NODE => index_node_header_bytes(key_width) - 6,
        _ => panic!("未登记的类"),
    };
    u32::from_le_bytes(unit[offset..offset + 4].try_into().expect("切了 4 字节"))
}

fn publish_first_file(pool: &mut RecordingPool, parameters: &PoolParameters, genesis: &MkfsOutput, file_bytes: &[u8]) -> TransactionOutput {
    let instance = InstanceGeneration(FIRST_INSTANCE_GENERATION);
    let txg = CheckpointTxg(1);
    let transaction = TransactionNumber(1);
    let write_order = WriteOrder { instance, transaction };
    let fsid = &parameters.fsid;
    let mut sequences = BirthSequenceAllocator::default();
    let mut units: Vec<(SlotNumber, &'static str, Vec<u8>)> = Vec::new();
    let mut index_node_header_widths = Vec::new();

    // t1 数据单元
    let data_identity = DataUnitIdentity { tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), object: FIRST_INODE_NUMBER, object_birth: txg, anchor_offset: 0 };
    let data_unit = build_data_unit(data_identity, txg, fsid, write_order, file_bytes);
    let data_pointer = DataPointer {
        head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(SLOT_DATA_UNIT), &data_unit),
        write_order,
    };

    // t3 inode 树叶容器
    let inode_leaf_identity = PackedIdentity { birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE), record_type: PACKED_TYPE_INODE, container: FIRST_INODE_NUMBER, container_birth: txg };
    let inode_leaf_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
    let inode_record = InodeRecord { inode: FIRST_INODE_NUMBER, object_birth: txg, size: file_bytes.len() as u64, change_count: 1 };
    let inode_leaf_unit = build_packed_unit(inode_leaf_identity, INODE_RECORD_BYTES as u16, &[inode_record.to_bytes()], txg, fsid, write_order, inode_leaf_sequence);
    let inode_leaf_pointer = NodePointer {
        head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE), birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(SLOT_INODE_LEAF), &inode_leaf_unit),
        instance,
        birth_sequence: inode_leaf_sequence,
    };

    // t2 extent 树根兼叶
    let extent_key = build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer)[..24].to_vec();
    let extent_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
    let extent_unit = build_index_node(TreeIdentifier(TREE_IDENTIFIER_EXTENT), 0, 24, &extent_key, &extent_key, txg, fsid, instance, extent_sequence, EXTENT_LEAF_RECORD_BYTES as u16, &[build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer)]);
    index_node_header_widths.push(("extent", index_node_header_bytes(24)));

    // t4 inode 树根
    let inode_root_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
    let inode_key = FIRST_INODE_NUMBER.to_le_bytes();
    let inode_root_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_INODE),
        1,
        8,
        &inode_key,
        &inode_key,
        txg,
        fsid,
        instance,
        inode_root_sequence,
        INODE_INTERNAL_ENTRY as u16,
        &[build_inode_internal_entry(FIRST_INODE_NUMBER, inode_leaf_identity, inode_leaf_pointer)],
    );
    index_node_header_widths.push(("inode", index_node_header_bytes(8)));

    // t5 分配记录树：mkfs 的 m1 / m2 分配代 0，其余 1；每盘各一条（字节表五）。
    let allocated: [(u64, u16, u64); 10] = [
        (SLOT_INSTANCE_TABLE, 2, 0),
        (SLOT_TREE_TABLE_GENESIS, 1, 0),
        (SLOT_EXTENT_ROOT, 1, 1),
        (SLOT_DATA_UNIT, 2, 1),
        (SLOT_INODE_LEAF, 2, 1),
        (SLOT_INODE_ROOT, 1, 1),
        (SLOT_ALLOCATION_ROOT, 1, 1),
        (SLOT_ACCOUNTING_ROOT, 1, 1),
        (SLOT_MAPPING_ROOT, 1, 1),
        (SLOT_TREE_TABLE_FIRST_PUBLISH, 1, 1),
    ];
    let mut allocation_records: Vec<AllocationRecord> = parameters
        .devices()
        .iter()
        .flat_map(|device| allocated.iter().map(move |(slot, span, generation)| AllocationRecord { device: *device, slot: SlotNumber(*slot), span_slots: *span, generation: CheckpointTxg(*generation) }))
        .collect();
    allocation_records.sort_by_key(AllocationRecord::key_bytes);
    let allocation_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), txg, instance);
    let allocation_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
        0,
        12,
        &allocation_records[0].key_bytes(),
        &allocation_records[allocation_records.len() - 1].key_bytes(),
        txg,
        fsid,
        instance,
        allocation_sequence,
        ALLOCATION_RECORD_BYTES as u16,
        &allocation_records.iter().map(AllocationRecord::to_bytes).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("allocation", index_node_header_bytes(12)));

    // t6 记账树：inode 号水位 2，已分配字节每盘 13 槽 × 16384（字节表五）。
    let allocated_slots: u64 = allocated.iter().map(|(_, span, _)| u64::from(*span)).sum();
    let mut accounting_entries = vec![AccountingEntry { statistic: STATISTIC_INODE_WATERMARK, tree: TreeIdentifier(TREE_IDENTIFIER_INODE), device: DeviceIdentity(0), generation: txg, value: FIRST_INODE_NUMBER + 1, sequence: 1 }];
    for (index, device) in parameters.devices().into_iter().enumerate() {
        accounting_entries.push(AccountingEntry { statistic: STATISTIC_ALLOCATED_BYTES, tree: TreeIdentifier(TREE_IDENTIFIER_NONE), device, generation: txg, value: allocated_slots * SLOT_BYTES, sequence: 2 + u32::try_from(index).expect("设备数") });
    }
    accounting_entries.sort_by_key(AccountingEntry::key_bytes);
    let accounting_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), txg, instance);
    let accounting_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
        0,
        22,
        &accounting_entries[0].key_bytes(),
        &accounting_entries[accounting_entries.len() - 1].key_bytes(),
        txg,
        fsid,
        instance,
        accounting_sequence,
        ACCOUNTING_ENTRY_BYTES as u16,
        &accounting_entries.iter().map(AccountingEntry::to_bytes).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("accounting", index_node_header_bytes(22)));

    // 五棵树的根指针（t7 映射与 t8 树表都要引它们）
    let node_pointer = |tree: TreeIdentifier, slot: u64, unit: &[u8], sequence: BirthSequence| NodePointer {
        head: PointerHead { birth_tree: tree, birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(slot), unit),
        instance,
        birth_sequence: sequence,
    };
    let extent_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_EXTENT), SLOT_EXTENT_ROOT, &extent_unit, extent_sequence);
    let inode_root_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_INODE), SLOT_INODE_ROOT, &inode_root_unit, inode_root_sequence);
    let allocation_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), SLOT_ALLOCATION_ROOT, &allocation_unit, allocation_sequence);
    let accounting_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), SLOT_ACCOUNTING_ROOT, &accounting_unit, accounting_sequence);

    // t7 中央映射树：码 1 一条 + 码 2 / 码 3 五条（D19 已定项 8；映射树自己、树表、实例表豁免）。
    let mut mapping_entries: Vec<(Vec<u8>, [LocationEntry; 2])> = vec![
        (mapping_key_for_data(data_pointer.head, data_pointer.write_order), data_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer), extent_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer), inode_leaf_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer), inode_root_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer), allocation_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer), accounting_pointer.locations),
    ];
    mapping_entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mapping_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_MAPPING), txg, instance);
    let mapping_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_MAPPING),
        0,
        MAPPING_KEY_BYTES as usize,
        &mapping_entries[0].0,
        &mapping_entries[mapping_entries.len() - 1].0,
        txg,
        fsid,
        instance,
        mapping_sequence,
        MAPPING_ENTRY_BYTES as u16,
        &mapping_entries.iter().map(|(key, locations)| build_mapping_entry(key, *locations)).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("mapping", index_node_header_bytes(MAPPING_KEY_BYTES as usize)));
    let mapping_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_MAPPING), SLOT_MAPPING_ROOT, &mapping_unit, mapping_sequence);

    // t8 树表单元第 1 版：五条条目按树 ID 升序（D8 已定项 8）。
    let tree_table_entries = [
        TreeTableEntry { kind: TREE_KIND_EXTENT, tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), root: extent_pointer, birth_txg: txg },
        TreeTableEntry { kind: TREE_KIND_INODE, tree: TreeIdentifier(TREE_IDENTIFIER_INODE), root: inode_root_pointer, birth_txg: txg },
        TreeTableEntry { kind: TREE_KIND_ALLOCATION, tree: TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), root: allocation_pointer, birth_txg: txg },
        TreeTableEntry { kind: TREE_KIND_ACCOUNTING, tree: TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), root: accounting_pointer, birth_txg: txg },
        TreeTableEntry { kind: TREE_KIND_MAPPING, tree: TreeIdentifier(TREE_IDENTIFIER_MAPPING), root: mapping_pointer, birth_txg: txg },
    ];
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
    let tree_table_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &TREE_IDENTIFIER_EXTENT.to_le_bytes(),
        &TREE_IDENTIFIER_MAPPING.to_le_bytes(),
        txg,
        fsid,
        instance,
        tree_table_sequence,
        TREE_TABLE_ENTRY_BYTES as u16,
        &tree_table_entries.iter().map(TreeTableEntry::to_bytes).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("tree_table", index_node_header_bytes(TREE_TABLE_KEY_WIDTH)));
    let tree_table_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_NONE), SLOT_TREE_TABLE_FIRST_PUBLISH, &tree_table_unit, tree_table_sequence);

    units.push((SlotNumber(SLOT_DATA_UNIT), "t1 数据单元", data_unit));
    units.push((SlotNumber(SLOT_EXTENT_ROOT), "t2 extent 根", extent_unit));
    units.push((SlotNumber(SLOT_INODE_LEAF), "t3 inode 叶", inode_leaf_unit));
    units.push((SlotNumber(SLOT_INODE_ROOT), "t4 inode 根", inode_root_unit));
    units.push((SlotNumber(SLOT_ALLOCATION_ROOT), "t5 分配树", allocation_unit));
    units.push((SlotNumber(SLOT_ACCOUNTING_ROOT), "t6 记账树", accounting_unit));
    units.push((SlotNumber(SLOT_MAPPING_ROOT), "t7 映射树", mapping_unit));
    units.push((SlotNumber(SLOT_TREE_TABLE_FIRST_PUBLISH), "t8 树表第 1 版", tree_table_unit));

    // 持久顺序第一段：单元与节点。
    for (slot, label, unit) in &units {
        write_unit_to_every_device(pool, parameters, *slot, unit, label);
    }
    if parameters.barriers == BarrierPolicy::Settled {
        pool.barrier();
    }

    // 第二段：journal 记录，点名 t1..t8 每个两盘。反向链：前一条不存在，写 0（gap G6）。
    let class_and_key_width = |label: &str| -> (u8, usize) {
        match label {
            "t1 数据单元" => (UNIT_CLASS_DATA, 0),
            "t3 inode 叶" => (UNIT_CLASS_PACKED, 0),
            "t2 extent 根" => (UNIT_CLASS_INDEX_NODE, 24),
            "t4 inode 根" => (UNIT_CLASS_INDEX_NODE, 8),
            "t5 分配树" => (UNIT_CLASS_INDEX_NODE, 12),
            "t6 记账树" => (UNIT_CLASS_INDEX_NODE, 22),
            "t7 映射树" => (UNIT_CLASS_INDEX_NODE, MAPPING_KEY_BYTES as usize),
            "t8 树表第 1 版" => (UNIT_CLASS_INDEX_NODE, TREE_TABLE_KEY_WIDTH),
            _ => panic!("没登记的单元"),
        }
    };
    let tree_of = |label: &str| -> TreeIdentifier {
        match label {
            "t1 数据单元" | "t2 extent 根" => TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            "t3 inode 叶" | "t4 inode 根" => TreeIdentifier(TREE_IDENTIFIER_INODE),
            "t5 分配树" => TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
            "t6 记账树" => TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
            "t7 映射树" => TreeIdentifier(TREE_IDENTIFIER_MAPPING),
            "t8 树表第 1 版" => TreeIdentifier(TREE_IDENTIFIER_NONE),
            _ => panic!("没登记的单元"),
        }
    };
    let named: Vec<NamedUnit> = units
        .iter()
        .map(|(slot, label, unit)| {
            let (unit_class, key_width) = class_and_key_width(label);
            NamedUnit {
                locations: parameters.location_entries(*slot, unit),
                tree: tree_of(label),
                birth_txg: txg,
                unit_class,
                unit_bytes: u32::try_from(unit.len()).expect("单元大小 4 字节"),
                payload_crc: payload_crc_of_unit(unit, unit_class, key_width),
            }
        })
        .collect();
    let record = JournalRecord { instance, counter: JournalCounter(1), checkpoint_txg: txg, transaction, is_commit: true, back_chain: 0, named };
    let record_bytes = record.to_bytes();
    for device in parameters.devices() {
        pool.write(device, DeviceOffset(JOURNAL_START_SLOT * SLOT_BYTES), &record_bytes, false, "t9 journal 记录");
    }
    if parameters.barriers == BarrierPolicy::Settled {
        pool.barrier();
    }

    // 第三段：根槽 FUA，区域 1 mod 3 的槽 0。
    let root = RootRecord {
        fsid: parameters.fsid,
        instance,
        checkpoint_txg: txg,
        tree_table: tree_table_pointer,
        tree_identifier_watermark: TREE_IDENTIFIER_MAPPING + 1,
        rollback_floor: CheckpointTxg(0),
        instance_table: genesis.root.instance_table,
    };
    let (region, ring_slot) = ring_target_for_publish(txg);
    pool.write(parameters.region_devices[region as usize], ring_slot_offset(region, ring_slot), &root.to_slot(), true, "t10 第 1 代根");

    // 根槽之后：超级块槽 1 轮换，世代号 2，tail 前移（预想）。
    for device in parameters.devices() {
        let superblock = Superblock {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation: 2,
            region_devices: parameters.region_devices,
            journal_tail: 1,
            journal_instance: instance,
        };
        pool.write(device, DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[1]), &superblock.to_slot(), false, "t11 超级块槽 1");
    }

    TransactionOutput {
        root,
        record,
        record_bytes,
        units_by_slot: units,
        data_pointer,
        mapping_keys: mapping_entries.into_iter().map(|(key, _)| key).collect(),
        allocation_records,
        accounting_entries,
        index_node_header_widths,
    }
}

// ───────────────────────── 冷启动恢复（里程碑步 6） ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum JournalPolicy {
    /// 全环扫描、按 D23 已定项 14 的五条口径取前缀、施加前逐项验证点名单元。
    Consult,
    /// 根本不看 journal：与 Consult 逐状态比，问 journal 在第一个事务里承不承重。
    Ignore,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum RecoveryOutcome {
    /// 择到的根下面没有文件（第 0 代）。
    NoFile { root: (InstanceGeneration, CheckpointTxg) },
    /// 沿树走到数据单元、校验和全过，读回内容。
    FileRead { root: (InstanceGeneration, CheckpointTxg), content: Vec<u8> },
    /// 走不下去：哪一步、为什么。
    Failed { root: Option<(InstanceGeneration, CheckpointTxg)>, reason: String },
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct JournalScanReport {
    valid_records: usize,
    above_water: usize,
    prefix_applied: usize,
    verification_passed: usize,
    verification_failed: usize,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct RecoveryReport {
    outcome: RecoveryOutcome,
    journal: JournalScanReport,
    mapping_fallbacks: usize,
}

/// 按指针里的位置条目读一个单元：逐条试，整单元 CRC32C 等于条目里的校验和才算读到（字节表三：无加密时那 4 字节就是整单元 CRC）。
fn read_unit_via_locations(reader: &dyn BlockReader, locations: &[LocationEntry; 2], unit_bytes: usize) -> Result<Vec<u8>, String> {
    for location in locations {
        if location.device.0 as usize >= reader.device_count() {
            continue;
        }
        let bytes = reader.read(location.device, location.slot.device_offset(), unit_bytes);
        if castagnoli_crc32(&bytes) == location.unit_checksum {
            return Ok(bytes);
        }
    }
    Err(format!("槽 {} 的两条位置条目都读不到校验和相符的单元", locations[0].slot.0))
}

fn choose_superblock(reader: &dyn BlockReader) -> Result<Superblock, String> {
    let mut chosen: Option<Superblock> = None;
    for device_index in 0..reader.device_count() {
        let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
        let mut best_on_device: Option<Superblock> = None;
        for slot_offset in SUPERBLOCK_SLOT_OFFSETS {
            let bytes = reader.read(device, DeviceOffset(slot_offset), PHYSICAL_BLOCK_BYTES as usize);
            if let Some(candidate) = Superblock::parse_slot(&bytes) {
                if best_on_device.as_ref().is_none_or(|best| candidate.slot_generation > best.slot_generation) {
                    best_on_device = Some(candidate);
                }
            }
        }
        let Some(best_on_device) = best_on_device else {
            return Err(format!("盘 {device_index} 两个超级块槽都无效"));
        };
        match &chosen {
            None => chosen = Some(best_on_device),
            Some(previous) => {
                if previous.fsid != best_on_device.fsid || previous.device_count != best_on_device.device_count {
                    return Err("两盘的超级块 fsid 或设备数对不上".to_string());
                }
            }
        }
    }
    chosen.ok_or_else(|| "池里没有设备".to_string())
}

fn choose_root(reader: &dyn BlockReader, superblock: &Superblock) -> Option<RootRecord> {
    let mut best: Option<RootRecord> = None;
    for region in 0..RING_REGIONS {
        let device = superblock.region_devices[region as usize];
        if device.0 as usize >= reader.device_count() {
            continue;
        }
        for slot in 0..RING_SLOTS_PER_REGION {
            let bytes = reader.read(device, ring_slot_offset(region, slot), PHYSICAL_BLOCK_BYTES as usize);
            if let Some(candidate) = RootRecord::parse_slot(&bytes, &superblock.fsid) {
                let candidate_key = (candidate.checkpoint_txg, candidate.instance);
                if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance)) {
                    best = Some(candidate);
                }
            }
        }
    }
    best
}

/// 全环扫描（D23 已定项 3：不先信 tail），两份镜像任一份合法即算记录在（gap G9）。
fn scan_journal(reader: &dyn BlockReader) -> BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord> {
    let mut records = BTreeMap::new();
    let ring_start = JOURNAL_START_SLOT * SLOT_BYTES;
    for device_index in 0..reader.device_count() {
        let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
        for sector in reader.written_sectors(device, DeviceOffset(ring_start), JOURNAL_RING_BYTES) {
            let offset = sector * PHYSICAL_BLOCK_BYTES;
            if (offset - ring_start) % JOURNAL_RECORD_BYTES != 0 {
                continue;
            }
            let bytes = reader.read(device, DeviceOffset(offset), JOURNAL_RECORD_BYTES as usize);
            if let Some(record) = JournalRecord::parse(&bytes) {
                records.entry((record.instance, record.counter)).or_insert(record);
            }
        }
    }
    records
}

/// D23 已定项 14 的五条口径里第一个事务碰得到的三条：jsn 严格连续、(实例代号, checkpoint_txg) 大于根的水位、提交标记齐全；
/// 「施加」= 逐项验证点名单元（D16 已定项 7），指针层上做什么无定义（C284，gap G7），装置不改状态。
fn replay_journal(reader: &dyn BlockReader, root: &RootRecord, records: &BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord>) -> JournalScanReport {
    let mut report = JournalScanReport { valid_records: records.len(), ..JournalScanReport::default() };
    let water = (root.instance, root.checkpoint_txg);
    let mut above: Vec<&JournalRecord> = records.values().filter(|record| (record.instance, record.checkpoint_txg) > water).collect();
    above.sort_by_key(|record| (record.instance, record.counter));
    report.above_water = above.len();
    let mut expected: Option<(InstanceGeneration, JournalCounter)> = None;
    for record in above {
        if let Some(expected_key) = expected {
            if (record.instance, record.counter) != expected_key {
                break;
            }
        }
        expected = Some((record.instance, JournalCounter(record.counter.0 + 1)));
        if !record.is_commit {
            break;
        }
        let mut all_verified = true;
        for named in &record.named {
            for location in &named.locations {
                if location.device.0 as usize >= reader.device_count() {
                    continue;
                }
                let bytes = reader.read(location.device, location.slot.device_offset(), named.unit_bytes as usize);
                if castagnoli_crc32(&bytes) != location.unit_checksum {
                    all_verified = false;
                }
            }
        }
        if all_verified {
            report.verification_passed += 1;
            report.prefix_applied += 1;
        } else {
            report.verification_failed += 1;
            break;
        }
    }
    report
}

struct TreeRoots {
    extent: IndexNodeHeader,
    inode: IndexNodeHeader,
    allocation: IndexNodeHeader,
    accounting: IndexNodeHeader,
    mapping: IndexNodeHeader,
}

fn read_tree_root(reader: &dyn BlockReader, entry: &TreeTableEntry, root: &RootRecord, expected_fsid: u64) -> Result<IndexNodeHeader, String> {
    let key_width = key_width_for_kind(entry.kind).ok_or_else(|| format!("树的种类 {} 没登记", entry.kind))?;
    let bytes = read_unit_via_locations(reader, &entry.root.locations, NODE_BYTES as usize)?;
    let node = parse_index_node(&bytes, key_width).map_err(|error| format!("树 {} 的根 {error:?}", entry.tree.0))?;
    if node.tree != entry.tree {
        return Err(format!("树 {} 的根头里写的树 ID 是 {}", entry.tree.0, node.tree.0)); // I-1.3
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err(format!("树 {} 的根诞生于根之后", entry.tree.0)); // I-1.2 的第一版读法
    }
    if node.fsid != expected_fsid {
        return Err(format!("树 {} 的根 fsid 不符", entry.tree.0)); // I-1.4
    }
    if node.birth_sequence != entry.root.birth_sequence {
        return Err(format!("树 {} 的根出生序号与指针不符", entry.tree.0));
    }
    if let (Some(first), Some(last)) = (node.entries.first(), node.entries.last()) {
        if first[..key_width] != node.smallest_key[..] || last[..key_width] != node.largest_key[..] {
            return Err(format!("树 {} 的根 key 区间与条目不符", entry.tree.0)); // I-1.1 索引节点那一半
        }
    }
    Ok(node)
}

fn walk_to_file(reader: &dyn BlockReader, root: &RootRecord, fsid: &[u8; 16], mapping_fallbacks: &mut usize) -> Result<Option<Vec<u8>>, String> {
    let expected_fsid = unit_fsid(fsid);
    // 实例表单元由根记录直接持有（D22 已定项 7）。
    let instance_table_bytes = read_unit_via_locations(reader, &root.instance_table.locations, DATA_UNIT_BYTES as usize)?;
    let instance_table = parse_packed_unit(&instance_table_bytes).map_err(|error| format!("实例表 {error:?}"))?;
    if instance_table.identity.record_type != PACKED_TYPE_INSTANCE_TABLE || instance_table.records.is_empty() {
        return Err("实例表单元不是类型 4 或没有链指针记录".to_string());
    }
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, NODE_BYTES as usize)?;
    let tree_table = parse_index_node(&tree_table_bytes, TREE_TABLE_KEY_WIDTH).map_err(|error| format!("树表 {error:?}"))?;
    if tree_table.entries.is_empty() {
        return Ok(None);
    }
    let entries: Vec<TreeTableEntry> = tree_table.entries.iter().map(|bytes| TreeTableEntry::parse(bytes)).collect::<Result<_, _>>().map_err(|error| format!("树表条目 {error:?}"))?;
    let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
    for entry in &entries {
        if entry.tree.0 >= root.tree_identifier_watermark {
            return Err("树 ID 不低于水位".to_string()); // I-7.8
        }
        by_kind.insert(entry.kind, read_tree_root(reader, entry, root, expected_fsid)?);
    }
    let mut take = |kind: u16, name: &str| by_kind.remove(&kind).ok_or_else(|| format!("树表里没有{name}"));
    let roots = TreeRoots {
        extent: take(TREE_KIND_EXTENT, "extent 树")?,
        inode: take(TREE_KIND_INODE, "inode 树")?,
        allocation: take(TREE_KIND_ALLOCATION, "分配记录树")?,
        accounting: take(TREE_KIND_ACCOUNTING, "记账树")?,
        mapping: take(TREE_KIND_MAPPING, "中央映射树")?,
    };
    if roots.allocation.entries.len() != 10 * reader.device_count() {
        return Err(format!("分配记录数 {} 不是 10 × 盘数", roots.allocation.entries.len()));
    }
    if roots.accounting.entries.len() != 1 + reader.device_count() {
        return Err(format!("记账条目数 {} 不是 1 + 盘数", roots.accounting.entries.len()));
    }
    if roots.mapping.entries.len() != 6 {
        return Err(format!("映射条目数 {} 不是 6", roots.mapping.entries.len()));
    }
    for record_bytes in &roots.allocation.entries {
        let record = AllocationRecord::parse(record_bytes);
        if record.span_slots & 0x8000 != 0 || record.generation > root.checkpoint_txg || record.span_slots == 0 {
            return Err("分配记录带已释放标志、跨度为 0 或分配代晚于根".to_string()); // 第一个事务没有释放
        }
    }
    for entry_bytes in &roots.accounting.entries {
        let entry = AccountingEntry::parse(entry_bytes);
        if entry.generation > root.checkpoint_txg || entry.sequence == 0 {
            return Err("记账条目的代晚于根或 seq 为 0".to_string());
        }
    }

    // inode 树：根（码 2）→ 内部条目 → 叶容器（码 3）→ 记录。
    if roots.inode.level != 1 {
        return Err("inode 树根层级不是 1".to_string());
    }
    let mut inode_record: Option<InodeRecord> = None;
    for entry in &roots.inode.entries {
        let (separator_key, identity, child) = parse_inode_internal_entry(entry);
        if identity.record_type != PACKED_TYPE_INODE {
            return Err("inode 内部条目类型段不是 2".to_string());
        }
        let leaf_bytes = read_unit_via_locations(reader, &child.locations, DATA_UNIT_BYTES as usize)?;
        let leaf = parse_packed_unit(&leaf_bytes).map_err(|error| format!("inode 叶 {error:?}"))?;
        if leaf.identity != identity || leaf.record_width as u64 != INODE_RECORD_BYTES || leaf.fsid != expected_fsid {
            return Err("inode 叶头与条目身份引用不符".to_string()); // I-9.2
        }
        if child.head.birth_tree != identity.birth_tree || leaf.birth_sequence != child.birth_sequence {
            return Err("inode 叶的出生身份与子指针不符".to_string());
        }
        if leaf.birth_txg > root.checkpoint_txg || leaf.write_order.instance != child.instance {
            return Err("inode 叶诞生于根之后或写序实例与子指针不符".to_string()); // I-1.2 码 3 那一半
        }
        for record_bytes in &leaf.records {
            let record = InodeRecord::parse(record_bytes).map_err(|error| format!("inode 记录 {error:?}"))?;
            if record.inode < separator_key || record.inode < identity.container {
                return Err("inode 记录号小于分隔 key 或容器号".to_string()); // I-9.4 / I-9.12
            }
            if record.inode == FIRST_INODE_NUMBER {
                inode_record = Some(record);
            }
        }
    }
    let Some(inode_record) = inode_record else {
        return Ok(None);
    };

    // extent 树：按 (locality 0, inode 1, offset 0) 找指针，解引用先按位置提示、校验和不对再查映射（D19 已定项 5）。
    let mut wanted_key = [0u8; 24];
    wanted_key[8..16].copy_from_slice(&FIRST_INODE_NUMBER.to_le_bytes());
    let mut data_unit: Option<(DataPointer, Vec<u8>)> = None;
    for record_bytes in &roots.extent.entries {
        let (key, pointer) = parse_extent_record(record_bytes);
        if key != wanted_key {
            continue;
        }
        let bytes = match read_unit_via_locations(reader, &pointer.locations, DATA_UNIT_BYTES as usize) {
            Ok(bytes) => bytes,
            Err(hint_error) => {
                *mapping_fallbacks += 1;
                let mapping_key = mapping_key_for_data(pointer.head, pointer.write_order);
                let mapped = roots.mapping.entries.iter().map(|entry| parse_mapping_entry(entry)).find(|(key, _)| *key == mapping_key);
                let Some((_, locations)) = mapped else {
                    return Err(format!("{hint_error}；映射里也没有这个 key"));
                };
                read_unit_via_locations(reader, &locations, DATA_UNIT_BYTES as usize).map_err(|error| format!("经映射仍读不到：{error}"))?
            }
        };
        data_unit = Some((pointer, bytes));
    }
    let Some((pointer, bytes)) = data_unit else {
        return Err("extent 树里没有这个文件的记录".to_string());
    };
    let header = parse_data_unit(&bytes).map_err(|error| format!("数据单元 {error:?}"))?;
    if header.identity.tree != TreeIdentifier(TREE_IDENTIFIER_EXTENT) || header.identity.object != FIRST_INODE_NUMBER || header.identity.anchor_offset != 0 {
        return Err("数据单元五元组与查找路径不符".to_string()); // I-1.1
    }
    if header.identity.object_birth != inode_record.object_birth {
        return Err("对象出生代与 inode 记录不符".to_string()); // I-9.10
    }
    if header.birth_txg != pointer.head.birth_txg || header.write_order != pointer.write_order || header.fsid != expected_fsid {
        return Err("数据单元头与指针的出生身份不符".to_string());
    }
    if u64::from(header.declared_length) != inode_record.size {
        return Err("声明长度与 inode size 不符".to_string());
    }
    if bytes[(DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize + header.declared_length as usize..].iter().any(|&byte| byte != 0) {
        return Err("补齐字节非零".to_string()); // I-2.3
    }
    Ok(Some(data_unit_payload(&bytes, header.declared_length).to_vec()))
}

fn recover(reader: &dyn BlockReader, policy: JournalPolicy) -> RecoveryReport {
    let mut mapping_fallbacks = 0;
    let superblock = match choose_superblock(reader) {
        Ok(superblock) => superblock,
        Err(reason) => return RecoveryReport { outcome: RecoveryOutcome::Failed { root: None, reason }, journal: JournalScanReport::default(), mapping_fallbacks },
    };
    let Some(root) = choose_root(reader, &superblock) else {
        return RecoveryReport { outcome: RecoveryOutcome::Failed { root: None, reason: "根环里一条合法根都没有".to_string() }, journal: JournalScanReport::default(), mapping_fallbacks };
    };
    let root_key = (root.instance, root.checkpoint_txg);
    let journal = match policy {
        JournalPolicy::Consult => replay_journal(reader, &root, &scan_journal(reader)),
        JournalPolicy::Ignore => JournalScanReport::default(),
    };
    let outcome = match walk_to_file(reader, &root, &superblock.fsid, &mut mapping_fallbacks) {
        Ok(Some(content)) => RecoveryOutcome::FileRead { root: root_key, content },
        Ok(None) => RecoveryOutcome::NoFile { root: root_key },
        Err(reason) => RecoveryOutcome::Failed { root: Some(root_key), reason },
    };
    RecoveryReport { outcome, journal, mapping_fallbacks }
}

// ───────────────────────── 层 0 崩溃点枚举（D13 已定项 4）与 oracle（E77 判据 1） ─────────────────────────

/// 屏障把写流切成段；FUA 写完成才发下一条，装置把它也当段边界（可关，字节表两种口径都报）。
fn split_into_segments(operations: &[RecordedOperation], fua_is_boundary: bool) -> (Vec<WriteRequest>, Vec<Vec<usize>>) {
    let mut writes = Vec::new();
    let mut segments: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for operation in operations {
        match operation {
            RecordedOperation::Write(write) => {
                let is_fua = write.is_fua;
                writes.push(write.clone());
                current.push(writes.len() - 1);
                if fua_is_boundary && is_fua {
                    segments.push(std::mem::take(&mut current));
                }
            }
            RecordedOperation::Barrier => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    (writes, segments)
}

/// E77 的闭式：1 + Σ(2^|段| − 1)。
fn closed_form_state_count(segments: &[Vec<usize>]) -> u64 {
    1 + segments.iter().map(|segment| (1u64 << segment.len()) - 1).sum::<u64>()
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct Layer0Tally {
    states: u64,
    violations: u64,
    root_persisted_states: u64,
    no_file_states: u64,
    file_read_states: u64,
    failed_states: u64,
    journal_differing_states: u64,
    verification_ran_states: u64,
    verification_failed_states: u64,
    first_violation: Option<String>,
}

fn oracle_violation(outcome: &RecoveryOutcome, root_persisted: bool, expected_content: &[u8]) -> Option<String> {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => (content != expected_content).then(|| "读回的内容不对".to_string()),
        RecoveryOutcome::NoFile { .. } => root_persisted.then(|| "根槽已持久而恢复到旧态".to_string()),
        RecoveryOutcome::Failed { reason, .. } => Some(format!("走读失败：{reason}")),
    }
}

fn evaluate_state(base: &Pool, writes: &[WriteRequest], persisted: Vec<bool>, root_index: usize, expected_content: &[u8], tally: &mut Layer0Tally) {
    let root_persisted = persisted[root_index];
    let image = CrashImage { base, writes, persisted };
    let consulted = recover(&image, JournalPolicy::Consult);
    let ignored = recover(&image, JournalPolicy::Ignore);
    tally.states += 1;
    if root_persisted {
        tally.root_persisted_states += 1;
    }
    if consulted.outcome != ignored.outcome {
        tally.journal_differing_states += 1;
    }
    if consulted.journal.verification_passed + consulted.journal.verification_failed > 0 {
        tally.verification_ran_states += 1;
    }
    if consulted.journal.verification_failed > 0 {
        tally.verification_failed_states += 1;
    }
    match &consulted.outcome {
        RecoveryOutcome::NoFile { .. } => tally.no_file_states += 1,
        RecoveryOutcome::FileRead { .. } => tally.file_read_states += 1,
        RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
    }
    if let Some(reason) = oracle_violation(&consulted.outcome, root_persisted, expected_content) {
        tally.violations += 1;
        if tally.first_violation.is_none() {
            let persisted_labels: Vec<&str> = image.persisted.iter().zip(writes).filter(|(is_persisted, _)| **is_persisted).map(|(_, write)| write.label).collect();
            tally.first_violation = Some(format!("{reason}（持久的写：{}）", persisted_labels.join("|")));
        }
    }
}

fn enumerate_layer0(base: &Pool, writes: &[WriteRequest], segments: &[Vec<usize>], expected_content: &[u8]) -> Layer0Tally {
    let root_index = writes.iter().position(|write| write.label.starts_with("t10")).expect("写流里有根槽那一条");
    let mut tally = Layer0Tally::default();
    let mut persisted_before = vec![false; writes.len()];
    for segment in segments {
        let full_mask = (1u64 << segment.len()) - 1;
        for mask in 0..full_mask {
            let mut persisted = persisted_before.clone();
            for (bit, write_index) in segment.iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    persisted[*write_index] = true;
                }
            }
            evaluate_state(base, writes, persisted, root_index, expected_content, &mut tally);
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    evaluate_state(base, writes, persisted_before, root_index, expected_content, &mut tally);
    tally
}

// ───────────────────────── 坏字节探针（里程碑步 6 验收） ─────────────────────────

fn flip_byte(pool: &mut Pool, device: DeviceIdentity, offset: DeviceOffset, byte_index: u64) {
    let sector_offset = DeviceOffset(offset.0 + byte_index - byte_index % PHYSICAL_BLOCK_BYTES);
    let mut sector = pool.devices[device.0 as usize].read(sector_offset, PHYSICAL_BLOCK_BYTES as usize);
    sector[(byte_index % PHYSICAL_BLOCK_BYTES) as usize] ^= 0xff;
    pool.devices[device.0 as usize].write(sector_offset, &sector);
}

fn outcome_kind(outcome: &RecoveryOutcome) -> &'static str {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "no_file",
        RecoveryOutcome::FileRead { .. } => "file_read",
        RecoveryOutcome::Failed { .. } => "failed",
    }
}

fn outcome_root(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { root } | RecoveryOutcome::FileRead { root, .. } => format!("{}:{}", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { root: Some(root), .. } => format!("{}:{}", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { root: None, .. } => "none".to_string(),
    }
}

struct Probe {
    name: &'static str,
    flips: Vec<(DeviceIdentity, DeviceOffset, u64)>,
}

fn probes(parameters: &PoolParameters) -> Vec<Probe> {
    let region_one_device = parameters.region_devices[1];
    let journal_start = DeviceOffset(JOURNAL_START_SLOT * SLOT_BYTES);
    let data_offset = SlotNumber(SLOT_DATA_UNIT).device_offset();
    let tree_table_offset = SlotNumber(SLOT_TREE_TABLE_FIRST_PUBLISH).device_offset();
    let both = |offset: DeviceOffset, byte: u64| vec![(DeviceIdentity(0), offset, byte), (DeviceIdentity(1), offset, byte)];
    vec![
        Probe { name: "newest_root_slot_one_byte", flips: vec![(region_one_device, ring_slot_offset(1, 0), 100)] },
        Probe { name: "journal_record_both_copies", flips: both(journal_start, 200) },
        Probe { name: "journal_record_one_copy", flips: vec![(DeviceIdentity(0), journal_start, 200)] },
        Probe { name: "data_payload_one_copy", flips: vec![(DeviceIdentity(0), data_offset, 200)] },
        Probe { name: "data_payload_both_copies", flips: both(data_offset, 200) },
        Probe { name: "data_header_last_byte_both_copies", flips: both(data_offset, DATA_UNIT_HEADER_BYTES - 1) },
        Probe { name: "tree_table_both_copies", flips: both(tree_table_offset, 300) },
        Probe { name: "superblock_slot_one_both_devices", flips: both(DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[1]), 50) },
    ]
}

fn run_probe(full: &Pool, probe: &Probe) -> RecoveryReport {
    let mut damaged = full.clone();
    for (device, offset, byte_index) in &probe.flips {
        flip_byte(&mut damaged, *device, *offset, *byte_index);
    }
    recover(&damaged, JournalPolicy::Consult)
}

// ───────────────────────── 装置自己补的取法：空白清单 ─────────────────────────

/// 每一条是仓里没有条款、装置不得不自己取一个值才写得出字节的地方。文字里不用空格，好让结果行按空格切段。
const GAPS: &[(&str, &str)] = &[
    ("G1", "码2头宽：字节表写84，按D18已定项7字段集合相加是81+2×key宽（inode树97、extent树129、分配树105、记账树125、映射树135），三处预想数（84、68+44、72/81/90三档）互不相等"),
    ("G2", "码2头不自描述key宽：key区间的宽随树走而头里没有key宽字段，扫描期没有树表就找不到头校验和的终点，I-2.4对码2判不了"),
    ("G3", "映射树同一棵树装两种key宽（码1的27与码2/码3的25），码2头的key区间与条目定宽都要一个宽；装置把码2/码3的key补两字节零到27"),
    ("G4", "根记录字段序：D22已定项7的表与字节表七的表行序不同（水位、F、校验和、实例表指针四行的先后），两处都没写偏移；装置按D22的表序"),
    ("G5", "32字节头校验和与自证校验和的算法全仓没定（E76与D23已定项13都自陈只定宽度与覆盖范围）；装置取SHA-256"),
    ("G6", "第一条journal记录的反向链：前一条记录不存在，D23已定项10只定了覆盖前一条的整个记录头；装置写0"),
    ("G7", "记录里没有新根：记录头与点名项都不装树表指针或根记录，「施加一条记录」在指针层上做什么无定义（C284）；装置只验点名单元不改状态"),
    ("G8", "码1映射key靠D16的事务切分纪律（一个事务最多写一个单元的用户数据）才唯一：同一事务写两个数据单元key相同；按D23已定项7一条记录一个事务，每个数据单元一条4KiB记录"),
    ("G9", "journal两份镜像何时算「记录在」没有条款（一份合法即在、还是两份都要）；装置取任一份合法即在"),
    ("G10", "里程碑步4验收「把位置提示改坏、经映射仍读到」在字节层做不到：提示住父节点、父节点被树表指针里的整单元CRC罩着，改坏提示先红父节点；要验的是搬走单元那条路"),
    ("G11", "里程碑步1验收「同参数两次mkfs逐字节相同」与字节表「fsid=mkfs随机」矛盾：fsid必须是mkfs参数"),
    ("G12", "超级块feature位全0与D12已定项1「每套布局各占一个incompat位」不合：第一条线（纯SSD）的位没赋值"),
    ("G13", "码2节点的条目数与条目宽不在头字段表里，只能住载荷内部布局（D15第4层）；装置在预留位之后放u16条目数+u16条目宽"),
    ("G14", "实例表链指针行宽在这一轮里从64改成88（C304，D18已定项11）；写装置那天kb还是64，跑之前改成了88，行里的83宽指针无下一片时清零"),
    ("G15", "码2的「声明长度」语义未定（I-2.3把码2排除在外）；装置写载荷已用字节"),
    ("G16", "出生序号的分配规则只在E137源码里（C291）；装置按(树,txg,实例)从0计、码2与码3共用一个计数"),
    ("G17", "D13已定项4只说屏障切段，FUA写算不算段边界没写；装置把FUA当边界，两种口径的状态数都报"),
    ("G18", "D23已定项12按「12项事务恰占1条记录」算余量，而D16的事务切分纪律让一次带8个数据单元的fsync至少是8个事务、8条记录；两条已定条款对同一负载算出的记录数不同"),
];

// ───────────────────────── main：发结果行 ─────────────────────────

fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn main() {
    let mut emitter = Emitter::new();
    let file_bytes: Vec<u8> = (0..3000u32).map(|index| u8::try_from((index * 7 + 3) % 251).expect("小于 256")).collect();
    emit(&mut emitter, &format!("name=config devices=2 physical_block_bytes={PHYSICAL_BLOCK_BYTES} file_bytes={} fsid=fixed", file_bytes.len()));

    // 判据 1：宽度对账。
    let width_rows: [(&str, u64, u64); 17] = [
        ("pointer_head", 47, POINTER_HEAD_BYTES),
        ("data_unit_header", 105, DATA_UNIT_HEADER_BYTES),
        ("packed_unit_header", 107, PACKED_UNIT_HEADER_BYTES),
        ("root_record", 250, ROOT_RECORD_BYTES),
        ("tree_table_entry", 145, TREE_TABLE_ENTRY_BYTES),
        ("journal_header", 95, JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES),
        ("journal_named_entry", 56, JOURNAL_NAMED_ENTRY_BYTES),
        ("data_pointer", 85, DATA_POINTER_BYTES),
        ("node_pointer", 83, NODE_POINTER_BYTES),
        ("inode_record", 140, INODE_RECORD_BYTES),
        ("inode_internal_entry", 117, INODE_INTERNAL_ENTRY),
        ("extent_leaf_record", 109, EXTENT_LEAF_RECORD_BYTES),
        ("allocation_record", 20, ALLOCATION_RECORD_BYTES),
        ("accounting_entry", 34, ACCOUNTING_ENTRY_BYTES),
        ("mapping_entry", 55, MAPPING_ENTRY_BYTES),
        ("location_entry", 14, LOC_ENTRY),
        ("superblock", 413, SUPERBLOCK_BYTES),
    ];
    let mut width_mismatches = 0u64;
    for (structure, expected, actual) in width_rows {
        if expected != actual {
            width_mismatches += 1;
        }
        emit(&mut emitter, &format!("name=width structure={structure} expected={expected} actual={actual}"));
    }

    let parameters = PoolParameters::settled_two_devices();
    let (mut recording, genesis) = mkfs(&parameters);
    let mkfs_operation_count = recording.operations.len();
    let output = publish_first_file(&mut recording, &parameters, &genesis, &file_bytes);
    let transaction_operations = &recording.operations[mkfs_operation_count..];
    let write_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count();
    let barrier_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier)).count();
    let fua_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua)).count();
    let instance_table = parse_packed_unit(&genesis.instance_table_unit).expect("mkfs 写出的实例表单元自检要过");
    let tree_table_genesis = parse_index_node(&genesis.tree_table_genesis_unit, TREE_TABLE_KEY_WIDTH).expect("mkfs 写出的树表单元自检要过");
    emit(&mut emitter, &format!("name=mkfs_units instance_table_records={} instance_table_row_bytes={} tree_table_entries={} genesis_root_watermark={}", instance_table.records.len(), instance_table.record_width, tree_table_genesis.entries.len(), genesis.root.tree_identifier_watermark));
    emit(&mut emitter, &format!("name=root_record checkpoint_txg={} instance={} tree_identifier_watermark={} rollback_floor={} record_bytes={} back_chain={}", output.root.checkpoint_txg.0, output.root.instance.0, output.root.tree_identifier_watermark, output.root.rollback_floor.0, output.record_bytes.len(), output.record.back_chain));
    let slots: Vec<String> = output.units_by_slot.iter().map(|(slot, label, unit)| format!("{}@{}x{}", label.split(' ').next().expect("标签"), slot.0, unit.len())).collect();
    emit(&mut emitter, &format!("name=write_list writes={write_count} barriers={barrier_count} fua={fua_count} named={} units={}", output.record.named.len(), slots.join(",")));
    for (tree, width) in &output.index_node_header_widths {
        emit(&mut emitter, &format!("name=index_node_header tree={tree} header_bytes={width} with_reserved={} layout_table_says=84", width + NONCE_MAC_RESERVED_BYTES as usize));
    }

    // 判据 3：读回。
    let full = recording.pool.clone();
    let full_report = recover(&full, JournalPolicy::Consult);
    let content_matches = matches!(&full_report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == file_bytes);
    emit(&mut emitter, &format!(
        "name=recover_full outcome={} root={} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
        outcome_kind(&full_report.outcome), outcome_root(&full_report.outcome), full_report.journal.valid_records, full_report.journal.above_water, full_report.journal.prefix_applied, full_report.journal.verification_passed, full_report.mapping_fallbacks
    ));

    // 判据 4：层 0 主臂。基线是 mkfs 之后的池（事务的写全没持久那一态）。
    let (base_pool, _) = mkfs(&parameters);
    let base_pool = base_pool.pool;
    let (writes, segments) = split_into_segments(transaction_operations, true);
    let segment_sizes: Vec<String> = segments.iter().map(|segment| segment.len().to_string()).collect();
    let closed_form = closed_form_state_count(&segments);
    let tally = enumerate_layer0(&base_pool, &writes, &segments, &file_bytes);
    emit(&mut emitter, &format!(
        "name=layer0 arm=settled_two_devices segments={} states={} closed_form={closed_form} violations={} root_persisted_states={} no_file={} file_read={} failed={} verification_ran={} verification_failed={} first_violation={}",
        segment_sizes.join("+"), tally.states, tally.violations, tally.root_persisted_states, tally.no_file_states, tally.file_read_states, tally.failed_states, tally.verification_ran_states, tally.verification_failed_states,
        tally.first_violation.clone().unwrap_or_else(|| "none".to_string()).replace(' ', "_")
    ));
    let (_, segments_fua_free) = split_into_segments(transaction_operations, false);
    emit(&mut emitter, &format!("name=layer0_fua_not_boundary segments={} closed_form={}", segments_fua_free.iter().map(|segment| segment.len().to_string()).collect::<Vec<_>>().join("+"), closed_form_state_count(&segments_fua_free)));
    emit(&mut emitter, &format!("name=journal_effect arm=settled_two_devices states={} differing_states={}", tally.states, tally.journal_differing_states));

    // 判据 5：阳性对照——一块盘、不放屏障，oracle 必须分得出「根在而单元不在」。
    let control = PoolParameters::control_one_device_no_barriers();
    let (mut control_recording, control_genesis) = mkfs(&control);
    let control_mkfs_operation_count = control_recording.operations.len();
    let _ = publish_first_file(&mut control_recording, &control, &control_genesis, &file_bytes);
    let (control_base, _) = mkfs(&control);
    let (control_writes, control_segments) = split_into_segments(&control_recording.operations[control_mkfs_operation_count..], false);
    let control_closed_form = closed_form_state_count(&control_segments);
    let control_tally = enumerate_layer0(&control_base.pool, &control_writes, &control_segments, &file_bytes);
    let control_expected_violations = (1u64 << (control_writes.len() - 1)) - (1u64 << (control_writes.len() - 1 - 8));
    emit(&mut emitter, &format!(
        "name=layer0_control arm=one_device_no_barriers writes={} states={} closed_form={control_closed_form} violations={} expected_violations={control_expected_violations} root_persisted_states={} failed={}",
        control_writes.len(), control_tally.states, control_tally.violations, control_tally.root_persisted_states, control_tally.failed_states
    ));

    // 判据 7：坏字节探针。
    for probe in probes(&parameters) {
        let report = run_probe(&full, &probe);
        let reason = match &report.outcome {
            RecoveryOutcome::Failed { reason, .. } => reason.replace(' ', "_"),
            _ => "none".to_string(),
        };
        let content_matches = matches!(&report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == file_bytes);
        emit(&mut emitter, &format!(
            "name=probe probe={} outcome={} root={} content_matches={content_matches} valid_records={} mapping_fallbacks={} reason={reason}",
            probe.name, outcome_kind(&report.outcome), outcome_root(&report.outcome), report.journal.valid_records, report.mapping_fallbacks
        ));
    }

    // 算术：码 1 映射 key 在同一事务写两个数据单元时相同；D16 的事务切分纪律下 1 MiB 写要几条记录。
    let first_key = mapping_key_for_data(output.data_pointer.head, output.data_pointer.write_order);
    let second_unit_same_transaction = mapping_key_for_data(output.data_pointer.head, output.data_pointer.write_order);
    let distinct_keys = if first_key == second_unit_same_transaction { 1 } else { 2 };
    emit(&mut emitter, &format!("name=mapping_key_collision same_transaction_data_units=2 distinct_keys={distinct_keys} mapping_entries_first_transaction={}", output.mapping_keys.len()));
    let one_mebibyte_units = (1u64 << 20) / DATA_UNIT_BYTES;
    let journal_bytes_two_devices = one_mebibyte_units * JOURNAL_RECORD_BYTES * 2;
    emit(&mut emitter, &format!(
        "name=journal_per_mebibyte data_units={one_mebibyte_units} transactions_under_split_rule={one_mebibyte_units} records={one_mebibyte_units} journal_bytes_two_devices={journal_bytes_two_devices} journal_over_data_percent={}",
        journal_bytes_two_devices * 100 / (2 << 20)
    ));
    emit(&mut emitter, &format!("name=accounting entries={} allocated_bytes_per_device={} inode_watermark={}", output.accounting_entries.len(), output.accounting_entries.iter().find(|entry| entry.statistic == STATISTIC_ALLOCATED_BYTES).map_or(0, |entry| entry.value), output.accounting_entries.iter().find(|entry| entry.statistic == STATISTIC_INODE_WATERMARK).map_or(0, |entry| entry.value)));
    emit(&mut emitter, &format!("name=allocation records={} first_slot={} last_slot={}", output.allocation_records.len(), output.allocation_records[0].slot.0, output.allocation_records[output.allocation_records.len() - 1].slot.0));

    // 判据 8：空白清单。
    for (gap_identifier, text) in GAPS {
        emit(&mut emitter, &format!("name=gap id={gap_identifier} text={text}"));
    }
    emit(&mut emitter, &format!("name=gaps count={}", GAPS.len()));

    emit(&mut emitter, &format!(
        "name=verdict width_mismatches={width_mismatches} write_list_ok={} recover_full_ok={content_matches} layer0_states_ok={} layer0_violations={} control_states_ok={} control_violations_ok={} journal_differing_states={}",
        write_count == 21 && barrier_count == 2 && fua_count == 1,
        tally.states == closed_form && closed_form == 65543,
        tally.violations,
        control_tally.states == control_closed_form && control_closed_form == 2048,
        control_tally.violations == control_expected_violations,
        tally.journal_differing_states
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_file() -> Vec<u8> {
        (0..3000u32).map(|index| u8::try_from((index * 7 + 3) % 251).expect("小于 256")).collect()
    }

    fn built_pool() -> (RecordingPool, MkfsOutput, TransactionOutput, usize) {
        let parameters = PoolParameters::settled_two_devices();
        let (mut recording, genesis) = mkfs(&parameters);
        let mkfs_operation_count = recording.operations.len();
        let output = publish_first_file(&mut recording, &parameters, &genesis, &sample_file());
        (recording, genesis, output, mkfs_operation_count)
    }

    #[test]
    fn crc32c_matches_the_published_check_value() {
        assert_eq!(castagnoli_crc32(b"123456789"), 0xE306_9283);
        assert_eq!(castagnoli_crc32(&[]), 0);
    }

    #[test]
    fn sha256_matches_the_published_test_vector() {
        let digest = sha256(b"abc");
        let expected: [u8; 32] = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
            0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
        ];
        assert_eq!(digest, expected);
        assert_eq!(sha256(b"")[0], 0xe3);
    }

    #[test]
    fn widths_equal_the_byte_table_and_the_root_record_is_250() {
        assert_eq!(ROOT_RECORD_BYTES, 250);
        assert_eq!(ROOT_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES + NODE_POINTER_BYTES, ROOT_RECORD_BYTES);
        assert_eq!(POINTER_HEAD_BYTES + 2 * LOC_ENTRY + 4 + 4, NODE_POINTER_BYTES);
        assert_eq!(POINTER_HEAD_BYTES + 2 * LOC_ENTRY + 10, DATA_POINTER_BYTES);
        assert_eq!(JOURNAL_HEADER_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES + 8 + 1 + 4 + 4, JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES);
        assert_eq!(JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES, 95);
        assert_eq!(SUPERBLOCK_CHECKSUM_OFFSET, 134);
        assert_eq!(MAPPING_KEY_BYTES + 2 * LOC_ENTRY, MAPPING_ENTRY_BYTES);
        assert_eq!(24 + DATA_POINTER_BYTES, EXTENT_LEAF_RECORD_BYTES);
        assert_eq!(8 + 26 + NODE_POINTER_BYTES, INODE_INTERNAL_ENTRY);
    }

    #[test]
    fn index_node_header_is_81_plus_twice_the_key_width_not_84() {
        assert_eq!(index_node_header_bytes(8), 97);
        assert_eq!(index_node_header_bytes(24), 129);
        assert_eq!(index_node_header_bytes(22), 125);
        assert_eq!(index_node_header_bytes(27), 135);
        assert_ne!(index_node_header_bytes(8), 84, "字节表那格 84 与字段集合相加对不上");
    }

    #[test]
    fn transaction_issues_21_writes_2_barriers_1_fua_in_the_settled_order() {
        let (recording, _, output, mkfs_operation_count) = built_pool();
        let operations = &recording.operations[mkfs_operation_count..];
        let labels: Vec<String> = operations
            .iter()
            .map(|operation| match operation {
                RecordedOperation::Write(write) => format!("{}{}", write.label.split(' ').next().expect("标签"), if write.is_fua { "!" } else { "" }),
                RecordedOperation::Barrier => "|".to_string(),
            })
            .collect();
        assert_eq!(labels.iter().filter(|label| label.as_str() != "|").count(), 21);
        assert_eq!(labels.iter().filter(|label| label.as_str() == "|").count(), 2);
        assert_eq!(labels.iter().filter(|label| label.ends_with('!')).count(), 1);
        assert_eq!(labels[16], "|");
        assert_eq!(labels[19], "|");
        assert_eq!(labels[20], "t10!");
        assert_eq!(output.record.named.len(), 8);
        assert_eq!(output.record.named.iter().map(|named| named.locations.len()).sum::<usize>(), 16);
        let slots: Vec<u64> = output.units_by_slot.iter().map(|(slot, _, _)| slot.0).collect();
        assert_eq!(slots, vec![5124, 5123, 5126, 5128, 5129, 5130, 5131, 5132]);
    }

    #[test]
    fn mkfs_seeds_three_generation_zero_roots_readable_from_all_regions() {
        let parameters = PoolParameters::settled_two_devices();
        let (recording, genesis) = mkfs(&parameters);
        let superblock = choose_superblock(&recording.pool).expect("超级块");
        let mut readable = 0;
        for region in 0..RING_REGIONS {
            let bytes = recording.pool.read(superblock.region_devices[region as usize], ring_slot_offset(region, 0), 512);
            if RootRecord::parse_slot(&bytes, &FIXED_FSID) == Some(genesis.root) {
                readable += 1;
            }
        }
        assert_eq!(readable, 3);
        assert_eq!(ring_region_offset(2).0, 7 << 20);
    }

    #[test]
    fn cold_start_reads_the_file_back_and_chooses_root_one_one() {
        let (recording, _, _, _) = built_pool();
        let report = recover(&recording.pool, JournalPolicy::Consult);
        assert_eq!(report.outcome, RecoveryOutcome::FileRead { root: (InstanceGeneration(1), CheckpointTxg(1)), content: sample_file() });
        assert_eq!(report.journal.valid_records, 1);
        assert_eq!(report.journal.above_water, 0, "记录 txg 1 不高于根的水位 1，不施加");
        assert_eq!(report.mapping_fallbacks, 0);
    }

    #[test]
    fn layer0_state_count_is_65543_with_zero_violations() {
        let (recording, _, _, mkfs_operation_count) = built_pool();
        let (base, _) = mkfs(&PoolParameters::settled_two_devices());
        let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
        assert_eq!(segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![16, 2, 1, 2]);
        assert_eq!(closed_form_state_count(&segments), 65543);
        let tally = enumerate_layer0(&base.pool, &writes, &segments, &sample_file());
        assert_eq!(tally.states, 65543);
        assert_eq!(tally.violations, 0, "{:?}", tally.first_violation);
        assert_eq!(tally.root_persisted_states, 4);
        assert_eq!(tally.file_read_states, 4);
        assert_eq!(tally.no_file_states, 65539);
        assert_eq!(tally.journal_differing_states, 0);
        assert_eq!(tally.verification_ran_states, 3);
        assert_eq!(tally.verification_failed_states, 0);
    }

    #[test]
    fn fua_not_a_boundary_gives_65546_states() {
        let (recording, _, _, mkfs_operation_count) = built_pool();
        let (_, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], false);
        assert_eq!(closed_form_state_count(&segments), 65546);
    }

    #[test]
    fn positive_control_without_barriers_has_1020_violations_out_of_2048() {
        let control = PoolParameters::control_one_device_no_barriers();
        let (mut recording, genesis) = mkfs(&control);
        let mkfs_operation_count = recording.operations.len();
        let _ = publish_first_file(&mut recording, &control, &genesis, &sample_file());
        let (base, _) = mkfs(&control);
        let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], false);
        assert_eq!(writes.len(), 11);
        let tally = enumerate_layer0(&base.pool, &writes, &segments, &sample_file());
        assert_eq!(tally.states, 2048);
        assert_eq!(tally.violations, 1020);
        assert_eq!(tally.root_persisted_states, 1024);
    }

    #[test]
    fn flipping_the_last_header_byte_is_caught_by_the_header_checksum() {
        let (_, _, output, _) = built_pool();
        let mut unit = output.units_by_slot[0].2.clone();
        unit[DATA_UNIT_HEADER_BYTES as usize - 1] ^= 0x01;
        assert_eq!(parse_data_unit(&unit).unwrap_err(), UnitError::HeaderChecksum);
        let mut payload_flipped = output.units_by_slot[0].2.clone();
        payload_flipped[DATA_UNIT_HEADER_BYTES as usize + NONCE_MAC_RESERVED_BYTES as usize + 10] ^= 0x01;
        assert_eq!(parse_data_unit(&payload_flipped).unwrap_err(), UnitError::PayloadChecksum);
        let mut reserved_flipped = output.units_by_slot[0].2.clone();
        reserved_flipped[DATA_UNIT_HEADER_BYTES as usize + 3] ^= 0x01;
        assert_eq!(parse_data_unit(&reserved_flipped).unwrap_err(), UnitError::PayloadChecksum, "预留位在载荷 CRC 覆盖内");
    }

    #[test]
    fn flipping_the_payload_checksum_field_breaks_the_journal_header_checksum() {
        let (_, _, output, _) = built_pool();
        let mut record = output.record_bytes.clone();
        record[JOURNAL_HEADER_WITH_SETTLED_INCREMENTS_BYTES as usize - 1] ^= 0x01;
        assert!(JournalRecord::parse(&record).is_none(), "载荷校验和字段要落在头校验和覆盖内（D23 已定项 13）");
        assert!(JournalRecord::parse(&output.record_bytes).is_some());
    }

    #[test]
    fn probes_behave_as_milestone_step_six_expects() {
        let (recording, _, _, _) = built_pool();
        let parameters = PoolParameters::settled_two_devices();
        let mut by_name = BTreeMap::new();
        for probe in probes(&parameters) {
            by_name.insert(probe.name, run_probe(&recording.pool, &probe));
        }
        assert_eq!(by_name["newest_root_slot_one_byte"].outcome, RecoveryOutcome::NoFile { root: (InstanceGeneration(1), CheckpointTxg(0)) });
        assert!(matches!(by_name["journal_record_both_copies"].outcome, RecoveryOutcome::FileRead { .. }));
        assert_eq!(by_name["journal_record_both_copies"].journal.valid_records, 0);
        assert_eq!(by_name["journal_record_one_copy"].journal.valid_records, 1);
        assert!(matches!(by_name["data_payload_one_copy"].outcome, RecoveryOutcome::FileRead { .. }));
        assert!(matches!(by_name["data_payload_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
        assert_eq!(by_name["data_payload_both_copies"].mapping_fallbacks, 1);
        assert!(matches!(by_name["data_header_last_byte_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
        assert!(matches!(by_name["tree_table_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
        assert!(matches!(by_name["superblock_slot_one_both_devices"].outcome, RecoveryOutcome::FileRead { .. }));
    }

    #[test]
    fn two_data_units_in_one_transaction_share_a_mapping_key() {
        let (_, _, output, _) = built_pool();
        let first = mapping_key_for_data(output.data_pointer.head, output.data_pointer.write_order);
        let second = mapping_key_for_data(PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), birth_txg: CheckpointTxg(1) }, WriteOrder { instance: InstanceGeneration(1), transaction: TransactionNumber(1) });
        assert_eq!(first, second);
        assert_eq!(first.len(), 27);
        let distinct: std::collections::BTreeSet<Vec<u8>> = output.mapping_keys.iter().cloned().collect();
        assert_eq!(distinct.len(), 6);
    }

    #[test]
    fn allocation_and_accounting_trees_carry_the_byte_table_numbers() {
        let (_, _, output, _) = built_pool();
        assert_eq!(output.allocation_records.len(), 20);
        assert_eq!(output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(0)).count(), 4);
        assert_eq!(output.accounting_entries.len(), 3);
        let allocated = output.accounting_entries.iter().find(|entry| entry.statistic == STATISTIC_ALLOCATED_BYTES).expect("已分配字节");
        assert_eq!(allocated.value, 212_992);
        let watermark = output.accounting_entries.iter().find(|entry| entry.statistic == STATISTIC_INODE_WATERMARK).expect("水位");
        assert_eq!(watermark.value, 2);
        assert_eq!(output.root.tree_identifier_watermark, 6);
        assert_eq!(output.root.rollback_floor, CheckpointTxg(0));
    }

    #[test]
    fn instance_table_row_is_64_bytes_and_the_inode_leaf_holds_one_140_byte_record() {
        let (_, genesis, output, _) = built_pool();
        let instance_table = parse_packed_unit(&genesis.instance_table_unit).expect("实例表");
        assert_eq!(instance_table.record_width as u64, INSTANCE_ROW_BYTES);
        assert_eq!(instance_table.records.len(), 1);
        assert_eq!(instance_table.identity.record_type, PACKED_TYPE_INSTANCE_TABLE);
        let leaf = parse_packed_unit(&output.units_by_slot[2].2).expect("inode 叶");
        assert_eq!(leaf.record_width as u64, INODE_RECORD_BYTES);
        assert_eq!(leaf.records.len(), 1);
        assert_eq!(InodeRecord::parse(&leaf.records[0]).expect("记录").size, 3000);
        let tree_table = parse_index_node(&genesis.tree_table_genesis_unit, TREE_TABLE_KEY_WIDTH).expect("树表第 0 版");
        assert_eq!(tree_table.entries.len(), 0);
    }
}
