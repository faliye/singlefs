//! 块值的生成器与校验和、块键、杀点计划，以及产物行、候选与写路径、锚点行。

use super::*;

// ============================== 随机源与校验和 ==============================

/// SplitMix64（登记 5.1）：全部 64 位回绕。
pub(super) struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub(super) fn new(state: u64) -> Self {
        Self { state }
    }

    pub(super) fn next_output(&mut self) -> u64 {
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
    pub(super) fn new() -> Self {
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

pub(super) fn crc32c(parts: &[&[u8]]) -> u32 {
    thread_local! {
        static TABLE: Crc32cTable = Crc32cTable::new();
    }
    TABLE.with(|table| table.checksum(parts))
}

// ============================== 块值、键、次序、杀点 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ValuePattern {
    /// P-rand：从起始状态连取 V/8 个输出，逐个小端拼成 V 字节。
    Random,
    /// P-sparse：每状态取一个输出 r，r >> 56 == 0 时是 1 + (r & 0xFF) mod 255，否则 0。
    Sparse,
}

impl ValuePattern {
    pub(super) fn label(self) -> &'static str {
        match self {
            ValuePattern::Random => "random",
            ValuePattern::Sparse => "sparse",
        }
    }

    pub(super) fn parse(text: &str) -> ValuePattern {
        match text {
            "random" => ValuePattern::Random,
            "sparse" => ValuePattern::Sparse,
            other => panic!("认不出的块值形态 {other}"),
        }
    }
}

fn block_start_state(seed: u64, block_index: u64) -> u64 {
    seed ^ block_index.wrapping_mul(BLOCK_START_MULTIPLIER)
}

/// 第 block_index 块的块值；每状态 1 字节，所以块值字节数 = 每块状态数。
pub(super) fn block_value(pattern: ValuePattern, seed: u64, block_index: u64, states_per_block: u64) -> Vec<u8> {
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

/// 输入指纹：SplitMix64(seed_FP) 连取两个输出、小端拼成 16 字节。
pub(super) fn input_fingerprint() -> [u8; FINGERPRINT_BYTES] {
    let mut generator = SplitMix64::new(SEED_FINGERPRINT);
    let mut fingerprint = [0u8; FINGERPRINT_BYTES];
    fingerprint[..8].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint[8..].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint
}

pub(super) type BlockKey = [u8; BLOCK_KEY_BYTES];

/// 块键：节点 u32 大端｜段号 u32 大端｜起点 u64 大端｜输入指纹 16 字节。
pub(super) fn compose_block_key(node: u32, segment: u32, first_state: u64, fingerprint: &[u8; FINGERPRINT_BYTES]) -> BlockKey {
    let mut key = [0u8; BLOCK_KEY_BYTES];
    key[0..4].copy_from_slice(&node.to_be_bytes());
    key[4..8].copy_from_slice(&segment.to_be_bytes());
    key[8..16].copy_from_slice(&first_state.to_be_bytes());
    key[16..32].copy_from_slice(fingerprint);
    key
}

/// 交错次序下第 block_index 块的键：流号 s = j mod 64、流内块号 b = j div 64，起点 = b × 每块状态数。
pub(super) fn interleaved_block_key(block_index: u64, states_per_block: u64, fingerprint: &[u8; FINGERPRINT_BYTES]) -> BlockKey {
    let stream = block_index % INTERLEAVED_STREAM_COUNT;
    let block_in_stream = block_index / INTERLEAVED_STREAM_COUNT;
    let node = u32::try_from(stream / SEGMENTS_PER_NODE).expect("节点号小于 8");
    let segment = u32::try_from(stream % SEGMENTS_PER_NODE).expect("段号小于 8");
    compose_block_key(node, segment, block_in_stream * states_per_block, fingerprint)
}

/// [0, N) 的 Fisher–Yates 置换：i 从 N−1 降到 1，k = 输出 mod (i+1)，交换 a[i] 与 a[k]。
pub(super) fn random_permutation(element_count: u64, seed: u64) -> Vec<u64> {
    let mut permutation: Vec<u64> = (0..element_count).collect();
    let mut generator = SplitMix64::new(seed);
    let mut position = permutation.len();
    while position > 1 {
        position -= 1;
        let bound = u64::try_from(position).expect("位置装得进 u64") + 1;
        let swap_with = usize::try_from(generator.next_output() % bound).expect("小于元素数");
        permutation.swap(position, swap_with);
    }
    permutation
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum KillPhase {
    /// 等到本趟第 n 行 `C` 之后再睡 u × 4 × m 杀。
    DuringWrite { confirmations_before_kill: u64 },
    /// 起子进程之后睡 u × 2 × o 杀。
    DuringOpen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct KillPlanRow {
    pub(super) cycle_index: usize,
    pub(super) phase: KillPhase,
    pub(super) delay_fraction: f64,
}

/// 杀点计划：第 i 次先后取 r1、r2；u = (r2 >> 11) ÷ 2⁵³（登记 5.5）。
pub(super) fn kill_plan(seed: u64, cycle_count: usize) -> Vec<KillPlanRow> {
    let mut generator = SplitMix64::new(seed);
    (0..cycle_count)
        .map(|cycle_index| {
            let first_output = generator.next_output();
            let second_output = generator.next_output();
            let phase = if cycle_index % OPEN_PHASE_CYCLE_MODULUS == OPEN_PHASE_CYCLE_REMAINDER {
                KillPhase::DuringOpen
            } else {
                KillPhase::DuringWrite {
                    confirmations_before_kill: 1 + first_output % WRITE_PHASE_CONFIRMATION_MODULUS,
                }
            };
            let delay_fraction = (second_output >> 11) as f64 / (1u64 << 53) as f64;
            KillPlanRow { cycle_index, phase, delay_fraction }
        })
        .collect()
}

pub(super) fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 产物行：每行 `E7RESULT ` 起头，收尾 `name=done emitted=N`（N 连 done 行自己都算）。
pub(super) struct ProductLines {
    emitter: Emitter,
}

impl ProductLines {
    pub(super) fn new() -> Self {
        Self { emitter: Emitter::new() }
    }

    pub(super) fn line(&mut self, body: String) {
        let text = self.emitter.emit_raw(&body);
        let mut standard_output = std::io::stdout().lock();
        writeln!(standard_output, "{text}").expect("写得进标准输出");
        standard_output.flush().expect("刷得出标准输出");
    }

    pub(super) fn finish(&mut self) {
        let text = self.emitter.finish();
        println!("{text}");
    }
}

// ============================== 锚点（登记 7.2，复跑逐字节比） ==============================

pub(super) fn run_anchors(output: &mut ProductLines) -> i32 {
    let mut mismatches = 0usize;
    let mut record = |output: &mut ProductLines, name: &str, got: String, expected: &str| {
        let matches = got == expected;
        if !matches {
            mismatches += 1;
        }
        output.line(format!("name=anchor id={name} value={got} matches_registration={matches}"));
    };
    let mut zero_generator = SplitMix64::new(0);
    let first_three: Vec<String> = (0..3).map(|_| format!("{:#018x}", zero_generator.next_output())).collect();
    record(output, "A1", first_three.join(","), "0xe220a8397b1dcdaf,0x6e789e6aa1b965f4,0x06c45d188009454f");
    record(output, "A2", format!("{:#010x}", crc32c(&[b"123456789"])), "0xe3069283");
    for (block_index, expected) in [
        (0u64, "76020f1b59444d587a6607195c0f8fed/8351809/0x08eefd5c"),
        (12_345, "b3c1f0df0c77c9e98c9df5b21d0395bf/8321879/0x565d4801"),
    ] {
        let value = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        let byte_sum: u64 = value.iter().map(|&byte| u64::from(byte)).sum();
        let summary = format!("{}/{byte_sum}/{:#010x}", hexadecimal(&value[..16]), crc32c(&[&value]));
        record(output, &format!("A3_j{block_index}"), summary, expected);
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
    let permutation = random_permutation(THROUGHPUT_BLOCK_COUNT, SEED_ORDER);
    let head: Vec<String> = permutation[..6].iter().map(u64::to_string).collect();
    record(output, "A7", format!("{}/{}", head.join(","), permutation[99_999]), "40238,12140,20790,5207,14245,28683/91647");
    let plan = kill_plan(SEED_KILL, CRASH_MAIN_KILL_CYCLES);
    for (row, expected) in plan.iter().take(5).zip([
        "0/write/12/0.3236817149915837",
        "1/write/51/0.5669778417801824",
        "2/write/50/0.6058528089881228",
        "3/write/59/0.14049280271075437",
        "4/write/7/0.8853027823689169",
    ]) {
        record(output, &format!("A8_row{}", row.cycle_index), kill_plan_row_text(row), expected);
    }
    let open_rows = plan.iter().filter(|row| row.phase == KillPhase::DuringOpen).count();
    let write_confirmations: u64 = plan.iter().map(|row| confirmations_of(row.phase)).sum();
    record(output, "A8b", format!("{open_rows}/{write_confirmations}"), "20/6178");
    let milestone_blocks = (MILESTONE_TWO_STATES / PRIMARY_STATES_PER_BLOCK as f64).ceil();
    record(output, "A9", format!("{milestone_blocks}"), "289917");
    record(
        output,
        "A10",
        format!("{}/{}", THROUGHPUT_BLOCK_COUNT * PRIMARY_STATES_PER_BLOCK, THROUGHPUT_BLOCK_COUNT * SMALL_STATES_PER_BLOCK),
        "6553600000/102400000",
    );
    let frame_bytes = 4 + BLOCK_KEY_BYTES + 4 + 65_536 + 4;
    record(output, "A11", format!("{frame_bytes}/{}", FILE_HEADER_BYTES + 65_536), "65580/65584");
    output.line(format!("name=verdict part=anchors anchor_mismatches={mismatches}"));
    if mismatches == 0 {
        0
    } else {
        1
    }
}

pub(super) fn kill_plan_row_text(row: &KillPlanRow) -> String {
    let (phase_label, confirmations) = match row.phase {
        KillPhase::DuringWrite { confirmations_before_kill } => ("write", confirmations_before_kill),
        KillPhase::DuringOpen => ("open", 0),
    };
    format!("{}/{phase_label}/{confirmations}/{}", row.cycle_index, row.delay_fraction)
}

pub(super) fn confirmations_of(phase: KillPhase) -> u64 {
    match phase {
        KillPhase::DuringWrite { confirmations_before_kill } => confirmations_before_kill,
        KillPhase::DuringOpen => 0,
    }
}

// ============================== 候选与写法 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Candidate {
    /// R1：redb，每个写事务 Durability::Immediate + quick-repair。
    RedbQuickRepair,
    /// R0：redb，Durability::Immediate，quick-repair 关（默认）。
    RedbDefault,
    /// K：RocksDB，LZ4，同步写、WAL 开。
    RocksDatabase,
    /// F：加固的文件，一块一个文件，临时文件 → fsync → 改名 → 目录 fsync。
    HardenedFile,
}

impl Candidate {
    pub(super) fn label(self) -> &'static str {
        match self {
            Candidate::RedbQuickRepair => "R1",
            Candidate::RedbDefault => "R0",
            Candidate::RocksDatabase => "K",
            Candidate::HardenedFile => "F",
        }
    }

    pub(super) fn parse(text: &str) -> Option<Candidate> {
        match text {
            "R1" => Some(Candidate::RedbQuickRepair),
            "R0" => Some(Candidate::RedbDefault),
            "K" => Some(Candidate::RocksDatabase),
            "F" => Some(Candidate::HardenedFile),
            _ => None,
        }
    }

    /// PC-L 用的坏形态：R-None、K-noWAL、F-lazyrename（登记 5.3）。
    pub(super) fn lost_block_control_write_path(self) -> WritePath {
        match self {
            Candidate::RedbQuickRepair | Candidate::RedbDefault => WritePath::RedbDurabilityNone,
            Candidate::RocksDatabase => WritePath::RocksDatabaseWithoutWriteAheadLog,
            Candidate::HardenedFile => WritePath::FileLazyRename,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WritePath {
    /// 臂定义里的写法（登记 5.2）。
    Registered,
    /// R-None：每次提交 Durability::None。
    RedbDurabilityNone,
    /// K-noWAL：disable_wal(true)（RocksDB 不许同步写关 WAL，所以 sync 同时关掉）。
    RocksDatabaseWithoutWriteAheadLog,
    /// F-lazyrename：临时文件 sync_all 后就报已确认，改名与目录 fsync 攒到第 8、16… 次确认。
    FileLazyRename,
    /// F-direct：直接写终名，头一次、块值分 16 次写，每次之间睡 100 µs，不 fsync、不改名。
    FileDirectInPlace,
}

impl WritePath {
    pub(super) fn label(self) -> &'static str {
        match self {
            WritePath::Registered => "registered",
            WritePath::RedbDurabilityNone => "redb-durability-none",
            WritePath::RocksDatabaseWithoutWriteAheadLog => "rocksdb-no-wal",
            WritePath::FileLazyRename => "file-lazy-rename",
            WritePath::FileDirectInPlace => "file-direct",
        }
    }

    pub(super) fn parse(text: &str) -> WritePath {
        match text {
            "registered" => WritePath::Registered,
            "redb-durability-none" => WritePath::RedbDurabilityNone,
            "rocksdb-no-wal" => WritePath::RocksDatabaseWithoutWriteAheadLog,
            "file-lazy-rename" => WritePath::FileLazyRename,
            "file-direct" => WritePath::FileDirectInPlace,
            other => panic!("认不出的写法 {other}"),
        }
    }
}
