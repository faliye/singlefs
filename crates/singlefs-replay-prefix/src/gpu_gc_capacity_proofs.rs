//! GPU GC 的切批准入、最大分片、显存账本和派发尺寸；产品调用同一份 exec 核。
//! G21–G30 续接 gpu_gc_proofs；设备完成信号的真实性和外围容器不在本模块证明。
use crate::gpu_gc_proofs::{checked_gpu_chunk_bytes, checked_gpu_input_bytes};
use vstd::prelude::*;

verus! {

pub struct GpuChunkCapacity {
    pub fixed_bytes: u64,
    pub bytes_per_entry: u64,
    pub entry_buffer_bytes: u64,
    pub binding_bytes: u64,
    pub dispatch_entries: u64,
    pub memory_limit: Option<u64>,
}

pub open spec fn chunk_fits(input: u64, entries: u64, capacity: GpuChunkCapacity) -> bool {
    crate::gpu_gc_proofs::aligned_input_size(input) <= capacity.binding_bytes
        && entries * capacity.entry_buffer_bytes <= capacity.binding_bytes
        && entries <= capacity.dispatch_entries
        && match capacity.memory_limit {
            None => true,
            Some(limit) => crate::gpu_gc_proofs::aligned_input_size(input) + entries * capacity.bytes_per_entry + capacity.fixed_bytes <= limit,
        }
}

/// G21：显存、输入绑定、条目绑定和派发四道界必须同时满足；乘加溢出不能误放行。
pub fn gpu_chunk_fits(input: u64, entries: u64, capacity: &GpuChunkCapacity) -> (fits: bool)
    ensures fits == chunk_fits(input, entries, *capacity),
{
    let aligned = match checked_gpu_input_bytes(input) {
        Some(bytes) => bytes,
        None => return false,
    };
    let entry_bytes = match entries.checked_mul(capacity.entry_buffer_bytes) {
        Some(bytes) => bytes,
        None => return false,
    };
    if aligned > capacity.binding_bytes || entry_bytes > capacity.binding_bytes
        || entries > capacity.dispatch_entries { return false; }
    match capacity.memory_limit {
        None => true,
        Some(limit) => match checked_gpu_chunk_bytes(input, entries, capacity.bytes_per_entry, capacity.fixed_bytes) {
            Some(bytes) => bytes <= limit,
            None => false,
        },
    }
}

/// G22：最大单条分片是装得下的最大四字节倍数；None 当且仅当最小输入也放不下。
#[allow(clippy::question_mark, reason = "被验函数体逐支写出溢出那一支，与 ensures 的 match 对照")]
pub fn largest_gpu_piece_input(capacity: &GpuChunkCapacity) -> (answer: Option<u64>)
    ensures
        match answer {
            None => !chunk_fits(0, 1, *capacity),
            Some(bytes) => bytes >= 4 && bytes % 4 == 0
                && chunk_fits(bytes, 1, *capacity)
                && forall|input: u64| #[trigger] chunk_fits(input, 1, *capacity) ==> input <= bytes,
        },
{
    if capacity.entry_buffer_bytes > capacity.binding_bytes || capacity.dispatch_entries == 0 {
        return None;
    }
    let room = match capacity.memory_limit {
        None => u64::MAX,
        Some(limit) => match limit.checked_sub(capacity.fixed_bytes) {
            None => return None,
            Some(left) => match left.checked_sub(capacity.bytes_per_entry) {
                None => return None,
                Some(input_room) => input_room,
            },
        },
    };
    let bounded = if room < capacity.binding_bytes { room } else { capacity.binding_bytes };
    let bytes = bounded / 4 * 4;
    if bytes < 4 { None } else { Some(bytes) }
}

/// G23：动态收紧显存上限，原来无限额时采用新上限；已有限额时绝不放宽。
pub fn tightened_gpu_memory_limit(previous: Option<u64>, available: u64) -> (limit: u64)
    ensures limit == match previous { None => available, Some(old) => if old < available { old } else { available } },
        limit <= available,
{
    match previous {
        None => available,
        Some(old) => if old < available { old } else { available },
    }
}

/// G24：尚有未完成提交、账本溢出或超限时拒绝新增占用；无上限也不允许整数回绕。
pub fn checked_gpu_memory_hold(held: u64, bytes: u64, pending: bool, limit: Option<u64>) -> (answer: Option<u64>)
    ensures match answer {
        Some(total) => !pending && total == held + bytes
            && match limit { None => true, Some(bound) => total <= bound },
        None => pending || held + bytes > u64::MAX
            || match limit { None => false, Some(bound) => held + bytes > bound },
    },
{
    if pending { return None; }
    match held.checked_add(bytes) {
        None => None,
        Some(total) => match limit {
            Some(bound) => if total > bound { None } else { Some(total) },
            None => Some(total),
        },
    }
}

/// G25：只释放已记账的字节；拒绝超额释放，成功时保留所有未释放的占用。
pub fn checked_gpu_memory_release(held: u64, bytes: u64) -> (answer: Option<u64>)
    ensures match answer {
        None => bytes > held,
        Some(left) => bytes <= held && left + bytes == held,
    },
{
    held.checked_sub(bytes)
}

/// G26：等待未成功时保留全部占用，只有完成才释放本段；两种结局都拒绝虚构的段大小。
pub fn gpu_bytes_after_submission_wait(held: u64, chunk: u64, completed: bool) -> (answer: Option<u64>)
    ensures match answer {
        None => chunk > held,
        Some(left) => chunk <= held && left == if completed { held - chunk } else { held as int },
    },
{
    if chunk > held { None }
    else if completed { Some(held - chunk) }
    else { Some(held) }
}

/// G27：显存不足后把失败段占用折半，正占用时严格下降，零时仍为零，取整误差至多一。
pub fn gpu_memory_cap_after_allocation_failure(failed_bytes: u64) -> (next: u64)
    ensures
        2 * next <= failed_bytes < 2 * next + 2,
        failed_bytes > 0 ==> next < failed_bytes,
        failed_bytes == 0 ==> next == 0,
{
    failed_bytes / 2
}

/// G28：派发覆盖恰好所需的工作组数，零条目不派发；全宽条数不因加法取整溢出。
#[allow(clippy::cast_possible_truncation, reason = "ensures 证了 groups 不超过 entries，装得进 u32")]
pub fn gpu_workgroups_for_entries(entries: u32, per_group: u32) -> (groups: u32)
    requires per_group > 0,
    ensures
        groups <= entries,
        groups == 0 <==> entries == 0,
        entries <= groups * per_group,
        entries > 0 ==> (groups - 1) * per_group < entries,
{
    let wide = crate::file_range_proofs::units_covering_size(entries as u64, per_group as u64);
    wide as u32
}

pub open spec fn mapping_entries_at_least_one(entries: u64) -> int {
    if entries == 0 { 1 } else { entries as int }
}

/// G29：映射条目/落点/排序索引三张表含空表占位；返回总量与最大绑定量，任何溢出拒绝。
#[allow(clippy::question_mark, reason = "被验函数体逐支写出溢出那一支，与 ensures 的 match 对照")]
pub fn checked_gpu_mapping_table_bytes(entries: u64, locations: u64) -> (answer: Option<(u64, u64)>)
    ensures match answer {
        None => 60 * mapping_entries_at_least_one(entries) + 16 * mapping_entries_at_least_one(locations) > u64::MAX,
        Some((total, largest)) => total == 60 * mapping_entries_at_least_one(entries) + 16 * mapping_entries_at_least_one(locations)
            && largest == if 56 * mapping_entries_at_least_one(entries) > 16 * mapping_entries_at_least_one(locations) {
                56 * mapping_entries_at_least_one(entries)
            } else { 16 * mapping_entries_at_least_one(locations) },
    },
{
    let count = if entries == 0 { 1 } else { entries };
    let location_count = if locations == 0 { 1 } else { locations };
    let entry_bytes = match count.checked_mul(56) { Some(bytes) => bytes, None => return None };
    let location_bytes = match location_count.checked_mul(16) { Some(bytes) => bytes, None => return None };
    let sorted_bytes = match count.checked_mul(4) { Some(bytes) => bytes, None => return None };
    let subtotal = match entry_bytes.checked_add(location_bytes) { Some(bytes) => bytes, None => return None };
    let total = match subtotal.checked_add(sorted_bytes) { Some(bytes) => bytes, None => return None };
    let largest = if entry_bytes > location_bytes { entry_bytes } else { location_bytes };
    Some((total, largest))
}

/// G30：向当前批追加输入，只有和可表示且整批满足四道界才接受；拒绝不能把和回绕成小批。
#[allow(clippy::manual_filter, reason = "被验函数体不用闭包，逐支写")]
pub fn extended_gpu_chunk_input(previous: u64, additional: u64, entries: u64, capacity: &GpuChunkCapacity) -> (answer: Option<u64>)
    ensures match answer {
        Some(total) => total == previous + additional && chunk_fits(total, entries, *capacity),
        None => previous + additional > u64::MAX
            || !chunk_fits((previous + additional) as u64, entries, *capacity),
    },
{
    match previous.checked_add(additional) {
        None => None,
        Some(total) => if gpu_chunk_fits(total, entries, capacity) { Some(total) } else { None },
    }
}

/// 非空见证：恰好装满、超出一字节、超时保留、完成释放、空表占位及全宽派发。
#[allow(unused_variables, reason = "assert 在普通 Rust 构建中擦除，执行调用保留")]
pub fn witness_gpu_capacity_and_accounting_boundaries() {
    let capacity = GpuChunkCapacity { fixed_bytes: 16, bytes_per_entry: 40, entry_buffer_bytes: 16,
        binding_bytes: 128, dispatch_entries: 2, memory_limit: Some(100) };
    let largest = largest_gpu_piece_input(&capacity);
    let accepted = gpu_chunk_fits(44, 1, &capacity);
    let rejected = gpu_chunk_fits(45, 1, &capacity);
    let held = checked_gpu_memory_hold(60, 40, false, Some(100));
    let pending = checked_gpu_memory_hold(60, 0, true, None);
    let kept = gpu_bytes_after_submission_wait(100, 40, false);
    let released = gpu_bytes_after_submission_wait(100, 40, true);
    let tables = checked_gpu_mapping_table_bytes(0, 0);
    let groups = gpu_workgroups_for_entries(u32::MAX, 64);
    assert(largest == Some(44));
    assert(accepted && !rejected);
    assert(held == Some(100) && pending == None);
    assert(kept == Some(100) && released == Some(60));
    assert(tables == Some((76, 56)));
    assert(groups == 67108864);
}

} // verus!
