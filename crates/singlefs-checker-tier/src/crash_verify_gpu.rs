//! 崩溃放量三段流的第 ② 段在 GPU 上：只读第 ① 段的事实表（`crate::crash_facts::FlowFacts`），一次派活核一整块状态——每个调用判一个状态：
//! 按序号现算持久掩码（与崩溃点重放同一套混合进制）、找最新持久的根、判它自证与四条指针的落点、判持久的单元写两道校验和都对。
//! 判据与 `crate::crash_facts::verify_state_from_facts` 逐条相同，两份逐状态相等是这一份的判别力自证；结论按位交回
//! （`crate::crash_facts::verdict_bits`），第 ③ 段再与 CPU 判器比。
//!
//! 事实表按流上卡一次（[`GpuFactsVerifier::new`]），之后每块只送一个头（块起点、块里几个状态）、读回每状态一个 u32；
//! 不再逐状态建缓冲、提交、等——那一版（`gpu_unit_checks` 逐状态算校验和）比 CPU 慢 18 倍，慢的是派活不是算。
//! 核对代码的摘要 [`gpu_verifier_digest`] 进核对表的键：着色器改一个字，旧行留着当历史，新行另起。
//! 段内序号按段的展开方式拆（全量、甲二；判决 `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md` 第二节 L6-2），
//! 满 64 位（同一判决 L1-7）：与 `crate::crash_facts::persisted_writes_of_state_by_the_segment_expansions` 同一套数字次序。
//! 分片的段（C790）：段内编号 = 片起点 + 片内序号，片起点拆成的各位随段写表上卡，内核从那几位起加片内序号、逐位进位（只用 u32，不做宽整数除法）。
use std::ops::Range;

use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash_facts::{
    check_the_slice_starts_against_the_expansions, segment_expansion_code_wgsl_constants,
    verdict_bits, SegmentExpansionFact, BASE_IMAGE_SOURCE, MAXIMUM_ENUMERATED_WRITES,
};
use crate::gpu_unit_checks::GpuCard;

/// 每个工作组 64 个状态。
const WORKGROUP_SIZE: usize = 64;
/// 每条录制流的写在 `recorded` 表里占几个 u32：取几态、撕裂镜像下标、重放起点、重放条数。
/// 这几个表宽公开给崩溃放量记显存账（`crate::crash_amplification` 的 `facts_verifier_buffer_footprint`，今天在那边抄了一份）。
pub const FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE: usize = 4;
/// 每一段在 `segments` 表里占几个 u32：数位那一组写的起点与条数、状态起点 lo/hi、状态终点 lo/hi、展开方式码
/// （[`SegmentExpansionFact::code`]）、整组落的单元写那一组的起点与条数（只有甲二段有，别的段 0 条；两组合起来是段里全部的写）、
/// 分片的段（C790）片起点各位在 `segment_writes` 里的起点与位数（不分片的段是 NONE 与 0；各位一位一个字，接在段写表里）。
pub const FACTS_VERIFIER_WORDS_PER_SEGMENT: usize = 11;
/// 每个单元在 `units` 表里占几个 u32：来源、盘、槽 lo/hi、校验和都对、整单元 CRC。
pub const FACTS_VERIFIER_WORDS_PER_UNIT: usize = 6;
/// 每条根在 `roots` 表里占几个 u32：来源、自证过、四条指针各（全零、两份位置条目各 盘、槽 lo/hi、校验和）。
pub const FACTS_VERIFIER_WORDS_PER_ROOT: usize = 2 + 4 * (1 + 2 * 4);
pub const FACTS_VERIFIER_HEADER_WORDS: usize = 12;
const NONE: u32 = u32::MAX;

/// 核对内核的本体；整份源码是 [`verifier_shader_source`]（段表宽与展开方式码的常量接在它前面）。
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
// 分片的段片起点的各位加片内序号越过了段的域（或片内序号装不进 32 位）：main 记 BIT_SLICE_PAST_ITS_DOMAIN
var<private> slice_past_its_domain: bool;

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
struct QuotientAndRemainder {
    quotient: vec2<u32>,
    remainder: u32,
};
// 64 位的数（低、高 32 位）除以一个小于 65536 的除数：按 32、16、16 位三段做长除法，每段的被除数装得进 u32
// （上一段的余数小于除数、左移 16 位不越过 32 位）。段内序号满 64 位时照它拆（判决 m3-crash-enumeration-merge-r2 第二节 L1-7）
fn divide_by_a_divisor_below_65536(value: vec2<u32>, divisor: u32) -> QuotientAndRemainder {
    var out: QuotientAndRemainder;
    let quotient_high = value.y / divisor;
    let upper = ((value.y % divisor) << 16u) | (value.x >> 16u);
    let lower = ((upper % divisor) << 16u) | (value.x & 0xFFFFu);
    out.quotient = vec2<u32>(((upper / divisor) << 16u) | (lower / divisor), quotient_high);
    out.remainder = lower % divisor;
    return out;
}

// 块里第 local 个状态的持久掩码拆进 persisted（撕裂态先记进 torn，再按撕裂镜像与重放落定）：main 与对拍用的探针入口共用这一份
fn decode_the_persisted_set_of_the_state(local: u32) {
    var ordinal_lo = header.block_start_lo + local;
    var ordinal_hi = header.block_start_hi;
    if (ordinal_lo < header.block_start_lo) {
        ordinal_hi = ordinal_hi + 1u;
    }
    for (var word = 0u; word < 64u; word = word + 1u) {
        persisted[word] = 0u;
        torn[word] = 0u;
    }
    slice_past_its_domain = false;
    // 序号落在哪一段：第一段终点大于序号的；都不大于就是最后全部持久那一个状态
    var segment_of_state = header.segment_count;
    for (var s = 0u; s < header.segment_count; s = s + 1u) {
        let end_lo = segments[s * SEGMENT_WORDS + 4u];
        let end_hi = segments[s * SEGMENT_WORDS + 5u];
        if (less_u64(ordinal_lo, ordinal_hi, end_lo, end_hi)) {
            segment_of_state = s;
            break;
        }
    }
    // 之前的段整段持久：数位那一组与整组落的单元写那一组合起来就是段里全部的写
    for (var s = 0u; s < segment_of_state; s = s + 1u) {
        let digit_first = segments[s * SEGMENT_WORDS];
        let digit_count = segments[s * SEGMENT_WORDS + 1u];
        for (var k = 0u; k < digit_count; k = k + 1u) {
            set_persisted(segment_writes[digit_first + k]);
        }
        let group_first = segments[s * SEGMENT_WORDS + 7u];
        let group_count = segments[s * SEGMENT_WORDS + 8u];
        for (var k = 0u; k < group_count; k = k + 1u) {
            set_persisted(segment_writes[group_first + k]);
        }
    }
    if (segment_of_state < header.segment_count) {
        let s = segment_of_state;
        let expansion = segments[s * SEGMENT_WORDS + 6u];
        // 段内序号满 64 位：序号减段的起点（借位进高 32 位）
        let start_lo = segments[s * SEGMENT_WORDS + 2u];
        let start_hi = segments[s * SEGMENT_WORDS + 3u];
        var borrow = 0u;
        if (ordinal_lo < start_lo) {
            borrow = 1u;
        }
        var remaining = vec2<u32>(ordinal_lo - start_lo, ordinal_hi - start_hi - borrow);
        // 不展开的段序号区间是空的，落不到这里；落到了就不拆（CPU 参照在那里断言）
        if (expansion != SEGMENT_EXPANSION_CODE_NOT_EXPANDED) {
            let digit_first = segments[s * SEGMENT_WORDS];
            let digit_count = segments[s * SEGMENT_WORDS + 1u];
            let group_first = segments[s * SEGMENT_WORDS + 7u];
            let group_count = segments[s * SEGMENT_WORDS + 8u];
            let slice_digit_first = segments[s * SEGMENT_WORDS + 9u];
            let slice_digit_count = segments[s * SEGMENT_WORDS + 10u];
            if (slice_digit_count > 0u) {
                // 分片的段：片起点的各位（单元写那一组那一位在前）加片内序号、逐位进位（crash::add_to_the_mixed_radix_digits 同一个算法）
                if (remaining.y != 0u) {
                    slice_past_its_domain = true;
                } else {
                    var carry = remaining.x;
                    var position = 0u;
                    if (group_count > 0u) {
                        let sum = segment_writes[slice_digit_first] + carry % 2u;
                        carry = carry / 2u + sum / 2u;
                        if (sum % 2u == 1u) {
                            for (var k = 0u; k < group_count; k = k + 1u) {
                                set_persisted(segment_writes[group_first + k]);
                            }
                        }
                        position = 1u;
                    }
                    for (var k = 0u; k < digit_count; k = k + 1u) {
                        let write = segment_writes[digit_first + k];
                        let choices = select(2u, 3u, recorded[write * 4u] != 0u);
                        let sum = segment_writes[slice_digit_first + position + k] + carry % choices;
                        carry = carry / choices + sum / choices;
                        let digit = sum % choices;
                        if (digit == choices - 1u) {
                            set_persisted(write);
                        } else if (digit == 1u) {
                            set_torn(write);
                        }
                    }
                    if (carry != 0u) {
                        slice_past_its_domain = true;
                    }
                }
            } else {
                // 甲二有单元写：最低一位是「单元写全落」，其余拆到原地写上
                if (group_count > 0u) {
                    if ((remaining.x & 1u) == 1u) {
                        for (var k = 0u; k < group_count; k = k + 1u) {
                            set_persisted(segment_writes[group_first + k]);
                        }
                    }
                    remaining = vec2<u32>((remaining.x >> 1u) | (remaining.y << 31u), remaining.y >> 1u);
                }
                for (var k = 0u; k < digit_count; k = k + 1u) {
                    let write = segment_writes[digit_first + k];
                    let choices = select(2u, 3u, recorded[write * 4u] != 0u);
                    let divided = divide_by_a_divisor_below_65536(remaining, choices);
                    let digit = divided.remainder;
                    remaining = divided.quotient;
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
}

// 对拍用的探针入口（`GpuFactsVerifier::persisted_masks_of_states`）：每个状态把拆出的持久掩码 64 个字写进结论缓冲
@compute @workgroup_size(64)
fn probe_persisted_masks(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= header.state_count) {
        return;
    }
    decode_the_persisted_set_of_the_state(local);
    for (var word = 0u; word < 64u; word = word + 1u) {
        verdicts[local * 64u + word] = persisted[word];
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= header.state_count) {
        return;
    }
    decode_the_persisted_set_of_the_state(local);
    if (slice_past_its_domain) {
        verdicts[local] = BIT_SLICE_PAST_ITS_DOMAIN;
        return;
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

/// 核对内核的整份源码：段表宽与段的展开方式码（[`segment_expansion_code_wgsl_constants`]）从 Rust 生成、接在本体前面，不在 WGSL 里另写一份。
#[must_use]
pub fn verifier_shader_source() -> String {
    format!(
        "const SEGMENT_WORDS: u32 = {FACTS_VERIFIER_WORDS_PER_SEGMENT}u;\nconst BIT_SLICE_PAST_ITS_DOMAIN: u32 = {}u;\n{}\n{GPU_VERIFIER_SHADER_SOURCE}",
        verdict_bits::SLICE_PAST_ITS_DOMAIN,
        segment_expansion_code_wgsl_constants()
    )
}

/// GPU 核对代码的摘要：进核对表的键。
#[must_use]
pub fn gpu_verifier_digest() -> [u8; DIGEST_BYTES] {
    let mut message = b"gpu-facts-verifier-v4\n".to_vec();
    message.extend_from_slice(verifier_shader_source().as_bytes());
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
    header_words: [u32; FACTS_VERIFIER_HEADER_WORDS],
    recorded: wgpu::Buffer,
    replays: wgpu::Buffer,
    segments: wgpu::Buffer,
    segment_writes: wgpu::Buffer,
    units: wgpu::Buffer,
    roots: wgpu::Buffer,
}

impl<'card> GpuFactsVerifier<'card> {
    /// 把事实表铺成几张 u32 表上卡，每一段按全量拆（同 `crate::crash_facts::verify_states_from_facts`：每段要么全量展开、要么不展开的流）。
    ///
    /// # Panics
    /// 同 [`Self::new_by_the_segment_expansions`]。
    #[must_use]
    pub fn new(card: &'card GpuCard, facts: &crate::crash_facts::FlowFactsReadBack) -> Self {
        Self::new_by_the_segment_expansions(
            card,
            facts,
            &vec![SegmentExpansionFact::EveryProperSubset; facts.segments.len()],
        )
    }

    /// 同 [`Self::new`]，每一段照 `segment_expansions` 那一段的拆法拆序号（有甲二段的流；CPU 参照是
    /// `crate::crash_facts::verify_states_from_facts_by_the_segment_expansions`）。
    ///
    /// # Panics
    /// 枚举写表超过掩码装得下的数；`segment_expansions` 与段数不一样长；展开了的不分片的段按它的拆法展开出的状态数与它的序号区间对不上
    /// （拆法给错了）；分片的段片起点与它的各位对不上、或片越过了段的域（`crate::crash_facts::check_the_slice_starts_against_the_expansions`）。
    #[must_use]
    pub fn new_by_the_segment_expansions(
        card: &'card GpuCard,
        facts: &crate::crash_facts::FlowFactsReadBack,
        segment_expansions: &[SegmentExpansionFact],
    ) -> Self {
        assert!(
            usize::try_from(facts.enumerated_write_count).expect("装得进")
                <= MAXIMUM_ENUMERATED_WRITES,
            "枚举写表 {} 次写，掩码只装得下 {MAXIMUM_ENUMERATED_WRITES}",
            facts.enumerated_write_count
        );
        let mut recorded =
            vec![0u32; facts.is_tearable.len() * FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE];
        for (index, tearable) in facts.is_tearable.iter().enumerate() {
            recorded[index * FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE] = u32::from(*tearable);
            recorded[index * FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE + 1] = NONE;
        }
        let mut replays: Vec<u32> = Vec::new();
        for torn in &facts.torn_images {
            let base = usize::try_from(torn.write_index).expect("装得进")
                * FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE;
            recorded[base + 1] = torn.torn_image_index;
            recorded[base + 2] = u32::try_from(replays.len() / 2).expect("装得进");
            recorded[base + 3] = u32::try_from(torn.replays.len()).expect("装得进");
            for (replay_index, replayed) in &torn.replays {
                replays.push(*replay_index);
                replays.push(*replayed);
            }
        }
        assert_eq!(
            segment_expansions.len(),
            facts.segments.len(),
            "每一段一个拆法"
        );
        let mut segments =
            Vec::with_capacity(facts.segments.len() * FACTS_VERIFIER_WORDS_PER_SEGMENT);
        let mut segment_writes = Vec::new();
        if let Err(segment_index) = check_the_slice_starts_against_the_expansions(
            &facts.segments,
            segment_expansions,
            &facts.is_tearable,
        ) {
            panic!("段 {segment_index} 的片起点与它的各位对不上，或片越过了段的域：事实表坏了，重跑录入段");
        }
        for (segment_index, (segment, expansion)) in
            facts.segments.iter().zip(segment_expansions).enumerate()
        {
            let states_of_the_segment = segment.states.end - segment.states.start;
            assert!(
                states_of_the_segment == 0
                    || segment.slice_start.is_some()
                    || expansion.state_count(segment, &facts.is_tearable)
                        == Some(states_of_the_segment),
                "段 {segment_index} 的拆法（{expansion:?}）展开出的状态数与它的序号区间 {:?} 对不上",
                segment.states
            );
            let (digit_writes, copy_on_write_group) =
                expansion.digit_writes_and_the_copy_on_write_group(segment);
            let (start_lo, start_hi) = split_u64(segment.states.start);
            let (end_lo, end_hi) = split_u64(segment.states.end);
            let digit_first = u32::try_from(segment_writes.len()).expect("装得进");
            segment_writes.extend_from_slice(digit_writes);
            let group_first = u32::try_from(segment_writes.len()).expect("装得进");
            segment_writes.extend_from_slice(copy_on_write_group);
            let (slice_digit_first, slice_digit_count) = match &segment.slice_start {
                None => (NONE, 0),
                Some(slice_start) => {
                    let first = u32::try_from(segment_writes.len()).expect("装得进");
                    segment_writes.extend(slice_start.digits.iter().map(|digit| u32::from(*digit)));
                    (
                        first,
                        u32::try_from(slice_start.digits.len()).expect("装得进"),
                    )
                }
            };
            segments.extend_from_slice(&[
                digit_first,
                u32::try_from(digit_writes.len()).expect("装得进"),
                start_lo,
                start_hi,
                end_lo,
                end_hi,
                expansion.code(),
                group_first,
                u32::try_from(copy_on_write_group.len()).expect("装得进"),
                slice_digit_first,
                slice_digit_count,
            ]);
        }
        let mut units = Vec::with_capacity(facts.units.len() * FACTS_VERIFIER_WORDS_PER_UNIT);
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
        let mut roots = Vec::with_capacity(facts.roots.len() * FACTS_VERIFIER_WORDS_PER_ROOT);
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
        debug_assert_eq!(
            roots.len(),
            facts.roots.len() * FACTS_VERIFIER_WORDS_PER_ROOT
        );
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
            source: wgpu::ShaderSource::Wgsl(verifier_shader_source().into()),
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

    /// 对拍用：一段连续的状态各拆出的持久掩码（每个 [`PERSISTED_MASK_WORDS`] 个字，位 i 在第 i / 32 字的第 i % 32 位），按序号次序；
    /// 走的是核对内核里同一个拆法函数（探针入口 `probe_persisted_masks`），与 `crate::crash_facts` 的 CPU 拆法逐序号比。
    ///
    /// # Panics
    /// 一块超过 u32 个状态；回读的长度对不上。
    #[must_use]
    pub fn persisted_masks_of_states(
        &self,
        ordinals: Range<u64>,
    ) -> Vec<[u32; PERSISTED_MASK_WORDS]> {
        let count =
            usize::try_from(ordinals.end - ordinals.start).expect("一块的状态数装得进 usize");
        if count == 0 {
            return Vec::new();
        }
        let device = self.card.device();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("crash-facts-verifier-persisted-mask-probe"),
            source: wgpu::ShaderSource::Wgsl(verifier_shader_source().into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("crash-facts-verifier-persisted-mask-probe"),
            layout: None,
            module: &module,
            entry_point: Some("probe_persisted_masks"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let (start_lo, start_hi) = split_u64(ordinals.start);
        let mut header = self.header_words;
        header[6] = start_lo;
        header[7] = start_hi;
        header[8] = u32::try_from(count).expect("一块的状态数装得进 u32");
        let header_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &words_to_bytes(&header),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let result_bytes = u64::try_from(count * PERSISTED_MASK_WORDS * 4).expect("装得进 u64");
        let masks = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &vec![0xFFu8; count * PERSISTED_MASK_WORDS * 4],
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            },
        );
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: result_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = pipeline.get_bind_group_layout(0);
        fn entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            }
        }
        // 探针入口只读拆掩码那几张表：单元与根两张不进它的绑定
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                entry(0, &header_buffer),
                entry(1, &self.recorded),
                entry(2, &self.replays),
                entry(3, &self.segments),
                entry(4, &self.segment_writes),
                entry(7, &masks),
            ],
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                u32::try_from(count.div_ceil(WORKGROUP_SIZE)).expect("工作组数装得进 u32"),
                1,
                1,
            );
        }
        encoder.copy_buffer_to_buffer(&masks, 0, &staging, 0, result_bytes);
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
        assert_eq!(
            entries.len(),
            count * PERSISTED_MASK_WORDS,
            "回读的字数与派的相同"
        );
        let out = entries
            .chunks(PERSISTED_MASK_WORDS)
            .map(|state| {
                let mut words = [0u32; PERSISTED_MASK_WORDS];
                for (word, chunk) in words.iter_mut().zip(state) {
                    *word = u32::from_le_bytes(*chunk);
                }
                words
            })
            .collect();
        drop(mapped);
        staging.unmap();
        out
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

/// 持久掩码几个字：内核里的 `array<u32, 64>`，装得下 [`MAXIMUM_ENUMERATED_WRITES`] 次写。
pub const PERSISTED_MASK_WORDS: usize = 64;
const _: () = assert!(PERSISTED_MASK_WORDS * 32 == MAXIMUM_ENUMERATED_WRITES);

/// 探针交回的一个掩码换成按枚举写表的持久集合（前 `enumerated_write_count` 位）。
///
/// # Panics
/// `enumerated_write_count` 超过掩码的宽。
#[must_use]
pub fn persisted_set_of_the_mask_words(
    words: &[u32; PERSISTED_MASK_WORDS],
    enumerated_write_count: usize,
) -> Vec<bool> {
    assert!(enumerated_write_count <= MAXIMUM_ENUMERATED_WRITES);
    (0..enumerated_write_count)
        .map(|write_index| words[write_index / 32] & (1 << (write_index % 32)) != 0)
        .collect()
}

/// 基线来源号在两侧是同一个数。
const _: () = assert!(BASE_IMAGE_SOURCE == u32::MAX);
