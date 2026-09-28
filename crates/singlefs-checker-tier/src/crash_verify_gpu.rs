//! 崩溃放量三段流的第 ② 段在 GPU 上：只读第 ① 段的事实表（`crate::crash_facts::FlowFacts`），一次派活核一整块状态——每个调用判一个状态：
//! 按序号现算持久掩码（与层 0 同一套混合进制）、找最新持久的根、判它自证与四条指针的落点、判持久的单元写两道校验和都对。
//! 判据与 `crate::crash_facts::verify_state_from_facts` 逐条相同，两份逐状态相等是这一份的判别力自证；结论按位交回
//! （`crate::crash_facts::verdict_bits`），第 ③ 段再与 CPU 判器比。
//!
//! 事实表按流上卡一次（[`GpuFactsVerifier::new`]），之后每块只送一个头（块起点、块里几个状态）、读回每状态一个 u32；
//! 不再逐状态建缓冲、提交、等——那一版（`gpu_unit_checks` 逐状态算校验和）比 CPU 慢 18 倍，慢的是派活不是算。
//! 核对代码的摘要 [`gpu_verifier_digest`] 进核对表的键：着色器改一个字，旧行留着当历史，新行另起。
use std::ops::Range;

use singlefs_harness::sha256::sha256_digest;

use crate::crash_facts::{FlowFacts, BASE_IMAGE_SOURCE, MAXIMUM_ENUMERATED_WRITES};
use crate::crash_identity::SHA256_BYTES;
use crate::gpu_unit_checks::GpuCard;

/// 每个工作组 64 个状态。
const WORKGROUP_SIZE: usize = 64;
/// 每条录制流的写在 `recorded` 表里占几个 u32：取几态、撕裂镜像下标、重放起点、重放条数。
const RECORDED_WORDS: usize = 4;
/// 每一段在 `segments` 表里占几个 u32：写的起点、写的条数、状态起点 lo/hi、状态终点 lo/hi。
const SEGMENT_WORDS: usize = 6;
/// 每个单元在 `units` 表里占几个 u32：来源、盘、槽 lo/hi、校验和都对、整单元 CRC。
const UNIT_WORDS: usize = 6;
/// 每条根在 `roots` 表里占几个 u32：来源、自证过、四条指针各（全零、两份位置条目各 盘、槽 lo/hi、校验和）。
const ROOT_WORDS: usize = 2 + 4 * (1 + 2 * 4);
const HEADER_WORDS: usize = 12;
const NONE: u32 = u32::MAX;

pub const GPU_VERIFIER_SHADER_SOURCE: &str = r#"
struct Header {
    recorded_write_count: u32,
    enumerated_write_count: u32,
    segment_count: u32,
    unit_count: u32,
    root_count: u32,
    newest_base_root: u32,
    block_start_lo: u32,
    block_start_hi: u32,
    state_count: u32,
    padding0: u32,
    padding1: u32,
    padding2: u32,
};

@group(0) @binding(0) var<uniform> header: Header;
@group(0) @binding(1) var<storage, read> recorded: array<u32>;
@group(0) @binding(2) var<storage, read> replays: array<u32>;
@group(0) @binding(3) var<storage, read> segments: array<u32>;
@group(0) @binding(4) var<storage, read> segment_writes: array<u32>;
@group(0) @binding(5) var<storage, read> units: array<u32>;
@group(0) @binding(6) var<storage, read> roots: array<u32>;
@group(0) @binding(7) var<storage, read_write> verdicts: array<u32>;

const NONE: u32 = 0xFFFFFFFFu;
const BIT_UNIT_CHECKSUMS: u32 = 1u;
const BIT_NEWEST_ROOT_SELF_CHECKSUM: u32 = 2u;
const BIT_NEWEST_ROOT_POINTER_TARGET: u32 = 4u;
const BIT_NO_ROOT_AT_ALL: u32 = 8u;

var<private> persisted: array<u32, 64>;
var<private> torn: array<u32, 64>;

fn set_persisted(index: u32) {
    persisted[index >> 5u] = persisted[index >> 5u] | (1u << (index & 31u));
}
fn clear_persisted(index: u32) {
    persisted[index >> 5u] = persisted[index >> 5u] & ~(1u << (index & 31u));
}
fn is_persisted(index: u32) -> bool {
    return (persisted[index >> 5u] & (1u << (index & 31u))) != 0u;
}
fn set_torn(index: u32) {
    torn[index >> 5u] = torn[index >> 5u] | (1u << (index & 31u));
}
fn is_torn(index: u32) -> bool {
    return (torn[index >> 5u] & (1u << (index & 31u))) != 0u;
}
fn less_u64(a_lo: u32, a_hi: u32, b_lo: u32, b_hi: u32) -> bool {
    return a_hi < b_hi || (a_hi == b_hi && a_lo < b_lo);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= header.state_count) {
        return;
    }
    var ordinal_lo = header.block_start_lo + local;
    var ordinal_hi = header.block_start_hi;
    if (ordinal_lo < header.block_start_lo) {
        ordinal_hi = ordinal_hi + 1u;
    }
    for (var word = 0u; word < 64u; word = word + 1u) {
        persisted[word] = 0u;
        torn[word] = 0u;
    }
    // 序号落在哪一段：第一段终点大于序号的；都不大于就是最后全部持久那一个状态
    var segment_of_state = header.segment_count;
    for (var s = 0u; s < header.segment_count; s = s + 1u) {
        let end_lo = segments[s * 6u + 4u];
        let end_hi = segments[s * 6u + 5u];
        if (less_u64(ordinal_lo, ordinal_hi, end_lo, end_hi)) {
            segment_of_state = s;
            break;
        }
    }
    for (var s = 0u; s < segment_of_state; s = s + 1u) {
        let first = segments[s * 6u];
        let count = segments[s * 6u + 1u];
        for (var k = 0u; k < count; k = k + 1u) {
            set_persisted(segment_writes[first + k]);
        }
    }
    if (segment_of_state < header.segment_count) {
        let s = segment_of_state;
        // 段内序号：CPU 保证每段的状态数装得进 u32，低 32 位相减（回绕）就是它
        var remaining = ordinal_lo - segments[s * 6u + 2u];
        let first = segments[s * 6u];
        let count = segments[s * 6u + 1u];
        for (var k = 0u; k < count; k = k + 1u) {
            let write = segment_writes[first + k];
            let choices = select(2u, 3u, recorded[write * 4u] != 0u);
            let digit = remaining % choices;
            remaining = remaining / choices;
            if (choices == 2u) {
                if (digit == 1u) {
                    set_persisted(write);
                }
            } else {
                if (digit == 2u) {
                    set_persisted(write);
                } else if (digit == 1u) {
                    set_torn(write);
                }
            }
        }
    }
    // 撕裂那一态：原写没持久、它的撕裂镜像持久；撕裂镜像之后的重放跟着被重放的那次写落不落
    for (var write = 0u; write < header.recorded_write_count; write = write + 1u) {
        if (!is_torn(write)) {
            continue;
        }
        set_persisted(recorded[write * 4u + 1u]);
        let replay_first = recorded[write * 4u + 2u];
        let replay_count = recorded[write * 4u + 3u];
        for (var r = 0u; r < replay_count; r = r + 1u) {
            let replay_index = replays[(replay_first + r) * 2u];
            let replayed = replays[(replay_first + r) * 2u + 1u];
            if (is_persisted(replayed)) {
                set_persisted(replay_index);
            } else {
                clear_persisted(replay_index);
            }
        }
    }
    var bits = 0u;
    var newest = NONE;
    var newest_source = 0u;
    var found = false;
    for (var r = 0u; r < header.root_count; r = r + 1u) {
        let source = roots[r * 38u];
        if (source != NONE && is_persisted(source)) {
            if (!found || source > newest_source) {
                found = true;
                newest = r;
                newest_source = source;
            }
        }
    }
    if (!found) {
        newest = header.newest_base_root;
    }
    if (newest == NONE) {
        verdicts[local] = bits | BIT_NO_ROOT_AT_ALL;
        return;
    }
    if (roots[newest * 38u + 1u] == 0u) {
        verdicts[local] = bits | BIT_NEWEST_ROOT_SELF_CHECKSUM;
        return;
    }
    for (var p = 0u; p < 4u; p = p + 1u) {
        let pointer = newest * 38u + 2u + p * 9u;
        if (roots[pointer] != 0u) {
            continue;
        }
        var reachable = false;
        for (var l = 0u; l < 2u; l = l + 1u) {
            if (reachable) {
                break;
            }
            let location = pointer + 1u + l * 4u;
            let device = roots[location];
            let slot_lo = roots[location + 1u];
            let slot_hi = roots[location + 2u];
            let checksum = roots[location + 3u];
            var best_found = false;
            var best_rank = 0u;
            var best_crc = 0u;
            var best_checksums_hold = 0u;
            for (var u = 0u; u < header.unit_count; u = u + 1u) {
                if (units[u * 6u + 1u] != device || units[u * 6u + 2u] != slot_lo || units[u * 6u + 3u] != slot_hi) {
                    continue;
                }
                let source = units[u * 6u];
                var present = false;
                var rank = 0u;
                if (source == NONE) {
                    present = true;
                    rank = 0u;
                } else if (is_persisted(source)) {
                    present = true;
                    rank = source + 1u;
                }
                if (present && (!best_found || rank > best_rank)) {
                    best_found = true;
                    best_rank = rank;
                    best_crc = units[u * 6u + 5u];
                    best_checksums_hold = units[u * 6u + 4u];
                }
            }
            if (best_found && best_crc == checksum) {
                reachable = true;
                if (best_checksums_hold == 0u) {
                    bits = bits | BIT_UNIT_CHECKSUMS;
                }
            }
        }
        if (!reachable) {
            bits = bits | BIT_NEWEST_ROOT_POINTER_TARGET;
        }
    }
    verdicts[local] = bits;
}
"#;

/// GPU 核对代码的摘要：进核对表的键。
#[must_use]
pub fn gpu_verifier_digest() -> [u8; SHA256_BYTES] {
    let mut message = b"gpu-facts-verifier-v2\n".to_vec();
    message.extend_from_slice(GPU_VERIFIER_SHADER_SOURCE.as_bytes());
    sha256_digest(&message)
}

fn words_to_bytes(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn split_u64(value: u64) -> (u32, u32) {
    (
        u32::try_from(value & 0xFFFF_FFFF).expect("低 32 位"),
        u32::try_from(value >> 32).expect("高 32 位"),
    )
}

/// 一条流的事实表在一张卡上的样子：上卡一次，每块只送头、读回结论。
pub struct GpuFactsVerifier<'card> {
    card: &'card GpuCard,
    pipeline: wgpu::ComputePipeline,
    header_words: [u32; HEADER_WORDS],
    recorded: wgpu::Buffer,
    replays: wgpu::Buffer,
    segments: wgpu::Buffer,
    segment_writes: wgpu::Buffer,
    units: wgpu::Buffer,
    roots: wgpu::Buffer,
}

impl<'card> GpuFactsVerifier<'card> {
    /// 把事实表铺成几张 u32 表上卡。
    ///
    /// # Panics
    /// 枚举写表超过掩码装得下的数；某一段展开出来的状态数装不进 u32（内核按低 32 位算段内序号）。
    #[must_use]
    pub fn new(card: &'card GpuCard, facts: &FlowFacts) -> Self {
        assert!(
            usize::try_from(facts.enumerated_write_count).expect("装得进")
                <= MAXIMUM_ENUMERATED_WRITES,
            "枚举写表 {} 次写，掩码只装得下 {MAXIMUM_ENUMERATED_WRITES}",
            facts.enumerated_write_count
        );
        let mut recorded = vec![0u32; facts.is_tearable.len() * RECORDED_WORDS];
        for (index, tearable) in facts.is_tearable.iter().enumerate() {
            recorded[index * RECORDED_WORDS] = u32::from(*tearable);
            recorded[index * RECORDED_WORDS + 1] = NONE;
        }
        let mut replays: Vec<u32> = Vec::new();
        for torn in &facts.torn_images {
            let base = usize::try_from(torn.write_index).expect("装得进") * RECORDED_WORDS;
            recorded[base + 1] = torn.torn_image_index;
            recorded[base + 2] = u32::try_from(replays.len() / 2).expect("装得进");
            recorded[base + 3] = u32::try_from(torn.replays.len()).expect("装得进");
            for (replay_index, replayed) in &torn.replays {
                replays.push(*replay_index);
                replays.push(*replayed);
            }
        }
        let mut segments = Vec::with_capacity(facts.segments.len() * SEGMENT_WORDS);
        let mut segment_writes = Vec::new();
        for segment in &facts.segments {
            assert!(
                segment.states.end - segment.states.start < u64::from(u32::MAX),
                "一段展开出 {} 个状态，段内序号装不进 u32",
                segment.states.end - segment.states.start
            );
            let (start_lo, start_hi) = split_u64(segment.states.start);
            let (end_lo, end_hi) = split_u64(segment.states.end);
            segments.extend_from_slice(&[
                u32::try_from(segment_writes.len()).expect("装得进"),
                u32::try_from(segment.writes.len()).expect("装得进"),
                start_lo,
                start_hi,
                end_lo,
                end_hi,
            ]);
            segment_writes.extend_from_slice(&segment.writes);
        }
        let mut units = Vec::with_capacity(facts.units.len() * UNIT_WORDS);
        for unit in &facts.units {
            let (slot_lo, slot_hi) = split_u64(unit.slot);
            units.extend_from_slice(&[
                unit.source,
                unit.device,
                slot_lo,
                slot_hi,
                u32::from(unit.checksums_hold),
                unit.whole_crc32c,
            ]);
        }
        let mut roots = Vec::with_capacity(facts.roots.len() * ROOT_WORDS);
        for root in &facts.roots {
            roots.push(root.source);
            roots.push(u32::from(root.self_checksum_holds));
            for pointer in &root.pointers {
                roots.push(u32::from(pointer.all_zero));
                for location in &pointer.locations {
                    let (slot_lo, slot_hi) = split_u64(location.slot);
                    roots.extend_from_slice(&[
                        location.device,
                        slot_lo,
                        slot_hi,
                        location.checksum,
                    ]);
                }
            }
        }
        debug_assert_eq!(roots.len(), facts.roots.len() * ROOT_WORDS);
        let header_words = [
            facts.recorded_write_count,
            facts.enumerated_write_count,
            u32::try_from(facts.segments.len()).expect("装得进"),
            u32::try_from(facts.units.len()).expect("装得进"),
            u32::try_from(facts.roots.len()).expect("装得进"),
            facts.newest_base_root.unwrap_or(NONE),
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        let device = card.device();
        let storage = |words: &[u32]| {
            // 空表也要一个合法的缓冲区：给一个字
            let bytes = if words.is_empty() {
                vec![0u8; 4]
            } else {
                words_to_bytes(words)
            };
            wgpu::util::DeviceExt::create_buffer_init(
                device,
                &wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &bytes,
                    usage: wgpu::BufferUsages::STORAGE,
                },
            )
        };
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("crash-facts-verifier"),
            source: wgpu::ShaderSource::Wgsl(GPU_VERIFIER_SHADER_SOURCE.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("crash-facts-verifier"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self {
            card,
            pipeline,
            header_words,
            recorded: storage(&recorded),
            replays: storage(&replays),
            segments: storage(&segments),
            segment_writes: storage(&segment_writes),
            units: storage(&units),
            roots: storage(&roots),
        }
    }

    /// 核一段连续的状态（一块），按序号次序交回每个状态的结论位。
    ///
    /// # Panics
    /// 一块超过 u32 个状态；回读的长度对不上。
    #[must_use]
    pub fn verify_states(&self, ordinals: Range<u64>) -> Vec<u32> {
        let count =
            usize::try_from(ordinals.end - ordinals.start).expect("一块的状态数装得进 usize");
        if count == 0 {
            return Vec::new();
        }
        let (start_lo, start_hi) = split_u64(ordinals.start);
        let mut header = self.header_words;
        header[6] = start_lo;
        header[7] = start_hi;
        header[8] = u32::try_from(count).expect("一块的状态数装得进 u32");
        let device = self.card.device();
        let header_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &words_to_bytes(&header),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let result_bytes = u64::try_from(count * 4).expect("装得进 u64");
        let verdicts = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &vec![0xFFu8; count * 4],
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            },
        );
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: result_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = self.pipeline.get_bind_group_layout(0);
        fn entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            }
        }
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                entry(0, &header_buffer),
                entry(1, &self.recorded),
                entry(2, &self.replays),
                entry(3, &self.segments),
                entry(4, &self.segment_writes),
                entry(5, &self.units),
                entry(6, &self.roots),
                entry(7, &verdicts),
            ],
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                u32::try_from(count.div_ceil(WORKGROUP_SIZE)).expect("工作组数装得进 u32"),
                1,
                1,
            );
        }
        encoder.copy_buffer_to_buffer(&verdicts, 0, &staging, 0, result_bytes);
        self.card.queue().submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).expect("回调送得出去")
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("等 GPU 算完");
        receiver.recv().expect("回调送达").expect("回读映射成功");
        let mapped = slice.get_mapped_range().expect("读得到回读缓冲区");
        let (entries, remainder) = mapped.as_chunks::<4>();
        assert!(remainder.is_empty(), "回读缓冲区的长度是 4 的整数倍");
        assert_eq!(entries.len(), count, "回读的状态数与派的相同");
        let bits: Vec<u32> = entries
            .iter()
            .map(|chunk| u32::from_le_bytes(*chunk))
            .collect();
        drop(mapped);
        staging.unmap();
        assert!(
            bits.iter().all(|value| *value != u32::MAX),
            "有状态没被内核写到（结论仍是初值）"
        );
        bits
    }
}

/// 基线来源号在两侧是同一个数。
const _: () = assert!(BASE_IMAGE_SOURCE == u32::MAX);
