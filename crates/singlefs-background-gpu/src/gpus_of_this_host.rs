//! 挂载那一刻开着的本机全部独显里，此刻照哪一组设置用哪几张、每张怎么切、等多久（D26（后台整理与放置回收） 已定项 13 第 1–3 条；
//! `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 43 批交回那一行的定法：挂载那一刻开全部独显并量代价，设置决定哪几张参与，切换不重开）。
//! 纯逻辑，不碰 GPU：外壳（`crate::background_gpu`）每张卡一份 [`GpuBudgetUnderTheSettings`]，参与的卡由 [`ParticipatingGpusOnThisHost`] 定；
//! 核心层交来一组新设置（`BackgroundGpuCompute::use_the_settings_from_the_next_batch`，只在两批之间）就换这两样，从下一批起照它分活、切段、等。
//!
//! 挂载那一刻一张卡没开成挡不挡挂载（第 2、4 条与同一份记录第 45 批交回那一行的定法）在 [`gpus_opened_at_the_mount`]：参与的卡没开成就拒，
//! 给了 `background_gpu_devices` 而没点名的卡没开成就跳过、报出来（[`DiscreteGpusAfterTheOpening`]），卡清单照列它、并报它没开成，
//! 核心层挂着切换点到它照第 4 条拒（校验同挂载那一刻）。
//!
//! 一批的活按卡的次序填满（用户 2026-11-02 定「按卡的顺序填满」）：参与的卡排成参与的次序，前面的卡先填到它照显存上限切片之后一次能放下的量，
//! 装不下的才给后面的卡，排在最后的那张拿剩下的全部（[`plans_of_a_batch_filling_the_gpus_in_order`]）；挂载时量的代价照量、不用来分活。
//!
//! 换设置不清账：一张卡上等超过上限、还在 GPU 上跑的那次提交照旧记着，下一批在这张卡上交活之前先在新的等的上限之内等它做完，
//! 再按新的显存上限扣掉还占着的切（第 3 条）；连续不确定的计数不在这里，换设置不碰它（第 8 条）。
//!
//! 显存上限连最小的一片都装不下的卡不用（用户 2026-10-28 定「连一片都装不下就不用这张卡」，原话在 `records/2026-10-19-里程碑四GPU垃圾回收开工.md`
//! 末尾「用户定案（弹窗两次）」那一行）：挂载那一刻与每次换设置都照 [`bytes_of_the_smallest_piece_every_participating_gpu_holds`] 判，
//! 装不下的卡不参与、报出来（[`GpuLeftOutUnderItsMemoryLimit`]，核心层定的那一份），挂载不因此拒绝；参与的卡因此一张都不剩时，三段每一批都交回
//! 「参与的卡一张都不剩」，核心层挂载那一刻就读到、当场报警停着（`singlefs_core::background_compute::BackgroundGpuCompute::failure_under_the_settings_handed_in`）。量代价那一段按开卡时的上限切成几片逐片交，代价取各片之和
//! （[`measured_cost_summed_over_the_pieces`]）。
//!
//! 丢后台 GPU 时不等还在 GPU 上跑的提交（同一定案「卸载时同样不等、泄漏设备」）：[`gpus_dropped_leaking_those_still_running`]。

use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use singlefs_core::background_compute::{
    AdmittedGpuSettings, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuInventory,
    GpuMemoryInBytes,
};

use crate::adapter_inventory::{DiscreteGpuOnThisHost, ParticipatingGpu};
use crate::background_gpu::{
    BackgroundGpuOpenRefused, GpuFailureCause, SmallestPieceNotHeldOnTheGpu,
};
use crate::background_gpu_compute::core_failure_of_the_gpu;
use crate::checksum_kernels::bytes_of_the_smallest_checksum_piece;
use crate::live_slot_kernels::bytes_of_the_smallest_live_slot_piece;
use crate::measured_host_cpu_time_per_submission::HostCpuNanosecondsPerSubmission;
use crate::memory_limit_and_chunks::{
    pieces_cut_to_fit_on_the_gpu, ChunkBoundsOfAGpu, ChunkFootprint, ChunksNotCutOnTheGpu,
    GpuMemoryAccount, PieceOfAnEntry, SmallestPieceLargerThanTheRoom,
};
use crate::region_record_kernels::bytes_of_the_smallest_region_record_piece;
use crate::waiting_for_the_gpu::{CompletionCheck, GpuWaitDeadline, MonotonicClock};

/// 一张卡此刻照的设置管的那几样：显存上限（记在账上）、等 GPU 的上限，连同与设置无关的绑定上限与派发上限、本包在它上面的账。
#[derive(Debug)]
pub struct GpuBudgetUnderTheSettings<Submission> {
    largest_storage_binding_in_bytes: u64,
    largest_entry_count_per_dispatch: u64,
    memory_account: GpuMemoryAccount<Submission>,
    wait_limit: Duration,
    /// 开卡那一刻这个进程里此前几次挂载泄漏在这张卡上的字节（账从它起）：丢的时候扣掉它，算这一次挂载自己泄漏的（[`GpuLeakedWhenDropped`]）。
    bytes_leaked_by_earlier_mounts: u64,
}

impl<Submission> GpuBudgetUnderTheSettings<Submission> {
    /// 开卡那一刻：一个字节都没占，照开卡时给的上限。
    #[must_use]
    pub fn with_nothing_held(
        largest_storage_binding_in_bytes: u64,
        largest_entry_count_per_dispatch: u64,
        memory_limit: Option<GpuMemoryInBytes>,
        wait_limit: Duration,
    ) -> Self {
        Self::with_the_bytes_leaked_by_earlier_mounts(
            largest_storage_binding_in_bytes,
            largest_entry_count_per_dispatch,
            memory_limit,
            wait_limit,
            0,
        )
    }

    /// 开卡那一刻，这个进程里此前几次挂载在这张卡上泄漏了 `bytes_leaked_by_earlier_mounts` 字节（[`GpuMemoryLeakedInThisProcess`]）：
    /// 账从它起、一直记着，这张卡的可用余量先扣掉它（第 3 条：泄漏的那一片仍占着这张卡）。
    #[must_use]
    pub fn with_the_bytes_leaked_by_earlier_mounts(
        largest_storage_binding_in_bytes: u64,
        largest_entry_count_per_dispatch: u64,
        memory_limit: Option<GpuMemoryInBytes>,
        wait_limit: Duration,
        bytes_leaked_by_earlier_mounts: u64,
    ) -> Self {
        Self {
            largest_storage_binding_in_bytes,
            largest_entry_count_per_dispatch,
            memory_account: GpuMemoryAccount::with_the_bytes_leaked_by_earlier_mounts(
                memory_limit,
                bytes_leaked_by_earlier_mounts,
            ),
            wait_limit,
            bytes_leaked_by_earlier_mounts,
        }
    }

    /// 此刻还能切一段的界：显存上限扣掉此刻还占着的（泄漏的、等超过上限还在跑的那一次提交的）。
    #[must_use]
    pub fn room_left_now(&self) -> ChunkBoundsOfAGpu {
        self.chunk_bounds()
            .with_the_room_left_after(self.memory_account.bytes_held())
    }

    /// 换上一组新设置给这张卡的显存上限与等的上限（两批之间调）。账照旧记着。
    pub fn use_the_settings_from_the_next_batch(
        &mut self,
        memory_limit: Option<GpuMemoryInBytes>,
        wait_limit: Duration,
    ) {
        self.memory_account
            .use_the_memory_limit_from_the_next_batch(memory_limit);
        self.wait_limit = wait_limit;
    }

    /// 切这张卡的段用的界：显存上限取此刻设置给的（还没扣已占的）。
    #[must_use]
    pub fn chunk_bounds(&self) -> ChunkBoundsOfAGpu {
        ChunkBoundsOfAGpu {
            memory_limit: self.memory_account.memory_limit(),
            largest_storage_binding_in_bytes: self.largest_storage_binding_in_bytes,
            largest_entry_count_per_dispatch: self.largest_entry_count_per_dispatch,
        }
    }

    #[must_use]
    pub fn wait_limit(&self) -> Duration {
        self.wait_limit
    }

    /// 开卡那一刻已经算上的、这个进程里此前几次挂载泄漏在这张卡上的字节。
    #[must_use]
    pub fn bytes_leaked_by_earlier_mounts(&self) -> u64 {
        self.bytes_leaked_by_earlier_mounts
    }

    #[must_use]
    pub fn memory_account(&self) -> &GpuMemoryAccount<Submission> {
        &self.memory_account
    }

    pub fn memory_account_mut(&mut self) -> &mut GpuMemoryAccount<Submission> {
        &mut self.memory_account
    }

    /// 交活之前调：前一次等超过上限、还在跑的那次提交先在此刻设置给的等的上限之内等它做完、放掉它的账，交回此刻剩下的界
    /// （`GpuMemoryAccount::room_once_the_submission_running_past_its_wait_completed`）。
    ///
    /// # Errors
    /// 等不到它做完 ⇒ [`GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit`]（等的时候报的出错照原样），账照旧记着。
    pub fn room_once_the_submission_running_past_its_wait_completed(
        &mut self,
        clock: &impl MonotonicClock,
        check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
    ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
        let deadline = GpuWaitDeadline::starting_now(self.wait_limit, clock);
        let bounds = self.chunk_bounds();
        self.memory_account
            .room_once_the_submission_running_past_its_wait_completed(
                &deadline,
                clock,
                check_once_waiting_at_most,
                bounds,
            )
    }

    /// 重交之前调：前一次等超过上限、还在跑的那次提交在此刻设置给的等的上限之内等它做完、放掉它的账；没有就当场交回。
    ///
    /// # Errors
    /// 同 [`Self::room_once_the_submission_running_past_its_wait_completed`]。
    pub fn wait_for_the_submission_running_past_its_wait(
        &mut self,
        clock: &impl MonotonicClock,
        check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
    ) -> Result<(), GpuFailureCause> {
        let deadline = GpuWaitDeadline::starting_now(self.wait_limit, clock);
        self.memory_account
            .wait_for_the_submission_running_past_its_wait(
                &deadline,
                clock,
                check_once_waiting_at_most,
            )
    }

    /// 这张卡分到的几条（每条的输入字节数）切段，交这几段之前调：前一次等超过上限、还在跑的那次提交先在此刻设置给的等的上限之内等它做完、
    /// 放掉它的账，再按此刻设置给的显存上限扣掉还占着的切（`GpuMemoryAccount::chunks_cut_once_the_submission_running_past_its_wait_completed`）。
    ///
    /// # Errors
    /// 同 `GpuMemoryAccount::chunks_cut_once_the_submission_running_past_its_wait_completed`。
    pub fn chunks_cut_once_the_submission_running_past_its_wait_completed(
        &mut self,
        clock: &impl MonotonicClock,
        check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
        entry_input_lengths_in_bytes: &[u64],
        footprint: ChunkFootprint,
    ) -> Result<Vec<Range<usize>>, ChunksNotCutOnTheGpu> {
        let deadline = GpuWaitDeadline::starting_now(self.wait_limit, clock);
        let bounds = self.chunk_bounds();
        self.memory_account
            .chunks_cut_once_the_submission_running_past_its_wait_completed(
                &deadline,
                clock,
                check_once_waiting_at_most,
                entry_input_lengths_in_bytes,
                footprint,
                bounds,
            )
    }
}

/// 此刻参与的卡：在本机全部独显（按总线地址排的那一份）里的位置，按参与的次序（设置点名的次序，没点名是卡清单的次序）。不重；
/// 设置点的卡都因显存上限装不下最小的一片而不用时一张都没有（[`Self::leaving_out`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipatingGpusOnThisHost {
    positions_on_this_host: Vec<usize>,
}

impl ParticipatingGpusOnThisHost {
    /// 核心层交来的那一组里参与的卡，在 `identifiers_on_this_host`（挂载那一刻开成的本机独显，按总线地址排）里找到各自的位置。
    ///
    /// # Panics
    /// 有一张参与的卡不在开成的独显里：核心层照这份实现答的卡清单（本机全部独显，连同它报的没开成的那几张）校验过这一组，
    /// 点名不在本机的、参与的卡里有没开成的都拒了。
    #[must_use]
    pub fn of_the_settings(
        settings: &AdmittedGpuSettings,
        identifiers_on_this_host: &[GpuDeviceIdentifier],
    ) -> Self {
        let positions_on_this_host: Vec<usize> = settings
            .participating_devices()
            .iter()
            .map(|participating| {
                identifiers_on_this_host
                    .iter()
                    .position(|on_this_host| *on_this_host == participating.device)
                    .expect("核心层照卡清单校验过：参与的卡都在本机、都不是挂载那一刻没开成的")
            })
            .collect();
        assert!(
            !positions_on_this_host.is_empty(),
            "核心层校验过的一组至少一张卡"
        );
        Self {
            positions_on_this_host,
        }
    }

    /// 开卡那一刻照开卡那一组定的参与的卡（位置按参与的次序；显存上限装不下最小的一片的卡已经不在里面，可以一张都没有）。
    #[must_use]
    pub fn of_the_positions(positions_on_this_host: Vec<usize>) -> Self {
        Self {
            positions_on_this_host,
        }
    }

    /// 去掉 `is_left_out` 判成不用的那几张（显存上限装不下最小的一片），次序不变；可以一张都不剩。
    #[must_use]
    pub fn leaving_out(self, is_left_out: impl Fn(usize) -> bool) -> Self {
        Self {
            positions_on_this_host: self
                .positions_on_this_host
                .into_iter()
                .filter(|position| !is_left_out(*position))
                .collect(),
        }
    }

    #[must_use]
    pub fn positions_on_this_host(&self) -> &[usize] {
        &self.positions_on_this_host
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.positions_on_this_host.is_empty()
    }
}

/// 本机的一张独显挂载那一刻没开成（起不了设备、编不了管线、量代价那几次提交出错或超时）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscreteGpuNotOpenedAtTheMount {
    pub discrete_gpu: DiscreteGpuOnThisHost,
    pub cause: GpuFailureCause,
}

impl DiscreteGpusAfterTheOpening {
    /// 只留 `identifiers_present_now`（挂着切换那一刻现查的那一次枚举里在的独显）里在的：挂载之后掉了的卡不再算本机的卡，切换点它照「点名的卡不存在」拒，
    /// 不点名时它不再参与（第 4 条「切换那一刻……GPU 不在……就拒绝切换」）。
    #[must_use]
    pub fn still_present(&self, identifiers_present_now: &[GpuDeviceIdentifier]) -> Self {
        Self {
            opened: self
                .opened
                .iter()
                .filter(|opened| identifiers_present_now.contains(&opened.identifier))
                .cloned()
                .collect(),
            not_opened: self
                .not_opened
                .iter()
                .filter(|not_opened| {
                    identifiers_present_now.contains(&not_opened.discrete_gpu.identifier)
                })
                .cloned()
                .collect(),
        }
    }
}

/// 挂载那一刻开卡之后的本机独显：开成的那几张与没开成而跳过的那几张，各按总线地址排。
/// 卡清单（核心层 `GpuInventory`）照它答：本机全部独显连同没开成的（两样合起来按总线地址排，没给 `background_gpu_devices` 时上限按这个次序
/// 一张一个，第 3 条），与其中没开成的那几张、原因照挂着时这张卡出错的报法（`core_failure_of_the_gpu`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscreteGpusAfterTheOpening {
    pub opened: Vec<DiscreteGpuOnThisHost>,
    pub not_opened: Vec<DiscreteGpuNotOpenedAtTheMount>,
}

impl GpuInventory for DiscreteGpusAfterTheOpening {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        let mut every_discrete_gpu: Vec<&DiscreteGpuOnThisHost> = self
            .opened
            .iter()
            .chain(
                self.not_opened
                    .iter()
                    .map(|not_opened| &not_opened.discrete_gpu),
            )
            .collect();
        every_discrete_gpu.sort_by(|left, right| left.pci_bus_address.cmp(&right.pci_bus_address));
        every_discrete_gpu
            .into_iter()
            .map(|discrete_gpu| (discrete_gpu.identifier.clone(), discrete_gpu.total_memory))
            .collect()
    }

    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
        self.not_opened
            .iter()
            .map(|not_opened| GpuDeviceNotOpenedAtTheMount {
                device: not_opened.discrete_gpu.identifier.clone(),
                failure_at_the_mount: core_failure_of_the_gpu(
                    &not_opened.discrete_gpu.identifier,
                    &not_opened.cause,
                ),
            })
            .collect()
    }
}

/// 挂载那一刻量代价交的那一段（外壳的那一段是 `crate::background_gpu::COST_PROBE_CHUNK`）：一条 `input_bytes` 字节的单元，`footprint` 是它那一段在卡上的占用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CostProbeChunk {
    pub input_bytes: u64,
    pub footprint: ChunkFootprint,
}

impl CostProbeChunk {
    /// 整条一段在卡上合占的字节数。
    #[must_use]
    pub fn bytes_held(&self) -> u64 {
        self.footprint.bytes_held_by_a_chunk(self.input_bytes, 1)
    }

    /// 照这张卡此刻的界切这一条：装得下就一段一片，装不下就切成几片、几段（切法与挂着时校验和那一段同一个，[`pieces_cut_to_fit_on_the_gpu`]）。
    ///
    /// # Errors
    /// 连最小的一片都装不下（[`SmallestPieceLargerThanTheRoom`]）。
    pub fn pieces_under_the_bounds(
        &self,
        bounds: ChunkBoundsOfAGpu,
    ) -> Result<Vec<Vec<PieceOfAnEntry>>, SmallestPieceLargerThanTheRoom> {
        pieces_cut_to_fit_on_the_gpu(&[self.input_bytes], self.footprint, bounds)
    }
}

/// 量代价（第 2 条：一次提交的主机 CPU 时间）按几片交时，这张卡的代价取各片之和：先把各段空跑一遍（管线刚编好时驱动的懒初始化不算进去），
/// 再量 `measurements` 轮，每一轮把各段各交一次、`cpu_time_of_submitting` 量出的时间加起来，取各轮之和的中位数。
///
/// 两层循环：外层 `measurements` 轮、内层每轮 `chunks` 段，都有界；跨轮只带量出的那几个和。
///
/// # Errors
/// 任何一次交出错照原样交回（这张卡没量成）。
///
/// # Panics
/// `chunks` 为空或 `measurements` 为 0：量代价那一段至少一段，至少量一轮。
pub fn measured_cost_summed_over_the_pieces<Chunk, Failure>(
    chunks: &[Chunk],
    measurements: usize,
    mut cpu_time_of_submitting: impl FnMut(&Chunk) -> Result<Duration, Failure>,
) -> Result<HostCpuNanosecondsPerSubmission, Failure> {
    assert!(!chunks.is_empty(), "量代价那一段至少切出一段");
    assert!(measurements > 0, "至少量一轮");
    for chunk in chunks {
        cpu_time_of_submitting(chunk)?;
    }
    let mut sums_in_nanoseconds = Vec::with_capacity(measurements);
    for _measurement in 0..measurements {
        let mut sum_of_this_round = Duration::ZERO;
        for chunk in chunks {
            sum_of_this_round += cpu_time_of_submitting(chunk)?;
        }
        sums_in_nanoseconds.push(
            u64::try_from(sum_of_this_round.as_nanos())
                .expect("一轮各段的 CPU 时间之和装得进 u64 纳秒"),
        );
    }
    sums_in_nanoseconds.sort_unstable();
    Ok(HostCpuNanosecondsPerSubmission(
        sums_in_nanoseconds[measurements / 2],
    ))
}

/// 参与的卡显存上限至少要装得下的那一片：三段各自与输入无关、定得下来的最小的一片取最大的那个——校验和那一段（量代价那一段同它）一个字节的单元，
/// 选段打分那一段一个段连同一条记录的记录表，区域扫描那一段切开的单元头一片连同只放一条条目的映射（单元能切开之后它不再随最大那条记录变，
/// 用户 2026-10-28 定「装不下应该自适应的分片，缩小逐次进行」，`crate::region_record_kernels::bytes_of_the_smallest_region_record_piece`）。
/// 区域扫描一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判（[`plans_of_a_batch_filling_the_gpus_in_order`]）。
#[must_use]
pub fn bytes_of_the_smallest_piece_every_participating_gpu_holds() -> u64 {
    bytes_of_the_smallest_checksum_piece()
        .max(bytes_of_the_smallest_live_slot_piece())
        .max(bytes_of_the_smallest_region_record_piece())
}

/// 一张卡因显存上限装不下最小的一片而不用：核心层定的那一份（参与的卡一张都不剩时原样报给核心层），本包照它报。
pub use singlefs_core::background_compute::GpuLeftOutUnderItsMemoryLimit;

/// 给这张卡的显存上限扣掉卡上已经占着的 `bytes_already_held_on_the_device`（这个进程里此前几次挂载泄漏的、等超过上限还在跑的那一次提交的）
/// 装不下 `smallest_piece_bytes` 时交回它不用的那一条；不给上限、装得下都交回 `None`。
#[must_use]
pub fn gpu_left_out_under_its_memory_limit(
    device: &GpuDeviceIdentifier,
    memory_limit: Option<GpuMemoryInBytes>,
    smallest_piece_bytes: u64,
    bytes_already_held_on_the_device: u64,
) -> Option<GpuLeftOutUnderItsMemoryLimit> {
    memory_limit
        .filter(|memory_limit| {
            memory_limit
                .0
                .saturating_sub(bytes_already_held_on_the_device)
                < smallest_piece_bytes
        })
        .map(|memory_limit| GpuLeftOutUnderItsMemoryLimit {
            device: device.clone(),
            memory_limit,
            smallest_piece_bytes,
            bytes_already_held_on_the_device,
        })
}

/// 一张参与的卡这一批交活之前那一刻的地方（交活之前先等前一次超时的那次提交做完之后）：剩下的界（给的上限扣掉卡上已占的），
/// 连同报出来要的那几样——哪张卡、给它的上限、已经占着的字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomOfAGpuForTheBatch {
    pub device: GpuDeviceIdentifier,
    pub room: ChunkBoundsOfAGpu,
    pub memory_limit: Option<GpuMemoryInBytes>,
    pub bytes_already_held_on_the_device: u64,
}

/// 一张参与的卡这一批分到的活与切法：它在本机里的位置、分到的那一段相连的条目、切的时候照的界、切法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanOfAGpuForTheBatch<Plan> {
    pub position_on_this_host: usize,
    pub entries: Range<usize>,
    pub room: ChunkBoundsOfAGpu,
    pub plan: Plan,
}

/// 一批切不成、一张卡都还没交活（[`plans_of_a_batch_filling_the_gpus_in_order`]）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchNotPlannedOverTheGpus {
    /// 一张卡交活之前先等前一次超时的那次提交没等成（含这一批在这张卡上第 4 次超时、报这张卡用不了）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
    /// 参与的卡这一批全都因显存上限（扣掉已占的）连这一批最小的一片都切不出：按不用的先后列出每一张。
    EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
    /// 一张卡切不出，而装不下的不是显存上限（是单个缓冲的绑定上限）：第一版不支持。
    GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
}

/// 一张卡从 `entries_not_handed_out` 的头一条起填得满的条数（D26（后台整理与放置回收） 已定项 13 第 2 条，用户 2026-11-02 定「按卡的顺序填满」：
/// 「填到它的上限（照显存上限切片之后能放下的量）」的读法）：照这张卡的地方 `room` 切（`cut`），交的次数（`submissions_of_the_plan`）不比只拿头一条时多的
/// 最多条数。头一条一次交得下的，就是一次交得下的最多条数；头一条自己就要切成几片、交几次的（单元比这张卡的地方大，或区域扫描的映射要切成几片），
/// 是跟着它那几次交、不多添一次的最多条数。至少交回 1：头一条连最小的一片都切不出时也交回 1（调用方切它时照切不出办）。
///
/// 先按 2、4、8……倍增，找到头一个交的次数变多（或切不出）的条数，再在它与上一个之间二分。两个循环都保持「下界交得下、上界交不下」：
/// 交回的条数交得下、多一条就交不下（交的次数随条数不单调时也成立）；倍增至多 log₂ 条数 轮，二分也是，切法的调用次数是条数的对数级。
///
/// # Panics
/// `entries_not_handed_out` 是空的：调用方只在还有没分出去的条目时问。
fn entry_count_filling_the_gpu<Plan>(
    entries_not_handed_out: Range<usize>,
    room: ChunkBoundsOfAGpu,
    cut: &mut impl FnMut(
        Range<usize>,
        ChunkBoundsOfAGpu,
    ) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
    submissions_of_the_plan: &impl Fn(&Plan) -> usize,
) -> usize {
    assert!(
        !entries_not_handed_out.is_empty(),
        "调用方只在还有没分出去的条目时问"
    );
    let start_of_the_entries = entries_not_handed_out.start;
    let entries_left = entries_not_handed_out.len();
    let mut submissions_of_the_entries_from_the_start = |entry_count: usize| -> Option<usize> {
        cut(
            start_of_the_entries..start_of_the_entries + entry_count,
            room,
        )
        .ok()
        .map(|plan| submissions_of_the_plan(&plan))
    };
    let Some(submissions_of_the_entry_at_the_start) = submissions_of_the_entries_from_the_start(1)
    else {
        return 1;
    };
    let mut entry_count_held = 1;
    let mut entry_count_not_held = None;
    while entry_count_not_held.is_none() && entry_count_held < entries_left {
        let probed = entry_count_held.saturating_mul(2).min(entries_left);
        if submissions_of_the_entries_from_the_start(probed)
            == Some(submissions_of_the_entry_at_the_start)
        {
            entry_count_held = probed;
        } else {
            entry_count_not_held = Some(probed);
        }
    }
    let Some(mut entry_count_not_held) = entry_count_not_held else {
        return entry_count_held;
    };
    while entry_count_not_held - entry_count_held > 1 {
        let probed = entry_count_held + (entry_count_not_held - entry_count_held) / 2;
        if submissions_of_the_entries_from_the_start(probed)
            == Some(submissions_of_the_entry_at_the_start)
        {
            entry_count_held = probed;
        } else {
            entry_count_not_held = probed;
        }
    }
    entry_count_held
}

/// 一批（`entry_count` 条）按卡的次序填满（D26（后台整理与放置回收） 已定项 13 第 2 条，用户 2026-11-02 定「按卡的顺序填满」）：参与的卡按参与的次序
/// （点名了是 `background_gpu_devices` 的次序，没点名是总线地址的次序，[`ParticipatingGpusOnThisHost`]），每张卡照交活之前那一刻的地方
/// （`room_of_the_position`，先等前一次超时的那次提交）从还没分出去的头一条起拿它填得满的那么多条（[`entry_count_filling_the_gpu`]），剩下的才给后面的卡；
/// 排在最后的那张卡拿剩下的全部、照它的地方切成几次交（活多到每张卡都填满时）。各卡照分到的那几条切好（`cut`）。活少时后面的卡一条都不分、不等、
/// 不切、不在交回的里面；挂载时量的代价不用来分活。
/// 一张卡这一批连最小的一片都切不出、而装不下的是显存上限（扣掉已占的），这一批就不分给它（第 3 条「参与的卡的上限连最小的一片都装不下，这张卡就不用」
/// 按这一批的最小的一片判：乙段一个落点上不止一条条目的那一片随这一批的映射变），整批照剩下的卡从头重新填；参与的卡全都这样才交回
/// [`BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut`]。各卡都切好才交回，一张卡都还没交活。一次调用里每张卡的地方至多问一次。
///
/// 外层循环每一轮要么交回、要么多一张这一批不用的卡（它在这一轮还参与），所以至多比参与的卡数多一轮；内层按参与的次序每张卡至多一回。
///
/// # Errors
/// [`BatchNotPlannedOverTheGpus`] 的三种；`room_of_the_position` 交回的出错照原样交回。
///
/// # Panics
/// `participating` 一张卡都没有（调用方先看 `ParticipatingGpusOnThisHost::is_empty`）。
pub fn plans_of_a_batch_filling_the_gpus_in_order<Plan>(
    participating: &ParticipatingGpusOnThisHost,
    entry_count: usize,
    mut room_of_the_position: impl FnMut(
        usize,
    )
        -> Result<RoomOfAGpuForTheBatch, BatchNotPlannedOverTheGpus>,
    mut cut: impl FnMut(Range<usize>, ChunkBoundsOfAGpu) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
    submissions_of_the_plan: impl Fn(&Plan) -> usize,
) -> Result<Vec<PlanOfAGpuForTheBatch<Plan>>, BatchNotPlannedOverTheGpus> {
    assert!(!participating.is_empty(), "调用方先看过：至少一张参与的卡");
    let mut positions_left_out: Vec<usize> = Vec::new();
    let mut gpus_left_out: Vec<GpuLeftOutUnderItsMemoryLimit> = Vec::new();
    let mut rooms_asked: Vec<(usize, RoomOfAGpuForTheBatch)> = Vec::new();
    loop {
        let participating_this_round = participating
            .clone()
            .leaving_out(|position| positions_left_out.contains(&position));
        if participating_this_round.is_empty() {
            return Err(BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(
                gpus_left_out,
            ));
        }
        let order = participating_this_round.positions_on_this_host();
        let mut plans = Vec::new();
        let mut next_entry_to_hand_out = 0;
        let mut left_out_this_round = None;
        for (place_in_the_order, position_on_this_host) in order.iter().copied().enumerate() {
            if next_entry_to_hand_out == entry_count {
                break;
            }
            let room_of_the_gpu = match rooms_asked
                .iter()
                .find(|(position_asked, _room)| *position_asked == position_on_this_host)
            {
                Some((_position_asked, room)) => room.clone(),
                None => {
                    let room = room_of_the_position(position_on_this_host)?;
                    rooms_asked.push((position_on_this_host, room.clone()));
                    room
                }
            };
            let entries_not_handed_out = next_entry_to_hand_out..entry_count;
            let is_the_last_gpu_in_the_order = place_in_the_order + 1 == order.len();
            let entries = if is_the_last_gpu_in_the_order {
                entries_not_handed_out
            } else {
                let entry_count_of_the_gpu = entry_count_filling_the_gpu(
                    entries_not_handed_out.clone(),
                    room_of_the_gpu.room,
                    &mut cut,
                    &submissions_of_the_plan,
                );
                next_entry_to_hand_out..next_entry_to_hand_out + entry_count_of_the_gpu
            };
            match cut(entries.clone(), room_of_the_gpu.room) {
                Ok(plan) => {
                    next_entry_to_hand_out = entries.end;
                    plans.push(PlanOfAGpuForTheBatch {
                        position_on_this_host,
                        entries,
                        room: room_of_the_gpu.room,
                        plan,
                    });
                }
                Err(smallest_piece) => {
                    let short_under_the_memory_limit = match (
                        room_of_the_gpu.memory_limit,
                        room_of_the_gpu.room.memory_limit,
                    ) {
                        (Some(memory_limit), Some(room_left))
                            if room_left.0 < smallest_piece.smallest_piece_bytes =>
                        {
                            Some(memory_limit)
                        }
                        (Some(_), Some(_) | None) | (None, _) => None,
                    };
                    let Some(memory_limit) = short_under_the_memory_limit else {
                        return Err(BatchNotPlannedOverTheGpus::GpuCannotHoldTheSmallestPiece(
                            SmallestPieceNotHeldOnTheGpu {
                                device: room_of_the_gpu.device,
                                entry_position: entries.start + smallest_piece.entry_position,
                                memory_limit: room_of_the_gpu.memory_limit,
                                smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
                            },
                        ));
                    };
                    left_out_this_round = Some((
                        position_on_this_host,
                        GpuLeftOutUnderItsMemoryLimit {
                            device: room_of_the_gpu.device,
                            memory_limit,
                            smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
                            bytes_already_held_on_the_device: room_of_the_gpu
                                .bytes_already_held_on_the_device,
                        },
                    ));
                    break;
                }
            }
        }
        let Some((position_left_out, gpu_left_out)) = left_out_this_round else {
            return Ok(plans);
        };
        positions_left_out.push(position_left_out);
        gpus_left_out.push(gpu_left_out);
    }
}

/// 一张卡丢后台 GPU 时泄漏了（还有一次等超过上限、没在 GPU 上做完的提交，设备不析构）：哪张卡、它上面那一刻本包占着的字节（泄漏的那一片），
/// 其中开卡那一刻已经算上的、这个进程里此前几次挂载泄漏在它上面的字节（[`GpuBudgetUnderTheSettings::bytes_leaked_by_earlier_mounts`]，
/// 恒不超过占着的：账从它起、这一次挂载里不放）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuLeakedWhenDropped {
    pub device: GpuDeviceIdentifier,
    pub bytes_held_on_the_device: u64,
    pub bytes_leaked_by_earlier_mounts: u64,
}

impl GpuLeakedWhenDropped {
    /// 这一次挂载自己泄漏的字节：那一刻占着的扣掉开卡时已算上的此前泄漏的那一片。
    ///
    /// # Panics
    /// 此前泄漏的比那一刻占着的多：账从此前泄漏的那一片起，这一次挂载里一直记着、不放（`GpuMemoryAccount::with_the_bytes_leaked_by_earlier_mounts`）。
    #[must_use]
    pub fn bytes_leaked_by_this_mount(&self) -> u64 {
        self.bytes_held_on_the_device
            .checked_sub(self.bytes_leaked_by_earlier_mounts)
            .expect("账从此前泄漏的那一片起、这一次挂载里不放：占着的不比它少")
    }
}

/// 一次挂载的后台 GPU 丢掉（卸载、丢掉核心层收的把手）那一刻泄漏了哪几张卡：调用方开卡时拿到一份（`background_gpu_compute::OpenedBackgroundGpuCompute`），
/// 丢掉之后读（`.claude/rules/fs-design.md` 五条硬要求第 4 条：泄漏没泄漏、泄漏了几张看得出来）。几份把手共用同一份。
#[derive(Clone, Debug, Default)]
pub struct GpusLeakedWhenTheBackgroundGpuIsDropped {
    leaked: Arc<Mutex<Option<Vec<GpuLeakedWhenDropped>>>>,
}

impl GpusLeakedWhenTheBackgroundGpuIsDropped {
    /// 还没丢。
    #[must_use]
    pub fn not_yet_dropped() -> Self {
        Self {
            leaked: Arc::new(Mutex::new(None)),
        }
    }

    /// 丢掉那一刻泄漏的卡，按本机的次序（总线地址）；还没丢是 `None`，丢了而一张都没泄漏是空。
    #[must_use]
    pub fn gpus_leaked(&self) -> Option<Vec<GpuLeakedWhenDropped>> {
        self.leaked
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub(crate) fn note_the_drop(&self, gpus_leaked: Vec<GpuLeakedWhenDropped>) {
        *self
            .leaked
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(gpus_leaked);
    }
}

/// 这个进程里每张卡泄漏着的字节（几次挂载的后台 GPU 丢掉时泄漏的设备不析构，那一片显存这个进程活着就一直占着）：调用方在进程里留一份、
/// 每次开卡都交同一份（`background_gpu_compute::open_background_gpu_compute`），新一次挂载这张卡的可用余量先扣掉它（第 3 条）。
/// 同一张卡上每丢一次泄漏一次就按次累加这一次挂载自己泄漏的字节（[`GpuLeakedWhenDropped::bytes_leaked_by_this_mount`]），不覆盖：
/// 两次挂载同时开着时各自开卡那一刻都没算上对方的那一片，后丢的那一次也不含先丢的（判决 m4-gpugc-code-r2 第三节第 4 条）。几份把手共用同一份。
#[derive(Clone, Debug, Default)]
pub struct GpuMemoryLeakedInThisProcess {
    bytes_leaked_on_each_device: Arc<Mutex<Vec<(GpuDeviceIdentifier, u64)>>>,
}

impl GpuMemoryLeakedInThisProcess {
    /// 进程里还一张卡都没泄漏过。
    #[must_use]
    pub fn with_nothing_leaked() -> Self {
        Self {
            bytes_leaked_on_each_device: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 这张卡上泄漏着的字节；没泄漏过是 0。
    #[must_use]
    pub fn bytes_leaked_on(&self, device: &GpuDeviceIdentifier) -> u64 {
        self.bytes_leaked_on_each_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .find(|(leaked_device, _bytes)| leaked_device == device)
            .map_or(0, |(_device, bytes)| *bytes)
    }

    /// 记下一次丢后台 GPU 时泄漏的卡：每张加上这一次挂载自己泄漏的字节（那一刻占着的扣掉开卡时已算上的此前泄漏的那一片）。
    pub fn note_the_gpus_leaked_when_dropped(&self, gpus_leaked: &[GpuLeakedWhenDropped]) {
        let mut bytes_leaked_on_each_device = self
            .bytes_leaked_on_each_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for leaked in gpus_leaked {
            match bytes_leaked_on_each_device
                .iter_mut()
                .find(|(leaked_device, _bytes)| *leaked_device == leaked.device)
            {
                Some((_device, bytes)) => *bytes += leaked.bytes_leaked_by_this_mount(),
                None => bytes_leaked_on_each_device
                    .push((leaked.device.clone(), leaked.bytes_leaked_by_this_mount())),
            }
        }
    }
}

/// 开一张卡时量不量它的代价：参与而显存上限装不下最小的一片的卡不量（不越过上限去量，第 3 条），只起设备与管线，之后换设置给它装得下的上限时再量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostAtTheOpening {
    MeasuredUnderTheMemoryLimit,
    NotMeasuredForTheGpuIsLeftOut,
}

/// 挂载那一刻开卡的结局：开成的每张卡开出来的东西（与 `discrete_gpus.opened` 一一对应、同序）、开卡之后的本机独显、开卡那一组参与的卡
/// （在开成的那几张里的位置；显存上限装不下最小的一片的不在里面，可以一张都没有）、因显存上限不用的那几张（按参与的次序）。
#[derive(Debug)]
pub struct GpusOpenedAtTheMount<Opened> {
    pub opened: Vec<Opened>,
    pub discrete_gpus: DiscreteGpusAfterTheOpening,
    pub participating: ParticipatingGpusOnThisHost,
    pub left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
}

/// 挂载那一刻在本机每一张独显上开卡（按 `discrete_gpus` 的次序，即总线地址的次序）：`open_the_gpu` 是外壳起设备、编管线、量代价那一步，
/// 交给它这张卡、开卡那一组给它的显存上限、量不量代价；测试里交造出来的结局。参与的卡显存上限装不下 `smallest_piece_bytes`（最小的一片）的照开、
/// 不量代价，不进参与的卡、报出来（[`GpuLeftOutUnderItsMemoryLimit`]），挂载不因此拒绝（用户 2026-10-28 定「连一片都装不下就不用这张卡」）；
/// 装得下的照它的上限量代价（外壳切成几片逐片交）。一张卡没开成挡不挡挂载（D26（后台整理与放置回收） 已定项 13 第 2、4 条，
/// `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 45 批交回那一行的定法）：它参与（`participating` 里有它——给了 `background_gpu_devices` 时是点名的卡，
/// 没给时本机全部独显都参与）就拒绝挂载、报出它与原因，后面的卡不再开；不参与（给了列表而没点它）就跳过，不进参与的卡，记进
/// [`DiscreteGpusAfterTheOpening::not_opened`] 报出来，挂载照常。
/// `participating` 是开卡那一组照同一份 `discrete_gpus` 校验过、取出来的参与的卡（`adapter_inventory::participating_gpus_admitted_by_the_mount_checks`）。
/// 这个进程里此前几次挂载泄漏在一张卡上的字节（`memory_leaked_in_this_process`）先从它的上限里扣掉再判装不装得下（第 3 条；外壳开这张卡时账也从它起）。
///
/// # Errors
/// [`BackgroundGpuOpenRefused::GpuUnusable`]：一张参与的卡没开成。
///
/// # Panics
/// `participating` 里有一张不在 `discrete_gpus` 里：它取自同一份清单。
pub fn gpus_opened_at_the_mount<Opened>(
    discrete_gpus: Vec<DiscreteGpuOnThisHost>,
    participating: &[ParticipatingGpu],
    smallest_piece_bytes: u64,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
    mut open_the_gpu: impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        CostAtTheOpening,
    ) -> Result<Opened, GpuFailureCause>,
) -> Result<GpusOpenedAtTheMount<Opened>, BackgroundGpuOpenRefused> {
    let mut opened = Vec::with_capacity(discrete_gpus.len());
    let mut opened_discrete_gpus: Vec<DiscreteGpuOnThisHost> =
        Vec::with_capacity(discrete_gpus.len());
    let mut not_opened = Vec::new();
    let mut left_out_under_their_memory_limit = Vec::new();
    for discrete_gpu in discrete_gpus {
        let participating_gpu = participating.iter().find(|participating_gpu| {
            participating_gpu.discrete_gpu.identifier == discrete_gpu.identifier
        });
        let memory_limit_at_the_opening =
            participating_gpu.and_then(|participating_gpu| participating_gpu.memory_limit);
        let left_out = gpu_left_out_under_its_memory_limit(
            &discrete_gpu.identifier,
            memory_limit_at_the_opening,
            smallest_piece_bytes,
            memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier),
        );
        let cost_at_the_opening = match &left_out {
            Some(_left_out) => CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut,
            None => CostAtTheOpening::MeasuredUnderTheMemoryLimit,
        };
        match open_the_gpu(
            &discrete_gpu,
            memory_limit_at_the_opening,
            cost_at_the_opening,
        ) {
            Ok(opened_gpu) => {
                opened.push(opened_gpu);
                opened_discrete_gpus.push(discrete_gpu);
                left_out_under_their_memory_limit.extend(left_out);
            }
            Err(cause) => match participating_gpu {
                Some(_participating) => {
                    return Err(BackgroundGpuOpenRefused::GpuUnusable {
                        device: discrete_gpu.identifier,
                        cause,
                    });
                }
                None => not_opened.push(DiscreteGpuNotOpenedAtTheMount {
                    discrete_gpu,
                    cause,
                }),
            },
        }
    }
    let participating_positions: Vec<usize> = participating
        .iter()
        .map(|participating_gpu| {
            opened_discrete_gpus
                .iter()
                .position(|opened_gpu| {
                    opened_gpu.identifier == participating_gpu.discrete_gpu.identifier
                })
                .expect("参与的卡取自同一份独显清单；没开成的那一张在上面就拒了挂载")
        })
        .collect();
    let left_out_positions: Vec<usize> = left_out_under_their_memory_limit
        .iter()
        .map(|left_out| {
            opened_discrete_gpus
                .iter()
                .position(|opened_gpu| opened_gpu.identifier == left_out.device)
                .expect("不用的卡是开成了的参与的卡")
        })
        .collect();
    left_out_under_their_memory_limit.sort_by_key(|left_out| {
        participating
            .iter()
            .position(|participating_gpu| {
                participating_gpu.discrete_gpu.identifier == left_out.device
            })
            .expect("不用的卡是参与的卡")
    });
    Ok(GpusOpenedAtTheMount {
        opened,
        discrete_gpus: DiscreteGpusAfterTheOpening {
            opened: opened_discrete_gpus,
            not_opened,
        },
        participating: ParticipatingGpusOnThisHost::of_the_positions(participating_positions)
            .leaving_out(|position| left_out_positions.contains(&position)),
        left_out_under_their_memory_limit,
    })
}

/// 丢后台 GPU 时（卸载、丢掉核心层收的把手）逐张丢开着的卡：还有一次等超过上限、没在 GPU 上做完的提交的那几张不丢、泄漏
/// （它的设备与队列不析构：wgpu-core 30.0.1 的 `Queue` 析构时调 `wait_for_idle`，即 Vulkan 的 `vkQueueWaitIdle`，不带上限），
/// 别的照常丢（用户 2026-10-28 定「卸载时同样不等、泄漏设备」）。交回泄漏的那几张（按交进来的次序，各带它上面那一刻占着的字节，
/// `leaked_when_dropped` 报）。不等任何一次提交：调用即返回。
pub fn gpus_dropped_leaking_those_still_running<Gpu>(
    gpus: Vec<Gpu>,
    has_a_submission_still_running: impl Fn(&Gpu) -> bool,
    leaked_when_dropped: impl Fn(&Gpu) -> GpuLeakedWhenDropped,
) -> Vec<GpuLeakedWhenDropped> {
    let mut leaked = Vec::new();
    for gpu in gpus {
        if has_a_submission_still_running(&gpu) {
            leaked.push(leaked_when_dropped(&gpu));
            std::mem::forget(gpu);
        } else {
            drop(gpu);
        }
    }
    leaked
}

/// 本机的这张卡在这一组设置里给的显存上限：参与而没给上限的、不参与的（它分不到活）都交回 `None`。
#[must_use]
pub fn memory_limit_of_the_device_under_the_settings(
    settings: &AdmittedGpuSettings,
    identifier_on_this_host: &GpuDeviceIdentifier,
) -> Option<GpuMemoryInBytes> {
    settings
        .participating_devices()
        .iter()
        .find(|participating| participating.device == *identifier_on_this_host)
        .and_then(|participating| participating.memory_limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use singlefs_core::background_compute::{
        admit_gpu_settings_against_the_inventory, BackgroundComputeBackend,
        BackgroundComputeSettings, GpuInventory,
    };

    use singlefs_core::background_compute::{BackgroundGpuComputeFailed, GpuFailureOnTheDevice};

    use crate::adapter_inventory::{
        discrete_gpus_of_the_enumeration, participating_gpus_admitted_by_the_mount_checks,
        AdapterDeviceKind, BackgroundGpuSelection, EnumeratedAdapterDescription,
    };
    use crate::background_gpu::COST_PROBE_CHUNK;
    use crate::memory_limit_and_chunks::SubmissionOfTheChunk;

    const GIBIBYTE: u64 = 1 << 30;

    /// 本机三张独显，按总线地址排：代价各不相同。
    struct ThreeCardsOnThisHost;

    fn card(name: &str) -> GpuDeviceIdentifier {
        GpuDeviceIdentifier::of_opaque_bytes(name.as_bytes().to_vec())
    }

    impl GpuInventory for ThreeCardsOnThisHost {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            vec![
                (card("0000:01:00.0"), GpuMemoryInBytes(32 * GIBIBYTE)),
                (card("0000:41:00.0"), GpuMemoryInBytes(16 * GIBIBYTE)),
                (card("0000:81:00.0"), GpuMemoryInBytes(16 * GIBIBYTE)),
            ]
        }
    }

    fn identifiers_on_this_host() -> Vec<GpuDeviceIdentifier> {
        ThreeCardsOnThisHost
            .devices_with_total_memory()
            .into_iter()
            .map(|(identifier, _total_memory)| identifier)
            .collect()
    }

    fn admitted(
        named_devices: Option<Vec<GpuDeviceIdentifier>>,
        memory_limits: Option<Vec<GpuMemoryInBytes>>,
        wait_limit: Duration,
    ) -> AdmittedGpuSettings {
        admit_gpu_settings_against_the_inventory(
            &BackgroundComputeSettings {
                background_compute: BackgroundComputeBackend::Gpu,
                background_gpu_devices: named_devices,
                background_gpu_memory_limit: memory_limits,
                background_gpu_wait_limit: wait_limit,
            },
            &ThreeCardsOnThisHost,
        )
        .expect("用例给的一组都合法")
    }

    /// 第 2 条：没点名时本机三张都参与、按总线地址的次序；切换缩成点名的两张（后一张在前）之后，参与的只有这两张、按点名的次序
    /// （按卡的顺序填满时就是这个次序），中间那张不参与。
    #[test]
    fn after_a_switch_to_fewer_cards_only_the_named_cards_take_part_in_the_named_order() {
        let at_the_mount = ParticipatingGpusOnThisHost::of_the_settings(
            &admitted(None, None, Duration::from_secs(10)),
            &identifiers_on_this_host(),
        );
        assert_eq!(
            at_the_mount.positions_on_this_host(),
            &[0, 1, 2],
            "没点名：本机三张都参与，按总线地址的次序"
        );
        let after_the_switch = ParticipatingGpusOnThisHost::of_the_settings(
            &admitted(
                Some(vec![card("0000:81:00.0"), card("0000:01:00.0")]),
                None,
                Duration::from_secs(10),
            ),
            &identifiers_on_this_host(),
        );
        assert_eq!(
            after_the_switch.positions_on_this_host(),
            &[2, 0],
            "点名两张：只有它们参与、按点名的次序；中间那张不参与"
        );
    }

    /// 第 3 条：显存上限从切换生效的那一批起照新的切；上限改小之后同一批条目切得更碎，记账超过新上限就断言。
    #[test]
    fn after_a_switch_to_a_smaller_memory_limit_the_next_batch_is_cut_by_the_smaller_limit() {
        let footprint = ChunkFootprint {
            bytes_per_chunk_besides_the_entries: 16,
            bytes_per_entry_besides_its_input: 40,
            bytes_per_entry_in_the_largest_entry_buffer: 16,
        };
        let lengths = [100_u64; 6];
        let clock = ClockThatNeverMoves;
        let mut budget: GpuBudgetUnderTheSettings<u32> =
            GpuBudgetUnderTheSettings::with_nothing_held(
                1 << 30,
                1 << 20,
                Some(GpuMemoryInBytes(1000)),
                Duration::from_secs(10),
            );
        assert_eq!(
            budget.chunks_cut_once_the_submission_running_past_its_wait_completed(
                &clock,
                |_submission, _time_left| panic!("没有还在跑的提交，不该等"),
                &lengths,
                footprint,
            ),
            Ok(vec![Range { start: 0, end: 6 }]),
            "上限 1000 字节：六条一段 856 字节"
        );
        budget.use_the_settings_from_the_next_batch(
            Some(GpuMemoryInBytes(300)),
            Duration::from_secs(10),
        );
        assert_eq!(
            budget.chunks_cut_once_the_submission_running_past_its_wait_completed(
                &clock,
                |_submission, _time_left| panic!("没有还在跑的提交，不该等"),
                &lengths,
                footprint,
            ),
            Ok(vec![0..2, 2..4, 4..6]),
            "切换成 300 字节之后：两条一段 296 字节"
        );
        assert_eq!(
            budget.chunk_bounds().memory_limit,
            Some(GpuMemoryInBytes(300)),
            "切段的界照新上限"
        );
        let held_past_the_smaller_limit =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                budget.memory_account_mut().hold(301);
            }));
        assert!(
            held_past_the_smaller_limit.is_err(),
            "记账照新上限断言：301 字节超过 300"
        );
    }

    /// 等的上限从切换生效的那一批起照新的：这张卡上前一次等超时的提交，下一批交活之前在新的上限之内等它。
    /// 上限从 2 秒改成 8 秒：那次提交 5 秒后做完，照新的等得到它、放掉账再切；照旧的 2 秒就报前一次超时的提交没做完（带的是 2 秒）。
    #[test]
    fn after_a_switch_of_the_wait_limit_the_next_batch_waits_by_the_new_limit() {
        let footprint = ChunkFootprint {
            bytes_per_chunk_besides_the_entries: 16,
            bytes_per_entry_besides_its_input: 40,
            bytes_per_entry_in_the_largest_entry_buffer: 16,
        };
        let (mut budget_kept_at_the_old_limit, clock_of_the_old_limit) =
            budget_with_a_timed_out_submission_completing_after(
                Duration::from_secs(2),
                Duration::from_secs(5),
            );
        assert_eq!(
            budget_kept_at_the_old_limit
                .chunks_cut_once_the_submission_running_past_its_wait_completed(
                    &clock_of_the_old_limit,
                    |_submission, time_left| clock_of_the_old_limit
                        .wait_for_the_submission(time_left),
                    &[100],
                    footprint,
                ),
            Err(ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning(
                GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                    wait_limit: Duration::from_secs(2)
                }
            )),
            "照旧的 2 秒：等不到"
        );
        let (mut budget_switched_to_the_new_limit, clock_of_the_new_limit) =
            budget_with_a_timed_out_submission_completing_after(
                Duration::from_secs(2),
                Duration::from_secs(5),
            );
        budget_switched_to_the_new_limit
            .use_the_settings_from_the_next_batch(None, Duration::from_secs(8));
        assert_eq!(
            budget_switched_to_the_new_limit.wait_limit(),
            Duration::from_secs(8)
        );
        assert_eq!(
            budget_switched_to_the_new_limit
                .chunks_cut_once_the_submission_running_past_its_wait_completed(
                    &clock_of_the_new_limit,
                    |_submission, time_left| clock_of_the_new_limit
                        .wait_for_the_submission(time_left),
                    &[100],
                    footprint,
                ),
            Ok(vec![Range { start: 0, end: 1 }]),
            "切换成 8 秒：等到它做完、放了账，照常切"
        );
        assert_eq!(
            budget_switched_to_the_new_limit
                .memory_account()
                .bytes_held(),
            0,
            "那次提交的账放了"
        );
    }

    /// 钟从不走：只给不等的用例用。
    struct ClockThatNeverMoves;

    impl MonotonicClock for ClockThatNeverMoves {
        fn time_since_the_origin(&self) -> Duration {
            Duration::ZERO
        }
    }

    /// 假钟连同一次提交：钟只在等那次提交时走，每看一次至多走交给它的那么久，走到 `completes_at` 那一刻它做完。
    struct ClockWithASubmissionCompletingAt {
        now: Cell<Duration>,
        completes_at: Duration,
    }

    impl MonotonicClock for ClockWithASubmissionCompletingAt {
        fn time_since_the_origin(&self) -> Duration {
            self.now.get()
        }
    }

    impl ClockWithASubmissionCompletingAt {
        fn wait_for_the_submission(&self, time_left: Duration) -> CompletionCheck<()> {
            let waited_this_check = Duration::from_secs(1)
                .min(time_left)
                .max(Duration::from_nanos(1));
            self.now.set(self.now.get() + waited_this_check);
            if self.now.get() >= self.completes_at {
                CompletionCheck::Completed(())
            } else {
                CompletionCheck::NotYetComplete
            }
        }
    }

    /// 一张卡，等的上限 `wait_limit`，账上记着一段 200 字节、等超时还在跑的提交；钟在 0，那次提交在 `completes_after` 做完。
    fn budget_with_a_timed_out_submission_completing_after(
        wait_limit: Duration,
        completes_after: Duration,
    ) -> (
        GpuBudgetUnderTheSettings<u32>,
        ClockWithASubmissionCompletingAt,
    ) {
        let mut budget =
            GpuBudgetUnderTheSettings::with_nothing_held(1 << 30, 1 << 20, None, wait_limit);
        budget.memory_account_mut().hold(200);
        budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                200,
                SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(7),
            );
        (
            budget,
            ClockWithASubmissionCompletingAt {
                now: Cell::new(Duration::ZERO),
                completes_at: completes_after,
            },
        )
    }

    /// 本机三张独显的枚举，按总线地址：「0000:01:00.0」32 GiB、「0000:41:00.0」16 GiB、「0000:81:00.0」16 GiB。
    fn three_discrete_gpus_of_the_enumeration() -> Vec<DiscreteGpuOnThisHost> {
        let discrete_gpu =
            |pci_bus_address: &str, memory_in_gibibytes: u64| EnumeratedAdapterDescription {
                name: format!("card at {pci_bus_address}"),
                device_kind: AdapterDeviceKind::DiscreteGpu,
                pci_bus_address: pci_bus_address.to_string(),
                device_local_memory: Some(GpuMemoryInBytes(memory_in_gibibytes * GIBIBYTE)),
            };
        discrete_gpus_of_the_enumeration(&[
            discrete_gpu("0000:81:00.0", 16),
            discrete_gpu("0000:01:00.0", 32),
            discrete_gpu("0000:41:00.0", 16),
        ])
        .expect("三张独显都有总线地址与显存")
    }

    fn reason_the_card_did_not_open(card_name: &str) -> GpuFailureCause {
        GpuFailureCause::DeviceOrPipelineDidNotStart {
            reason: format!("起不了设备：{card_name} 造出来的坏卡"),
        }
    }

    /// 开卡那一组：照三张独显校验过、取出参与的卡；再在三张卡上开卡，`card_that_does_not_open` 那一张交回没开成，别的交回（总线地址，给它的上限）。
    fn opening_under_the_selection(
        selection: &BackgroundGpuSelection,
        card_that_does_not_open: &str,
    ) -> Result<GpusOpenedAtTheMount<(String, Option<GpuMemoryInBytes>)>, BackgroundGpuOpenRefused>
    {
        let discrete_gpus = three_discrete_gpus_of_the_enumeration();
        let participating =
            participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus)
                .expect("用例给的一组照三张独显校验都合法");
        gpus_opened_at_the_mount(
            discrete_gpus,
            &participating,
            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            |discrete_gpu, memory_limit, _cost_at_the_opening| {
                if discrete_gpu.pci_bus_address == card_that_does_not_open {
                    Err(reason_the_card_did_not_open(card_that_does_not_open))
                } else {
                    Ok((discrete_gpu.pci_bus_address.clone(), memory_limit))
                }
            },
        )
    }

    /// 第 2 条「点名了就只用点名的卡」：点名的两张都开成、没点名的那张没开成——照挂，没开成的那张跳过、不进参与的卡，卡清单照列它并报出它与原因。
    #[test]
    fn an_unnamed_card_that_did_not_open_is_skipped_and_reported_while_the_named_cards_take_part() {
        let opening = opening_under_the_selection(
            &BackgroundGpuSelection {
                background_gpu_devices: Some(vec![card("0000:81:00.0"), card("0000:01:00.0")]),
                background_gpu_memory_limit: Some(vec![
                    GpuMemoryInBytes(2 * GIBIBYTE),
                    GpuMemoryInBytes(3 * GIBIBYTE),
                ]),
                background_gpu_wait_limit: Duration::from_secs(10),
            },
            "0000:41:00.0",
        )
        .expect("没点名的卡开不了：不挡挂载");
        assert_eq!(
            opening.opened,
            vec![
                (
                    "0000:01:00.0".to_string(),
                    Some(GpuMemoryInBytes(3 * GIBIBYTE))
                ),
                (
                    "0000:81:00.0".to_string(),
                    Some(GpuMemoryInBytes(2 * GIBIBYTE))
                ),
            ],
            "开成的是点名的两张，按总线地址排，各带点名给的上限"
        );
        assert_eq!(
            opening.participating.positions_on_this_host(),
            &[1, 0],
            "参与的卡是点名的两张、按点名的次序（在开成的卡里的位置）"
        );
        assert_eq!(
            opening.discrete_gpus.not_opened,
            vec![DiscreteGpuNotOpenedAtTheMount {
                discrete_gpu: three_discrete_gpus_of_the_enumeration()[1].clone(),
                cause: reason_the_card_did_not_open("0000:41:00.0"),
            }],
            "跳过的那张报出来：哪张卡、什么原因"
        );
        assert_eq!(
            opening
                .discrete_gpus
                .devices_with_total_memory()
                .into_iter()
                .map(|(device, _total_memory)| device)
                .collect::<Vec<_>>(),
            vec![
                card("0000:01:00.0"),
                card("0000:41:00.0"),
                card("0000:81:00.0")
            ],
            "卡清单照列本机全部独显，连同没开成的那张，按总线地址排"
        );
        assert_eq!(
            opening.discrete_gpus.devices_not_opened_at_the_mount(),
            vec![GpuDeviceNotOpenedAtTheMount {
                device: card("0000:41:00.0"),
                failure_at_the_mount: BackgroundGpuComputeFailed::GpuFailed {
                    device: card("0000:41:00.0"),
                    cause: GpuFailureOnTheDevice::DeviceOrPipelineDidNotStart {
                        reason: "起不了设备：0000:41:00.0 造出来的坏卡".to_string(),
                    },
                },
            }],
            "卡清单报给核心层的没开成的那张：卡与原因同挂着时这张卡出错的报法"
        );
    }

    /// 第 4 条：点名的卡开不了，拒绝挂载，报出它与原因。
    #[test]
    fn one_named_card_that_did_not_open_refuses_the_mount() {
        let refused = opening_under_the_selection(
            &BackgroundGpuSelection {
                background_gpu_devices: Some(vec![card("0000:41:00.0"), card("0000:01:00.0")]),
                background_gpu_memory_limit: None,
                background_gpu_wait_limit: Duration::from_secs(10),
            },
            "0000:41:00.0",
        );
        assert_eq!(
            refused.map(|opening| opening.opened),
            Err(BackgroundGpuOpenRefused::GpuUnusable {
                device: card("0000:41:00.0"),
                cause: reason_the_card_did_not_open("0000:41:00.0"),
            }),
            "点名的那张没开成：拒绝挂载"
        );
    }

    /// 第 2、4 条：没给 `background_gpu_devices` 时本机全部独显都参与，任何一张开不了就拒绝挂载。
    #[test]
    fn without_named_cards_any_card_that_did_not_open_refuses_the_mount() {
        let refused = opening_under_the_selection(
            &BackgroundGpuSelection {
                background_gpu_devices: None,
                background_gpu_memory_limit: None,
                background_gpu_wait_limit: Duration::from_secs(10),
            },
            "0000:81:00.0",
        );
        assert_eq!(
            refused.map(|opening| opening.opened),
            Err(BackgroundGpuOpenRefused::GpuUnusable {
                device: card("0000:81:00.0"),
                cause: reason_the_card_did_not_open("0000:81:00.0"),
            }),
            "没给卡列表：总线地址最后那张没开成也拒绝挂载"
        );
    }

    /// 开卡那一组点「0000:01:00.0」「0000:41:00.0」两张、各给 `memory_limits_in_bytes` 的显存上限；开卡那一步照外壳那样：量代价的卡拿给它的上限建账、
    /// 把量代价那一段照这张卡的界切成几片、逐段记账（`GpuMemoryAccount::hold`，记下之后超上限就断言）再放掉，交回（总线地址，量没量代价）。
    fn opening_that_holds_the_cost_probe_in_pieces(
        memory_limits_in_bytes: [u64; 2],
    ) -> Result<GpusOpenedAtTheMount<(String, CostAtTheOpening)>, BackgroundGpuOpenRefused> {
        let selection = BackgroundGpuSelection {
            background_gpu_devices: Some(vec![card("0000:01:00.0"), card("0000:41:00.0")]),
            background_gpu_memory_limit: Some(
                memory_limits_in_bytes
                    .into_iter()
                    .map(GpuMemoryInBytes)
                    .collect(),
            ),
            background_gpu_wait_limit: Duration::from_secs(10),
        };
        let discrete_gpus = three_discrete_gpus_of_the_enumeration();
        let participating =
            participating_gpus_admitted_by_the_mount_checks(&selection, &discrete_gpus)
                .expect("点两张卡、各给一个不超过显存总量的上限：校验合法");
        gpus_opened_at_the_mount(
            discrete_gpus,
            &participating,
            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            |discrete_gpu, memory_limit, cost_at_the_opening| {
                if cost_at_the_opening == CostAtTheOpening::MeasuredUnderTheMemoryLimit {
                    let mut account = GpuMemoryAccount::<u32>::with_nothing_held(memory_limit);
                    let chunks = COST_PROBE_CHUNK
                        .pieces_under_the_bounds(ChunkBoundsOfAGpu {
                            memory_limit,
                            largest_storage_binding_in_bytes: 1 << 30,
                            largest_entry_count_per_dispatch: 1 << 20,
                        })
                        .expect("量代价的卡装得下最小的一片");
                    for chunk in &chunks {
                        let input_bytes: u64 =
                            chunk.iter().map(PieceOfAnEntry::input_byte_count).sum();
                        let held = COST_PROBE_CHUNK.footprint.bytes_held_by_a_chunk(
                            input_bytes,
                            u64::try_from(chunk.len()).expect("小数"),
                        );
                        account.hold(held);
                        account.release(held);
                    }
                }
                Ok((discrete_gpu.pci_bus_address.clone(), cost_at_the_opening))
            },
        )
    }

    /// 用户 2026-10-28 定「这部分不能因为装不下就拒绝，装不下应该自适应的分片」：给参与的卡的上限（1000 字节）比量代价那一段（4156 字节）小，
    /// 照这张卡的上限切成几片，每段记账都不超上限（`GpuMemoryAccount::hold` 超了就断言），开得成、参与，挂载不拒。
    #[test]
    fn the_memory_limit_smaller_than_the_cost_probe_cuts_it_into_pieces_instead_of_refusing_the_mount(
    ) {
        let opening = opening_that_holds_the_cost_probe_in_pieces([1000, 1000])
            .expect("上限比量代价那一段小：切开量，不拒挂载");
        assert_eq!(
            opening.opened,
            vec![
                (
                    "0000:01:00.0".to_string(),
                    CostAtTheOpening::MeasuredUnderTheMemoryLimit
                ),
                (
                    "0000:41:00.0".to_string(),
                    CostAtTheOpening::MeasuredUnderTheMemoryLimit
                ),
                (
                    "0000:81:00.0".to_string(),
                    CostAtTheOpening::MeasuredUnderTheMemoryLimit
                ),
            ],
            "三张卡都开、都量代价"
        );
        assert_eq!(opening.participating.positions_on_this_host(), &[0, 1]);
        assert!(opening.left_out_under_their_memory_limit.is_empty());
        let chunks = COST_PROBE_CHUNK
            .pieces_under_the_bounds(ChunkBoundsOfAGpu {
                memory_limit: Some(GpuMemoryInBytes(1000)),
                largest_storage_binding_in_bytes: 1 << 30,
                largest_entry_count_per_dispatch: 1 << 20,
            })
            .expect("1000 字节装得下最小的一片");
        assert_eq!(
            chunks
                .iter()
                .map(|chunk| chunk
                    .iter()
                    .map(|piece| piece.input_bytes.clone())
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![
                vec![0..940],
                vec![940..1880],
                vec![1880..2820],
                vec![2820..3760],
                vec![3760..4096]
            ],
            "1000 字节扣掉一段 16 字节与一条 44 字节的固定占用剩 940：4096 字节切成五片、一片一段"
        );
    }

    /// 用户 2026-10-28 定「代价取各片之和」：量代价那一段切成几段时，每一轮把各段的 CPU 时间加起来，代价是各轮之和的中位数；
    /// 头一遍各段空跑、不算进去。
    #[test]
    fn the_cost_of_a_cost_probe_cut_into_pieces_is_the_sum_over_the_pieces() {
        let chunks = [100_u64, 200, 300, 400, 500];
        let rounds_seen = Cell::new(0_u32);
        let cost = measured_cost_summed_over_the_pieces(&chunks, 5, |nanoseconds_of_the_chunk| {
            rounds_seen.set(rounds_seen.get() + 1);
            Ok::<_, GpuFailureCause>(Duration::from_nanos(*nanoseconds_of_the_chunk))
        });
        assert_eq!(
            cost,
            Ok(HostCpuNanosecondsPerSubmission(1500)),
            "五段各 100、200、300、400、500 纳秒：代价是它们之和"
        );
        assert_eq!(rounds_seen.get(), 6 * 5, "空跑一遍、再量五轮，每轮五段");
    }

    /// 用户 2026-10-28 定「连一片都装不下就不用这张卡」：一张参与的卡的显存上限连最小的一片都装不下——照开、不量代价，不参与、报出那张卡、
    /// 它的上限与至少要多少字节，另一张参与的卡照常；挂载不拒、不 panic。
    #[test]
    fn the_memory_limit_below_the_smallest_piece_leaves_the_gpu_out_without_refusing_the_mount() {
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
        let opening = opening_that_holds_the_cost_probe_in_pieces([smallest_piece_bytes - 1, 1000])
            .expect("一张卡的上限装不下最小的一片：不用它，不拒挂载");
        assert_eq!(
            opening.left_out_under_their_memory_limit,
            vec![GpuLeftOutUnderItsMemoryLimit {
                device: card("0000:01:00.0"),
                memory_limit: GpuMemoryInBytes(smallest_piece_bytes - 1),
                smallest_piece_bytes,
                bytes_already_held_on_the_device: 0,
            }],
            "报出不用的那张卡、它的上限与至少要多少"
        );
        assert_eq!(
            opening.participating.positions_on_this_host(),
            &[1],
            "只剩另一张参与"
        );
        assert_eq!(
            opening.opened[0],
            (
                "0000:01:00.0".to_string(),
                CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut
            ),
            "不用的那张照开（核心层照卡清单认它），不越过上限去量代价"
        );
        let refused_none = opening_that_holds_the_cost_probe_in_pieces([
            smallest_piece_bytes - 1,
            smallest_piece_bytes - 1,
        ])
        .expect("两张都不用：照样不拒挂载");
        assert!(
            refused_none.participating.is_empty(),
            "参与的卡一张都不剩：交核心层当 GPU 用不上那一态"
        );
        assert_eq!(refused_none.left_out_under_their_memory_limit.len(), 2);
    }

    /// 最小的一片：校验和那一段一个字节的单元（补到 4 字节、12 字节描述、两份核各 8 字节的结果与回读暂存、16 字节参数）、
    /// 选段打分那一段一个段连同一条记录（16 字节段界、两份核各 12 字节的结果与回读暂存、16 字节参数、20 字节记录与 4 字节序号）、
    /// 区域扫描那一段切开的单元头一片连同一条条目一个落点的映射（628 字节头一片、44 字节描述、两份核各 52 字节的结果与回读暂存、16 字节参数、
    /// 56 字节条目、16 字节落点项与 4 字节按 key 排的序号）三者取最大的，是区域扫描那一段的 972 字节。
    #[test]
    fn the_smallest_piece_every_participating_gpu_holds_is_the_largest_of_the_three_segments() {
        assert_eq!(bytes_of_the_smallest_checksum_piece(), 4 + 12 + 4 * 8 + 16);
        assert_eq!(
            bytes_of_the_smallest_live_slot_piece(),
            16 + 4 * 12 + 16 + 20 + 4
        );
        assert_eq!(
            bytes_of_the_smallest_region_record_piece(),
            628 + 44 + 4 * 52 + 16 + 56 + 16 + 4
        );
        assert_eq!(
            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
            972
        );
    }

    /// 丢掉的一张卡：记下它丢没丢；`waits_for_its_queue` 的那一张要是被丢，就是丢的时候等了它的队列（析构不带上限地等）。
    struct GpuDroppedByTheTest<'record> {
        name: &'static str,
        waits_for_its_queue: bool,
        dropped: &'record std::cell::RefCell<Vec<&'static str>>,
    }

    impl Drop for GpuDroppedByTheTest<'_> {
        fn drop(&mut self) {
            assert!(
                !self.waits_for_its_queue,
                "{} 还有没做完的提交，丢它就等了它的队列",
                self.name
            );
            self.dropped.borrow_mut().push(self.name);
        }
    }

    /// 用户 2026-10-28 定「卸载时同样不等、泄漏设备」：丢后台 GPU 时还有没做完的提交的那张卡不丢（泄漏），别的照常丢，调用即返回。
    #[test]
    fn dropping_the_gpus_leaks_the_one_with_a_running_submission_and_drops_the_others() {
        let dropped = std::cell::RefCell::new(Vec::new());
        let gpus = vec![
            GpuDroppedByTheTest {
                name: "跑完了的卡",
                waits_for_its_queue: false,
                dropped: &dropped,
            },
            GpuDroppedByTheTest {
                name: "还在跑的卡",
                waits_for_its_queue: true,
                dropped: &dropped,
            },
            GpuDroppedByTheTest {
                name: "没交过活的卡",
                waits_for_its_queue: false,
                dropped: &dropped,
            },
        ];
        let leaked = gpus_dropped_leaking_those_still_running(
            gpus,
            |gpu| gpu.waits_for_its_queue,
            |gpu| GpuLeakedWhenDropped {
                device: card(gpu.name),
                bytes_held_on_the_device: 600,
                bytes_leaked_by_earlier_mounts: 0,
            },
        );
        assert_eq!(
            leaked,
            vec![GpuLeakedWhenDropped {
                device: card("还在跑的卡"),
                bytes_held_on_the_device: 600,
                bytes_leaked_by_earlier_mounts: 0,
            }],
            "泄漏还在跑的那一张"
        );
        assert_eq!(
            *dropped.borrow(),
            vec!["跑完了的卡", "没交过活的卡"],
            "别的照常丢"
        );
    }

    /// 测试自己定的占用：每段另占 16 字节、每条另占 40 字节、按条排的最大缓冲每条 16 字节；最小的一片 4 + 40 + 16 = 60 字节。
    const FOOTPRINT_OF_THE_PLANNING_CASES: ChunkFootprint = ChunkFootprint {
        bytes_per_chunk_besides_the_entries: 16,
        bytes_per_entry_besides_its_input: 40,
        bytes_per_entry_in_the_largest_entry_buffer: 16,
    };

    /// 切法是 `pieces_cut_to_fit_on_the_gpu` 那一种（几段，每段几片）的一批的结局。
    type PlansOfPiecesOverTheGpus =
        Result<Vec<PlanOfAGpuForTheBatch<Vec<Vec<PieceOfAnEntry>>>>, BatchNotPlannedOverTheGpus>;

    /// 本机三张卡（位置 0、1、2）的总线地址。
    const ADDRESSES_OF_THE_PLANNING_CASES: [&str; 3] =
        ["0000:01:00.0", "0000:41:00.0", "0000:81:00.0"];

    /// 参与的卡按 `order`（本机位置，按参与的次序）、各自这一批交活之前那一刻给的上限 `memory_limits_in_bytes`（按本机位置，不设别的界），
    /// 一批各条的输入字节数 `lengths` 照 `pieces_cut_to_fit_on_the_gpu` 切、按卡的次序填满。交回切法与每张卡（按本机位置）被问了几次地方。
    fn plans_filling_the_gpus_in_order(
        order: &[usize],
        memory_limits_in_bytes: [u64; 3],
        lengths: &[u64],
    ) -> (PlansOfPiecesOverTheGpus, [u32; 3]) {
        let mut rooms_asked = [0_u32; 3];
        let plans = plans_of_a_batch_filling_the_gpus_in_order(
            &ParticipatingGpusOnThisHost::of_the_positions(order.to_vec()),
            lengths.len(),
            |position| {
                rooms_asked[position] += 1;
                Ok(RoomOfAGpuForTheBatch {
                    device: card(ADDRESSES_OF_THE_PLANNING_CASES[position]),
                    room: ChunkBoundsOfAGpu {
                        memory_limit: Some(GpuMemoryInBytes(memory_limits_in_bytes[position])),
                        largest_storage_binding_in_bytes: 1 << 30,
                        largest_entry_count_per_dispatch: 1 << 20,
                    },
                    memory_limit: Some(GpuMemoryInBytes(memory_limits_in_bytes[position])),
                    bytes_already_held_on_the_device: 0,
                })
            },
            |entries, room| {
                pieces_cut_to_fit_on_the_gpu(
                    &lengths[entries],
                    FOOTPRINT_OF_THE_PLANNING_CASES,
                    room,
                )
            },
            Vec::len,
        );
        (plans, rooms_asked)
    }

    /// 两张参与的卡（本机位置 0、1）这一批各自给的上限，四条各 100 字节（[`plans_filling_the_gpus_in_order`]）。
    fn plans_over_two_gpus_with_the_memory_limits(
        memory_limits_in_bytes: [u64; 2],
    ) -> (PlansOfPiecesOverTheGpus, [u32; 3]) {
        plans_filling_the_gpus_in_order(
            &[0, 1],
            [memory_limits_in_bytes[0], memory_limits_in_bytes[1], 0],
            &[100; 4],
        )
    }

    /// 切法交回的各卡：（本机位置，分到的条目，交几次）。
    fn positions_entries_and_submissions(
        plans: &[PlanOfAGpuForTheBatch<Vec<Vec<PieceOfAnEntry>>>],
    ) -> Vec<(usize, Range<usize>, usize)> {
        plans
            .iter()
            .map(|plan| {
                (
                    plan.position_on_this_host,
                    plan.entries.clone(),
                    plan.plan.len(),
                )
            })
            .collect()
    }

    /// 用户 2026-11-02 定「按卡的顺序填满」：两张卡各 1000 字节、四条各 100 字节一次交得下（4 × 140 + 16 = 576）——头一张全拿，
    /// 第二张一条都不分、不问它的地方（活少时只有头一张卡分到活，不要求每张卡都有活）。
    #[test]
    fn the_batch_that_one_gpu_holds_goes_wholly_to_the_first_gpu_in_the_order() {
        let (plans, rooms_asked) = plans_over_two_gpus_with_the_memory_limits([1000, 1000]);
        assert_eq!(
            positions_entries_and_submissions(&plans.expect("装得下：照常交回")),
            vec![(0, 0..4, 1)],
            "头一张全拿、一次交"
        );
        assert_eq!(rooms_asked, [1, 0, 0], "第二张没分到活：不问它的地方");
    }

    /// 同一条：头一张 300 字节一次只交得下两条（296 字节），第三条起才给第二张；第二张排在最后，拿剩下的两条。
    #[test]
    fn the_batch_overflowing_the_first_gpu_hands_the_rest_to_the_next_gpu_in_the_order() {
        let (plans, rooms_asked) = plans_over_two_gpus_with_the_memory_limits([300, 1000]);
        assert_eq!(
            positions_entries_and_submissions(&plans.expect("装得下：照常交回")),
            vec![(0, 0..2, 1), (1, 2..4, 1)],
            "头一张填满两条，剩下的给第二张"
        );
        assert_eq!(rooms_asked, [1, 1, 0]);
    }

    /// 同一条的饱和：三张卡各 300 字节、八条——前两张各填满两条、交一次，排在最后的那张拿剩下的四条、照它的地方切成两次交。
    #[test]
    fn the_batch_saturating_every_gpu_fills_each_gpu_and_leaves_the_rest_to_the_last_gpu() {
        let (plans, rooms_asked) =
            plans_filling_the_gpus_in_order(&[0, 1, 2], [300, 300, 300], &[100; 8]);
        assert_eq!(
            positions_entries_and_submissions(&plans.expect("装得下：照常交回")),
            vec![(0, 0..2, 1), (1, 2..4, 1), (2, 4..8, 2)],
            "每张都填满、各自都交，最后那张多交一次"
        );
        assert_eq!(rooms_asked, [1, 1, 1]);
    }

    /// 同一条：参与的次序是点名的次序（本机位置 1 在前）——活少时只有点名在前的那张分到活。
    #[test]
    fn the_batch_fills_the_gpus_in_the_named_order_rather_than_the_bus_address_order() {
        let (plans, rooms_asked) =
            plans_filling_the_gpus_in_order(&[1, 0], [1000, 1000, 0], &[100; 4]);
        assert_eq!(
            positions_entries_and_submissions(&plans.expect("装得下：照常交回")),
            vec![(1, 0..4, 1)],
            "点名在前的本机位置 1 全拿"
        );
        assert_eq!(rooms_asked, [0, 1, 0]);
    }

    /// 同一条：头一条 500 字节比头一张的地方（300 字节）大，切成 244、244、12 字节三片交三次；后面两条各 10 字节跟着最后那一片交、
    /// 不多添一次，头一张拿前三条；第四条（100 字节）要多交一次，给第二张。
    #[test]
    fn the_gpu_whose_first_entry_is_cut_into_pieces_also_takes_the_entries_riding_along_its_last_piece(
    ) {
        let (plans, rooms_asked) =
            plans_filling_the_gpus_in_order(&[0, 1], [300, 1000, 0], &[500, 10, 10, 100]);
        assert_eq!(
            positions_entries_and_submissions(&plans.expect("装得下：照常交回")),
            vec![(0, 0..3, 3), (1, 3..4, 1)],
            "头一张交三次、带上跟着最后一片的两条"
        );
        assert_eq!(rooms_asked, [1, 1, 0]);
    }

    /// 判决 m4-gpugc-code-r1 第三节第 4 条：两张参与的卡，一张这一批连最小的一片（60 字节）都切不出（上限 50）——这一批不分给它，四条全分给另一张、
    /// 照它的上限切，照常交回，不停整理（攻方探针 T3-1 那一种，改前这一批交回切不出、整理停下）。
    #[test]
    fn one_gpu_short_of_the_smallest_piece_of_the_batch_is_left_out_and_the_other_gpu_takes_the_whole_batch(
    ) {
        let (plans, rooms_asked) = plans_over_two_gpus_with_the_memory_limits([50, 1000]);
        let plans = plans.expect("另一张卡切得出：照常交回");
        assert_eq!(
            positions_entries_and_submissions(&plans),
            vec![(1, 0..4, 1)],
            "四条全分给本机位置 1 那张卡，一段装下（4 × 140 + 16 = 576 字节不超 1000）"
        );
        assert_eq!(
            rooms_asked,
            [1, 1, 0],
            "头一轮问到切不出的那张就重分；另一张只在重分之后问一次"
        );
    }

    /// 同一条：参与的卡这一批全都切不出最小的一片，才报参与的卡一张都不剩，按不用的先后带每张卡、上限、最小的一片与已占的字节。
    #[test]
    fn every_participating_gpu_short_of_the_smallest_piece_of_the_batch_is_reported_as_every_gpu_left_out(
    ) {
        let (plans, _rooms_asked) = plans_over_two_gpus_with_the_memory_limits([50, 59]);
        assert_eq!(
            plans,
            Err(BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(
                vec![
                    GpuLeftOutUnderItsMemoryLimit {
                        device: card("0000:01:00.0"),
                        memory_limit: GpuMemoryInBytes(50),
                        smallest_piece_bytes: 60,
                        bytes_already_held_on_the_device: 0,
                    },
                    GpuLeftOutUnderItsMemoryLimit {
                        device: card("0000:41:00.0"),
                        memory_limit: GpuMemoryInBytes(59),
                        smallest_piece_bytes: 60,
                        bytes_already_held_on_the_device: 0,
                    },
                ]
            ))
        );
    }

    /// 判决第三节第 6 条：一次丢后台 GPU 时泄漏了一张卡 600 字节，进程里记下；同一张卡新一次挂载、上限 1000 字节，账从 600 起、
    /// 可用余量只剩 400：五条各 100 字节切成两条一段（296 字节）的三段，与只剩 400 字节时相同。同一张卡这一次挂载再泄漏（那一刻占着 900，其中 600 是开卡时已算上的），加上这一次自己的 300，记 900、不重算已含的那一片。
    #[test]
    fn memory_leaked_by_an_earlier_mount_is_taken_off_the_room_of_the_next_mount_on_that_gpu() {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
            device: card("0000:01:00.0"),
            bytes_held_on_the_device: 600,
            bytes_leaked_by_earlier_mounts: 0,
        }]);
        assert_eq!(
            (
                memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
                memory_leaked.bytes_leaked_on(&card("0000:41:00.0"))
            ),
            (600, 0),
            "只记泄漏的那一张"
        );
        let mut budget: GpuBudgetUnderTheSettings<u32> =
            GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
                1 << 30,
                1 << 20,
                Some(GpuMemoryInBytes(1000)),
                Duration::from_secs(10),
                memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
            );
        assert_eq!(
            budget.room_left_now().memory_limit,
            Some(GpuMemoryInBytes(400)),
            "上限 1000 先扣掉泄漏的 600"
        );
        assert_eq!(
            budget.chunks_cut_once_the_submission_running_past_its_wait_completed(
                &ClockThatNeverMoves,
                |_submission, _time_left| panic!("没有还在跑的提交，不该等"),
                &[100, 100, 100, 100, 100],
                FOOTPRINT_OF_THE_PLANNING_CASES,
            ),
            Ok(vec![0..2, 2..4, 4..5]),
            "只用剩下的 400 字节：两条一段"
        );
        memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
            device: card("0000:01:00.0"),
            bytes_held_on_the_device: 900,
            bytes_leaked_by_earlier_mounts: 600,
        }]);
        assert_eq!(memory_leaked.bytes_leaked_on(&card("0000:01:00.0")), 900);
    }

    /// 判决 m4-gpugc-code-r2 第三节第 4 条：同一进程里两次挂载同时开着、都在同一张卡上泄漏（开卡那一刻都没算上对方的），先丢的那一次 300 字节、
    /// 后丢的那一次 600 字节——进程里这张卡按次累加记 900，不让后丢的那一次盖掉先丢的。
    #[test]
    fn two_mounts_alive_at_once_leaking_on_one_gpu_add_up_in_the_process() {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        for bytes_leaked_by_the_mount in [300, 600] {
            memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
                device: card("0000:01:00.0"),
                bytes_held_on_the_device: bytes_leaked_by_the_mount,
                bytes_leaked_by_earlier_mounts: 0,
            }]);
        }
        assert_eq!(
            memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
            900,
            "两次各 300、600 字节：记 900"
        );
    }

    /// 同一条在开卡那一刻：一张参与的卡上限 1000 字节、进程里记着它泄漏了 600 字节——余量 400 装不下最小的一片（972 字节）也不拒挂载，
    /// 这张卡照开、不量代价、不参与，报出它的上限、最小的一片与已占的 600 字节；不泄漏时同一个上限照常参与。
    #[test]
    fn the_gpu_whose_room_after_the_memory_leaked_in_this_process_is_short_of_the_smallest_piece_is_left_out_at_the_mount(
    ) {
        let selection = BackgroundGpuSelection {
            background_gpu_devices: Some(vec![card("0000:01:00.0"), card("0000:41:00.0")]),
            background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(1000), GpuMemoryInBytes(1000)]),
            background_gpu_wait_limit: Duration::from_secs(10),
        };
        let discrete_gpus = three_discrete_gpus_of_the_enumeration();
        let participating =
            participating_gpus_admitted_by_the_mount_checks(&selection, &discrete_gpus)
                .expect("两张卡、各 1000 字节：校验合法");
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
            device: card("0000:01:00.0"),
            bytes_held_on_the_device: 600,
            bytes_leaked_by_earlier_mounts: 0,
        }]);
        let opening = gpus_opened_at_the_mount(
            discrete_gpus,
            &participating,
            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
            &memory_leaked,
            |discrete_gpu, _memory_limit, cost_at_the_opening| {
                Ok((discrete_gpu.pci_bus_address.clone(), cost_at_the_opening))
            },
        )
        .expect("余量装不下最小的一片：不用它，不拒挂载");
        assert_eq!(
            opening.left_out_under_their_memory_limit,
            vec![GpuLeftOutUnderItsMemoryLimit {
                device: card("0000:01:00.0"),
                memory_limit: GpuMemoryInBytes(1000),
                smallest_piece_bytes: 972,
                bytes_already_held_on_the_device: 600,
            }]
        );
        assert_eq!(
            opening.participating.positions_on_this_host(),
            &[1],
            "只剩没泄漏的那一张参与"
        );
        assert_eq!(
            opening.opened[0],
            (
                "0000:01:00.0".to_string(),
                CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut
            )
        );
    }
}
