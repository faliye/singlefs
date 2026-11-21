//! 显存上限与切批（D26（后台整理与放置回收） 已定项 13 第 3 条）：每张卡上本包占的缓冲（输入、输出、暂存一起）任何时刻不超过那张卡的上限，
//! 一批放不下就切成几次交，不给上限就不设限（只受这张卡的绑定上限与派发上限管）。
//! 暂存有两种，都算进占用：两份核的回读暂存（本包自己建的缓冲），与映射着建的每一份缓冲另带的一份同尺寸暂存
//! （[`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`]）。切批、记账、最小的一片与建完之后核真建尺寸都按同一个口径（[`ChunkFootprint`]）。
//!
//! 被验的 `gpu_chunk_fits` 只把对齐后的输入算一遍，映射着建的输入连同它的暂存在卡上占两遍：交给它之前把显存上限先扣掉一份对齐后的输入
//! （`chunk_fits_with_the_staging_of_its_input`），算最大的一片之前把扣掉一段与一条的固定占用之后剩下的地方分成两份
//! （`largest_piece_input_bytes`），两处都只用被验的函数判，不另写一份判法。
//!
//! 切法是纯函数（[`chunks_that_fit_on_the_gpu`]）；外壳每建一个缓冲先在这张卡的账上记下（[`GpuMemoryAccount::hold`]），
//! 记下之后超过上限就断言：切批算对了就走不到那一步，走到了是切批的 bug，不带着超限往下跑。
//!
//! 一条自己就装不进一段时把它切成几片逐片交（装不下自适应分片，缩小逐次进行）：[`pieces_cut_to_fit_on_the_gpu`] 按输入字节切，
//! 每片单独成一段装得下；连最小的一片（一个字节的输入加这一段的固定占用）都装不下才交回 [`SmallestPieceLargerThanTheRoom`]，
//! 调用方照「连一片都装不下就不用这张卡」办。各段每段都交一份的表（区域扫描的映射、选段打分的记录表）大到装不下时的切法在各段自己的模块里。
//!
//! 等一段的提交超过上限（这一批按出错交回）之后，GPU 上可能还在跑它：wgpu 留着被提交用到的缓冲直到提交做完，卡上的显存照旧占着。
//! 所以这一段的账不在交回出错时放，记成这张卡上还在跑的那次提交（[`SubmissionOfTheChunk`]、[`GpuMemoryAccount::settle_the_chunk_after_its_submission`]），
//! 等它的完成信号来了才放；下一次在这张卡上切段交活之前，先在上限之内等它做完
//! （[`GpuMemoryAccount::chunks_cut_once_the_submission_running_past_its_wait_completed`]），等不到就交回出错、这张卡上一段都不交，不越过上限去交。

use std::time::Duration;

use singlefs_core::background_compute::GpuMemoryInBytes;
use singlefs_replay_prefix::gpu_gc_capacity_proofs::{
    checked_gpu_memory_hold, checked_gpu_memory_release, gpu_bytes_after_submission_wait,
    gpu_chunk_fits, largest_gpu_piece_input, tightened_gpu_memory_limit, GpuChunkCapacity,
};
use singlefs_replay_prefix::gpu_gc_proofs::checked_gpu_input_bytes;

use crate::background_gpu::GpuFailureCause;
use crate::waiting_for_the_gpu::{
    wait_for_completion_before_the_deadline, CompletionCheck, GpuWaitDeadline, MonotonicClock,
};

/// GPU 缓冲的字节数要按 4 字节对齐（wgpu 的 `COPY_BUFFER_ALIGNMENT`；着色器里输入按 `array<u32>` 读）。
pub const GPU_BUFFER_ALIGNMENT_IN_BYTES: u64 =
    singlefs_replay_prefix::gpu_gc_proofs::GPU_INPUT_ALIGNMENT;

/// 主机上一条输入的字节长度，对齐前（切批按它切、按它取片）；一片在它那一条里的字节偏移、一段几条合起来的输入长度也用它。
/// 量的是主机内存里那几个字节有多长，不是卡上的显存：卡上那个输入缓冲占的显存由 [`input_buffer_bytes`] 换成 [`GpuMemoryInBytes`]，
/// 两者交反编不过。交给被验函数、切 `&[u8]` 的地方取内值。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputLengthOnTheHostInBytes(pub u64);

impl InputLengthOnTheHostInBytes {
    /// 主机上 `byte_count` 个字节（调用方手里是切片长度、`usize`）。
    ///
    /// # Panics
    /// `byte_count` 装不进 `u64`（`usize` 不比 `u64` 宽的平台上走不到）。
    #[must_use]
    pub fn of_this_many_bytes(byte_count: usize) -> Self {
        Self(u64::try_from(byte_count).expect("主机上的输入长度装得进 u64"))
    }

    /// 两段输入长度之和；超出 `u64` 交回 `None`。
    #[must_use]
    pub const fn checked_add(self, more: Self) -> Option<Self> {
        match self.0.checked_add(more.0) {
            Some(sum_in_bytes) => Some(Self(sum_in_bytes)),
            None => None,
        }
    }

    /// 两段输入长度之和，用在和不超过一条输入长度的地方（一片在它那一条里的偏移挪过头一片）。
    ///
    /// # Panics
    /// 和超出 `u64`。
    #[must_use]
    pub const fn plus(self, more: Self) -> Self {
        match self.checked_add(more) {
            Some(sum) => sum,
            None => panic!("主机上的输入长度之和超出 u64"),
        }
    }

    /// 扣掉 `less` 之后剩下的输入长度（一片的终点扣起点、一条扣掉头一片）。
    ///
    /// # Panics
    /// `less` 比它长。
    #[must_use]
    pub const fn minus(self, less: Self) -> Self {
        match self.0.checked_sub(less.0) {
            Some(difference_in_bytes) => Self(difference_in_bytes),
            None => panic!("扣掉的输入长度比剩下的长"),
        }
    }
}

/// 几段输入长度合起来（一段里几片的输入加起来）。
///
/// # Panics
/// 和超出 `u64`。
impl std::iter::Sum for InputLengthOnTheHostInBytes {
    fn sum<Lengths: Iterator<Item = Self>>(lengths: Lengths) -> Self {
        lengths.fold(Self(0), Self::plus)
    }
}

/// 输入缓冲的字节数：输入字节按 4 字节向上取整；一个字节都没有时也占 4 字节（wgpu 不许绑定长度为 0 的缓冲）。
/// `input_bytes` 是对齐前的输入数据长度，交回的是卡上那个缓冲占的显存。
#[must_use]
pub fn input_buffer_bytes(input_bytes: InputLengthOnTheHostInBytes) -> GpuMemoryInBytes {
    GpuMemoryInBytes(
        singlefs_replay_prefix::gpu_gc_proofs::checked_gpu_input_bytes(input_bytes.0)
            .expect("四字节对齐后的输入长度装得进 u64"),
    )
}

/// 一份缓冲在卡上占几份它的字节（缓冲本身、另带的暂存、回读暂存各算一份）：份数，不是字节，也不是条数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CopiesOnTheGpu(pub u64);

impl CopiesOnTheGpu {
    /// 这么多份、每份 `bytes_of_one_copy` 字节合占的显存；超出 `u64` 交回 `None`。
    #[must_use]
    pub const fn checked_bytes_of_the_copies(
        self,
        bytes_of_one_copy: GpuMemoryInBytes,
    ) -> Option<GpuMemoryInBytes> {
        match bytes_of_one_copy.0.checked_mul(self.0) {
            Some(bytes_of_the_copies) => Some(GpuMemoryInBytes(bytes_of_the_copies)),
            None => None,
        }
    }

    /// 这么多份、每份 `bytes_of_one_copy` 字节合占的显存，用在乘出来装得进 `u64` 的地方（各段固定占用的常量、一段真建的缓冲）。
    ///
    /// # Panics
    /// 乘出来超出 `u64`。
    #[must_use]
    pub const fn bytes_of_the_copies(
        self,
        bytes_of_one_copy: GpuMemoryInBytes,
    ) -> GpuMemoryInBytes {
        match self.checked_bytes_of_the_copies(bytes_of_one_copy) {
            Some(bytes_of_the_copies) => bytes_of_the_copies,
            None => panic!("份数乘每份的显存字节数超出 u64"),
        }
    }
}

/// 一段里有几条（一次交给一张卡的那几条输入；一次派发最多几条也按它数）：条数，不是字节，也不是份数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryCountOfAChunk(pub u64);

impl EntryCountOfAChunk {
    /// 一段里 `entry_count` 条（调用方手里是切片长度、`usize`）。
    ///
    /// # Panics
    /// `entry_count` 装不进 `u64`（`usize` 不比 `u64` 宽的平台上走不到）。
    #[must_use]
    pub fn of_this_many_entries(entry_count: usize) -> Self {
        Self(u64::try_from(entry_count).expect("一段的条数装得进 u64"))
    }

    /// 这么多条、每条 `bytes_of_one_entry` 字节合占的显存（每条一项的缓冲：两份核每一份的结果缓冲按它建）。
    ///
    /// # Panics
    /// 乘出来超出 `u64`。
    #[must_use]
    pub const fn bytes_of_the_entries(
        self,
        bytes_of_one_entry: GpuMemoryInBytes,
    ) -> GpuMemoryInBytes {
        match bytes_of_one_entry.0.checked_mul(self.0) {
            Some(bytes_of_the_entries) => GpuMemoryInBytes(bytes_of_the_entries),
            None => panic!("条数乘每条的显存字节数超出 u64"),
        }
    }

    /// 这么多条、每条的输入都是 `input_bytes_of_one_entry` 字节时，这一段对齐前的输入合起来多长（选段打分每个段一项上下界）。
    ///
    /// # Panics
    /// 乘出来超出 `u64`。
    #[must_use]
    pub const fn input_bytes_of_the_entries(
        self,
        input_bytes_of_one_entry: InputLengthOnTheHostInBytes,
    ) -> InputLengthOnTheHostInBytes {
        match input_bytes_of_one_entry.0.checked_mul(self.0) {
            Some(input_bytes_of_the_entries) => {
                InputLengthOnTheHostInBytes(input_bytes_of_the_entries)
            }
            None => panic!("条数乘每条的输入长度超出 u64"),
        }
    }
}

/// 一份映射着建的缓冲（建的时候映射着写内容，用法里没有 `MAP_WRITE`）在卡上占几份它的字节：缓冲本身，加 wgpu-core 30.0.1 为它另建的
/// 一份同尺寸暂存（`src/device/resource.rs` 里 `mapped_at_creation` 那一支建的 `StagingBuffer`，尺寸是对齐后的缓冲尺寸；本包建的缓冲尺寸都是
/// 4 的整数倍，对齐后不变）。解开映射时暂存挂进队列的待写，冲掉它的那一次提交做完才放。各段的参数、输入、描述、表与两份结果缓冲都这样建。
pub const COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION: CopiesOnTheGpu = CopiesOnTheGpu(2);

/// 映射着建的一份缓冲另带的暂存是几份：[`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`] 扣掉缓冲本身那一份。
const STAGING_COPIES_OF_A_BUFFER_MAPPED_AT_CREATION: CopiesOnTheGpu =
    CopiesOnTheGpu(COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION.0 - 1);

/// 两份核每一项结果在卡上合占几份它的字节：两份核各一个结果缓冲（映射着建，各带暂存）与一个回读暂存（`MAP_READ`、不映射着建，不另带暂存）。
pub const COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS: CopiesOnTheGpu =
    CopiesOnTheGpu(2 * COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION.0 + 2);

/// 一段（这一次交给一张卡的那几条）在卡上占多少：输入缓冲（映射着建，连同它的暂存占 [`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`] 份）之外，
/// 每一条占的固定字节（描述、两份核的输出与回读暂存，映射着建的连同暂存），加上与条数无关的那几样（参数、每段都交一份的表，连同暂存）。
/// 由每一段的核自己给（校验和核对那一段在 `checksum_kernels` 里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkFootprint {
    pub bytes_per_chunk_besides_the_entries: GpuMemoryInBytes,
    pub bytes_per_entry_besides_its_input: GpuMemoryInBytes,
    /// 每一条在它最大的那一个按条排的缓冲里占几字节：那个缓冲也要装得进这张卡的绑定上限。
    pub bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes,
}

impl ChunkFootprint {
    /// 这一段在卡上合占的字节数：对齐后的输入连同它的暂存，加上每条与每段的固定占用。`input_bytes` 是这一段对齐前的输入数据长度
    /// （同 [`input_buffer_bytes`]）。
    ///
    /// 条数与输入长度各是各的类型，交反了编不过；照对的次序交编得过（下面两段只差交的次序）：
    ///
    /// ```compile_fail
    /// use singlefs_background_gpu::checksum_kernels::CHECKSUM_CHUNK_FOOTPRINT;
    /// use singlefs_background_gpu::memory_limit_and_chunks::{
    ///     EntryCountOfAChunk, InputLengthOnTheHostInBytes,
    /// };
    /// let _held = CHECKSUM_CHUNK_FOOTPRINT
    ///     .bytes_held_by_a_chunk(EntryCountOfAChunk(1), InputLengthOnTheHostInBytes(4096));
    /// ```
    ///
    /// ```
    /// use singlefs_background_gpu::checksum_kernels::CHECKSUM_CHUNK_FOOTPRINT;
    /// use singlefs_background_gpu::memory_limit_and_chunks::{
    ///     EntryCountOfAChunk, InputLengthOnTheHostInBytes,
    /// };
    /// let _held = CHECKSUM_CHUNK_FOOTPRINT
    ///     .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(4096), EntryCountOfAChunk(1));
    /// ```
    ///
    /// 主机上的输入长度与卡上的显存字节数各是各的类型：显存字节数交到输入长度那一格编不过，
    /// 输入长度交到显存字节数那一格（剩下的地方扣掉已占的）也编不过：
    ///
    /// ```compile_fail
    /// use singlefs_background_gpu::checksum_kernels::CHECKSUM_CHUNK_FOOTPRINT;
    /// use singlefs_background_gpu::memory_limit_and_chunks::EntryCountOfAChunk;
    /// use singlefs_core::background_compute::GpuMemoryInBytes;
    /// let _held =
    ///     CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(GpuMemoryInBytes(4096), EntryCountOfAChunk(1));
    /// ```
    ///
    /// ```compile_fail
    /// use singlefs_background_gpu::memory_limit_and_chunks::{
    ///     ChunkBoundsOfAGpu, EntryCountOfAChunk, InputLengthOnTheHostInBytes,
    /// };
    /// use singlefs_core::background_compute::GpuMemoryInBytes;
    /// let bounds = ChunkBoundsOfAGpu {
    ///     memory_limit: Some(GpuMemoryInBytes(1 << 20)),
    ///     largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
    ///     largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
    /// };
    /// let _room = bounds.with_the_room_left_after(InputLengthOnTheHostInBytes(4096));
    /// ```
    ///
    /// ```
    /// use singlefs_background_gpu::memory_limit_and_chunks::{ChunkBoundsOfAGpu, EntryCountOfAChunk};
    /// use singlefs_core::background_compute::GpuMemoryInBytes;
    /// let bounds = ChunkBoundsOfAGpu {
    ///     memory_limit: Some(GpuMemoryInBytes(1 << 20)),
    ///     largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
    ///     largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
    /// };
    /// let _room = bounds.with_the_room_left_after(GpuMemoryInBytes(4096));
    /// ```
    ///
    /// 份数交到每份字节数那一格同样编不过：
    ///
    /// ```compile_fail
    /// use singlefs_background_gpu::memory_limit_and_chunks::{
    ///     COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS, COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,
    /// };
    /// let _bytes = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
    ///     .bytes_of_the_copies(COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS);
    /// ```
    ///
    /// # Panics
    /// 乘加超出 `u64`：调用方交的是已经切过、装得下这张卡的一段，或最小的一片。
    #[must_use]
    pub fn bytes_held_by_a_chunk(
        &self,
        input_bytes: InputLengthOnTheHostInBytes,
        entry_count: EntryCountOfAChunk,
    ) -> GpuMemoryInBytes {
        checked_bytes_held_by_a_chunk(*self, input_bytes, entry_count)
            .expect("GPU 缓冲占用的乘加装得进 u64")
    }
}

/// [`ChunkFootprint::bytes_held_by_a_chunk`] 的算法，乘加超出 `u64` 交回 `None`：被验的 `checked_gpu_chunk_bytes` 算一份对齐后的输入与固定占用，
/// 再加输入的暂存那一份（被验的 `checked_gpu_input_bytes`）。
fn checked_bytes_held_by_a_chunk(
    footprint: ChunkFootprint,
    input_bytes: InputLengthOnTheHostInBytes,
    entry_count: EntryCountOfAChunk,
) -> Option<GpuMemoryInBytes> {
    let bytes_besides_the_staging_of_the_input = GpuMemoryInBytes(
        singlefs_replay_prefix::gpu_gc_proofs::checked_gpu_chunk_bytes(
            input_bytes.0,
            entry_count.0,
            footprint.bytes_per_entry_besides_its_input.0,
            footprint.bytes_per_chunk_besides_the_entries.0,
        )?,
    );
    STAGING_COPIES_OF_A_BUFFER_MAPPED_AT_CREATION
        .checked_bytes_of_the_copies(GpuMemoryInBytes(checked_gpu_input_bytes(input_bytes.0)?))?
        .checked_add(bytes_besides_the_staging_of_the_input)
}

/// 一张卡上一段装得下多少：显存上限（不给是不设限）、单个缓冲能绑定的最大字节数、一次派发最多几条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkBoundsOfAGpu {
    pub memory_limit: Option<GpuMemoryInBytes>,
    pub largest_storage_binding_in_bytes: GpuMemoryInBytes,
    pub largest_entry_count_per_dispatch: EntryCountOfAChunk,
}

impl ChunkBoundsOfAGpu {
    /// 这张卡上已经有 `bytes_already_held_on_the_gpu` 字节被本包别的段占着时，再切一段能用的界：显存上限扣掉已占的（第 3 条：全部后台任务
    /// 在那张卡上合占的显存不超过上限，三段同时在一张卡上时一起算），绑定上限与派发上限不变。不给上限的照旧不设限；已占的不比上限少时一字节都不剩。
    #[must_use]
    pub fn with_the_room_left_after(self, bytes_already_held_on_the_gpu: GpuMemoryInBytes) -> Self {
        Self {
            memory_limit: self.memory_limit.map(|memory_limit| {
                GpuMemoryInBytes(singlefs_replay_prefix::gpu_gc_proofs::remaining_gpu_memory(
                    memory_limit.0,
                    bytes_already_held_on_the_gpu.0,
                ))
            }),
            largest_storage_binding_in_bytes: self.largest_storage_binding_in_bytes,
            largest_entry_count_per_dispatch: self.largest_entry_count_per_dispatch,
        }
    }

    /// 卡上此刻空闲的显存比剩下的地方少（建缓冲显存不足，第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）时，显存那一项压到至多
    /// `bytes_at_most`：给了上限的取两者小的，不给上限的取它。绑定上限与派发上限不变。
    #[must_use]
    pub fn with_the_memory_limit_at_most(self, bytes_at_most: GpuMemoryInBytes) -> Self {
        Self {
            memory_limit: Some(GpuMemoryInBytes(tightened_gpu_memory_limit(
                self.memory_limit.map(|limit| limit.0),
                bytes_at_most.0,
            ))),
            largest_storage_binding_in_bytes: self.largest_storage_binding_in_bytes,
            largest_entry_count_per_dispatch: self.largest_entry_count_per_dispatch,
        }
    }
}

/// 交给被验函数的那几个字段，显存上限是 `memory_limit_handed_to_the_verified_function`（照被验函数只把对齐后的输入算一遍的口径折过的）。
fn verified_chunk_capacity_under_the_memory_limit(
    footprint: ChunkFootprint,
    bounds: ChunkBoundsOfAGpu,
    memory_limit_handed_to_the_verified_function: Option<u64>,
) -> GpuChunkCapacity {
    GpuChunkCapacity {
        fixed_bytes: footprint.bytes_per_chunk_besides_the_entries.0,
        bytes_per_entry: footprint.bytes_per_entry_besides_its_input.0,
        entry_buffer_bytes: footprint.bytes_per_entry_in_the_largest_entry_buffer.0,
        binding_bytes: bounds.largest_storage_binding_in_bytes.0,
        dispatch_entries: bounds.largest_entry_count_per_dispatch.0,
        memory_limit: memory_limit_handed_to_the_verified_function,
    }
}

/// `entry_count` 条、对齐前 `input_bytes` 字节输入的一段装不装得下 `bounds`（输入连同它的暂存算在里面）：被验的 `gpu_chunk_fits` 判绑定、派发与显存四道界，
/// 交给它的显存上限先扣掉输入的暂存那几份（被验函数里 `对齐后的输入 + 条数 × 每条 + 每段 ≤ 上限 − 暂存`，即 [`ChunkFootprint::bytes_held_by_a_chunk`] ≤ 上限）。
/// 输入的暂存比上限还大、或对齐后的输入超出 `u64`，装不下。不给上限的只判绑定与派发。
fn chunk_fits_with_the_staging_of_its_input(
    input_bytes: InputLengthOnTheHostInBytes,
    entry_count: EntryCountOfAChunk,
    footprint: ChunkFootprint,
    bounds: ChunkBoundsOfAGpu,
) -> bool {
    let memory_limit_handed_to_the_verified_function = match bounds.memory_limit {
        None => None,
        Some(memory_limit) => {
            let Some(staging_of_the_input) =
                checked_gpu_input_bytes(input_bytes.0).and_then(|aligned_input| {
                    STAGING_COPIES_OF_A_BUFFER_MAPPED_AT_CREATION
                        .checked_bytes_of_the_copies(GpuMemoryInBytes(aligned_input))
                })
            else {
                return false;
            };
            let Some(limit_left_besides_the_staging) =
                memory_limit.checked_sub(staging_of_the_input)
            else {
                return false;
            };
            Some(limit_left_besides_the_staging.0)
        }
    };
    gpu_chunk_fits(
        input_bytes.0,
        entry_count.0,
        &verified_chunk_capacity_under_the_memory_limit(
            footprint,
            bounds,
            memory_limit_handed_to_the_verified_function,
        ),
    )
}

/// 一条单独就装不进一段（[`chunks_that_fit_on_the_gpu`] 不切单条；要切单条的用 [`pieces_cut_to_fit_on_the_gpu`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryLargerThanAChunkOnTheGpu {
    pub entry_position: usize,
    pub input_bytes: InputLengthOnTheHostInBytes,
}

/// 把一批（每条的输入字节数按批里的次序给）切成相连的几段，每段在卡上合占的字节不超过上限、输入与按条排的缓冲都装得进绑定上限、
/// 条数不超过一次派发的上限。从前往后贪心：能装就装进当前这一段，装不下就另起一段。空批交回空。
///
/// # Errors
/// 某一条单独成一段也装不下（[`EntryLargerThanAChunkOnTheGpu`]），一段都不交。
pub fn chunks_that_fit_on_the_gpu(
    entry_input_lengths_in_bytes: &[InputLengthOnTheHostInBytes],
    footprint: ChunkFootprint,
    bounds: ChunkBoundsOfAGpu,
) -> Result<Vec<std::ops::Range<usize>>, EntryLargerThanAChunkOnTheGpu> {
    let mut chunks = Vec::new();
    let mut chunk_start = 0;
    let mut chunk_input_bytes = InputLengthOnTheHostInBytes(0);
    for (entry_position, input_bytes) in entry_input_lengths_in_bytes.iter().copied().enumerate() {
        let entry_count_with_this_entry =
            EntryCountOfAChunk::of_this_many_entries(entry_position - chunk_start + 1);
        if let Some(combined) = chunk_input_bytes
            .checked_add(input_bytes)
            .filter(|combined| {
                chunk_fits_with_the_staging_of_its_input(
                    *combined,
                    entry_count_with_this_entry,
                    footprint,
                    bounds,
                )
            })
        {
            chunk_input_bytes = combined;
            continue;
        }
        if !chunk_fits_with_the_staging_of_its_input(
            input_bytes,
            EntryCountOfAChunk(1),
            footprint,
            bounds,
        ) {
            return Err(EntryLargerThanAChunkOnTheGpu {
                entry_position,
                input_bytes,
            });
        }
        chunks.push(chunk_start..entry_position);
        chunk_start = entry_position;
        chunk_input_bytes = input_bytes;
    }
    if chunk_start < entry_input_lengths_in_bytes.len() {
        chunks.push(chunk_start..entry_input_lengths_in_bytes.len());
    }
    Ok(chunks)
}

/// 一条里的一片：这一条在批里的位置，与这一片罩住的输入字节（从这一条的头一个字节数起，左闭右开）。没切的那一条是罩住它全部字节的一片。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PieceOfAnEntry {
    pub entry_position: usize,
    pub input_bytes: std::ops::Range<InputLengthOnTheHostInBytes>,
}

impl PieceOfAnEntry {
    #[must_use]
    pub fn input_byte_count(&self) -> InputLengthOnTheHostInBytes {
        self.input_bytes.end.minus(self.input_bytes.start)
    }
}

/// 一张卡此刻剩下的地方连最小的一片都装不下：`entry_position` 是要切的那一条在批里的位置，`smallest_piece_bytes` 是最小的一片
/// （一个字节的输入，补到一个整字）连同一段的固定占用合占的字节数，即这张卡至少要剩这么多。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmallestPieceLargerThanTheRoom {
    pub entry_position: usize,
    pub smallest_piece_bytes: GpuMemoryInBytes,
}

/// 一片单独成一段时，它的输入最多几个字节装得下 `bounds`：按显存上限扣掉一段的固定占用与一条的固定占用、剩下的分给输入连同它的暂存
/// （[`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`] 份）、按绑定上限，取 4 的整数倍（输入缓冲按整字建，[`input_buffer_bytes`]）。
/// 一个字节的一片都装不下交回 `None`。
///
/// 被验的 `largest_gpu_piece_input` 只把输入算一遍：交给它的显存上限是「一段与一条的固定占用」加「剩下的除以份数」，它交回的最大 4 的整数倍
/// 照份数乘回去不超过剩下的；比它大的 4 的整数倍乘份数都超过剩下的。显存上限连一段与一条的固定占用都装不下就交回 `None`。
fn largest_piece_input_bytes(
    footprint: ChunkFootprint,
    bounds: ChunkBoundsOfAGpu,
) -> Option<InputLengthOnTheHostInBytes> {
    let memory_limit_handed_to_the_verified_function = match bounds.memory_limit {
        None => None,
        Some(memory_limit) => {
            let bytes_of_a_chunk_and_an_entry_besides_the_input = footprint
                .bytes_per_chunk_besides_the_entries
                .checked_add(footprint.bytes_per_entry_besides_its_input)?;
            let room_for_the_input_and_its_staging =
                memory_limit.checked_sub(bytes_of_a_chunk_and_an_entry_besides_the_input)?;
            Some(
                bytes_of_a_chunk_and_an_entry_besides_the_input.0
                    + room_for_the_input_and_its_staging.0
                        / COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION.0,
            )
        }
    };
    largest_gpu_piece_input(&verified_chunk_capacity_under_the_memory_limit(
        footprint,
        bounds,
        memory_limit_handed_to_the_verified_function,
    ))
    .map(InputLengthOnTheHostInBytes)
}

/// 把一批切成相连的几段，每段是几片；装得下的一条整条是一片，单独成一段都装不下的一条从头切成几片，每片取单独成一段装得下的最大字节数
/// （4 的整数倍），最后一片是余下的。片按批里的次序、一条之内按字节的次序排，再照 [`chunks_that_fit_on_the_gpu`] 从前往后贪心装段：
/// 每一片单独成一段都装得下，所以装段不会失败。整条都装得下的批与 [`chunks_that_fit_on_the_gpu`] 切出同样的段。空批交回空。
///
/// # Errors
/// 某一条要切而连一个字节的一片都装不下（[`SmallestPieceLargerThanTheRoom`]），一段都不交。
pub fn pieces_cut_to_fit_on_the_gpu(
    entry_input_lengths_in_bytes: &[InputLengthOnTheHostInBytes],
    footprint: ChunkFootprint,
    bounds: ChunkBoundsOfAGpu,
) -> Result<Vec<Vec<PieceOfAnEntry>>, SmallestPieceLargerThanTheRoom> {
    let largest_piece = largest_piece_input_bytes(footprint, bounds);
    let mut pieces: Vec<PieceOfAnEntry> = Vec::with_capacity(entry_input_lengths_in_bytes.len());
    for (entry_position, input_bytes) in entry_input_lengths_in_bytes.iter().copied().enumerate() {
        let whole_entry_fits =
            chunks_that_fit_on_the_gpu(&[input_bytes], footprint, bounds).is_ok();
        if whole_entry_fits {
            pieces.push(PieceOfAnEntry {
                entry_position,
                input_bytes: InputLengthOnTheHostInBytes(0)..input_bytes,
            });
            continue;
        }
        let Some(largest_piece_bytes) = largest_piece else {
            return Err(SmallestPieceLargerThanTheRoom {
                entry_position,
                smallest_piece_bytes: footprint
                    .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(1), EntryCountOfAChunk(1)),
            });
        };
        assert!(
            largest_piece_bytes < input_bytes,
            "最大的一片装得下而整条装不下：整条比最大的一片长"
        );
        let mut piece_start = InputLengthOnTheHostInBytes(0);
        while piece_start < input_bytes {
            let piece_end = InputLengthOnTheHostInBytes(
                singlefs_replay_prefix::gpu_gc_proofs::next_gpu_record_slice_end(
                    piece_start.0,
                    input_bytes.0,
                    largest_piece_bytes.0,
                ),
            );
            pieces.push(PieceOfAnEntry {
                entry_position,
                input_bytes: piece_start..piece_end,
            });
            piece_start = piece_end;
        }
    }
    let piece_lengths: Vec<InputLengthOnTheHostInBytes> = pieces
        .iter()
        .map(PieceOfAnEntry::input_byte_count)
        .collect();
    let chunks = chunks_that_fit_on_the_gpu(&piece_lengths, footprint, bounds)
        .expect("每一片单独成一段都装得下：整条装得下的是整条，切开的每片不超过最大的一片");
    Ok(chunks
        .into_iter()
        .map(|chunk| pieces[chunk].to_vec())
        .collect())
}

/// 一段的提交等完之后，它的缓冲在卡上还占不占显存。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmissionOfTheChunk<Submission> {
    /// 等到它在 GPU 上做完了：这一段的缓冲随它丢掉，显存还回去。
    CompletedOnTheGpu,
    /// 没等到它做完（等超过上限、调用方在停而放弃了那一等，或等的时候出错）：GPU 上可能还在跑它，这一段的缓冲在卡上照旧占着显存。
    MayStillBeRunningOnTheGpu(Submission),
}

impl<Submission> SubmissionOfTheChunk<Submission> {
    /// 按等这次提交做完的结局判：等到了才算做完；超时与别的出错都当它可能还在跑（等的那一样报了错，GPU 上的提交不一定停了）。
    #[must_use]
    pub fn after_waiting_for_its_completion(
        submission: Submission,
        waited_for_its_completion: &Result<(), GpuFailureCause>,
    ) -> Self {
        match waited_for_its_completion {
            Ok(()) => Self::CompletedOnTheGpu,
            Err(_cause_of_the_failed_wait) => Self::MayStillBeRunningOnTheGpu(submission),
        }
    }
}

/// 等超过上限之后还在 GPU 上跑的那次提交，与它那一段在账上占着的字节。
#[derive(Debug)]
struct SubmissionRunningPastItsWait<Submission> {
    submission: Submission,
    bytes_held_by_its_chunk: GpuMemoryInBytes,
}

/// 一张卡上切不出这一批分到它的几条（这张卡上一段都不交）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChunksNotCutOnTheGpu {
    /// 这张卡上前一次等超过上限的那次提交，这一次在上限之内又没等到它做完（或等的时候出错）：它的缓冲还占着卡上的显存，
    /// 不越过上限去交（第 3、4 条）。
    EarlierSubmissionStillRunning(GpuFailureCause),
    /// 一条单独就装不进一段。
    EntryLargerThanAChunk(EntryLargerThanAChunkOnTheGpu),
}

/// 一张卡上本包此刻占着的缓冲字节数与上限，连同等超过上限之后还在 GPU 上跑的那次提交。外壳建缓冲之前记、那一段的提交做完之后放。
#[derive(Debug)]
pub struct GpuMemoryAccount<Submission> {
    memory_limit: Option<GpuMemoryInBytes>,
    /// 合占的字节数，含还在跑的那次提交那一段。
    bytes_held: GpuMemoryInBytes,
    /// 至多一次：它没做完之前这张卡不再交活（[`Self::hold`] 断言）。
    submission_running_past_its_wait: Option<SubmissionRunningPastItsWait<Submission>>,
}

impl<Submission> GpuMemoryAccount<Submission> {
    #[must_use]
    pub fn with_nothing_held(memory_limit: Option<GpuMemoryInBytes>) -> Self {
        Self::with_the_bytes_leaked_by_earlier_mounts(memory_limit, GpuMemoryInBytes(0))
    }

    /// 这个进程里之前几次挂载在这张卡上泄漏了 `bytes_leaked_by_earlier_mounts` 字节（丢后台 GPU 时还在跑的提交，设备不析构，显存不还）：
    /// 这一次的账从它起，这一次挂载里一直记着、不放（第 3 条「任何时刻都不许比它多」：泄漏的那一片仍占着这张卡）。
    #[must_use]
    pub fn with_the_bytes_leaked_by_earlier_mounts(
        memory_limit: Option<GpuMemoryInBytes>,
        bytes_leaked_by_earlier_mounts: GpuMemoryInBytes,
    ) -> Self {
        Self {
            memory_limit,
            bytes_held: bytes_leaked_by_earlier_mounts,
            submission_running_past_its_wait: None,
        }
    }

    #[must_use]
    pub fn bytes_held(&self) -> GpuMemoryInBytes {
        self.bytes_held
    }

    #[must_use]
    pub fn memory_limit(&self) -> Option<GpuMemoryInBytes> {
        self.memory_limit
    }

    /// 这张卡上记着一次等超过上限（或调用方在停而放弃了那一等）、还没在 GPU 上做完的提交：丢后台 GPU 时这张卡的设备与队列不析构
    /// （析构要等队列空，不带上限）。
    #[must_use]
    pub fn has_a_submission_running_past_its_wait(&self) -> bool {
        self.submission_running_past_its_wait.is_some()
    }

    /// 换上切换给的上限（第 1、3 条：从下一批起照它，外壳在两批之间调）。账不清：还占着的字节、等超过上限还在跑的那次提交照旧记着，
    /// 下一批交活之前先等它做完、放掉它的账，再按新上限扣掉还占着的切。
    pub fn use_the_memory_limit_from_the_next_batch(
        &mut self,
        memory_limit: Option<GpuMemoryInBytes>,
    ) {
        self.memory_limit = memory_limit;
    }

    /// 记下要建的那个缓冲。
    ///
    /// # Panics
    /// 这张卡上还有一次等超过上限、没做完的提交：交活之前先等它（[`Self::chunks_cut_once_the_submission_running_past_its_wait_completed`]），
    /// 走到这里是外壳跳过了那一步。
    /// 记下之后超过上限：切批（[`chunks_that_fit_on_the_gpu`]）按同一个上限切过，超了是切批与建缓冲对不上的 bug。
    pub fn hold(&mut self, bytes: GpuMemoryInBytes) {
        assert!(
            self.submission_running_past_its_wait.is_none(),
            "这张卡上还有一次等超过上限、没做完的提交：交下一段之前先等它做完"
        );
        self.bytes_held = GpuMemoryInBytes(
            checked_gpu_memory_hold(
                self.bytes_held.0,
                bytes.0,
                self.submission_running_past_its_wait.is_some(),
                self.memory_limit.map(|limit| limit.0),
            )
            .expect("本包在这张卡上占的缓冲超过了上限或 u64：切批与记账必须一致"),
        );
    }

    /// 放掉丢掉的那几个缓冲。
    ///
    /// # Panics
    /// 放的比记的多。
    pub fn release(&mut self, bytes: GpuMemoryInBytes) {
        self.bytes_held = GpuMemoryInBytes(
            checked_gpu_memory_release(self.bytes_held.0, bytes.0).expect("放掉的缓冲比记下的多"),
        );
    }

    /// 一段的提交等完之后结它的账：做完了就放掉这一段；没等到它做完的，账照旧记着，记成这张卡上还在跑的那次提交，
    /// 等它的完成信号来了才放（第 3 条：任何时刻都不许比上限多）。
    ///
    /// # Panics
    /// 放的比记的多；或这张卡上已经记着一次还在跑的提交（交这一段之前 [`Self::hold`] 断言过没有）。
    pub fn settle_the_chunk_after_its_submission(
        &mut self,
        bytes_held_by_the_chunk: GpuMemoryInBytes,
        submission_of_the_chunk: SubmissionOfTheChunk<Submission>,
    ) {
        let bytes_after_wait = gpu_bytes_after_submission_wait(
            self.bytes_held.0,
            bytes_held_by_the_chunk.0,
            matches!(
                &submission_of_the_chunk,
                SubmissionOfTheChunk::CompletedOnTheGpu
            ),
        )
        .expect("结算的那一段比记下的多");
        match submission_of_the_chunk {
            SubmissionOfTheChunk::CompletedOnTheGpu => {}
            SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(submission) => {
                assert!(
                    self.submission_running_past_its_wait.is_none(),
                    "这张卡上已经记着一次还在跑的提交：交这一段之前没有等它"
                );
                self.submission_running_past_its_wait = Some(SubmissionRunningPastItsWait {
                    submission,
                    bytes_held_by_its_chunk: bytes_held_by_the_chunk,
                });
            }
        }
        self.bytes_held = GpuMemoryInBytes(bytes_after_wait);
    }

    /// 一段建缓冲出错（显存不足，或等设备交回错误范围超过上限）之后结它的账：这一段已经建好、解开映射的那几份缓冲连同暂存挂在队列的待写里，
    /// 冲掉待写的那一次提交（`flushing_submission`，调用方交的一次空提交）做完才放。等到它做完（`waited_for_the_flush` 是 `Ok`）就放掉这一段、
    /// 交回建缓冲那一次的错；没等到就把这一段记成这张卡上还在跑的那次提交（同 [`Self::settle_the_chunk_after_its_submission`]），
    /// 这张卡下一次交之前先等它做完：等满了上限交回 [`GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit`]（算这一批这张卡一次超时，
    /// 这一段没交到卡上、不算重交），等的时候报的别的出错照原样交回（调用方在停而放弃的那一等交回 [`GpuFailureCause::GaveUpWaitingForTheCallerIsStopping`]）。
    ///
    /// # Panics
    /// 同 [`Self::settle_the_chunk_after_its_submission`]。
    #[must_use]
    pub fn settle_the_chunk_whose_buffers_were_not_created(
        &mut self,
        bytes_held_by_the_chunk: GpuMemoryInBytes,
        failure_creating_the_buffers: GpuFailureCause,
        flushing_submission: Submission,
        waited_for_the_flush: Result<(), GpuFailureCause>,
    ) -> GpuFailureCause {
        let submission_of_the_chunk = SubmissionOfTheChunk::after_waiting_for_its_completion(
            flushing_submission,
            &waited_for_the_flush,
        );
        self.settle_the_chunk_after_its_submission(
            bytes_held_by_the_chunk,
            submission_of_the_chunk,
        );
        match waited_for_the_flush {
            Ok(()) => failure_creating_the_buffers,
            Err(failure_waiting_for_the_flush) => {
                failure_of_waiting_for_the_flush_of_the_pending_writes(
                    failure_waiting_for_the_flush,
                )
            }
        }
    }

    /// 这张卡上那次等超过上限、还在跑的提交，在 `deadline` 之内等它做完（`check_once_waiting_at_most` 看一次它做没做完、最多等交给它的那么久）；
    /// 做完放掉它那一段的账。没有这样的提交就当场交回。
    ///
    /// # Errors
    /// 等满上限还没做完 ⇒ [`GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit`]；期限读调用方在停的标记而调用方在停 ⇒
    /// [`GpuFailureCause::GaveUpWaitingForTheCallerIsStopping`]；看的时候报的出错照原样交回。几种都不放账：那次提交照旧记成还在跑的那一次。
    pub fn wait_for_the_submission_running_past_its_wait(
        &mut self,
        deadline: &GpuWaitDeadline,
        clock: &impl MonotonicClock,
        mut check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
    ) -> Result<(), GpuFailureCause> {
        let Some(running) = self.submission_running_past_its_wait.as_ref() else {
            return Ok(());
        };
        let waited = wait_for_completion_before_the_deadline(deadline, clock, |time_left| {
            check_once_waiting_at_most(&running.submission, time_left)
        });
        if let Err(cause) = waited {
            return Err(failure_of_waiting_for_the_earlier_submission(cause));
        }
        let completed = self
            .submission_running_past_its_wait
            .take()
            .expect("上面刚看过这张卡上记着一次还在跑的提交");
        self.release(completed.bytes_held_by_its_chunk);
        Ok(())
    }

    /// 在这张卡上交活之前调（第 3、4 条）：这张卡上还有一次等超过上限、没做完的提交的，先在 `deadline` 之内等它做完、放掉它的账，
    /// 交回此刻剩下的界（`bounds` 的显存上限扣掉此刻还占着的），调用方按它切这一批分到这张卡的几条。
    ///
    /// # Errors
    /// 等不到它做完：同 [`Self::wait_for_the_submission_running_past_its_wait`]，账照旧记着。
    pub fn room_once_the_submission_running_past_its_wait_completed(
        &mut self,
        deadline: &GpuWaitDeadline,
        clock: &impl MonotonicClock,
        check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
        bounds: ChunkBoundsOfAGpu,
    ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
        self.wait_for_the_submission_running_past_its_wait(
            deadline,
            clock,
            check_once_waiting_at_most,
        )?;
        Ok(bounds.with_the_room_left_after(self.bytes_held))
    }

    /// 这张卡分到的几条（按批里的次序给每条的输入字节数）切段，在交这几段之前调（第 3、4 条）：分到了活而这张卡上还有一次等超过上限、
    /// 没做完的提交，先在 `deadline` 之内等它做完、放掉它的账，再按上限扣掉此刻还占着的切（[`chunks_that_fit_on_the_gpu`]）；
    /// 没分到活的不等（这一批不在这张卡上交活）。
    ///
    /// # Errors
    /// 等不到它做完 ⇒ [`ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning`]，一段都不切、账照旧记着；
    /// 一条单独装不下 ⇒ [`ChunksNotCutOnTheGpu::EntryLargerThanAChunk`]。
    pub fn chunks_cut_once_the_submission_running_past_its_wait_completed(
        &mut self,
        deadline: &GpuWaitDeadline,
        clock: &impl MonotonicClock,
        check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
        entry_input_lengths_in_bytes: &[InputLengthOnTheHostInBytes],
        footprint: ChunkFootprint,
        bounds: ChunkBoundsOfAGpu,
    ) -> Result<Vec<std::ops::Range<usize>>, ChunksNotCutOnTheGpu> {
        if entry_input_lengths_in_bytes.is_empty() {
            return Ok(Vec::new());
        }
        let room = self
            .room_once_the_submission_running_past_its_wait_completed(
                deadline,
                clock,
                check_once_waiting_at_most,
                bounds,
            )
            .map_err(ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning)?;
        chunks_that_fit_on_the_gpu(entry_input_lengths_in_bytes, footprint, room)
            .map_err(ChunksNotCutOnTheGpu::EntryLargerThanAChunk)
    }
}

/// 等冲掉待写的那一次空提交等满了上限：报成它自己的那一种（这一段没交到卡上）；别的出错照原样（调用方在停而放弃的那一等照原样报放弃，不报成等满上限）。
fn failure_of_waiting_for_the_flush_of_the_pending_writes(
    cause: GpuFailureCause,
) -> GpuFailureCause {
    match cause {
        GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit } => {
            GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { wait_limit }
        }
        reported_by_the_wait @ (GpuFailureCause::GaveUpWaitingForTheCallerIsStopping
        | GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
        | GpuFailureCause::OpeningThreadNotStarted { .. }
        | GpuFailureCause::OpeningPanicked { .. }
        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
        | GpuFailureCause::OpeningCalledOff
        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::ReadbackMappingFailed { .. }
        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
        | GpuFailureCause::UnusableAfterResubmissions { .. }
        | GpuFailureCause::NotOpenedAfterTheReopenings { .. }) => reported_by_the_wait,
    }
}

/// 等前一次超时的那次提交又等满了上限：报成它自己的那一种，与这一次交的段自己等超时分得开；别的出错照原样（调用方在停而放弃的那一等照原样报放弃）。
fn failure_of_waiting_for_the_earlier_submission(cause: GpuFailureCause) -> GpuFailureCause {
    match cause {
        GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit } => {
            GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { wait_limit }
        }
        reported_by_the_wait @ (GpuFailureCause::GaveUpWaitingForTheCallerIsStopping
        | GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
        | GpuFailureCause::OpeningThreadNotStarted { .. }
        | GpuFailureCause::OpeningPanicked { .. }
        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
        | GpuFailureCause::OpeningCalledOff
        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::ReadbackMappingFailed { .. }
        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
        | GpuFailureCause::UnusableAfterResubmissions { .. }
        | GpuFailureCause::NotOpenedAfterTheReopenings { .. }) => reported_by_the_wait,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// 每条另占 40 字节、每段另占 16 字节、按条排的最大缓冲每条 16 字节：校验和核对那一段的量级，数是测试自己定的。
    const FOOTPRINT_UNDER_TEST: ChunkFootprint = ChunkFootprint {
        bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(16),
        bytes_per_entry_besides_its_input: GpuMemoryInBytes(40),
        bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(16),
    };

    fn bounds(memory_limit_in_bytes: Option<u64>) -> ChunkBoundsOfAGpu {
        ChunkBoundsOfAGpu {
            memory_limit: memory_limit_in_bytes.map(GpuMemoryInBytes),
            largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
            largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
        }
    }

    /// 用例写的每条输入长度（主机上的字节）。
    fn lengths_on_the_host(lengths_in_bytes: &[u64]) -> Vec<InputLengthOnTheHostInBytes> {
        lengths_in_bytes
            .iter()
            .copied()
            .map(InputLengthOnTheHostInBytes)
            .collect()
    }

    /// 用例里交给 GPU 的一次提交：只是个编号。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct SubmissionNamedByTheTest(u32);

    /// 等超时的那一次提交。
    const TIMED_OUT_SUBMISSION: SubmissionNamedByTheTest = SubmissionNamedByTheTest(1);
    /// 等的上限：用例自己定的数。
    const WAIT_LIMIT_UNDER_TEST: Duration = Duration::from_secs(5);
    /// 这张卡的显存上限：用例自己定的数。
    const MEMORY_LIMIT_UNDER_TEST_IN_BYTES: u64 = 1000;
    /// 等超时的那一段占的字节。
    const BYTES_HELD_BY_THE_TIMED_OUT_CHUNK: GpuMemoryInBytes = GpuMemoryInBytes(600);

    /// 假钟：只在假完成信号让它走时才走。
    struct ClockMovedByTheCompletionSignal {
        now: Cell<Duration>,
    }

    impl MonotonicClock for ClockMovedByTheCompletionSignal {
        fn time_since_the_origin(&self) -> Duration {
            self.now.get()
        }
    }

    /// 假完成信号：`submission` 在钟走到 `completes_at` 那一刻在 GPU 上做完。每看一次钟走一秒（最多走到期限，至少走 1 纳秒：
    /// 被测的写错时用例照样走得完、在断言上红）；看的不是这一次提交就断言。
    fn completion_signal_of(
        clock: &ClockMovedByTheCompletionSignal,
        submission: SubmissionNamedByTheTest,
        completes_at: Duration,
    ) -> impl FnMut(&SubmissionNamedByTheTest, Duration) -> CompletionCheck<()> + '_ {
        move |submission_checked, time_left| {
            assert_eq!(*submission_checked, submission, "等的是超时的那一次提交");
            let waited_this_check = Duration::from_secs(1)
                .min(time_left)
                .max(Duration::from_nanos(1));
            clock.now.set(clock.now.get() + waited_this_check);
            if clock.now.get() >= completes_at {
                CompletionCheck::Completed(())
            } else {
                CompletionCheck::NotYetComplete
            }
        }
    }

    /// 照外壳交一段的次序走一遍：钟在 100 秒，记下 600 字节的一段、交、在 5 秒上限之内等它做完；那次提交到 `completes_at`（晚于 105 秒）
    /// 才在 GPU 上做完，等满上限交回超时出错；再照等的结局结这一段的账。交回账与钟（停在 105 秒）。
    fn account_after_a_chunk_whose_wait_timed_out(
        completes_at: Duration,
    ) -> (
        GpuMemoryAccount<SubmissionNamedByTheTest>,
        ClockMovedByTheCompletionSignal,
    ) {
        assert!(completes_at > Duration::from_secs(105), "要等超时");
        let mut account = GpuMemoryAccount::with_nothing_held(Some(GpuMemoryInBytes(
            MEMORY_LIMIT_UNDER_TEST_IN_BYTES,
        )));
        let clock = ClockMovedByTheCompletionSignal {
            now: Cell::new(Duration::from_secs(100)),
        };
        account.hold(BYTES_HELD_BY_THE_TIMED_OUT_CHUNK);
        let waited = {
            let mut completion_signal =
                completion_signal_of(&clock, TIMED_OUT_SUBMISSION, completes_at);
            wait_for_completion_before_the_deadline(
                &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
                &clock,
                |time_left| completion_signal(&TIMED_OUT_SUBMISSION, time_left),
            )
        };
        assert_eq!(
            waited,
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: WAIT_LIMIT_UNDER_TEST
            }),
            "等满 5 秒还没做完：这一批交回超时出错"
        );
        let submission_of_the_chunk =
            SubmissionOfTheChunk::after_waiting_for_its_completion(TIMED_OUT_SUBMISSION, &waited);
        assert_eq!(
            submission_of_the_chunk,
            SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(TIMED_OUT_SUBMISSION),
            "没等到做完的提交可能还在 GPU 上跑"
        );
        account.settle_the_chunk_after_its_submission(
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            submission_of_the_chunk,
        );
        assert_eq!(clock.now.get(), Duration::from_secs(105));
        (account, clock)
    }

    /// 第 3 条在等超时之后：一段的提交等超过上限、这一批按出错交回之后，GPU 上可能还在跑它，这一段的缓冲照旧占着卡上的显存。
    /// 账上照旧记着它，这期间切批扣掉它；完成信号来了才放。
    #[test]
    fn the_chunk_whose_wait_timed_out_stays_held_until_its_submission_completes_on_the_gpu() {
        let (mut account, clock) =
            account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(107));
        assert_eq!(
            account.bytes_held(),
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            "超时之后、完成之前：这一段在账上照旧占着 600 字节"
        );
        let lengths = [50_u64, 50, 50, 50, 50];
        let bounds_of_the_gpu = bounds(Some(MEMORY_LIMIT_UNDER_TEST_IN_BYTES));
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&lengths),
                FOOTPRINT_UNDER_TEST,
                bounds_of_the_gpu.with_the_room_left_after(account.bytes_held()),
            ),
            Ok(vec![0..2, 2..4, 4..5]),
            "这期间切批只用剩下的 400 字节；超时就放账的话整批一段 720 字节，与卡上还占着的 600 合 1320"
        );
        // 下一批在 105 秒来了，那次提交在 107 秒做完：切段之前等到了它、放掉它的账，再按整个上限切
        let chunks = account.chunks_cut_once_the_submission_running_past_its_wait_completed(
            &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
            &clock,
            completion_signal_of(&clock, TIMED_OUT_SUBMISSION, Duration::from_secs(107)),
            &lengths_on_the_host(&lengths),
            FOOTPRINT_UNDER_TEST,
            bounds_of_the_gpu,
        );
        assert_eq!(
            chunks,
            Ok(std::iter::once(0..5).collect::<Vec<_>>()),
            "那次提交做完、账放了：五条 720 字节一段"
        );
        assert_eq!(
            clock.now.get(),
            Duration::from_secs(107),
            "等到它做完那一刻"
        );
        assert_eq!(
            account.bytes_held(),
            GpuMemoryInBytes(0),
            "完成信号来了才放"
        );
    }

    /// 第 3、4 条：前一次等超时的那次提交还没在 GPU 上做完时，下一批不在这张卡上交活：切段之前在上限之内等它，等满上限还没做完就交回出错，
    /// 一段都不切，它的账照旧记着。
    #[test]
    fn the_gpu_whose_timed_out_submission_is_still_running_gets_no_chunk_of_the_next_batch() {
        let (mut account, clock) =
            account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(200));
        let chunks = account.chunks_cut_once_the_submission_running_past_its_wait_completed(
            &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
            &clock,
            completion_signal_of(&clock, TIMED_OUT_SUBMISSION, Duration::from_secs(200)),
            &lengths_on_the_host(&[100, 100, 100, 100, 100]),
            FOOTPRINT_UNDER_TEST,
            bounds(Some(MEMORY_LIMIT_UNDER_TEST_IN_BYTES)),
        );
        assert_eq!(
            chunks,
            Err(ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning(
                GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                    wait_limit: WAIT_LIMIT_UNDER_TEST
                }
            )),
            "那次提交要到 200 秒才做完，下一批在 105 秒来、等满 5 秒：这张卡上一段都不交，交回出错"
        );
        assert_eq!(clock.now.get(), Duration::from_secs(110), "等满 5 秒");
        assert_eq!(
            account.bytes_held(),
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            "它的账照旧记着"
        );
        assert!(
            account.submission_running_past_its_wait.is_some(),
            "它仍记成还在跑的那次提交"
        );
    }

    /// 第 4 条「卸载时不等还在跑的提交」：交活之前等前一次超时的那次提交时调用方在停（标记已置）——醒来就放弃，交回放弃那一种
    /// （不报成前一次又没等到），它的账照旧记着、仍记成还在跑的那次提交（丢的时候这张卡因此整张泄漏）。
    #[test]
    fn the_wait_for_the_timed_out_submission_given_up_for_the_caller_stopping_keeps_it_held_as_still_running(
    ) {
        let (mut account, clock) =
            account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(200));
        let (caller_is_stopping, handle_setting_the_flag) =
            crate::waiting_for_the_gpu::caller_stopping_for_tests::caller_stopping_flag_and_the_handle_setting_it();
        handle_setting_the_flag.tell_the_implementation_the_caller_is_stopping();
        let waited = account.wait_for_the_submission_running_past_its_wait(
            &GpuWaitDeadline::starting_now_seeing_the_caller_stopping(
                WAIT_LIMIT_UNDER_TEST,
                &clock,
                crate::waiting_for_the_gpu::CallerStoppingSeenByTheWait::ReadBetweenShortWaits(
                    caller_is_stopping,
                ),
            ),
            &clock,
            completion_signal_of(&clock, TIMED_OUT_SUBMISSION, Duration::from_secs(200)),
        );
        assert_eq!(
            waited,
            Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping),
            "调用方在停：放弃，不报成前一次又等满了上限"
        );
        assert_eq!(
            clock.now.get(),
            Duration::from_secs(105)
                + crate::waiting_for_the_gpu::SHORT_WAIT_BETWEEN_READINGS_OF_THE_CALLER_STOPPING,
            "只等了一个短等间隔就放弃"
        );
        assert_eq!(
            account.bytes_held(),
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            "它的账照旧记着"
        );
        assert!(
            account.has_a_submission_running_past_its_wait(),
            "它仍记成还在跑的那次提交"
        );
    }

    /// 同一条在建缓冲出错之后：冲掉待写的那一次空提交，等它时调用方在停而放弃——交回放弃那一种（不报成建缓冲的那个错，也不报成冲掉待写等满了上限），
    /// 这一段的账照旧记着、记成还在跑的那次提交。
    #[test]
    fn the_flush_of_the_pending_writes_given_up_for_the_caller_stopping_keeps_the_chunk_held_as_still_running(
    ) {
        let mut account = GpuMemoryAccount::with_nothing_held(Some(GpuMemoryInBytes(
            MEMORY_LIMIT_UNDER_TEST_IN_BYTES,
        )));
        account.hold(BYTES_HELD_BY_THE_TIMED_OUT_CHUNK);
        let reported = account.settle_the_chunk_whose_buffers_were_not_created(
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                bytes_held_by_the_chunk: BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
                reason: "用例造的：建缓冲显存不足".to_string(),
            },
            TIMED_OUT_SUBMISSION,
            Err(GpuFailureCause::GaveUpWaitingForTheCallerIsStopping),
        );
        assert_eq!(
            reported,
            GpuFailureCause::GaveUpWaitingForTheCallerIsStopping
        );
        assert_eq!(account.bytes_held(), BYTES_HELD_BY_THE_TIMED_OUT_CHUNK);
        assert!(
            account.has_a_submission_running_past_its_wait(),
            "冲掉待写的那一次没等到它做完：记成还在跑的那次提交"
        );
    }

    /// 外壳要是跳过了等那一步就交下一段：记账那一刻断言，不带着可能超限的显存往下交。
    #[test]
    #[should_panic(expected = "交下一段之前先等它做完")]
    fn holding_a_chunk_while_a_timed_out_submission_is_still_running_panics() {
        let (mut account, _clock) =
            account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(107));
        account.hold(GpuMemoryInBytes(100));
    }

    /// 这一批没分到这张卡的活：不在这张卡上交活，就不等它前一次超时的那次提交；一段不切，账照旧记着。
    #[test]
    fn the_gpu_given_no_entries_of_the_next_batch_does_not_wait_for_its_timed_out_submission() {
        let (mut account, clock) =
            account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(107));
        let chunks = account.chunks_cut_once_the_submission_running_past_its_wait_completed(
            &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
            &clock,
            |_submission, _time_left| panic!("没分到活的卡不该等"),
            &lengths_on_the_host(&[]),
            FOOTPRINT_UNDER_TEST,
            bounds(Some(MEMORY_LIMIT_UNDER_TEST_IN_BYTES)),
        );
        assert_eq!(chunks, Ok(Vec::new()));
        assert_eq!(
            account.bytes_held(),
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            "没等，账照旧记着"
        );
    }

    #[test]
    fn copies_on_the_gpu_times_the_bytes_of_one_copy_give_the_bytes_of_the_copies() {
        assert_eq!(
            CopiesOnTheGpu(3).bytes_of_the_copies(GpuMemoryInBytes(16)),
            GpuMemoryInBytes(48)
        );
        assert_eq!(
            COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS
                .bytes_of_the_copies(GpuMemoryInBytes(8)),
            GpuMemoryInBytes(48),
            "两份核每一项结果：两个结果缓冲各连同暂存两份、两个回读暂存各一份，合六份"
        );
        assert_eq!(
            STAGING_COPIES_OF_A_BUFFER_MAPPED_AT_CREATION
                .bytes_of_the_copies(GpuMemoryInBytes(4100)),
            GpuMemoryInBytes(4100),
            "映射着建的缓冲另带一份暂存"
        );
    }

    #[test]
    fn copies_on_the_gpu_checked_bytes_of_the_copies_refuse_past_u64() {
        assert_eq!(
            CopiesOnTheGpu(2).checked_bytes_of_the_copies(GpuMemoryInBytes(7)),
            Some(GpuMemoryInBytes(14))
        );
        assert_eq!(
            CopiesOnTheGpu(2).checked_bytes_of_the_copies(GpuMemoryInBytes(u64::MAX / 2 + 1)),
            None
        );
    }

    #[test]
    #[should_panic(expected = "份数乘每份的显存字节数超出 u64")]
    fn copies_on_the_gpu_bytes_of_the_copies_past_u64_panics() {
        let _bytes = CopiesOnTheGpu(2).bytes_of_the_copies(GpuMemoryInBytes(u64::MAX));
    }

    #[test]
    fn entry_count_of_a_chunk_times_the_bytes_of_one_entry_gives_the_bytes_of_the_entries() {
        assert_eq!(
            EntryCountOfAChunk(5).bytes_of_the_entries(GpuMemoryInBytes(12)),
            GpuMemoryInBytes(60)
        );
        assert_eq!(
            EntryCountOfAChunk::of_this_many_entries(7),
            EntryCountOfAChunk(7)
        );
    }

    #[test]
    #[should_panic(expected = "条数乘每条的显存字节数超出 u64")]
    fn entry_count_of_a_chunk_bytes_of_the_entries_past_u64_panics() {
        let _bytes = EntryCountOfAChunk(2).bytes_of_the_entries(GpuMemoryInBytes(u64::MAX));
    }

    #[test]
    fn entry_count_of_a_chunk_times_the_input_bytes_of_one_entry_gives_the_input_bytes_of_the_entries(
    ) {
        assert_eq!(
            EntryCountOfAChunk(3).input_bytes_of_the_entries(InputLengthOnTheHostInBytes(16)),
            InputLengthOnTheHostInBytes(48)
        );
    }

    #[test]
    #[should_panic(expected = "条数乘每条的输入长度超出 u64")]
    fn entry_count_of_a_chunk_input_bytes_of_the_entries_past_u64_panics() {
        let _input_bytes =
            EntryCountOfAChunk(2).input_bytes_of_the_entries(InputLengthOnTheHostInBytes(u64::MAX));
    }

    #[test]
    fn input_length_on_the_host_of_this_many_bytes_is_the_slice_length() {
        assert_eq!(
            InputLengthOnTheHostInBytes::of_this_many_bytes(4097),
            InputLengthOnTheHostInBytes(4097)
        );
    }

    #[test]
    fn input_length_on_the_host_checked_add_gives_the_sum_and_refuses_past_u64() {
        assert_eq!(
            InputLengthOnTheHostInBytes(628).checked_add(InputLengthOnTheHostInBytes(100)),
            Some(InputLengthOnTheHostInBytes(728))
        );
        assert_eq!(
            InputLengthOnTheHostInBytes(u64::MAX).checked_add(InputLengthOnTheHostInBytes(1)),
            None
        );
    }

    #[test]
    fn input_length_on_the_host_plus_gives_the_sum() {
        assert_eq!(
            InputLengthOnTheHostInBytes(628).plus(InputLengthOnTheHostInBytes(484)),
            InputLengthOnTheHostInBytes(1112)
        );
    }

    #[test]
    #[should_panic(expected = "主机上的输入长度之和超出 u64")]
    fn input_length_on_the_host_plus_past_u64_panics() {
        let _sum = InputLengthOnTheHostInBytes(u64::MAX).plus(InputLengthOnTheHostInBytes(1));
    }

    #[test]
    fn input_length_on_the_host_minus_gives_what_is_left() {
        assert_eq!(
            InputLengthOnTheHostInBytes(968).minus(InputLengthOnTheHostInBytes(484)),
            InputLengthOnTheHostInBytes(484)
        );
        assert_eq!(
            PieceOfAnEntry {
                entry_position: 0,
                input_bytes: InputLengthOnTheHostInBytes(3872)..InputLengthOnTheHostInBytes(4096),
            }
            .input_byte_count(),
            InputLengthOnTheHostInBytes(224),
            "一片罩住的字节数是终点扣起点"
        );
    }

    #[test]
    #[should_panic(expected = "扣掉的输入长度比剩下的长")]
    fn input_length_on_the_host_minus_more_than_there_is_panics() {
        let _left = InputLengthOnTheHostInBytes(484).minus(InputLengthOnTheHostInBytes(968));
    }

    #[test]
    fn input_lengths_on_the_host_summed_over_the_pieces_are_their_total() {
        let total: InputLengthOnTheHostInBytes =
            lengths_on_the_host(&[484, 484, 224]).into_iter().sum();
        assert_eq!(total, InputLengthOnTheHostInBytes(1192));
    }

    #[test]
    fn input_buffers_round_up_to_whole_words_and_never_have_zero_bytes() {
        assert_eq!(
            input_buffer_bytes(InputLengthOnTheHostInBytes(0)),
            GpuMemoryInBytes(4)
        );
        assert_eq!(
            input_buffer_bytes(InputLengthOnTheHostInBytes(1)),
            GpuMemoryInBytes(4)
        );
        assert_eq!(
            input_buffer_bytes(InputLengthOnTheHostInBytes(4)),
            GpuMemoryInBytes(4)
        );
        assert_eq!(
            input_buffer_bytes(InputLengthOnTheHostInBytes(5)),
            GpuMemoryInBytes(8)
        );
        assert_eq!(
            input_buffer_bytes(InputLengthOnTheHostInBytes(4097)),
            GpuMemoryInBytes(4100)
        );
    }

    #[test]
    fn without_a_memory_limit_a_batch_is_one_chunk() {
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[4096, 4096, 7, 0]),
                FOOTPRINT_UNDER_TEST,
                bounds(None)
            ),
            Ok(std::iter::once(0..4).collect::<Vec<_>>()),
            "不给上限就不设限：整批一段"
        );
    }

    #[test]
    fn the_batch_that_does_not_fit_the_memory_limit_is_cut_into_chunks_that_each_fit() {
        let lengths = [100_u64, 100, 100, 100, 100];
        // 两条一段：输入 200 字节连同暂存 400、条目 80、参数 16，合 496；三条要 736
        let memory_limit_in_bytes = 500;
        let chunks = chunks_that_fit_on_the_gpu(
            &lengths_on_the_host(&lengths),
            FOOTPRINT_UNDER_TEST,
            bounds(Some(memory_limit_in_bytes)),
        )
        .expect("单条 256 字节装得下");
        assert_eq!(chunks, vec![0..2, 2..4, 4..5], "两条一段，最后剩一条");
        for chunk in &chunks {
            let input_bytes = InputLengthOnTheHostInBytes(lengths[chunk.clone()].iter().sum());
            let held = FOOTPRINT_UNDER_TEST.bytes_held_by_a_chunk(
                input_bytes,
                EntryCountOfAChunk::of_this_many_entries(chunk.len()),
            );
            assert!(
                held <= GpuMemoryInBytes(memory_limit_in_bytes),
                "每一段合占 {held:?} 字节不超过上限 {memory_limit_in_bytes}"
            );
        }
    }

    #[test]
    fn the_chunk_exactly_at_the_memory_limit_is_kept_whole() {
        // 两条 100 字节：2 × 200 + 80 + 16 = 496，正好等于上限也算装得下
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[100, 100]),
                FOOTPRINT_UNDER_TEST,
                bounds(Some(496))
            ),
            Ok(std::iter::once(0..2).collect::<Vec<_>>())
        );
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[100, 100]),
                FOOTPRINT_UNDER_TEST,
                bounds(Some(495))
            ),
            Ok(vec![0..1, 1..2]),
            "少一个字节就切开"
        );
    }

    #[test]
    fn the_dispatch_and_binding_limits_also_cut_chunks() {
        let dispatch_bound = ChunkBoundsOfAGpu {
            memory_limit: None,
            largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
            largest_entry_count_per_dispatch: EntryCountOfAChunk(3),
        };
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[1; 7]),
                FOOTPRINT_UNDER_TEST,
                dispatch_bound
            ),
            Ok(vec![0..3, 3..6, 6..7]),
            "一次派发最多 3 条"
        );
        let binding_bound = ChunkBoundsOfAGpu {
            memory_limit: None,
            largest_storage_binding_in_bytes: GpuMemoryInBytes(8),
            largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
        };
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[4, 4, 4]),
                FOOTPRINT_UNDER_TEST,
                binding_bound
            ),
            Err(EntryLargerThanAChunkOnTheGpu {
                entry_position: 0,
                input_bytes: InputLengthOnTheHostInBytes(4)
            }),
            "一条在按条排的缓冲里就占 16 字节，超过 8 字节的绑定上限"
        );
    }

    /// 两段同时在一张卡上（第 3 条：全部后台任务在那张卡上合占的显存不超过上限）：另一段先占着 600 字节，这一段再切时只用剩下的 400 字节，
    /// 两段合占任何时刻都不超过上限 1000。
    #[test]
    fn two_segments_held_on_one_gpu_together_stay_within_its_memory_limit() {
        let memory_limit_in_bytes = 1000;
        let bounds_of_the_gpu = bounds(Some(memory_limit_in_bytes));
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
            GpuMemoryInBytes(memory_limit_in_bytes),
        ));
        // 先占着的那一段：每段另占 200 字节、每条另占 50 字节的四条 25 字节，合 2 × 100 + 200 + 200 = 600
        let footprint_of_the_other_segment = ChunkFootprint {
            bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(200),
            bytes_per_entry_besides_its_input: GpuMemoryInBytes(50),
            bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(50),
        };
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[25, 25, 25, 25]),
                footprint_of_the_other_segment,
                bounds_of_the_gpu.with_the_room_left_after(account.bytes_held()),
            ),
            Ok(std::iter::once(0..4).collect::<Vec<_>>())
        );
        let held_by_the_other_segment = footprint_of_the_other_segment
            .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(100), EntryCountOfAChunk(4));
        assert_eq!(held_by_the_other_segment, GpuMemoryInBytes(600));
        account.hold(held_by_the_other_segment);

        let lengths = [50_u64, 50, 50, 50, 50];
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&lengths),
                FOOTPRINT_UNDER_TEST,
                bounds_of_the_gpu
            ),
            Ok(std::iter::once(0..5).collect::<Vec<_>>()),
            "单看这一段，五条合 720 字节装得进 1000：不扣另一段占的就整批一段，两段合占 1320 字节"
        );
        let chunks = chunks_that_fit_on_the_gpu(
            &lengths_on_the_host(&lengths),
            FOOTPRINT_UNDER_TEST,
            bounds_of_the_gpu.with_the_room_left_after(account.bytes_held()),
        )
        .expect("单条 160 字节装得进剩下的 400 字节");
        assert_eq!(
            chunks,
            vec![0..2, 2..4, 4..5],
            "只用剩下的 400 字节：两条 296 一段，三条 440 装不下"
        );
        for chunk in &chunks {
            let input_bytes = InputLengthOnTheHostInBytes(lengths[chunk.clone()].iter().sum());
            let held_by_this_chunk = FOOTPRINT_UNDER_TEST.bytes_held_by_a_chunk(
                input_bytes,
                EntryCountOfAChunk::of_this_many_entries(chunk.len()),
            );
            account.hold(held_by_this_chunk);
            assert!(
                account.bytes_held() <= GpuMemoryInBytes(memory_limit_in_bytes),
                "两段合占 {:?} 字节不超过上限 {memory_limit_in_bytes}",
                account.bytes_held()
            );
            account.release(held_by_this_chunk);
        }
        assert_eq!(
            account.bytes_held(),
            held_by_the_other_segment,
            "这一段交完放掉，另一段的照旧占着"
        );
    }

    #[test]
    fn the_room_left_after_other_segments_without_a_memory_limit_is_still_unlimited() {
        assert_eq!(
            bounds(None).with_the_room_left_after(GpuMemoryInBytes(1 << 40)),
            bounds(None),
            "不给上限就不设限，别的段占多少都一样"
        );
        assert_eq!(
            bounds(Some(100))
                .with_the_room_left_after(GpuMemoryInBytes(150))
                .memory_limit,
            Some(GpuMemoryInBytes(0)),
            "已占的比上限多：一字节都不剩"
        );
    }

    #[test]
    fn an_entry_that_alone_exceeds_the_memory_limit_refuses_the_batch() {
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[10, 4096, 10]),
                FOOTPRINT_UNDER_TEST,
                bounds(Some(1024))
            ),
            Err(EntryLargerThanAChunkOnTheGpu {
                entry_position: 1,
                input_bytes: InputLengthOnTheHostInBytes(4096)
            })
        );
    }

    #[test]
    fn an_empty_batch_has_no_chunk() {
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[]),
                FOOTPRINT_UNDER_TEST,
                bounds(Some(64))
            ),
            Ok(Vec::new())
        );
    }

    #[test]
    fn holding_up_to_the_memory_limit_is_allowed_and_releasing_gives_it_back() {
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
            GpuMemoryInBytes(100),
        ));
        account.hold(GpuMemoryInBytes(60));
        account.hold(GpuMemoryInBytes(40));
        assert_eq!(account.bytes_held(), GpuMemoryInBytes(100), "正好到上限");
        account.release(GpuMemoryInBytes(100));
        account.hold(GpuMemoryInBytes(100));
        assert_eq!(
            account.bytes_held(),
            GpuMemoryInBytes(100),
            "放掉之后又能占满"
        );
    }

    #[test]
    #[should_panic(expected = "超过了上限")]
    fn holding_past_the_memory_limit_panics() {
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
            GpuMemoryInBytes(100),
        ));
        account.hold(GpuMemoryInBytes(60));
        account.hold(GpuMemoryInBytes(41));
    }

    #[test]
    fn without_a_memory_limit_holding_is_never_refused() {
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(None);
        account.hold(GpuMemoryInBytes(u64::MAX / 2));
        assert_eq!(account.bytes_held(), GpuMemoryInBytes(u64::MAX / 2));
    }

    /// 整条都装得下的批：每条一片，切出的段与不切单条的切法相同。
    #[test]
    fn the_batch_whose_entries_each_fit_is_cut_like_the_chunks_with_one_whole_piece_per_entry() {
        let lengths = [100_u64, 100, 100, 100, 100];
        let pieces = pieces_cut_to_fit_on_the_gpu(
            &lengths_on_the_host(&lengths),
            FOOTPRINT_UNDER_TEST,
            bounds(Some(500)),
        )
        .expect("单条 256 字节装得下");
        assert_eq!(
            pieces,
            vec![
                vec![
                    PieceOfAnEntry {
                        entry_position: 0,
                        input_bytes: InputLengthOnTheHostInBytes(0)
                            ..InputLengthOnTheHostInBytes(100)
                    },
                    PieceOfAnEntry {
                        entry_position: 1,
                        input_bytes: InputLengthOnTheHostInBytes(0)
                            ..InputLengthOnTheHostInBytes(100)
                    }
                ],
                vec![
                    PieceOfAnEntry {
                        entry_position: 2,
                        input_bytes: InputLengthOnTheHostInBytes(0)
                            ..InputLengthOnTheHostInBytes(100)
                    },
                    PieceOfAnEntry {
                        entry_position: 3,
                        input_bytes: InputLengthOnTheHostInBytes(0)
                            ..InputLengthOnTheHostInBytes(100)
                    }
                ],
                vec![PieceOfAnEntry {
                    entry_position: 4,
                    input_bytes: InputLengthOnTheHostInBytes(0)..InputLengthOnTheHostInBytes(100)
                }],
            ]
        );
    }

    /// 装不下自适应分片：一条 4096 字节的单独成一段都装不下 1024 字节的上限，切成每片 484 字节
    /// （1024 扣掉一段 16、一条 40 剩 968，输入连同暂存占两份）的几片，最后一片是余下的；前后装得下的整条照常，片按批里的次序、一条之内按字节的次序排，
    /// 每段都不超上限。
    #[test]
    fn an_entry_larger_than_the_memory_limit_is_cut_into_pieces_that_each_fit() {
        let lengths = [10_u64, 4096, 10];
        let memory_limit_in_bytes = 1024;
        let chunks = pieces_cut_to_fit_on_the_gpu(
            &lengths_on_the_host(&lengths),
            FOOTPRINT_UNDER_TEST,
            bounds(Some(memory_limit_in_bytes)),
        )
        .expect("最小的一片装得下");
        let pieces_in_order: Vec<(usize, std::ops::Range<u64>)> = chunks
            .iter()
            .flatten()
            .map(|piece| {
                (
                    piece.entry_position,
                    piece.input_bytes.start.0..piece.input_bytes.end.0,
                )
            })
            .collect();
        assert_eq!(
            pieces_in_order,
            vec![
                (0, 0..10),
                (1, 0..484),
                (1, 484..968),
                (1, 968..1452),
                (1, 1452..1936),
                (1, 1936..2420),
                (1, 2420..2904),
                (1, 2904..3388),
                (1, 3388..3872),
                (1, 3872..4096),
                (2, 0..10),
            ],
            "4096 字节切成八片 484 与一片 224"
        );
        for chunk in &chunks {
            let input_bytes: InputLengthOnTheHostInBytes =
                chunk.iter().map(PieceOfAnEntry::input_byte_count).sum();
            let held = FOOTPRINT_UNDER_TEST.bytes_held_by_a_chunk(
                input_bytes,
                EntryCountOfAChunk::of_this_many_entries(chunk.len()),
            );
            assert!(
                held <= GpuMemoryInBytes(memory_limit_in_bytes),
                "每一段合占 {held:?} 字节不超过上限"
            );
        }
    }

    /// 连一个字节的一片都装不下：交回要切的那一条与最小的一片合占多少（这张卡至少要剩这么多）。
    #[test]
    fn the_memory_limit_below_the_smallest_piece_reports_how_much_the_smallest_piece_needs() {
        let smallest_piece_bytes = FOOTPRINT_UNDER_TEST
            .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(1), EntryCountOfAChunk(1));
        assert_eq!(smallest_piece_bytes, GpuMemoryInBytes(2 * 4 + 40 + 16));
        assert_eq!(
            pieces_cut_to_fit_on_the_gpu(
                &lengths_on_the_host(&[0, 4096]),
                FOOTPRINT_UNDER_TEST,
                bounds(Some(smallest_piece_bytes.0 - 1))
            ),
            Err(SmallestPieceLargerThanTheRoom {
                entry_position: 0,
                smallest_piece_bytes,
            }),
            "空的那一条也占一个整字，装不下"
        );
        assert!(
            pieces_cut_to_fit_on_the_gpu(
                &lengths_on_the_host(&[4096]),
                FOOTPRINT_UNDER_TEST,
                bounds(Some(smallest_piece_bytes.0))
            )
            .is_ok(),
            "正好装得下最小的一片：切成 1024 片"
        );
    }

    /// 校验和那一段一个 4096 字节的单元：参数、输入、描述与两份结果缓冲都映射着建、各带一份同尺寸的暂存，两份回读暂存不带。
    /// 只算缓冲本身合占 4096 + 12 + 4 × 8 + 16 = 4156 字节；连同暂存合占 8296 字节。上限给 4156（只算缓冲本身正好装得下）时这一条整条装不下，
    /// 切成三片（剩下的 4052 字节分给输入与它的暂存两份，每片至多 2024 字节），每一段连同暂存都不超上限；上限给 8296 整条一段，少一个字节就装不下。
    #[test]
    fn checksum_chunk_fitting_without_the_staging_but_not_with_it_is_cut_into_pieces_that_fit_with_the_staging(
    ) {
        use crate::checksum_kernels::CHECKSUM_CHUNK_FOOTPRINT;
        let bytes_held_with_the_staging = CHECKSUM_CHUNK_FOOTPRINT
            .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(4096), EntryCountOfAChunk(1));
        assert_eq!(
            bytes_held_with_the_staging,
            GpuMemoryInBytes(2 * (16 + 4096 + 12 + 2 * 8) + 2 * 8),
            "参数、输入、描述、两份结果各两份，两份回读各一份"
        );
        let bytes_held_without_the_staging = 4096 + 12 + 4 * 8 + 16;
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[4096]),
                CHECKSUM_CHUNK_FOOTPRINT,
                bounds(Some(bytes_held_without_the_staging))
            ),
            Err(EntryLargerThanAChunkOnTheGpu {
                entry_position: 0,
                input_bytes: InputLengthOnTheHostInBytes(4096)
            }),
            "只算缓冲本身正好装得下的上限，连同暂存装不下"
        );
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[4096]),
                CHECKSUM_CHUNK_FOOTPRINT,
                bounds(Some(bytes_held_with_the_staging.0))
            ),
            Ok(std::iter::once(0..1).collect::<Vec<_>>()),
            "连同暂存正好装得下：整条一段"
        );
        assert_eq!(
            chunks_that_fit_on_the_gpu(
                &lengths_on_the_host(&[4096]),
                CHECKSUM_CHUNK_FOOTPRINT,
                bounds(Some(bytes_held_with_the_staging.0 - 1))
            ),
            Err(EntryLargerThanAChunkOnTheGpu {
                entry_position: 0,
                input_bytes: InputLengthOnTheHostInBytes(4096)
            }),
            "少一个字节就装不下"
        );
        let chunks = pieces_cut_to_fit_on_the_gpu(
            &lengths_on_the_host(&[4096]),
            CHECKSUM_CHUNK_FOOTPRINT,
            bounds(Some(bytes_held_without_the_staging)),
        )
        .expect("最小的一片连同暂存装得进 4156 字节的上限");
        let pieces_in_order: Vec<std::ops::Range<u64>> = chunks
            .iter()
            .flatten()
            .map(|piece| piece.input_bytes.start.0..piece.input_bytes.end.0)
            .collect();
        assert_eq!(
            pieces_in_order,
            vec![0..2024, 2024..4048, 4048..4096],
            "4096 字节切成两片 2024 与一片 48"
        );
        for chunk in &chunks {
            let input_bytes: InputLengthOnTheHostInBytes =
                chunk.iter().map(PieceOfAnEntry::input_byte_count).sum();
            let held = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
                input_bytes,
                EntryCountOfAChunk::of_this_many_entries(chunk.len()),
            );
            assert!(
                held <= GpuMemoryInBytes(bytes_held_without_the_staging),
                "每一段连同暂存合占 {held:?} 字节不超过上限"
            );
        }
    }

    /// 冲掉待写的那一次空提交。
    const FLUSHING_SUBMISSION: SubmissionNamedByTheTest = SubmissionNamedByTheTest(2);

    fn out_of_memory_creating_the_buffers_of_the_chunk() -> GpuFailureCause {
        GpuFailureCause::OutOfMemoryCreatingTheBuffers {
            bytes_held_by_the_chunk: BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            reason: "用例造的显存不足".to_string(),
        }
    }

    /// 建缓冲显存不足之后，冲掉待写的那一次空提交在上限之内做完了：这一段（缓冲连同暂存）的账放掉，交回显存不足（外壳照它重切）。
    #[test]
    fn the_chunk_whose_buffers_were_not_created_is_released_once_the_flush_of_the_pending_writes_completes(
    ) {
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
            GpuMemoryInBytes(MEMORY_LIMIT_UNDER_TEST_IN_BYTES),
        ));
        account.hold(BYTES_HELD_BY_THE_TIMED_OUT_CHUNK);
        let reported = account.settle_the_chunk_whose_buffers_were_not_created(
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            out_of_memory_creating_the_buffers_of_the_chunk(),
            FLUSHING_SUBMISSION,
            Ok(()),
        );
        assert_eq!(
            reported,
            out_of_memory_creating_the_buffers_of_the_chunk(),
            "交回建缓冲那一次的错"
        );
        assert_eq!(
            account.bytes_held(),
            GpuMemoryInBytes(0),
            "冲掉待写的提交做完了才放"
        );
        assert!(!account.has_a_submission_running_past_its_wait());
    }

    /// 冲掉待写的那一次空提交在上限之内没做完：已建的缓冲与暂存还挂在待写里占着卡上的显存，这一段的账照旧记着、记成还在跑的那次提交，
    /// 交回冲掉待写等满了上限的那一种（外壳照它先等那次提交、再重交；算超时、不算交到了卡上），不放账也不交回显存不足。
    #[test]
    fn the_chunk_whose_buffers_were_not_created_stays_held_while_the_flush_of_the_pending_writes_is_running(
    ) {
        let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
            GpuMemoryInBytes(MEMORY_LIMIT_UNDER_TEST_IN_BYTES),
        ));
        account.hold(BYTES_HELD_BY_THE_TIMED_OUT_CHUNK);
        let waited_past_the_limit = GpuFailureCause::WaitForTheGpuExceededTheLimit {
            wait_limit: WAIT_LIMIT_UNDER_TEST,
        };
        let reported = account.settle_the_chunk_whose_buffers_were_not_created(
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            out_of_memory_creating_the_buffers_of_the_chunk(),
            FLUSHING_SUBMISSION,
            Err(waited_past_the_limit),
        );
        assert_eq!(
            reported,
            GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit {
                wait_limit: WAIT_LIMIT_UNDER_TEST,
            },
            "交回冲掉待写等满了上限的那一种"
        );
        assert_eq!(
            account.bytes_held(),
            BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
            "没等到做完：账照旧记着"
        );
        assert_eq!(
            account
                .submission_running_past_its_wait
                .as_ref()
                .map(|running| (running.submission, running.bytes_held_by_its_chunk)),
            Some((FLUSHING_SUBMISSION, BYTES_HELD_BY_THE_TIMED_OUT_CHUNK)),
            "记成还在跑的那一次空提交"
        );
    }
}
