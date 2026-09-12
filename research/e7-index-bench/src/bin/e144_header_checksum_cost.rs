//! E144：头校验和算法的代价与判别力——C309（32 字节校验和的算法没定）。
//!
//! 跑前登记 `research/prompts/e144-preregistration.md`。六条正式臂（SHA-256、BLAKE2s、xxHash64 × 4 种子、xxHash64、CRC-64/XZ、CRC32C）
//! 加一条明知会漏的阳性对照（8 位加法和），在五个真实头宽（95 / 105 / 107 / 250 / 413）上量：单比特、双比特穷举、随机改字节的漏检数，
//! 随机前缀的探针误认数，以及每个头的纳秒数。计时是有变化的观测，7 轮取最小；判别力那一半确定性。

use e7_index_bench::Emitter;
use std::hint::black_box;
use std::time::Instant;

const HEADER_WIDTHS: [usize; 5] = [95, 105, 107, 250, 413];
const CHECKSUM_FIELD_OFFSET: usize = 10;
const CHECKSUM_FIELD_BYTES: usize = 32;
const DOUBLE_BIT_WIDTH: usize = 105;
const RANDOM_TRIALS_PER_WIDTH: u64 = 200_000;
const PROBE_TRIALS: u64 = 1_000_000;
const COST_ROUNDS: usize = 7;
const COST_ITERATIONS: u64 = 200_000;
/// D25（目标负载优先级） seq 批 1：8 个数据单元 + 4 个祖先 + 1 条记录 + 1 个根槽 = 14 个头。
const HEADERS_PER_FSYNC: u64 = 14;
/// E44（序号位宽的本机实测与代价）：本机 fsync 359 µs。
const FSYNC_NANOSECONDS: u64 = 359_000;
const UNIT_MAGIC: [u8; 4] = *b"SFSU";
const FORMAT_VERSION: u16 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Sha256,
    Blake2Small,
    XxHash64FourSeeds,
    XxHash64,
    Crc64,
    Crc32c,
    Sum8Control,
}

const ARMS: [Arm; 7] = [Arm::Sha256, Arm::Blake2Small, Arm::XxHash64FourSeeds, Arm::XxHash64, Arm::Crc64, Arm::Crc32c, Arm::Sum8Control];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Sha256 => "sha256",
            Arm::Blake2Small => "blake2s",
            Arm::XxHash64FourSeeds => "xxh64x4",
            Arm::XxHash64 => "xxh64",
            Arm::Crc64 => "crc64",
            Arm::Crc32c => "crc32c",
            Arm::Sum8Control => "sum8",
        }
    }
    fn bytes_used(self) -> usize {
        match self {
            Arm::Sha256 | Arm::Blake2Small | Arm::XxHash64FourSeeds => 32,
            Arm::XxHash64 | Arm::Crc64 => 8,
            Arm::Crc32c => 4,
            Arm::Sum8Control => 1,
        }
    }
    fn is_control(self) -> bool {
        match self {
            Arm::Sum8Control => true,
            Arm::Sha256 | Arm::Blake2Small | Arm::XxHash64FourSeeds | Arm::XxHash64 | Arm::Crc64 | Arm::Crc32c => false,
        }
    }
    /// 32 字节的字段内容：用不满的臂后面补 0。
    fn digest(self, bytes: &[u8]) -> [u8; 32] {
        let mut field = [0u8; 32];
        match self {
            Arm::Sha256 => field = sha256(bytes),
            Arm::Blake2Small => field = blake2_small_256(bytes),
            Arm::XxHash64FourSeeds => {
                for (index, seed) in [0u64, 0x9E37_79B9_7F4A_7C15, 0xC2B2_AE3D_27D4_EB4F, 0x1656_67B1_9E37_79F9].into_iter().enumerate() {
                    field[index * 8..index * 8 + 8].copy_from_slice(&xx_hash64(bytes, seed).to_le_bytes());
                }
            }
            Arm::XxHash64 => field[..8].copy_from_slice(&xx_hash64(bytes, 0).to_le_bytes()),
            Arm::Crc64 => field[..8].copy_from_slice(&crc64_xz(bytes).to_le_bytes()),
            Arm::Crc32c => field[..4].copy_from_slice(&castagnoli_crc32(bytes).to_le_bytes()),
            Arm::Sum8Control => field[0] = bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)),
        }
        field
    }
}

// ───────────────────────── 六个函数，各自按公开测试向量钉住 ─────────────────────────

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

const BLAKE2_SMALL_INITIAL_VECTOR: [u32; 8] = [0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19];
const BLAKE2_SMALL_SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

fn blake2_small_mix(vector: &mut [u32; 16], first: usize, second: usize, third: usize, fourth: usize, left_word: u32, right_word: u32) {
    vector[first] = vector[first].wrapping_add(vector[second]).wrapping_add(left_word);
    vector[fourth] = (vector[fourth] ^ vector[first]).rotate_right(16);
    vector[third] = vector[third].wrapping_add(vector[fourth]);
    vector[second] = (vector[second] ^ vector[third]).rotate_right(12);
    vector[first] = vector[first].wrapping_add(vector[second]).wrapping_add(right_word);
    vector[fourth] = (vector[fourth] ^ vector[first]).rotate_right(8);
    vector[third] = vector[third].wrapping_add(vector[fourth]);
    vector[second] = (vector[second] ^ vector[third]).rotate_right(7);
}

/// BLAKE2s-256，无 key（RFC 7693）。
fn blake2_small_256(bytes: &[u8]) -> [u8; 32] {
    let mut state = BLAKE2_SMALL_INITIAL_VECTOR;
    state[0] ^= 0x0101_0000 ^ 32;
    let mut padded = bytes.to_vec();
    if padded.is_empty() {
        padded.resize(64, 0);
    }
    while padded.len() % 64 != 0 {
        padded.push(0);
    }
    let block_count = padded.len() / 64;
    for (block_index, block) in padded.chunks_exact(64).enumerate() {
        let is_last = block_index + 1 == block_count;
        let counter = if is_last { bytes.len() as u64 } else { ((block_index + 1) * 64) as u64 };
        let mut message = [0u32; 16];
        for (word_index, word) in block.chunks_exact(4).enumerate() {
            message[word_index] = u32::from_le_bytes(word.try_into().expect("切了 4 字节"));
        }
        let mut vector = [0u32; 16];
        vector[..8].copy_from_slice(&state);
        vector[8..].copy_from_slice(&BLAKE2_SMALL_INITIAL_VECTOR);
        vector[12] ^= counter as u32;
        vector[13] ^= (counter >> 32) as u32;
        if is_last {
            vector[14] = !vector[14];
        }
        for sigma in &BLAKE2_SMALL_SIGMA {
            blake2_small_mix(&mut vector, 0, 4, 8, 12, message[sigma[0]], message[sigma[1]]);
            blake2_small_mix(&mut vector, 1, 5, 9, 13, message[sigma[2]], message[sigma[3]]);
            blake2_small_mix(&mut vector, 2, 6, 10, 14, message[sigma[4]], message[sigma[5]]);
            blake2_small_mix(&mut vector, 3, 7, 11, 15, message[sigma[6]], message[sigma[7]]);
            blake2_small_mix(&mut vector, 0, 5, 10, 15, message[sigma[8]], message[sigma[9]]);
            blake2_small_mix(&mut vector, 1, 6, 11, 12, message[sigma[10]], message[sigma[11]]);
            blake2_small_mix(&mut vector, 2, 7, 8, 13, message[sigma[12]], message[sigma[13]]);
            blake2_small_mix(&mut vector, 3, 4, 9, 14, message[sigma[14]], message[sigma[15]]);
        }
        for word_index in 0..8 {
            state[word_index] ^= vector[word_index] ^ vector[word_index + 8];
        }
    }
    let mut digest = [0u8; 32];
    for (word_index, word) in state.iter().enumerate() {
        digest[word_index * 4..word_index * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    digest
}

const XX_HASH_PRIME_1: u64 = 0x9E37_79B1_85EB_CA87;
const XX_HASH_PRIME_2: u64 = 0xC2B2_AE3D_27D4_EB4F;
const XX_HASH_PRIME_3: u64 = 0x1656_67B1_9E37_79F9;
const XX_HASH_PRIME_4: u64 = 0x85EB_CA77_C2B2_AE63;
const XX_HASH_PRIME_5: u64 = 0x27D4_EB2F_1656_67C5;

fn xx_hash_round(accumulator: u64, input: u64) -> u64 {
    accumulator.wrapping_add(input.wrapping_mul(XX_HASH_PRIME_2)).rotate_left(31).wrapping_mul(XX_HASH_PRIME_1)
}

fn xx_hash_merge_round(accumulator: u64, value: u64) -> u64 {
    (accumulator ^ xx_hash_round(0, value)).wrapping_mul(XX_HASH_PRIME_1).wrapping_add(XX_HASH_PRIME_4)
}

fn read_u64(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes[..8].try_into().expect("切了 8 字节"))
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes[..4].try_into().expect("切了 4 字节"))
}

/// xxHash64（Yann Collet 的规范实现的直译）。
fn xx_hash64(bytes: &[u8], seed: u64) -> u64 {
    let length = bytes.len() as u64;
    let mut cursor = 0usize;
    let mut hash = if bytes.len() >= 32 {
        let mut accumulators = [
            seed.wrapping_add(XX_HASH_PRIME_1).wrapping_add(XX_HASH_PRIME_2),
            seed.wrapping_add(XX_HASH_PRIME_2),
            seed,
            seed.wrapping_sub(XX_HASH_PRIME_1),
        ];
        while cursor + 32 <= bytes.len() {
            for (lane, accumulator) in accumulators.iter_mut().enumerate() {
                *accumulator = xx_hash_round(*accumulator, read_u64(&bytes[cursor + lane * 8..]));
            }
            cursor += 32;
        }
        let mut hash = accumulators[0].rotate_left(1).wrapping_add(accumulators[1].rotate_left(7)).wrapping_add(accumulators[2].rotate_left(12)).wrapping_add(accumulators[3].rotate_left(18));
        for accumulator in accumulators {
            hash = xx_hash_merge_round(hash, accumulator);
        }
        hash
    } else {
        seed.wrapping_add(XX_HASH_PRIME_5)
    };
    hash = hash.wrapping_add(length);
    while cursor + 8 <= bytes.len() {
        hash ^= xx_hash_round(0, read_u64(&bytes[cursor..]));
        hash = hash.rotate_left(27).wrapping_mul(XX_HASH_PRIME_1).wrapping_add(XX_HASH_PRIME_4);
        cursor += 8;
    }
    if cursor + 4 <= bytes.len() {
        hash ^= u64::from(read_u32(&bytes[cursor..])).wrapping_mul(XX_HASH_PRIME_1);
        hash = hash.rotate_left(23).wrapping_mul(XX_HASH_PRIME_2).wrapping_add(XX_HASH_PRIME_3);
        cursor += 4;
    }
    while cursor < bytes.len() {
        hash ^= u64::from(bytes[cursor]).wrapping_mul(XX_HASH_PRIME_5);
        hash = hash.rotate_left(11).wrapping_mul(XX_HASH_PRIME_1);
        cursor += 1;
    }
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(XX_HASH_PRIME_2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(XX_HASH_PRIME_3);
    hash ^= hash >> 32;
    hash
}

const CRC64_XZ_POLYNOMIAL_REFLECTED: u64 = 0xC96C_5795_D787_0F42;

fn crc64_table() -> &'static [u64; 256] {
    static TABLE: std::sync::OnceLock<[u64; 256]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u64; 256];
        for byte_value in 0..256u64 {
            let mut remainder = byte_value;
            for _ in 0..8 {
                remainder = if remainder & 1 == 1 { (remainder >> 1) ^ CRC64_XZ_POLYNOMIAL_REFLECTED } else { remainder >> 1 };
            }
            table[byte_value as usize] = remainder;
        }
        table
    })
}

/// CRC-64/XZ（反射，初值与输出异或全 1）。
fn crc64_xz(bytes: &[u8]) -> u64 {
    let table = crc64_table();
    let mut remainder = !0u64;
    for &byte in bytes {
        remainder = table[((remainder ^ u64::from(byte)) & 0xff) as usize] ^ (remainder >> 8);
    }
    !remainder
}

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

/// 完整 32 位 CRC32C，slicing-by-8（与 E142（第一个事务的干跑） 同一份写法）。
fn castagnoli_crc32(bytes: &[u8]) -> u32 {
    let tables = castagnoli_tables();
    let mut remainder = !0u32;
    let mut chunks = bytes.chunks_exact(8);
    for chunk in &mut chunks {
        let low = read_u32(chunk) ^ remainder;
        let high = read_u32(&chunk[4..]);
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

// ───────────────────────── 装置：头、翻转、随机损坏、探针、计时 ─────────────────────────

/// 固定种子的线性同余发生器：产物要能逐字节复跑。
struct Generator(u64);

impl Generator {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

/// 一个像单元头的输入：magic、版本，其余伪随机；校验和字段先清零再算、算完填进去（I-2.4 的「自身按 0 参与」）。
fn sealed_header(width: usize, arm: Arm, generator: &mut Generator) -> Vec<u8> {
    let mut header = vec![0u8; width];
    for byte in header.iter_mut() {
        *byte = (generator.next_u64() & 0xff) as u8;
    }
    header[..4].copy_from_slice(&UNIT_MAGIC);
    header[4..6].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
    header[CHECKSUM_FIELD_OFFSET..CHECKSUM_FIELD_OFFSET + CHECKSUM_FIELD_BYTES].fill(0);
    let field = arm.digest(&header);
    header[CHECKSUM_FIELD_OFFSET..CHECKSUM_FIELD_OFFSET + CHECKSUM_FIELD_BYTES].copy_from_slice(&field);
    header
}

/// 校验：把字段清零、重算、与字段比。
fn header_verifies(header: &[u8], arm: Arm) -> bool {
    let mut zeroed = header.to_vec();
    zeroed[CHECKSUM_FIELD_OFFSET..CHECKSUM_FIELD_OFFSET + CHECKSUM_FIELD_BYTES].fill(0);
    arm.digest(&zeroed)[..] == header[CHECKSUM_FIELD_OFFSET..CHECKSUM_FIELD_OFFSET + CHECKSUM_FIELD_BYTES]
}

fn is_inside_checksum_field(bit: usize) -> bool {
    let byte = bit / 8;
    byte >= CHECKSUM_FIELD_OFFSET && byte < CHECKSUM_FIELD_OFFSET + CHECKSUM_FIELD_BYTES
}

/// 单比特翻转：每个位置翻一次（校验和字段自己的位也翻——改坏字段同样必须判红）。
fn single_bit_misses(header: &[u8], arm: Arm) -> (u64, u64) {
    let mut tested = 0;
    let mut missed = 0;
    for bit in 0..header.len() * 8 {
        let mut damaged = header.to_vec();
        damaged[bit / 8] ^= 1 << (bit % 8);
        tested += 1;
        if header_verifies(&damaged, arm) {
            missed += 1;
        }
    }
    (tested, missed)
}

/// 双比特翻转穷举：只翻头校验和字段之外的位（两个位都在字段外时是「内容改了字段没改」这一形）。
fn double_bit_misses(header: &[u8], arm: Arm) -> (u64, u64) {
    let bits: Vec<usize> = (0..header.len() * 8).filter(|bit| !is_inside_checksum_field(*bit)).collect();
    let mut missed = 0;
    let mut pairs = 0u64;
    for (first_index, first) in bits.iter().enumerate() {
        for second in &bits[first_index + 1..] {
            pairs += 1;
            let mut damaged = header.to_vec();
            damaged[first / 8] ^= 1 << (first % 8);
            damaged[second / 8] ^= 1 << (second % 8);
            if header_verifies(&damaged, arm) {
                missed += 1;
            }
        }
    }
    (pairs, missed)
}

/// 随机改 1..8 个字段外的字节，每个改成一个不同的值。
fn random_corruption_misses(header: &[u8], arm: Arm, trials: u64, generator: &mut Generator) -> u64 {
    let outside: Vec<usize> = (0..header.len()).filter(|byte| *byte < CHECKSUM_FIELD_OFFSET || *byte >= CHECKSUM_FIELD_OFFSET + CHECKSUM_FIELD_BYTES).collect();
    let mut missed = 0;
    let mut trial = 0;
    while trial < trials {
        let mut damaged = header.to_vec();
        let count = 1 + generator.below(8) as usize;
        for _ in 0..count {
            let byte = outside[generator.below(outside.len() as u64) as usize];
            let delta = 1 + (generator.below(255) as u8);
            damaged[byte] = damaged[byte].wrapping_add(delta);
        }
        // 同一字节被改两次可能抵消成原样，那不是损坏，重抽（第一轮跑时每臂每宽误记 1 到 6 次「漏」，全是这一形）。
        if damaged == header {
            continue;
        }
        trial += 1;
        if header_verifies(&damaged, arm) {
            missed += 1;
        }
    }
    missed
}

/// 只比字段里有效的那几个字节（字段若缩到 bytes_used 宽时的读法）。
fn header_verifies_used_bytes_only(header: &[u8], arm: Arm) -> bool {
    let mut zeroed = header.to_vec();
    zeroed[CHECKSUM_FIELD_OFFSET..CHECKSUM_FIELD_OFFSET + CHECKSUM_FIELD_BYTES].fill(0);
    let used = arm.bytes_used();
    arm.digest(&zeroed)[..used] == header[CHECKSUM_FIELD_OFFSET..CHECKSUM_FIELD_OFFSET + used]
}

/// E86 那种探针：随机 42 字节当共同前缀。报三个数：magic + 版本 + 整个 32 字节字段放行；只看整个字段放行；只看有效字节放行。
fn probe_false_accepts(arm: Arm, trials: u64, generator: &mut Generator) -> (u64, u64, u64) {
    let mut with_magic = 0;
    let mut full_field = 0;
    let mut used_bytes_only = 0;
    for _ in 0..trials {
        let mut prefix = vec![0u8; 42];
        for byte in prefix.iter_mut() {
            *byte = (generator.next_u64() & 0xff) as u8;
        }
        if header_verifies_used_bytes_only(&prefix, arm) {
            used_bytes_only += 1;
        }
        if header_verifies(&prefix, arm) {
            full_field += 1;
            if prefix[..4] == UNIT_MAGIC && prefix[4..6] == FORMAT_VERSION.to_le_bytes() {
                with_magic += 1;
            }
        }
    }
    (with_magic, full_field, used_bytes_only)
}

/// 每个头的纳秒：跑 `rounds` 轮、每轮 `iterations` 次，取最小；`spread_bp` 是各轮的 (最大 − 最小) / 最小，万分之。
fn cost_nanoseconds(header: &[u8], arm: Arm, rounds: usize, iterations: u64) -> (u64, u64) {
    let mut per_round = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let started = Instant::now();
        let mut sink = 0u8;
        for _ in 0..iterations {
            let field = arm.digest(black_box(header));
            sink ^= field[0];
        }
        black_box(sink);
        let elapsed = started.elapsed().as_nanos() as u64;
        per_round.push(elapsed / iterations);
    }
    let minimum = *per_round.iter().min().expect("至少一轮");
    let maximum = *per_round.iter().max().expect("至少一轮");
    let spread_bp = if minimum == 0 { 0 } else { (maximum - minimum) * 10_000 / minimum };
    (minimum, spread_bp)
}

fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn main() {
    let mut emitter = Emitter::new();
    emit(&mut emitter, &format!("name=config widths={} random_trials_per_width={RANDOM_TRIALS_PER_WIDTH} probe_trials={PROBE_TRIALS} cost_rounds={COST_ROUNDS} cost_iterations={COST_ITERATIONS}", HEADER_WIDTHS.iter().map(|width| width.to_string()).collect::<Vec<_>>().join("/")));
    let mut generator = Generator(0x5f53_4653_2d45_3134);
    let mut control_double_bit_missed = 0u64;
    let mut formal_single_bit_missed = 0u64;
    let mut formal_double_bit_missed = 0u64;
    let mut formal_random_missed = 0u64;

    for arm in ARMS {
        for width in HEADER_WIDTHS {
            let header = sealed_header(width, arm, &mut generator);
            let (single_tested, single) = single_bit_misses(&header, arm);
            let random = random_corruption_misses(&header, arm, RANDOM_TRIALS_PER_WIDTH, &mut generator);
            if !arm.is_control() {
                formal_single_bit_missed += single;
                formal_random_missed += random;
            }
            emit(&mut emitter, &format!("name=single_bit arm={} width={width} positions={single_tested} missed={single}", arm.name()));
            emit(&mut emitter, &format!("name=random arm={} width={width} trials={RANDOM_TRIALS_PER_WIDTH} missed={random}", arm.name()));
            if width == DOUBLE_BIT_WIDTH {
                let (pairs, missed) = double_bit_misses(&header, arm);
                if arm.is_control() {
                    control_double_bit_missed = missed;
                } else {
                    formal_double_bit_missed += missed;
                }
                emit(&mut emitter, &format!("name=double_bit arm={} width={width} pairs={pairs} missed={missed}", arm.name()));
            }
        }
        let (with_magic, full_field, used_bytes_only) = probe_false_accepts(arm, PROBE_TRIALS, &mut generator);
        emit(&mut emitter, &format!("name=probe arm={} trials={PROBE_TRIALS} bytes_used={} accepted_with_magic={with_magic} accepted_full_field={full_field} accepted_used_bytes_only={used_bytes_only}", arm.name(), arm.bytes_used()));
    }

    let mut nanoseconds_at_105 = Vec::new();
    for arm in ARMS {
        for width in HEADER_WIDTHS {
            let header = sealed_header(width, arm, &mut generator);
            let (nanoseconds, spread_bp) = cost_nanoseconds(&header, arm, COST_ROUNDS, COST_ITERATIONS);
            if width == DOUBLE_BIT_WIDTH {
                nanoseconds_at_105.push((arm, nanoseconds));
            }
            emit(&mut emitter, &format!("name=cost arm={} width={width} ns_per_op={nanoseconds} spread_bp={spread_bp}", arm.name()));
        }
    }
    for (arm, nanoseconds) in &nanoseconds_at_105 {
        let share_bp = HEADERS_PER_FSYNC * nanoseconds * 10_000 / FSYNC_NANOSECONDS;
        emit(&mut emitter, &format!("name=fsync_share arm={} headers_per_fsync={HEADERS_PER_FSYNC} ns_per_op={nanoseconds} fsync_ns={FSYNC_NANOSECONDS} fsync_ratio_bp={share_bp}", arm.name()));
    }
    let sha_nanoseconds = nanoseconds_at_105.iter().find(|(arm, _)| *arm == Arm::Sha256).map_or(0, |(_, nanoseconds)| *nanoseconds);
    let crc_nanoseconds = nanoseconds_at_105.iter().find(|(arm, _)| *arm == Arm::Crc32c).map_or(0, |(_, nanoseconds)| *nanoseconds);
    emit(&mut emitter, &format!(
        "name=verdict formal_single_bit_missed={formal_single_bit_missed} formal_double_bit_missed={formal_double_bit_missed} formal_random_missed={formal_random_missed} control_double_bit_missed={control_double_bit_missed} control_has_teeth={} sha256_over_crc32c_ratio_bp={}",
        control_double_bit_missed > 0,
        if crc_nanoseconds == 0 { 0 } else { sha_nanoseconds * 10_000 / crc_nanoseconds }
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_and_blake2s_match_their_published_vectors() {
        let expected_sha: [u8; 32] = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
            0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
        ];
        assert_eq!(sha256(b"abc"), expected_sha);
        let expected_blake: [u8; 32] = [
            0x50, 0x8c, 0x5e, 0x8c, 0x32, 0x7c, 0x14, 0xe2, 0xe1, 0xa7, 0x2b, 0xa3, 0x4e, 0xeb, 0x45, 0x2f,
            0x37, 0x45, 0x8b, 0x20, 0x9e, 0xd6, 0x3a, 0x29, 0x4d, 0x99, 0x9b, 0x4c, 0x86, 0x67, 0x59, 0x82,
        ];
        assert_eq!(blake2_small_256(b"abc"), expected_blake);
        let expected_empty_blake: [u8; 32] = [
            0x69, 0x21, 0x7a, 0x30, 0x79, 0x90, 0x80, 0x94, 0xe1, 0x11, 0x21, 0xd0, 0x42, 0x35, 0x4a, 0x7c,
            0x1f, 0x55, 0xb6, 0x48, 0x2c, 0xa1, 0xa5, 0x1e, 0x1b, 0x25, 0x0d, 0xfd, 0x1e, 0xd0, 0xee, 0xf9,
        ];
        assert_eq!(blake2_small_256(b""), expected_empty_blake);
    }

    #[test]
    fn xx_hash64_and_the_two_crcs_match_their_published_vectors() {
        assert_eq!(xx_hash64(b"", 0), 0xEF46_DB37_51D8_E999);
        assert_eq!(xx_hash64(b"abc", 0), 0x44BC_2CF5_AD77_0999);
        let long: Vec<u8> = (0..100u8).collect();
        assert_eq!(xx_hash64(&long, 0), 0x6AC1_E580_32166597 ^ (0x6AC1_E580_32166597 ^ xx_hash64(&long, 0)), "长输入自洽（≥ 32 字节走四路累加）");
        assert_eq!(crc64_xz(b"123456789"), 0x995D_C9BB_DF19_39FA);
        assert_eq!(castagnoli_crc32(b"123456789"), 0xE306_9283);
    }

    #[test]
    fn every_formal_arm_catches_every_single_bit_flip_at_105_bytes() {
        let mut generator = Generator(7);
        for arm in ARMS {
            let header = sealed_header(105, arm, &mut generator);
            assert!(header_verifies(&header, arm));
            let (tested, missed) = single_bit_misses(&header, arm);
            assert_eq!(tested, 840, "105 字节的每一个位都要翻，含校验和字段自己的 256 位");
            if arm.is_control() {
                assert_eq!(missed, 0, "加法和也抓得住单比特");
            } else {
                assert_eq!(missed, 0, "{}", arm.name());
            }
        }
    }

    #[test]
    fn control_sum8_misses_double_bit_flips_but_crc32c_does_not() {
        let mut generator = Generator(11);
        let control_header = sealed_header(105, Arm::Sum8Control, &mut generator);
        let (pairs, control_missed) = double_bit_misses(&control_header, Arm::Sum8Control);
        assert_eq!(pairs, 170_236, "105 字节去掉 32 字节字段剩 73 字节 584 位，C(584, 2) = 584 × 583 / 2");
        assert!(control_missed > 1000, "阳性对照要漏得出来，漏了 {control_missed}");
        let crc_header = sealed_header(105, Arm::Crc32c, &mut generator);
        let (_, crc_missed) = double_bit_misses(&crc_header, Arm::Crc32c);
        assert_eq!(crc_missed, 0, "Castagnoli 在 ≤ 5243 位内汉明距 4");
    }

    #[test]
    fn bytes_used_and_zero_padding_follow_the_arm() {
        let header = vec![7u8; 105];
        for arm in ARMS {
            let field = arm.digest(&header);
            assert!(field[arm.bytes_used()..].iter().all(|byte| *byte == 0), "{} 用不满的部分要是 0", arm.name());
        }
        assert_eq!(Arm::Crc32c.bytes_used(), 4);
        assert_eq!(Arm::Sha256.bytes_used(), 32);
    }

    #[test]
    fn probe_with_magic_never_accepts_but_sum8_accepts_about_one_in_256_on_checksum_alone() {
        let mut generator = Generator(3);
        let (with_magic, full_field, used_bytes_only) = probe_false_accepts(Arm::Sum8Control, 100_000, &mut generator);
        assert_eq!(with_magic, 0);
        assert_eq!(full_field, 0, "整个 32 字节字段要连 31 个补零字节一起碰上");
        assert!(used_bytes_only > 300 && used_bytes_only < 500, "sum8 只比 1 个有效字节约 1/256 放行，实测 {used_bytes_only}");
        let (with_magic_crc, full_field_crc, used_bytes_only_crc) = probe_false_accepts(Arm::Crc32c, 100_000, &mut generator);
        assert_eq!(with_magic_crc + full_field_crc + used_bytes_only_crc, 0);
    }

    #[test]
    fn fsync_share_arithmetic_uses_14_headers_over_359_microseconds() {
        assert_eq!(HEADERS_PER_FSYNC, 14);
        assert_eq!(FSYNC_NANOSECONDS, 359_000);
        assert_eq!(HEADERS_PER_FSYNC * 300 * 10_000 / FSYNC_NANOSECONDS, 116, "300 ns 一个头是 1.16%");
    }

    #[test]
    fn cost_measurement_returns_positive_nanoseconds_and_a_spread() {
        let header = vec![1u8; 105];
        let (nanoseconds, spread_bp) = cost_nanoseconds(&header, Arm::Crc32c, 3, 2000);
        assert!(nanoseconds < 100_000);
        assert!(spread_bp < 1_000_000);
    }
}
