//! 挂载那一刻开着的本机全部独显里，此刻照哪一组设置用哪几张、每张怎么切、等多久（D26（后台整理与放置回收） 已定项 13 第 1–3 条；
//! `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 43 批交回那一行的定法：挂载那一刻开全部独显并量代价，设置决定哪几张参与，切换不重开）。
//! 纯逻辑，不碰 GPU：外壳（`crate::background_gpu`）每张卡一份 [`GpuBudgetUnderTheSettings`]，参与的卡由 [`ParticipatingGpusOnThisHost`] 定；
//! 核心层交来一组新设置（`BackgroundGpuCompute::use_the_settings_from_the_next_batch`，只在两批之间）就换这两样，从下一批起照它分活、切段、等。
//!
//! 挂载那一刻一张卡没开成挡不挡挂载（第 2、4 条与同一份记录第 45 批交回那一行的定法）在 [`gpus_opened_at_the_mount`]：参与的卡没开成就拒，
//! 给了 `background_gpu_devices` 而没点名的卡没开成就跳过、报出来（[`DiscreteGpusAfterTheOpening`]），卡清单照列它、并报它没开成，
//! 核心层挂着切换点到它照第 4 条拒（校验同挂载那一刻）。
//!
//! 换设置不清账：一张卡上等超过上限、还在 GPU 上跑的那次提交照旧记着，下一批在这张卡上交活之前先在新的等的上限之内等它做完，
//! 再按新的显存上限扣掉还占着的切（第 3 条）；连续不确定的计数不在这里，换设置不碰它（第 8 条）。

use std::ops::Range;
use std::time::Duration;

use singlefs_core::background_compute::{
    AdmittedGpuSettings, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuInventory,
    GpuMemoryInBytes,
};

use crate::adapter_inventory::{DiscreteGpuOnThisHost, ParticipatingGpu};
use crate::background_gpu::{BackgroundGpuOpenRefused, GpuFailureCause};
use crate::background_gpu_compute::core_failure_of_the_gpu;
use crate::memory_limit_and_chunks::{
    ChunkBoundsOfAGpu, ChunkFootprint, ChunksNotCutOnTheGpu, GpuMemoryAccount,
};
use crate::waiting_for_the_gpu::{CompletionCheck, GpuWaitDeadline, MonotonicClock};
use crate::work_split_by_measured_cost::{
    entries_per_gpu_by_measured_cost, entry_ranges_per_gpu, HostCpuNanosecondsPerSubmission,
};

/// 一张卡此刻照的设置管的那几样：显存上限（记在账上）、等 GPU 的上限，连同与设置无关的绑定上限与派发上限、本包在它上面的账。
#[derive(Debug)]
pub struct GpuBudgetUnderTheSettings<Submission> {
    largest_storage_binding_in_bytes: u64,
    largest_entry_count_per_dispatch: u64,
    memory_account: GpuMemoryAccount<Submission>,
    wait_limit: Duration,
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
        Self {
            largest_storage_binding_in_bytes,
            largest_entry_count_per_dispatch,
            memory_account: GpuMemoryAccount::with_nothing_held(memory_limit),
            wait_limit,
        }
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

    #[must_use]
    pub fn memory_account(&self) -> &GpuMemoryAccount<Submission> {
        &self.memory_account
    }

    pub fn memory_account_mut(&mut self) -> &mut GpuMemoryAccount<Submission> {
        &mut self.memory_account
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

/// 此刻参与的卡：在本机全部独显（按总线地址排的那一份）里的位置，按参与的次序（设置点名的次序，没点名是卡清单的次序）。至少一张、不重。
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

    /// 开卡那一刻照开卡那一组定的参与的卡（位置按参与的次序）。
    ///
    /// # Panics
    /// 一张都没有：开卡那一刻一张独显都没有就拒了。
    #[must_use]
    pub fn of_the_positions(positions_on_this_host: Vec<usize>) -> Self {
        assert!(
            !positions_on_this_host.is_empty(),
            "开卡那一刻至少一张参与的卡"
        );
        Self {
            positions_on_this_host,
        }
    }

    #[must_use]
    pub fn positions_on_this_host(&self) -> &[usize] {
        &self.positions_on_this_host
    }

    /// 一批的条目按挂载时量出的代价分给参与的卡（第 2 条，`work_split_by_measured_cost`）：每张参与的卡一段相连的条目，按参与的次序，
    /// 交回（它在本机里的位置，分到的那几条）。不参与的卡一条都不分。
    #[must_use]
    pub fn entry_ranges_by_measured_cost(
        &self,
        measured_costs_on_this_host: &[HostCpuNanosecondsPerSubmission],
        entry_count: usize,
    ) -> Vec<(usize, Range<usize>)> {
        let measured_costs: Vec<HostCpuNanosecondsPerSubmission> = self
            .positions_on_this_host
            .iter()
            .map(|position| measured_costs_on_this_host[*position])
            .collect();
        self.positions_on_this_host
            .iter()
            .copied()
            .zip(entry_ranges_per_gpu(&entries_per_gpu_by_measured_cost(
                &measured_costs,
                entry_count,
            )))
            .collect()
    }
}

/// 本机的一张独显挂载那一刻没开成（起不了设备、编不了管线、量代价那几次提交出错或超时）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscreteGpuNotOpenedAtTheMount {
    pub discrete_gpu: DiscreteGpuOnThisHost,
    pub cause: GpuFailureCause,
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

/// 挂载那一刻开卡的结局：开成的每张卡开出来的东西（与 `discrete_gpus.opened` 一一对应、同序）、开卡之后的本机独显、开卡那一组参与的卡
/// （在开成的那几张里的位置）。
#[derive(Debug)]
pub struct GpusOpenedAtTheMount<Opened> {
    pub opened: Vec<Opened>,
    pub discrete_gpus: DiscreteGpusAfterTheOpening,
    pub participating: ParticipatingGpusOnThisHost,
}

/// 挂载那一刻在本机每一张独显上开卡（按 `discrete_gpus` 的次序，即总线地址的次序）：`open_the_gpu` 是外壳起设备、编管线、量代价那一步，
/// 交给它这张卡与开卡那一组给它的显存上限；测试里交造出来的结局。一张卡没开成挡不挡挂载（D26（后台整理与放置回收） 已定项 13 第 2、4 条，
/// `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 45 批交回那一行的定法）：它参与（`participating` 里有它——给了 `background_gpu_devices` 时是点名的卡，
/// 没给时本机全部独显都参与）就拒绝挂载、报出它与原因，后面的卡不再开；不参与（给了列表而没点它）就跳过，不进参与的卡，记进
/// [`DiscreteGpusAfterTheOpening::not_opened`] 报出来，挂载照常。
/// `participating` 是开卡那一组照同一份 `discrete_gpus` 校验过、取出来的参与的卡（`adapter_inventory::participating_gpus_admitted_by_the_mount_checks`）。
///
/// # Errors
/// [`BackgroundGpuOpenRefused::GpuUnusable`]：一张参与的卡没开成。
///
/// # Panics
/// `participating` 里有一张不在 `discrete_gpus` 里：它取自同一份清单。
pub fn gpus_opened_at_the_mount<Opened>(
    discrete_gpus: Vec<DiscreteGpuOnThisHost>,
    participating: &[ParticipatingGpu],
    mut open_the_gpu: impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
    ) -> Result<Opened, GpuFailureCause>,
) -> Result<GpusOpenedAtTheMount<Opened>, BackgroundGpuOpenRefused> {
    let mut opened = Vec::with_capacity(discrete_gpus.len());
    let mut opened_discrete_gpus: Vec<DiscreteGpuOnThisHost> =
        Vec::with_capacity(discrete_gpus.len());
    let mut not_opened = Vec::new();
    for discrete_gpu in discrete_gpus {
        let participating_gpu = participating.iter().find(|participating_gpu| {
            participating_gpu.discrete_gpu.identifier == discrete_gpu.identifier
        });
        let memory_limit_at_the_opening =
            participating_gpu.and_then(|participating_gpu| participating_gpu.memory_limit);
        match open_the_gpu(&discrete_gpu, memory_limit_at_the_opening) {
            Ok(opened_gpu) => {
                opened.push(opened_gpu);
                opened_discrete_gpus.push(discrete_gpu);
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
    Ok(GpusOpenedAtTheMount {
        opened,
        discrete_gpus: DiscreteGpusAfterTheOpening {
            opened: opened_discrete_gpus,
            not_opened,
        },
        participating: ParticipatingGpusOnThisHost::of_the_positions(participating_positions),
    })
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

    /// 三张卡挂载时量出的代价：头一张最便宜。
    fn measured_costs_on_this_host() -> Vec<HostCpuNanosecondsPerSubmission> {
        vec![
            HostCpuNanosecondsPerSubmission(1_000),
            HostCpuNanosecondsPerSubmission(2_000),
            HostCpuNanosecondsPerSubmission(4_000),
        ]
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

    /// 第 2 条：挂载时三张都参与、按代价分；切换缩成点名的两张（后一张在前）之后，下一批只分给这两张、按点名的次序，第三张一条都不分。
    #[test]
    fn after_a_switch_to_fewer_cards_the_next_batch_is_split_only_over_the_named_cards() {
        let at_the_mount = ParticipatingGpusOnThisHost::of_the_settings(
            &admitted(None, None, Duration::from_secs(10)),
            &identifiers_on_this_host(),
        );
        assert_eq!(
            at_the_mount.entry_ranges_by_measured_cost(&measured_costs_on_this_host(), 7),
            vec![(0, 0..4), (1, 4..6), (2, 6..7)],
            "没点名：本机三张都分，份额与代价成反比"
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
            after_the_switch.entry_ranges_by_measured_cost(&measured_costs_on_this_host(), 10),
            vec![(2, 0..2), (0, 2..10)],
            "点名两张：只分给它们、按点名的次序，代价照挂载时量的；中间那张一条都不分"
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
            |discrete_gpu, memory_limit| {
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
    fn a_named_card_that_did_not_open_refuses_the_mount() {
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
}
