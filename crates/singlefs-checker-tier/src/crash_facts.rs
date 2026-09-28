//! 崩溃放量三段流的第 ① 段：CPU 把核对要的**事实**无脑写进 KV，不判对错。
//!
//! 一条流的枚举写表解析一次（用户 2026-09-28 定：CPU checker → 写入 KV → 读出 KV 核对（CPU 或 GPU）→ 写入 KV → 查出红汇总；
//! 同一种逻辑的活合在一段里做，不在每个状态上来回切）：每次单元写的类、两道校验和对不对、整单元 CRC、落在哪；每条根记录的字段、
//! 自证校验和对不对、四条指针；基线镜像（mkfs 之后）里已有的单元与根；哪几次写取三态、撕裂镜像与重放接在枚举写表的哪里；
//! 每一段展开出几个状态。**状态本身不存**：序号一到，掩码由 [`FlowFacts::persisted_writes_of_state`] 现算，与层 0 的
//! `Layer0StatePlan::persisted_writes_of_state` 同一套数字次序。
//!
//! 第 ② 段读事实判状态：GPU 内核在 `crash_verify_gpu`，CPU 参照实现是这里的 [`verify_state_from_facts`]——两份按同一张事实表、
//! 同一组判据算，逐状态相等是 GPU 那一份的判别力自证；第 ③ 段（`crash_amplification::compare_judge_and_verifier`）再把它们
//! 与 CPU 判器（`crash::judge_crash_image`）的判定逐状态比。
//!
//! 事实表进 KV 时键带**产出它的代码摘要**（[`FlowFacts::digest`] 连同 [`FACTS_EXTRACTOR_VERSION`]），核对表的键带核对代码摘要：
//! 换了代码就是新一轮，旧行留着当历史、不覆盖，每一段各自对账（用户 2026-09-28 定「事实表和核对表按代码摘要分版本存储、
//! 旧行保留为历史、每段独立对账」）。
use std::collections::BTreeMap;
use std::ops::Range;

use singlefs_checker::image::parse_node_pointer;
use singlefs_checker::{check_root_slot, check_unit, crc32_castagnoli_table};
use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::recovery::PoolReader;
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RECORD_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::sha256_digest;

use crate::crash::{layer0_enumeration_tables, Layer0EnumerationTables, Layer0SegmentExpansion};
use crate::crash_identity::SHA256_BYTES;

/// 事实抽取器的版本：解析规则变了就抬它，事实表的键跟着变。
pub const FACTS_EXTRACTOR_VERSION: u32 = 1;

/// 「基线镜像里本来就有」的单元或根，在事实里用这个来源号（不是任何一次写）。
pub const BASE_IMAGE_SOURCE: u32 = u32::MAX;

/// 根记录里四条节点指针的偏移（`singlefs_format::ROOT_RECORD_BYTES` 那一行的字段表：magic 4 + fsid 16 + flags 4 + 实例代号 4 +
/// checkpoint_txg 8 = 36 起是树表指针 86，之后树 ID 水位 8、回退下界 F 8、自证校验和 32，170 起实例表指针 86、256 起中央映射树根指针 86、
/// 342 起分配记录树根指针 86）。
pub const ROOT_POINTER_OFFSETS: [usize; 4] = [36, 170, 256, 342];
const ROOT_MAGIC: &[u8; 4] = b"SFSR";
const UNIT_MAGIC: &[u8; 4] = b"SFSU";
const UNIT_CLASS_OFFSET: usize = 6;
const SECTOR_BYTES: u64 = 512;

/// 核对结论的位：一位一条判据，0 就是这个状态在这几条上都成立。
pub mod verdict_bits {
    /// 最新持久的根指到的单元（整单元 CRC 与位置条目相符）自己的两道校验和有一道对不上。只判根指到的：崩溃状态里没被任何根引用的
    /// 半截单元校验和本来就对不上，池级 checker 也不判它们（第一版 GPU 对拍就打中过这一条：核对红、判器绿）。
    pub const UNIT_CHECKSUMS: u32 = 1 << 0;
    /// 最新持久的那条根自证校验和对不上（或 magic、flags 不合）。
    pub const NEWEST_ROOT_SELF_CHECKSUM: u32 = 1 << 1;
    /// 最新持久的那条根的某条非零指针，两份位置条目都指不到一个整单元 CRC 相符、且在这个状态里在盘上的单元。
    pub const NEWEST_ROOT_POINTER_TARGET: u32 = 1 << 2;
    /// 这个状态里一条根都没有（基线里也没有）：不该出现在层 0 的池上，出现就是事实表或掩码算错。
    pub const NO_ROOT_AT_ALL: u32 = 1 << 3;
}

/// 一个在盘上的单元（一次单元写，或基线镜像里本来就有的）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitFact {
    /// 枚举写表里的下标；基线里的是 [`BASE_IMAGE_SOURCE`]。
    pub source: u32,
    pub device: u32,
    pub slot: u64,
    pub unit_bytes: u32,
    pub class: u8,
    /// 池级 checker `check_unit` 判头校验和、载荷 CRC、magic、flags、预留位都过。
    pub checksums_hold: bool,
    /// 整单元 CRC-32C：位置条目里的校验和比的是它（`walk.rs` 的 `read_unit_without_judging`）。
    pub whole_crc32c: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationFact {
    pub device: u32,
    pub slot: u64,
    pub checksum: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerFact {
    pub all_zero: bool,
    pub locations: [LocationFact; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootFact {
    /// 枚举写表里的下标；基线里的是 [`BASE_IMAGE_SOURCE`]。
    pub source: u32,
    pub instance: u32,
    pub checkpoint_txg: u64,
    pub rollback_floor: u64,
    pub self_checksum_holds: bool,
    pub pointers: [PointerFact; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentFact {
    /// 这一段里录制流的写（写表下标，升序）。
    pub writes: Vec<u32>,
    /// 这一段展开出来的状态序号区间（不展开的段是空区间）。
    pub states: Range<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TornImageFact {
    pub write_index: u32,
    pub torn_image_index: u32,
    /// （重放在枚举写表里的下标，重放的是写表里哪一次写）。
    pub replays: Vec<(u32, u32)>,
}

/// 一条流的全部事实。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowFacts {
    pub recorded_write_count: u32,
    pub enumerated_write_count: u32,
    pub is_tearable: Vec<bool>,
    pub torn_images: Vec<TornImageFact>,
    pub segments: Vec<SegmentFact>,
    pub state_count: u64,
    pub units: Vec<UnitFact>,
    pub roots: Vec<RootFact>,
    /// 基线里最新的那条根（按 checkpoint_txg 取大）在 `roots` 里的下标；基线里没有根就是 None。
    pub newest_base_root: Option<u32>,
}

/// GPU 内核的持久掩码是 `array<u32, 64>`：2048 位。
pub const MAXIMUM_ENUMERATED_WRITES: usize = 2048;

/// 从一条流抽事实（第 ① 段）。`expansion` 与枚举时给的同一个。
///
/// # Panics
/// 枚举写表超过 GPU 内核掩码装得下的 [`MAXIMUM_ENUMERATED_WRITES`]；写表里有单元写不是普通写（清零写不是单元）。
#[must_use]
pub fn extract_flow_facts(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
) -> FlowFacts {
    let tables: Layer0EnumerationTables =
        layer0_enumeration_tables(base, writes, segments, expansion);
    let enumerated = &tables.enumerated_writes;
    assert!(
        enumerated.len() <= MAXIMUM_ENUMERATED_WRITES,
        "枚举写表 {} 次写，GPU 内核的掩码只装得下 {MAXIMUM_ENUMERATED_WRITES}",
        enumerated.len()
    );
    let mut units = base_units(base);
    let mut roots = base_roots(base);
    let newest_base_root = roots
        .iter()
        .enumerate()
        .max_by_key(|(_, root)| root.checkpoint_txg)
        .map(|(index, _)| u32::try_from(index).expect("根数装得进 u32"));
    for (write_index, write) in enumerated.iter().enumerate() {
        let source = u32::try_from(write_index).expect("写表下标装得进 u32");
        match write.kind {
            StepKind::UnitWrite => {
                let bytes = write.bytes().expect("单元写是普通写：清零不是单元");
                units.push(unit_fact(source, write.device.0, write.offset.0, bytes));
            }
            StepKind::RootRecordFua => {
                let bytes = write.bytes().expect("根槽写是普通写");
                roots.push(root_fact(source, bytes));
            }
            StepKind::ZeroFill
            | StepKind::JournalRecord
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => {}
        }
    }
    FlowFacts {
        recorded_write_count: u32::try_from(writes.len()).expect("写数装得进 u32"),
        enumerated_write_count: u32::try_from(enumerated.len()).expect("写数装得进 u32"),
        is_tearable: tables.is_tearable_by_write.clone(),
        torn_images: tables
            .torn_images
            .iter()
            .map(|torn| TornImageFact {
                write_index: u32::try_from(torn.write_index).expect("下标装得进 u32"),
                torn_image_index: u32::try_from(torn.torn_image_index).expect("下标装得进 u32"),
                replays: torn
                    .replays
                    .iter()
                    .map(|(replay_index, replayed)| {
                        (
                            u32::try_from(*replay_index).expect("下标装得进 u32"),
                            u32::try_from(*replayed).expect("下标装得进 u32"),
                        )
                    })
                    .collect(),
            })
            .collect(),
        segments: segments
            .iter()
            .zip(&tables.state_ranges_by_segment)
            .map(|(segment, states)| SegmentFact {
                writes: segment
                    .iter()
                    .map(|write| u32::try_from(*write).expect("下标装得进 u32"))
                    .collect(),
                states: states.clone(),
            })
            .collect(),
        state_count: tables.state_count,
        units,
        roots,
        newest_base_root,
    }
}

fn unit_fact(source: u32, device: u32, offset: u64, bytes: &[u8]) -> UnitFact {
    UnitFact {
        source,
        device,
        slot: offset / SLOT_BYTES,
        unit_bytes: u32::try_from(bytes.len()).expect("单元长度装得进 u32"),
        class: bytes.get(UNIT_CLASS_OFFSET).copied().unwrap_or(0),
        checksums_hold: check_unit(bytes).is_ok(),
        whole_crc32c: crc32_castagnoli_table(bytes),
    }
}

fn zero_pointer() -> PointerFact {
    PointerFact {
        all_zero: true,
        locations: [LocationFact {
            device: 0,
            slot: 0,
            checksum: 0,
        }; 2],
    }
}

fn root_fact(source: u32, bytes: &[u8]) -> RootFact {
    let record_bytes = usize::try_from(ROOT_RECORD_BYTES).expect("457");
    let mut filesystem_identifier = [0u8; 16];
    if bytes.len() >= 20 {
        filesystem_identifier.copy_from_slice(&bytes[4..20]);
    }
    // 自证：magic、整槽校验和、flags；fsid 拿它自己的（同一个池的判定归池级 checker，这里只判它自己站不站得住）
    let view = check_root_slot(bytes, &filesystem_identifier).ok();
    let pointers = ROOT_POINTER_OFFSETS.map(|offset| {
        if bytes.len() >= record_bytes {
            let pointer = parse_node_pointer(&bytes[offset..]);
            PointerFact {
                all_zero: pointer.all_zero,
                locations: pointer.locations.map(|location| LocationFact {
                    device: location.device,
                    slot: location.slot,
                    checksum: location.checksum,
                }),
            }
        } else {
            zero_pointer()
        }
    });
    RootFact {
        source,
        instance: view.as_ref().map_or(0, |view| view.instance),
        checkpoint_txg: view.as_ref().map_or(0, |view| view.checkpoint_txg),
        rollback_floor: view.as_ref().map_or(0, |view| view.rollback_floor),
        self_checksum_holds: view.is_some(),
        pointers,
    }
}

/// 基线镜像里的单元：写过的扇区里、16 KiB 槽起点上带单元 magic 的，按类读整单元。
fn base_units(base: &MemoryPool) -> Vec<UnitFact> {
    let mut units = Vec::new();
    for (device, sparse) in &base.devices {
        for (sector, bytes) in sparse.written_sectors() {
            let offset = sector * SECTOR_BYTES;
            if !offset.is_multiple_of(SLOT_BYTES) || bytes.len() < 8 || &bytes[..4] != UNIT_MAGIC {
                continue;
            }
            let unit_bytes = match bytes[UNIT_CLASS_OFFSET] {
                2 => NODE_BYTES,
                _ => DATA_UNIT_BYTES,
            };
            let Some(unit) = PoolReader::read(
                base,
                *device,
                DeviceOffsetInBytes(offset),
                usize::try_from(unit_bytes).expect("单元长度装得进 usize"),
            ) else {
                continue;
            };
            units.push(unit_fact(BASE_IMAGE_SOURCE, device.0, offset, &unit));
        }
    }
    units
}

/// 基线镜像里的根记录：写过的扇区里带根 magic 的（根槽写是一个物理块，层 0 的池按 512 字节建）。
fn base_roots(base: &MemoryPool) -> Vec<RootFact> {
    let mut roots = Vec::new();
    for sparse in base.devices.values() {
        for bytes in sparse.written_sectors().values() {
            if bytes.len() >= 4 && &bytes[..4] == ROOT_MAGIC {
                roots.push(root_fact(BASE_IMAGE_SOURCE, bytes));
            }
        }
    }
    roots
}

impl FlowFacts {
    /// 事实表的摘要（连同抽取器版本）：进 KV 键。
    #[must_use]
    pub fn digest(&self) -> [u8; SHA256_BYTES] {
        let mut message = FACTS_EXTRACTOR_VERSION.to_le_bytes().to_vec();
        message.extend_from_slice(&self.to_bytes());
        sha256_digest(&message)
    }

    /// 序号落在哪一段；等于段数就是最后全部持久那一个状态。
    #[must_use]
    pub fn segment_of_state(&self, ordinal: u64) -> usize {
        self.segments
            .partition_point(|segment| segment.states.end <= ordinal)
    }

    /// 这个状态的持久集合，按枚举写表给；与层 0 `Layer0StatePlan::persisted_writes_of_state` 同一套数字次序
    /// （两态的写数字 0 / 1 = 没持久 / 持久，三态的写 0 / 1 / 2 = 没持久 / 撕裂 / 持久；撕裂那一态记成原写没持久、撕裂镜像持久，
    /// 撕裂镜像之后的重放跟着被重放的那次写落不落）。
    ///
    /// # Panics
    /// 序号越过状态总数。
    #[must_use]
    pub fn persisted_writes_of_state(&self, ordinal: u64) -> Vec<bool> {
        assert!(
            ordinal < self.state_count,
            "序号 {ordinal} 越过状态总数 {}",
            self.state_count
        );
        let recorded = usize::try_from(self.recorded_write_count).expect("装得进 usize");
        let enumerated = usize::try_from(self.enumerated_write_count).expect("装得进 usize");
        // 0 没持久、1 撕裂、2 持久
        let mut landings = vec![0u8; recorded];
        let segment_of_state = self.segment_of_state(ordinal);
        for segment in &self.segments[..segment_of_state] {
            for write in &segment.writes {
                landings[*write as usize] = 2;
            }
        }
        if let Some(segment) = self.segments.get(segment_of_state) {
            let mut remaining = ordinal - segment.states.start;
            for write in &segment.writes {
                let index = *write as usize;
                let choices = if self.is_tearable[index] { 3 } else { 2 };
                let digit = remaining % choices;
                remaining /= choices;
                landings[index] = match (choices, digit) {
                    (2, 0) | (3, 0) => 0,
                    (2, 1) | (3, 2) => 2,
                    (3, 1) => 1,
                    _ => unreachable!("数字不超过态数"),
                };
            }
            assert_eq!(remaining, 0, "段内序号超出这几次写的组合数");
        }
        let mut persisted = vec![false; enumerated];
        for (index, landing) in landings.iter().enumerate() {
            persisted[index] = *landing == 2;
        }
        for torn in &self.torn_images {
            if landings[torn.write_index as usize] == 1 {
                persisted[torn.torn_image_index as usize] = true;
                for (replay_index, replayed) in &torn.replays {
                    persisted[*replay_index as usize] = landings[*replayed as usize] == 2;
                }
            }
        }
        persisted
    }

    /// 序列化：全部小端定宽，字段次序就是 [`Self::from_bytes`] 读的次序。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let put32 = |out: &mut Vec<u8>, value: u32| out.extend_from_slice(&value.to_le_bytes());
        let put64 = |out: &mut Vec<u8>, value: u64| out.extend_from_slice(&value.to_le_bytes());
        put32(&mut out, self.recorded_write_count);
        put32(&mut out, self.enumerated_write_count);
        put32(
            &mut out,
            u32::try_from(self.is_tearable.len()).expect("装得进"),
        );
        out.extend(self.is_tearable.iter().map(|tearable| u8::from(*tearable)));
        put32(
            &mut out,
            u32::try_from(self.torn_images.len()).expect("装得进"),
        );
        for torn in &self.torn_images {
            put32(&mut out, torn.write_index);
            put32(&mut out, torn.torn_image_index);
            put32(&mut out, u32::try_from(torn.replays.len()).expect("装得进"));
            for (replay_index, replayed) in &torn.replays {
                put32(&mut out, *replay_index);
                put32(&mut out, *replayed);
            }
        }
        put32(
            &mut out,
            u32::try_from(self.segments.len()).expect("装得进"),
        );
        for segment in &self.segments {
            put32(
                &mut out,
                u32::try_from(segment.writes.len()).expect("装得进"),
            );
            for write in &segment.writes {
                put32(&mut out, *write);
            }
            put64(&mut out, segment.states.start);
            put64(&mut out, segment.states.end);
        }
        put64(&mut out, self.state_count);
        put32(&mut out, u32::try_from(self.units.len()).expect("装得进"));
        for unit in &self.units {
            put32(&mut out, unit.source);
            put32(&mut out, unit.device);
            put64(&mut out, unit.slot);
            put32(&mut out, unit.unit_bytes);
            out.push(unit.class);
            out.push(u8::from(unit.checksums_hold));
            put32(&mut out, unit.whole_crc32c);
        }
        put32(&mut out, u32::try_from(self.roots.len()).expect("装得进"));
        for root in &self.roots {
            put32(&mut out, root.source);
            put32(&mut out, root.instance);
            put64(&mut out, root.checkpoint_txg);
            put64(&mut out, root.rollback_floor);
            out.push(u8::from(root.self_checksum_holds));
            for pointer in &root.pointers {
                out.push(u8::from(pointer.all_zero));
                for location in &pointer.locations {
                    put32(&mut out, location.device);
                    put64(&mut out, location.slot);
                    put32(&mut out, location.checksum);
                }
            }
        }
        match self.newest_base_root {
            Some(index) => put32(&mut out, index),
            None => put32(&mut out, u32::MAX),
        }
        out
    }

    /// 反序列化；字节不够或对不上返回 None。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut cursor = Cursor { bytes, at: 0 };
        let recorded_write_count = cursor.u32()?;
        let enumerated_write_count = cursor.u32()?;
        let tearable_count = cursor.u32()? as usize;
        let mut is_tearable = Vec::with_capacity(tearable_count);
        for _ in 0..tearable_count {
            is_tearable.push(cursor.u8()? != 0);
        }
        let torn_count = cursor.u32()? as usize;
        let mut torn_images = Vec::with_capacity(torn_count);
        for _ in 0..torn_count {
            let write_index = cursor.u32()?;
            let torn_image_index = cursor.u32()?;
            let replay_count = cursor.u32()? as usize;
            let mut replays = Vec::with_capacity(replay_count);
            for _ in 0..replay_count {
                replays.push((cursor.u32()?, cursor.u32()?));
            }
            torn_images.push(TornImageFact {
                write_index,
                torn_image_index,
                replays,
            });
        }
        let segment_count = cursor.u32()? as usize;
        let mut segments = Vec::with_capacity(segment_count);
        for _ in 0..segment_count {
            let write_count = cursor.u32()? as usize;
            let mut writes = Vec::with_capacity(write_count);
            for _ in 0..write_count {
                writes.push(cursor.u32()?);
            }
            let start = cursor.u64()?;
            let end = cursor.u64()?;
            segments.push(SegmentFact {
                writes,
                states: start..end,
            });
        }
        let state_count = cursor.u64()?;
        let unit_count = cursor.u32()? as usize;
        let mut units = Vec::with_capacity(unit_count);
        for _ in 0..unit_count {
            units.push(UnitFact {
                source: cursor.u32()?,
                device: cursor.u32()?,
                slot: cursor.u64()?,
                unit_bytes: cursor.u32()?,
                class: cursor.u8()?,
                checksums_hold: cursor.u8()? != 0,
                whole_crc32c: cursor.u32()?,
            });
        }
        let root_count = cursor.u32()? as usize;
        let mut roots = Vec::with_capacity(root_count);
        for _ in 0..root_count {
            let source = cursor.u32()?;
            let instance = cursor.u32()?;
            let checkpoint_txg = cursor.u64()?;
            let rollback_floor = cursor.u64()?;
            let self_checksum_holds = cursor.u8()? != 0;
            let mut pointers = [zero_pointer(); 4];
            for pointer in &mut pointers {
                pointer.all_zero = cursor.u8()? != 0;
                for location in &mut pointer.locations {
                    location.device = cursor.u32()?;
                    location.slot = cursor.u64()?;
                    location.checksum = cursor.u32()?;
                }
            }
            roots.push(RootFact {
                source,
                instance,
                checkpoint_txg,
                rollback_floor,
                self_checksum_holds,
                pointers,
            });
        }
        let newest_base_root = match cursor.u32()? {
            u32::MAX => None,
            index => Some(index),
        };
        if cursor.at != bytes.len() {
            return None;
        }
        Some(Self {
            recorded_write_count,
            enumerated_write_count,
            is_tearable,
            torn_images,
            segments,
            state_count,
            units,
            roots,
            newest_base_root,
        })
    }
}

struct Cursor<'bytes> {
    bytes: &'bytes [u8],
    at: usize,
}

impl Cursor<'_> {
    fn take(&mut self, length: usize) -> Option<&[u8]> {
        let slice = self.bytes.get(self.at..self.at + length)?;
        self.at += length;
        Some(slice)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|slice| slice[0])
    }
    fn u32(&mut self) -> Option<u32> {
        self.take(4)
            .map(|slice| u32::from_le_bytes(slice.try_into().expect("4 字节")))
    }
    fn u64(&mut self) -> Option<u64> {
        self.take(8)
            .map(|slice| u64::from_le_bytes(slice.try_into().expect("8 字节")))
    }
}

// ============================== 第 ② 段的 CPU 参照：只读事实判一个状态 ==============================

/// 在这个状态里，(盘, 槽) 上现在是哪个单元：持久的单元写里来源号最大的那一次；一次都没有就是基线里的（有的话）。
fn current_unit_at<'facts>(
    facts: &'facts FlowFacts,
    persisted: &[bool],
    device: u32,
    slot: u64,
) -> Option<&'facts UnitFact> {
    let rank = |unit: &UnitFact| {
        if unit.source == BASE_IMAGE_SOURCE {
            -1i64
        } else {
            i64::from(unit.source)
        }
    };
    let mut best: Option<&UnitFact> = None;
    for unit in &facts.units {
        if unit.device != device || unit.slot != slot {
            continue;
        }
        let present = unit.source == BASE_IMAGE_SOURCE || persisted[unit.source as usize];
        if !present {
            continue;
        }
        if best.is_none_or(|current| rank(unit) > rank(current)) {
            best = Some(unit);
        }
    }
    best
}

/// 只读事实判一个状态，交回 [`verdict_bits`] 的位。GPU 内核（`crash_verify_gpu`）按同一张表、同一组判据算，两份逐状态相等。
#[must_use]
pub fn verify_state_from_facts(facts: &FlowFacts, ordinal: u64) -> u32 {
    let persisted = facts.persisted_writes_of_state(ordinal);
    verify_persisted_set_from_facts(facts, &persisted)
}

/// [`verify_state_from_facts`] 拿到掩码之后的那一半（测试里直接喂掩码用）。
#[must_use]
pub fn verify_persisted_set_from_facts(facts: &FlowFacts, persisted: &[bool]) -> u32 {
    let mut bits = 0u32;
    // 最新持久的根：写表里持久的根槽写取来源号最大的；一条都没有就取基线里最新的
    let mut newest: Option<&RootFact> = None;
    for root in &facts.roots {
        if root.source == BASE_IMAGE_SOURCE || !persisted[root.source as usize] {
            continue;
        }
        if newest.is_none_or(|current| root.source > current.source) {
            newest = Some(root);
        }
    }
    if newest.is_none() {
        newest = facts
            .newest_base_root
            .and_then(|index| facts.roots.get(index as usize));
    }
    let Some(root) = newest else {
        return bits | verdict_bits::NO_ROOT_AT_ALL;
    };
    if !root.self_checksum_holds {
        return bits | verdict_bits::NEWEST_ROOT_SELF_CHECKSUM;
    }
    for pointer in &root.pointers {
        if pointer.all_zero {
            continue;
        }
        let target = pointer.locations.iter().find_map(|location| {
            current_unit_at(facts, persisted, location.device, location.slot)
                .filter(|unit| unit.whole_crc32c == location.checksum)
        });
        match target {
            None => bits |= verdict_bits::NEWEST_ROOT_POINTER_TARGET,
            Some(unit) if !unit.checksums_hold => bits |= verdict_bits::UNIT_CHECKSUMS,
            Some(_) => {}
        }
    }
    bits
}

/// 一批状态的核对位（第 ② 段按块调），按序号次序。
#[must_use]
pub fn verify_states_from_facts(facts: &FlowFacts, ordinals: Range<u64>) -> Vec<u32> {
    ordinals
        .map(|ordinal| verify_state_from_facts(facts, ordinal))
        .collect()
}

/// （盘，槽）→ 落在它上面的单元来源，测试与统计用。
#[must_use]
pub fn units_by_placement(facts: &FlowFacts) -> BTreeMap<(u32, u64), Vec<u32>> {
    let mut by_placement: BTreeMap<(u32, u64), Vec<u32>> = BTreeMap::new();
    for unit in &facts.units {
        by_placement
            .entry((unit.device, unit.slot))
            .or_default()
            .push(unit.source);
    }
    by_placement
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer_to(device: u32, slot: u64, checksum: u32) -> PointerFact {
        PointerFact {
            all_zero: false,
            locations: [
                LocationFact {
                    device,
                    slot,
                    checksum,
                },
                LocationFact {
                    device: device ^ 1,
                    slot,
                    checksum,
                },
            ],
        }
    }

    #[test]
    fn facts_round_trip_through_bytes() {
        let facts = FlowFacts {
            recorded_write_count: 3,
            enumerated_write_count: 4,
            is_tearable: vec![false, true, false],
            torn_images: vec![TornImageFact {
                write_index: 1,
                torn_image_index: 3,
                replays: vec![(3, 2)],
            }],
            segments: vec![
                SegmentFact {
                    writes: vec![0, 1],
                    states: 0..5,
                },
                SegmentFact {
                    writes: vec![2],
                    states: 5..6,
                },
            ],
            state_count: 7,
            units: vec![UnitFact {
                source: 0,
                device: 1,
                slot: 50176,
                unit_bytes: 32768,
                class: 1,
                checksums_hold: true,
                whole_crc32c: 0xdead_beef,
            }],
            roots: vec![RootFact {
                source: 2,
                instance: 1,
                checkpoint_txg: 3,
                rollback_floor: 0,
                self_checksum_holds: true,
                pointers: [pointer_to(1, 50176, 0xdead_beef); 4],
            }],
            newest_base_root: None,
        };
        let bytes = facts.to_bytes();
        assert_eq!(FlowFacts::from_bytes(&bytes), Some(facts.clone()));
        assert_eq!(
            FlowFacts::from_bytes(&bytes[..bytes.len() - 1]),
            None,
            "少一个字节认不回"
        );
        assert_ne!(facts.digest(), [0u8; SHA256_BYTES]);
    }

    #[test]
    fn the_state_ordinal_decodes_to_the_same_persisted_set_as_layer0_mixed_radix() {
        // 段 0：写 0（两态）、写 1（三态）：序号 r 拆成 写0 取 r % 2、写1 取 (r / 2) % 3；共 6 个组合、去掉全持久那一个 → 5 个状态
        let facts = FlowFacts {
            recorded_write_count: 3,
            enumerated_write_count: 4,
            is_tearable: vec![false, true, false],
            torn_images: vec![TornImageFact {
                write_index: 1,
                torn_image_index: 3,
                replays: vec![],
            }],
            segments: vec![
                SegmentFact {
                    writes: vec![0, 1],
                    states: 0..5,
                },
                SegmentFact {
                    writes: vec![2],
                    states: 5..6,
                },
            ],
            state_count: 7,
            units: vec![],
            roots: vec![],
            newest_base_root: None,
        };
        assert_eq!(
            facts.persisted_writes_of_state(0),
            vec![false, false, false, false]
        );
        assert_eq!(
            facts.persisted_writes_of_state(1),
            vec![true, false, false, false]
        );
        assert_eq!(
            facts.persisted_writes_of_state(2),
            vec![false, false, false, true],
            "写 1 取撕裂：原写没持久、撕裂镜像（枚举表下标 3）持久"
        );
        assert_eq!(
            facts.persisted_writes_of_state(4),
            vec![false, true, false, false]
        );
        assert_eq!(
            facts.persisted_writes_of_state(5),
            vec![true, true, false, false],
            "段 1 的第一个状态：段 0 整段持久、写 2 没持久"
        );
        assert_eq!(
            facts.persisted_writes_of_state(6),
            vec![true, true, true, false],
            "最后全部持久"
        );
    }

    #[test]
    fn a_root_whose_pointer_targets_a_missing_unit_is_red_and_becomes_green_once_the_unit_lands() {
        let unit = UnitFact {
            source: 0,
            device: 0,
            slot: 7,
            unit_bytes: 16384,
            class: 2,
            checksums_hold: true,
            whole_crc32c: 42,
        };
        let root = RootFact {
            source: 1,
            instance: 1,
            checkpoint_txg: 1,
            rollback_floor: 0,
            self_checksum_holds: true,
            pointers: [
                pointer_to(0, 7, 42),
                zero_pointer(),
                zero_pointer(),
                zero_pointer(),
            ],
        };
        let facts = FlowFacts {
            recorded_write_count: 2,
            enumerated_write_count: 2,
            is_tearable: vec![false, false],
            torn_images: vec![],
            segments: vec![SegmentFact {
                writes: vec![0, 1],
                states: 0..3,
            }],
            state_count: 4,
            units: vec![unit],
            roots: vec![root],
            newest_base_root: None,
        };
        assert_eq!(
            verify_persisted_set_from_facts(&facts, &[false, true]),
            verdict_bits::NEWEST_ROOT_POINTER_TARGET,
            "根落了、它指的单元没落"
        );
        assert_eq!(verify_persisted_set_from_facts(&facts, &[true, true]), 0);
        assert_eq!(
            verify_persisted_set_from_facts(&facts, &[true, false]),
            verdict_bits::NO_ROOT_AT_ALL,
            "没有根、基线里也没有"
        );
        let mut bad_unit = facts.clone();
        bad_unit.units[0].checksums_hold = false;
        assert_eq!(
            verify_persisted_set_from_facts(&bad_unit, &[true, true]),
            verdict_bits::UNIT_CHECKSUMS
        );
        let mut mismatch = facts.clone();
        mismatch.units[0].whole_crc32c = 43;
        assert_eq!(
            verify_persisted_set_from_facts(&mismatch, &[true, true]),
            verdict_bits::NEWEST_ROOT_POINTER_TARGET,
            "指针里的校验和与单元的整单元 CRC 对不上"
        );
    }
}
