//! 块值的生成器与校验和、候选存储、写设置与断电臂的全部定义。

use super::*;

// ============================== 随机源、校验和、SHA-256 ==============================

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
    pub(super) entries: [u32; 256],
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

pub(super) fn sha256_hexadecimal(message: &[u8]) -> String {
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
pub(super) enum ValuePattern {
    /// P-rand：从起始状态连取 V/8 个输出，逐个小端拼成 V 字节。
    Random,
    /// P-sparse：每状态取一个输出 r，r >> 56 == 0 时是 1 + (r & 0xFF) mod 255，否则 0（只给 A4 锚点用）。
    Sparse,
}

fn block_start_state(seed: u64, block_index: u64) -> u64 {
    seed ^ block_index.wrapping_mul(MIXING_MULTIPLIER)
}

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

pub(super) fn input_fingerprint() -> [u8; FINGERPRINT_BYTES] {
    let mut generator = SplitMix64::new(SEED_FINGERPRINT);
    let mut fingerprint = [0u8; FINGERPRINT_BYTES];
    fingerprint[..8].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint[8..].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint
}

pub(super) type BlockKey = [u8; BLOCK_KEY_BYTES];

pub(super) fn compose_block_key(node: u32, segment: u32, first_state: u64, fingerprint: &[u8; FINGERPRINT_BYTES]) -> BlockKey {
    let mut key = [0u8; BLOCK_KEY_BYTES];
    key[0..4].copy_from_slice(&node.to_be_bytes());
    key[4..8].copy_from_slice(&segment.to_be_bytes());
    key[8..16].copy_from_slice(&first_state.to_be_bytes());
    key[16..32].copy_from_slice(fingerprint);
    key
}

pub(super) fn interleaved_block_key(block_index: u64, states_per_block: u64, fingerprint: &[u8; FINGERPRINT_BYTES]) -> BlockKey {
    let stream = block_index % INTERLEAVED_STREAM_COUNT;
    let block_in_stream = block_index / INTERLEAVED_STREAM_COUNT;
    let node = u32::try_from(stream / SEGMENTS_PER_NODE).expect("节点号小于 8");
    let segment = u32::try_from(stream % SEGMENTS_PER_NODE).expect("段号小于 8");
    compose_block_key(node, segment, block_in_stream * states_per_block, fingerprint)
}

pub(super) fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ============================== 臂（补 7 B1）与写入设定 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Candidate {
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
    pub(super) fn label(self) -> &'static str {
        match self {
            Candidate::RedbQuickRepair => "R1",
            Candidate::RedbDefault => "R0",
            Candidate::RocksDatabase => "K",
            Candidate::HardenedFile => "F",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RedbCommitDurability {
    Immediate,
    NotPersisted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RedbRepairMode {
    QuickRepair,
    FullRepair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RocksDatabaseWriteSync {
    SyncEachWrite,
    NoSync,
}

/// F 的持久步骤：全做；临时文件与目录都不 fsync（F-nofsync）；只拿掉改名之后的父目录 fsync（F-nodirsync）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FileSyncSteps {
    TemporaryFileAndDirectory,
    NeitherTemporaryFileNorDirectory,
    TemporaryFileOnly,
}

/// ext4 的挂载选项：默认（barrier 开）或 `barrier=0`（PC4-NB）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FilesystemBarrier {
    Enabled,
    Disabled,
}

/// 一条臂的全部写入选项；对照形态与正确形态逐项比，必须恰好差登记的那一处（M26）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WriteSettings {
    pub(super) redb_commit_durability: RedbCommitDurability,
    pub(super) redb_repair_mode: RedbRepairMode,
    pub(super) rocksdb_write_sync: RocksDatabaseWriteSync,
    pub(super) file_sync_steps: FileSyncSteps,
    pub(super) filesystem_barrier: FilesystemBarrier,
}

impl WriteSettings {
    pub(super) fn registered_for(candidate: Candidate) -> WriteSettings {
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

    pub(super) fn differing_setting_names(self, other: WriteSettings) -> Vec<&'static str> {
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
pub(super) enum PowerCutArm {
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

pub(super) const ALL_POWER_CUT_ARMS: [PowerCutArm; 18] = [
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
pub(super) enum ArmPurpose {
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
    pub(super) fn index(self) -> u32 {
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

    pub(super) fn label(self) -> &'static str {
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

    pub(super) fn from_index(index: u32) -> Option<PowerCutArm> {
        ALL_POWER_CUT_ARMS.iter().copied().find(|arm| arm.index() == index)
    }

    /// 四条主臂（登记 R1、R0、K、F）；`s4-drive` 不给 `--arms` 就是它们全部。
    pub(super) const EVERY_MAIN_ARM: [PowerCutArm; 4] =
        [PowerCutArm::RedbQuickRepair, PowerCutArm::RedbDefault, PowerCutArm::RocksDatabase, PowerCutArm::HardenedFile];

    pub(super) fn main_arm_from_label(label: &str) -> Option<PowerCutArm> {
        PowerCutArm::EVERY_MAIN_ARM.into_iter().find(|arm| arm.label() == label)
    }

    pub(super) fn candidate(self) -> Option<Candidate> {
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

    pub(super) fn purpose(self) -> ArmPurpose {
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
    pub(super) fn correct_form(self) -> PowerCutArm {
        match self.candidate() {
            None => self,
            Some(Candidate::RedbQuickRepair) => PowerCutArm::RedbQuickRepair,
            Some(Candidate::RedbDefault) => PowerCutArm::RedbDefault,
            Some(Candidate::RocksDatabase) => PowerCutArm::RocksDatabase,
            Some(Candidate::HardenedFile) => PowerCutArm::HardenedFile,
        }
    }

    /// 登记里这个对照形态与正确形态差的那一处（补 5.2 阳性对照表）；主臂与取样点是 None。
    pub(super) fn registered_control_difference(self) -> Option<&'static str> {
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

    pub(super) fn write_settings(self) -> Option<WriteSettings> {
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

    pub(super) fn states_per_block(self) -> u64 {
        match self.purpose() {
            ArmPurpose::SmallBlockSamplingPoint => SMALL_STATES_PER_BLOCK,
            ArmPurpose::Main
            | ArmPurpose::LostPageCacheControl
            | ArmPurpose::LostWriteCacheControl
            | ArmPurpose::MissingDirectorySyncControl
            | ArmPurpose::WriteCacheModelControl => PRIMARY_STATES_PER_BLOCK,
        }
    }

    pub(super) fn registered_block_count(self) -> u64 {
        match self.purpose() {
            ArmPurpose::Main | ArmPurpose::SmallBlockSamplingPoint => MAIN_BLOCK_COUNT,
            ArmPurpose::LostPageCacheControl | ArmPurpose::LostWriteCacheControl | ArmPurpose::MissingDirectorySyncControl => {
                CONTROL_BLOCK_COUNT
            }
            ArmPurpose::WriteCacheModelControl => 2 * WRITE_CACHE_MODEL_BATCH_WRITES,
        }
    }

    pub(super) fn lost_page_cache_control(self) -> PowerCutArm {
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

    pub(super) fn lost_write_cache_control(self) -> PowerCutArm {
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

    pub(super) fn small_block_sampling_point(self) -> PowerCutArm {
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
