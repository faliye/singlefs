use super::*;
use std::cell::Cell;

/// 假钟：每读一次走一秒。
struct ClockMovingOneSecondPerReading {
    readings: Cell<u64>,
}

impl MonotonicClock for ClockMovingOneSecondPerReading {
    fn time_since_the_origin(&self) -> Duration {
        self.readings.set(self.readings.get() + 1);
        Duration::from_secs(self.readings.get())
    }
}

use std::collections::VecDeque;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{AllocationRecordQuarantineMark, UnitAreaStart};
use singlefs_core::background_compute::admit_gpu_settings_against_the_inventory;
use singlefs_core::background_compute::{
    BackgroundComputeBackend, BackgroundComputeSettings, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};

use crate::checksum_kernels::cpu_reference_outcome;
use crate::live_slot_kernels::live_slot_chunk_footprint;
use crate::memory_limit_and_chunks::{GpuMemoryAccount, SubmissionOfTheChunk};
use crate::pci_state_of_the_gpu::{
    pci_state_read_under_the_root_not_present_for_tests,
    pci_state_reader_under_a_root_not_present_for_tests,
};
use crate::two_kernel_reconciliation::EntryDecision;

use crate::background_gpu_compute::BackgroundGpuComputeThroughBothKernels;
use crate::waiting_for_the_gpu::caller_stopping_for_tests::caller_stopping_flag_and_the_handle_setting_it;
use singlefs_core::background_compute::{
    BackgroundGpuCompute, BackgroundGpuComputeFailed, BackgroundGpuComputeHandle,
    RegionRecordEntryKey, UnitChecksumToVerifyOnTheGpu,
};

mod batches_over_the_cards;
mod buffers_of_a_chunk_in_one_out_of_memory_scope;
mod caller_stopping;
mod cards_and_their_settings_between_batches;
mod cards_leaked_when_dropped_or_closed;
mod failed_openings_in_a_mount;
mod measurement_at_a_switch_after_a_timeout;
mod opening_of_the_gpus;
mod opening_thread_at_a_switch;
mod openings_not_returned_in_this_process;
mod panic_in_the_wait_for_a_submission;
mod pci_state_read_when_a_card_is_judged_unusable;
mod switching_back_while_an_opening_is_held;
mod waiting_for_wgpu_futures_and_thread_clocks;

const WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU: Duration = Duration::from_secs(10);
/// 用例里开卡开不成照产品那样至多重开 3 次，间隔为 0：不真等 1 分钟。
const REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS: ReopeningsOfAGpuNotOpened =
    ReopeningsOfAGpuNotOpened {
        interval: Duration::ZERO,
        reopenings_at_most: 3,
    };
const EIGHT_GIBIBYTES: u64 = 8 << 30;

/// 替身卡上的钟不走：替身等前一次提交照脚本当场交回，不真等。
struct ClockOfTheCardsOnTheCpu;

impl MonotonicClock for ClockOfTheCardsOnTheCpu {
    fn time_since_the_origin(&self) -> Duration {
        Duration::ZERO
    }
}

/// 在 CPU 上算的替身卡：两份核都照 CPU 参照算（校验和那一段 `cpu_reference_outcome`；区域扫描与选段打分那两段只交形状对的结果，用例只看交没交成、
/// 交了几次），账与产品那张卡同一份（`GpuBudgetUnderTheSettings`）。按脚本造三样：交活之前、重交之前等前一次超时的那次提交等满上限没等到
/// （`earlier_submission_waits_timing_out`，按次序，`true` 是没等到；卡上记着一次还在跑的提交时才等、才取脚本），每一次提交等超过上限
/// （`submissions_timing_out`，按次序，`true` 是超时，超时的那一段的账照产品记成还在跑的那次提交），卡上空闲的显存（`free_bytes`：一段合占的比它多就建不成缓冲）。
/// 脚本用完之后都等到、都做完；`next_submission_failing_with` 给了时下一次交一段当场交回那个错（只一次，例：设备丢了），放掉那一段的账。另记量了几次代价（`cost_measurements`；`cost_measurement_failing_with` 给了时每次量都交回那个错、不记代价；
/// `cost_measured_through_the_checksum_chunk` 时照产品那样把量代价那一段照此刻剩下的界切片、逐段当一次提交走账：先记账、超时就记成还在跑），可点名第几次交成的校验和那一段交回翻过的判定
/// （`checksum_submission_with_flipped_verdicts`，从 1 数：那一次的结果是坏的，用了它判定就与 CPU 参照不同）。
/// 析构一次就在 `drops_counted` 上加一（泄漏的卡不析构，计数不动）：用例在卡交出去之前取一份它的克隆。
/// 只供测试的钩子 `submission_waits_panicking`：交成一段之后等它时，那一次等交给产品等提交那一步（`completion_check_of_one_poll_with_its_panic_caught`），
/// 等的那一下 panic（造真卡掉线时 wgpu 的 `Device::poll` panic）；交活之前、重交之前等前一次还在跑的提交也走产品那一步（没 panic 过的当场等到），
/// panic 的原话记在 `panic_in_a_wait_of_the_device`。
/// `submissions_never_completing` 时交成一段之后那一次提交一直不做完：照产品从这张卡的账上取期限（读不读调用方在停的标记照账上记着的），
/// 在每读一次走一秒的钟上等它，等的结局（等满上限，或调用方在停而放弃）照原样交回，这一段记成还在跑的那次提交。
struct CardComputedOnTheCpuForTests {
    discrete_gpu: DiscreteGpuOnThisHost,
    budget: GpuBudgetUnderTheSettings<u32>,
    measured_cost: Option<HostCpuNanosecondsPerSubmission>,
    earlier_submission_waits_timing_out: VecDeque<bool>,
    submissions_timing_out: VecDeque<bool>,
    free_bytes: Option<u64>,
    submissions_made: u32,
    cost_measurements: u32,
    checksum_submission_with_flipped_verdicts: Option<u32>,
    cost_measurement_failing_with: Option<GpuFailureCause>,
    drops_counted: Arc<std::sync::atomic::AtomicU32>,
    cost_measured_through_the_checksum_chunk: bool,
    next_submission_failing_with: Option<GpuFailureCause>,
    submission_waits_panicking: bool,
    panic_in_a_wait_of_the_device: OnceLock<String>,
    submissions_never_completing: bool,
}

impl Drop for CardComputedOnTheCpuForTests {
    fn drop(&mut self) {
        self.drops_counted
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

impl CardComputedOnTheCpuForTests {
    /// 总线地址 `pci_bus_address` 的一张 8 GiB 替身卡，量过代价，给它的显存上限 `memory_limit_in_bytes`，这个进程里之前泄漏在它上面 `bytes_leaked` 字节。
    fn of_the_address(
        pci_bus_address: &str,
        memory_limit_in_bytes: Option<u64>,
        bytes_leaked: u64,
    ) -> Self {
        Self {
            discrete_gpu: DiscreteGpuOnThisHost {
                position_in_the_enumeration: 0,
                identifier: card_identifier(pci_bus_address),
                pci_bus_address: pci_bus_address.to_string(),
                name: format!("cpu card at {pci_bus_address}"),
                total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
            },
            budget: GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
                GpuMemoryInBytes(1 << 30),
                EntryCountOfAChunk(1 << 20),
                memory_limit_in_bytes.map(GpuMemoryInBytes),
                WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                GpuMemoryInBytes(bytes_leaked),
            ),
            measured_cost: Some(HostCpuNanosecondsPerSubmission(1_000)),
            earlier_submission_waits_timing_out: VecDeque::new(),
            submissions_timing_out: VecDeque::new(),
            free_bytes: None,
            submissions_made: 0,
            cost_measurements: 0,
            checksum_submission_with_flipped_verdicts: None,
            cost_measurement_failing_with: None,
            drops_counted: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            cost_measured_through_the_checksum_chunk: false,
            next_submission_failing_with: None,
            submission_waits_panicking: false,
            panic_in_a_wait_of_the_device: OnceLock::new(),
            submissions_never_completing: false,
        }
    }

    /// 卡上记着一次等超过上限、还在跑的提交（占 `bytes` 字节），交活之前要先等它。
    fn with_a_submission_running_past_its_wait(mut self, bytes: u64) -> Self {
        self.budget
            .memory_account_mut()
            .hold(GpuMemoryInBytes(bytes));
        self.budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                GpuMemoryInBytes(bytes),
                SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(0),
            );
        self
    }

    /// 交一段（合占 `bytes_held_by_the_chunk` 字节）：照产品记账；空闲的不够就放掉、交回显存不足；照脚本超时就把这一段记成还在跑的那次提交、交回超时；
    /// 做完了放账、交回 `outcome`。
    fn chunk_submitted_on_the_cpu<Outcome>(
        &mut self,
        bytes_held_by_the_chunk: GpuMemoryInBytes,
        outcome: impl FnOnce() -> Outcome,
    ) -> Result<Outcome, GpuFailureCause> {
        self.budget
            .memory_account_mut()
            .hold(bytes_held_by_the_chunk);
        if self
            .free_bytes
            .is_some_and(|free_bytes| bytes_held_by_the_chunk.0 > free_bytes)
        {
            self.budget
                .memory_account_mut()
                .release(bytes_held_by_the_chunk);
            return Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                bytes_held_by_the_chunk,
                reason: "替身卡上空闲的不够".to_string(),
            });
        }
        if let Some(cause) = self.next_submission_failing_with.take() {
            self.budget
                .memory_account_mut()
                .release(bytes_held_by_the_chunk);
            return Err(cause);
        }
        self.submissions_made += 1;
        if self.submission_waits_panicking {
            let waited = completion_check_of_one_poll_with_its_panic_caught(
                &self.panic_in_a_wait_of_the_device,
                || panic!("{PANIC_SAID_BY_THE_WAIT_OF_THE_TEST_CARD}"),
            );
            let cause = match waited {
                CompletionCheck::Failed(cause) => cause,
                CompletionCheck::Completed(()) | CompletionCheck::NotYetComplete => {
                    panic!("替身：等的那一下 panic 了，只交得回出错")
                }
            };
            self.budget
                .memory_account_mut()
                .settle_the_chunk_after_its_submission(
                    bytes_held_by_the_chunk,
                    SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(self.submissions_made),
                );
            return Err(cause);
        }
        if self.submissions_never_completing {
            let clock = ClockMovingOneSecondPerReading {
                readings: Cell::new(0),
            };
            let waited = wait_for_completion_before_the_deadline(
                &self
                    .budget
                    .deadline_of_a_wait_of_the_batch_starting_now(&clock),
                &clock,
                |_time_left| CompletionCheck::<()>::NotYetComplete,
            );
            let cause = waited.expect_err("替身：这一次提交一直不做完，等它只交得回出错");
            self.budget
                .memory_account_mut()
                .settle_the_chunk_after_its_submission(
                    bytes_held_by_the_chunk,
                    SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(self.submissions_made),
                );
            return Err(cause);
        }
        if self.submissions_timing_out.pop_front().unwrap_or(false) {
            self.budget
                .memory_account_mut()
                .settle_the_chunk_after_its_submission(
                    bytes_held_by_the_chunk,
                    SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(self.submissions_made),
                );
            return Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
            });
        }
        self.budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                bytes_held_by_the_chunk,
                SubmissionOfTheChunk::CompletedOnTheGpu,
            );
        Ok(outcome())
    }

    /// 卡上记着一次还在跑的提交、脚本这一回说没等到：交回等满上限。
    fn earlier_submission_still_running_this_time(&mut self) -> bool {
        self.budget
            .memory_account()
            .has_a_submission_running_past_its_wait()
            && self
                .earlier_submission_waits_timing_out
                .pop_front()
                .unwrap_or(false)
    }
}

impl CardOfTheBackgroundGpu for CardComputedOnTheCpuForTests {
    type Submission = u32;

    fn discrete_gpu(&self) -> &DiscreteGpuOnThisHost {
        &self.discrete_gpu
    }

    fn budget(&self) -> &GpuBudgetUnderTheSettings<u32> {
        &self.budget
    }

    fn budget_mut(&mut self) -> &mut GpuBudgetUnderTheSettings<u32> {
        &mut self.budget
    }

    fn measured_cost(&self) -> Option<HostCpuNanosecondsPerSubmission> {
        self.measured_cost
    }

    fn measure_the_cost_of_one_submission(&mut self) -> Result<(), GpuFailureCause> {
        self.cost_measurements += 1;
        if let Some(cause) = &self.cost_measurement_failing_with {
            return Err(cause.clone());
        }
        if self.cost_measured_through_the_checksum_chunk {
            let chunks_of_pieces = COST_PROBE_CHUNK
                .pieces_under_the_bounds(self.budget.room_left_now())
                .map_err(
                    |smallest_piece| GpuFailureCause::DeviceOrPipelineDidNotStart {
                        reason: format!(
                            "替身：量代价那一段最小的一片 {} 字节装不下",
                            smallest_piece.smallest_piece_bytes.0
                        ),
                    },
                )?;
            for pieces in &chunks_of_pieces {
                let input_bytes: InputLengthOnTheHostInBytes = pieces
                    .iter()
                    .map(crate::memory_limit_and_chunks::PieceOfAnEntry::input_byte_count)
                    .sum();
                let bytes_held_by_the_chunk = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
                    input_bytes,
                    EntryCountOfAChunk::of_this_many_entries(pieces.len()),
                );
                self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || ())?;
            }
        }
        self.measured_cost = Some(HostCpuNanosecondsPerSubmission(1_000));
        Ok(())
    }

    fn room_once_the_submission_running_past_its_wait_completed(
        &mut self,
    ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
        if self.earlier_submission_still_running_this_time() {
            return Err(
                GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                    wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                },
            );
        }
        let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
        self.budget
            .room_once_the_submission_running_past_its_wait_completed(
                &ClockOfTheCardsOnTheCpu,
                |_submission, _time_left| {
                    completion_check_of_one_poll_with_its_panic_caught(
                        panic_in_a_wait_of_the_device,
                        || Ok(wgpu::PollStatus::WaitSucceeded),
                    )
                },
            )
    }

    fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
        if self.earlier_submission_still_running_this_time() {
            return Err(
                GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                    wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                },
            );
        }
        let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
        self.budget.wait_for_the_submission_running_past_its_wait(
            &ClockOfTheCardsOnTheCpu,
            |_submission, _time_left| {
                completion_check_of_one_poll_with_its_panic_caught(
                    panic_in_a_wait_of_the_device,
                    || Ok(wgpu::PollStatus::WaitSucceeded),
                )
            },
        )
    }

    fn both_kernel_outcomes_of_a_checksum_chunk(
        &mut self,
        units: &[UnitChecksumToVerify<'_, ()>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
        let input_bytes: usize = units.iter().map(|unit| unit.unit_bytes.len()).sum();
        let bytes_held_by_the_chunk = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
            InputLengthOnTheHostInBytes::of_this_many_bytes(input_bytes),
            EntryCountOfAChunk::of_this_many_entries(units.len()),
        );
        let submission_with_flipped_verdicts = self.checksum_submission_with_flipped_verdicts;
        let submission_number = self.submissions_made + 1;
        self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || {
            units
                .iter()
                .map(|unit| {
                    let mut outcome = cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);
                    if submission_with_flipped_verdicts == Some(submission_number) {
                        outcome.computed_crc32c ^= 1;
                        outcome.matches_the_expected_crc32c = !outcome.matches_the_expected_crc32c;
                    }
                    (outcome, outcome)
                })
                .collect()
        })
    }

    fn both_kernel_outcomes_of_a_region_record_chunk(
        &mut self,
        pieces: &[RegionRecordPieceForTheKernels<'_>],
        mapping_tables: Option<&MappingTablesForTheKernels>,
    ) -> Result<BothKernelOutcomesOfARegionRecordChunk, GpuFailureCause> {
        let empty_mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&[])
            .expect("空映射没有长过核装得下的 key");
        let input_bytes: usize = pieces
            .iter()
            .map(|piece| piece.bytes().map_or(0, <[u8]>::len))
            .sum();
        let bytes_held_by_the_chunk =
            region_record_chunk_footprint(mapping_tables.unwrap_or(&empty_mapping_tables))
                .bytes_held_by_a_chunk(
                    InputLengthOnTheHostInBytes::of_this_many_bytes(input_bytes),
                    EntryCountOfAChunk::of_this_many_entries(pieces.len()),
                );
        self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || match mapping_tables {
            None => BothKernelOutcomesOfARegionRecordChunk::RegistersAfterThePieces(vec![
                    CrcRegistersOfTheTwoKernels::BEFORE_ANY_BYTE;
                    pieces.len()
                ]),
            Some(_tables) => {
                BothKernelOutcomesOfARegionRecordChunk::Classifications(vec![
                    (
                        AllocationInRegion::Pinned,
                        AllocationInRegion::Pinned
                    );
                    pieces.len()
                ])
            }
        })
    }

    fn both_kernel_live_slots_of_a_chunk(
        &mut self,
        segments: &[ClusterSegmentEntryKey],
        record_tables: &AllocationRecordTablesForTheKernels,
        _unit_area: UnitAreaOfOneDevice,
    ) -> Result<Vec<(u64, u64)>, GpuFailureCause> {
        let segment_count = EntryCountOfAChunk::of_this_many_entries(segments.len());
        let bytes_held_by_the_chunk = live_slot_chunk_footprint(record_tables)
            .bytes_held_by_a_chunk(
                segment_count.input_bytes_of_the_entries(SEGMENT_BOUND_BYTES),
                segment_count,
            );
        self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || vec![(0, 0); segments.len()])
    }
}

fn card_identifier(pci_bus_address: &str) -> GpuDeviceIdentifier {
    GpuDeviceIdentifier::of_opaque_bytes(pci_bus_address.as_bytes().to_vec())
}

/// 替身的「此刻本机在的独显」：用例改它模拟挂载之后掉了一张卡。
struct DiscreteGpusPresentForTests(Arc<Mutex<Vec<GpuDeviceIdentifier>>>);

impl DiscreteGpusPresentNow for DiscreteGpusPresentForTests {
    fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
        self.0.lock().expect("用例没在别处 panic 过").clone()
    }
}

/// 外壳装上这几张替身卡（按交进来的次序就是本机的次序，全都参与），进程里泄漏的那一份是 `memory_leaked`；交回外壳与「此刻本机在的独显」那一份（一开始全在）。
fn background_gpu_over_cards_on_the_cpu(
    cards: Vec<CardComputedOnTheCpuForTests>,
    memory_leaked: &GpuMemoryLeakedInThisProcess,
) -> (
    BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    Arc<Mutex<Vec<GpuDeviceIdentifier>>>,
) {
    let discrete_gpus: Vec<DiscreteGpuOnThisHost> =
        cards.iter().map(|card| card.discrete_gpu.clone()).collect();
    let present = Arc::new(Mutex::new(
        discrete_gpus
            .iter()
            .map(|discrete_gpu| discrete_gpu.identifier.clone())
            .collect(),
    ));
    let participating = ParticipatingGpusOnThisHost::of_the_positions((0..cards.len()).collect());
    (
        BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
            cards,
            DiscreteGpusAfterTheOpening {
                opened: discrete_gpus,
                not_opened_for_not_taking_part: Vec::new(),
            },
            Vec::new(),
            participating,
            Vec::new(),
            memory_leaked.clone(),
            Box::new(DiscreteGpusPresentForTests(Arc::clone(&present))),
            no_opening_at_a_switch(),
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            pci_state_reader_under_a_root_not_present_for_tests(),
        ),
        present,
    )
}

/// 本机的卡挂载那一刻全开了时交的开卡那一步：切换点不到没开的卡，调到它就是用例错了。
fn no_opening_at_a_switch() -> OpeningOfAGpuAtASwitch<CardComputedOnTheCpuForTests> {
    Box::new(
        |discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
            panic!(
                "用例的卡挂载那一刻全开了，切换不该开 {}",
                discrete_gpu.pci_bus_address
            )
        },
    )
}

/// `count` 个单元，第 i 个是 `bytes_per_unit` 个字节的 i，期望值照 CPU 参照算对。
fn unit_bytes_of(count: usize, bytes_per_unit: usize) -> Vec<Vec<u8>> {
    (0..count)
        .map(|unit| vec![u8::try_from(unit % 251).expect("小于 256"); bytes_per_unit])
        .collect()
}

fn units_to_verify(unit_bytes: &[Vec<u8>]) -> Vec<UnitChecksumToVerify<'_, usize>> {
    unit_bytes
        .iter()
        .enumerate()
        .map(|(entry_key, bytes)| UnitChecksumToVerify {
            entry_key,
            unit_bytes: bytes,
            expected_crc32c: singlefs_core::checksum::crc32_castagnoli(bytes),
        })
        .collect()
}

fn cpu_reference_outcomes_of(
    units: &[UnitChecksumToVerify<'_, usize>],
) -> Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)> {
    units
        .iter()
        .map(|unit| {
            let outcome = cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);
            (outcome, outcome)
        })
        .collect()
}

/// 开卡时量代价交回的错（等超过上限）：两格用同一个。
fn failure_of_the_cost_measurement_in_the_test() -> GpuFailureCause {
    GpuFailureCause::WaitForTheGpuExceededTheLimit {
        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
    }
}

/// 本机两张独显：「0000:01:00.0」挂载那一刻参与、开着；「0000:41:00.0」没参与、没开。切换点它时开它一次的那一步交 `opening_at_the_switch`，
/// 开不成照 `reopenings` 重开；开卡那一步被交了哪几张卡（每开一次记一次）记在交回的那一份里。进程里泄漏的那一份是 `memory_leaked`。
fn background_gpu_with_one_card_not_opened_at_the_mount(
    opening_at_the_switch: impl FnMut(
            &DiscreteGpuOnThisHost,
            Option<GpuMemoryInBytes>,
            Duration,
            CostAtTheOpening,
        ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
        + Send
        + 'static,
    reopenings: ReopeningsOfAGpuNotOpened,
    memory_leaked: &GpuMemoryLeakedInThisProcess,
) -> (
    BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    Arc<Mutex<Vec<String>>>,
) {
    background_gpu_with_one_card_not_opened_at_the_mount_read_by(
        opening_at_the_switch,
        reopenings,
        memory_leaked,
        pci_state_reader_under_a_root_not_present_for_tests(),
    )
}

/// 同 [`background_gpu_with_one_card_not_opened_at_the_mount`]，判一张卡用不了时照 `pci_state_reader` 读 PCI 状态。
fn background_gpu_with_one_card_not_opened_at_the_mount_read_by(
    opening_at_the_switch: impl FnMut(
            &DiscreteGpuOnThisHost,
            Option<GpuMemoryInBytes>,
            Duration,
            CostAtTheOpening,
        ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
        + Send
        + 'static,
    reopenings: ReopeningsOfAGpuNotOpened,
    memory_leaked: &GpuMemoryLeakedInThisProcess,
    pci_state_reader: PciStateReaderOfTheGpus,
) -> (
    BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    Arc<Mutex<Vec<String>>>,
) {
    let opened_card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    let opened = opened_card.discrete_gpu.clone();
    let not_opened = DiscreteGpuOnThisHost {
        position_in_the_enumeration: 1,
        identifier: card_identifier("0000:41:00.0"),
        pci_bus_address: "0000:41:00.0".to_string(),
        name: "cpu card not opened at the mount".to_string(),
        total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
    };
    let (opening_noting_the_cards, cards_handed_to_the_opening) =
        opening_noting_the_cards_handed_to_it(opening_at_the_switch);
    let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
        vec![opened_card],
        DiscreteGpusAfterTheOpening {
            opened: vec![opened],
            not_opened_for_not_taking_part: vec![not_opened],
        },
        Vec::new(),
        ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
        Vec::new(),
        memory_leaked.clone(),
        Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
            card_identifier("0000:01:00.0"),
            card_identifier("0000:41:00.0"),
        ])))),
        opening_noting_the_cards,
        reopenings,
        pci_state_reader,
    );
    (background_gpu, cards_handed_to_the_opening)
}

/// 开卡那一步包一层：每开一次把交来的卡（总线地址）记进交回的那一份，再交 `opening_at_the_switch` 开。
fn opening_noting_the_cards_handed_to_it(
    mut opening_at_the_switch: impl FnMut(
            &DiscreteGpuOnThisHost,
            Option<GpuMemoryInBytes>,
            Duration,
            CostAtTheOpening,
        ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
        + Send
        + 'static,
) -> (
    OpeningOfAGpuAtASwitch<CardComputedOnTheCpuForTests>,
    Arc<Mutex<Vec<String>>>,
) {
    let cards_handed_to_the_opening = Arc::new(Mutex::new(Vec::new()));
    let cards_noted_by_the_opening = Arc::clone(&cards_handed_to_the_opening);
    let opening_noting_the_cards: OpeningOfAGpuAtASwitch<CardComputedOnTheCpuForTests> = Box::new(
        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
            cards_noted_by_the_opening
                .lock()
                .expect("用例没在别处 panic 过")
                .push(discrete_gpu.pci_bus_address.clone());
            opening_at_the_switch(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
        },
    );
    (opening_noting_the_cards, cards_handed_to_the_opening)
}

/// 开卡那一步：每一次都开成交来的那张卡（替身卡，照给它的显存上限）。
fn opening_that_always_opens(
    discrete_gpu: &DiscreteGpuOnThisHost,
    memory_limit: Option<GpuMemoryInBytes>,
    _wait_limit: Duration,
    _cost_at_the_opening: CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause> {
    Ok(CardComputedOnTheCpuForTests::of_the_address(
        &discrete_gpu.pci_bus_address,
        memory_limit.map(|memory_limit| memory_limit.0),
        0,
    ))
}

/// 用例里切换那一刻开不成的原因。
fn reason_the_card_does_not_open_at_the_switch() -> GpuFailureCause {
    GpuFailureCause::DeviceOrPipelineDidNotStart {
        reason: "替身：切换那一刻起不了设备".to_string(),
    }
}

/// 本机两张独显「0000:01:00.0」「0000:41:00.0」挂载那一刻都参与、都开着（按这个次序），切换再点到关掉的卡时开它一次的那一步交 `opening_at_the_switch`；
/// 交回外壳、开卡那一步被交了哪几张卡，与「0000:41:00.0」那张替身卡析构了几次。「0000:41:00.0」那张交给用例改（例：记上一次还在跑的提交）。
fn background_gpu_with_both_cards_opened_at_the_mount(
    opening_at_the_switch: impl FnMut(
            &DiscreteGpuOnThisHost,
            Option<GpuMemoryInBytes>,
            Duration,
            CostAtTheOpening,
        ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
        + Send
        + 'static,
    card_at_forty_one_as_the_test_wants: impl FnOnce(
        CardComputedOnTheCpuForTests,
    ) -> CardComputedOnTheCpuForTests,
    memory_leaked: &GpuMemoryLeakedInThisProcess,
) -> (
    BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    Arc<Mutex<Vec<String>>>,
    Arc<std::sync::atomic::AtomicU32>,
) {
    let card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    let card_at_forty_one = card_at_forty_one_as_the_test_wants(
        CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
    );
    let drops_of_the_card_at_forty_one = Arc::clone(&card_at_forty_one.drops_counted);
    let opened = vec![
        card_at_one.discrete_gpu.clone(),
        card_at_forty_one.discrete_gpu.clone(),
    ];
    let (opening_noting_the_cards, cards_handed_to_the_opening) =
        opening_noting_the_cards_handed_to_it(opening_at_the_switch);
    let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
        vec![card_at_one, card_at_forty_one],
        DiscreteGpusAfterTheOpening {
            opened,
            not_opened_for_not_taking_part: Vec::new(),
        },
        Vec::new(),
        ParticipatingGpusOnThisHost::of_the_positions(vec![0, 1]),
        Vec::new(),
        memory_leaked.clone(),
        Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
            card_identifier("0000:01:00.0"),
            card_identifier("0000:41:00.0"),
        ])))),
        opening_noting_the_cards,
        REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
        pci_state_reader_under_a_root_not_present_for_tests(),
    );
    (
        background_gpu,
        cards_handed_to_the_opening,
        drops_of_the_card_at_forty_one,
    )
}

/// 核心层照外壳答的卡清单校验点「0000:01:00.0」「0000:41:00.0」两张的切换（照这个次序）。
fn switch_naming_both_cards_admitted_against(
    background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
) -> AdmittedGpuSettings {
    admit_gpu_settings_against_the_inventory(
        &BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0"),
            ]),
            background_gpu_memory_limit: None,
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        },
        background_gpu,
    )
    .expect("卡清单列着没开的那张：切换点它照收")
}

/// 核心层照外壳答的卡清单校验只点 `pci_bus_address` 那一张的切换。
fn switch_naming_one_card_admitted_against(
    background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    pci_bus_address: &str,
) -> AdmittedGpuSettings {
    admit_gpu_settings_against_the_inventory(
        &BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![card_identifier(pci_bus_address)]),
            background_gpu_memory_limit: None,
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        },
        background_gpu,
    )
    .expect("卡清单列着本机两张独显：切换点其中一张照收")
}

/// 一批校验和两核各自的结局，按单元的次序。
type BothChecksumKernelOutcomesOfTheBatch =
    Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed>;

/// 在外壳上交一批校验和那一段（一条 1000 字节的单元），交回这一批的结局与这一段分到活的卡。
fn checksum_batch_and_the_devices_given_work(
    background_gpu: &mut BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
) -> (
    BothChecksumKernelOutcomesOfTheBatch,
    Vec<GpuDeviceIdentifier>,
) {
    let unit_bytes = unit_bytes_of(1, 1000);
    let units = units_to_verify(&unit_bytes);
    let outcome = background_gpu.both_kernel_outcomes_of_the_batch(&units);
    let devices_given_work = background_gpu
        .take_the_submissions_of_the_batch()
        .unit_checksum_verification
        .devices_given_work;
    (outcome, devices_given_work)
}

/// 此刻参与的卡，按参与的次序。
fn participating_devices_of(
    background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
) -> Vec<GpuDeviceIdentifier> {
    background_gpu
        .participating_gpus()
        .into_iter()
        .map(|participating| participating.identifier)
        .collect()
}

/// 本机三张独显：「0000:01:00.0」挂载那一刻开着（给它的显存上限 `memory_limit_of_the_card_at_one`；`left_out_at_the_mount` 时它因显存上限不参与、
/// 参与的卡一张都不剩），「0000:41:00.0」「0000:81:00.0」没开。切换点到没开的卡时开它那一步交 `opening_at_the_switch`，重开照用例的规矩
/// （间隔 0、至多重开 3 次）；开卡那一步被交了哪几张卡记在交回的那一份里。
fn background_gpu_with_the_card_at_one_opened_and_two_not_opened(
    opening_at_the_switch: impl FnMut(
            &DiscreteGpuOnThisHost,
            Option<GpuMemoryInBytes>,
            Duration,
            CostAtTheOpening,
        ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
        + Send
        + 'static,
    memory_limit_of_the_card_at_one: Option<u64>,
    left_out_at_the_mount: bool,
) -> (
    BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    Arc<Mutex<Vec<String>>>,
) {
    let card_at_one = CardComputedOnTheCpuForTests::of_the_address(
        "0000:01:00.0",
        memory_limit_of_the_card_at_one,
        0,
    );
    let opened = card_at_one.discrete_gpu.clone();
    let not_opened_card =
        |pci_bus_address: &str, position_in_the_enumeration: usize| DiscreteGpuOnThisHost {
            position_in_the_enumeration,
            identifier: card_identifier(pci_bus_address),
            pci_bus_address: pci_bus_address.to_string(),
            name: format!("cpu card at {pci_bus_address} not opened at the mount"),
            total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
        };
    let (participating, left_out) = if left_out_at_the_mount {
        (
            ParticipatingGpusOnThisHost::of_the_positions(Vec::new()),
            vec![GpuLeftOutUnderItsMemoryLimit {
                device: card_identifier("0000:01:00.0"),
                memory_limit: GpuMemoryInBytes(memory_limit_of_the_card_at_one.unwrap_or(0)),
                smallest_piece_bytes: bytes_of_the_smallest_piece_every_participating_gpu_holds(),
                bytes_already_held_on_the_device: GpuMemoryInBytes(0),
            }],
        )
    } else {
        (
            ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
            Vec::new(),
        )
    };
    let (opening_noting_the_cards, cards_handed_to_the_opening) =
        opening_noting_the_cards_handed_to_it(opening_at_the_switch);
    let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
        vec![card_at_one],
        DiscreteGpusAfterTheOpening {
            opened: vec![opened],
            not_opened_for_not_taking_part: vec![
                not_opened_card("0000:41:00.0", 1),
                not_opened_card("0000:81:00.0", 2),
            ],
        },
        Vec::new(),
        participating,
        left_out,
        GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
            card_identifier("0000:01:00.0"),
            card_identifier("0000:41:00.0"),
            card_identifier("0000:81:00.0"),
        ])))),
        opening_noting_the_cards,
        REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
        pci_state_reader_under_a_root_not_present_for_tests(),
    );
    (background_gpu, cards_handed_to_the_opening)
}

/// 核心层照外壳答的卡清单校验点 `addresses` 那几张（按这个次序）、显存上限 `memory_limits` 的切换。
fn switch_naming_the_cards_admitted_against(
    background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    addresses: &[&str],
    memory_limits: Option<Vec<u64>>,
) -> AdmittedGpuSettings {
    admit_gpu_settings_against_the_inventory(
        &BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(
                addresses
                    .iter()
                    .map(|address| card_identifier(address))
                    .collect(),
            ),
            background_gpu_memory_limit: memory_limits
                .map(|limits| limits.into_iter().map(GpuMemoryInBytes).collect()),
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        },
        background_gpu,
    )
    .expect("卡清单列着这几张：切换照收")
}

fn openings_made_so_far(cards_handed_to_the_opening: &Arc<Mutex<Vec<String>>>) -> usize {
    cards_handed_to_the_opening
        .lock()
        .expect("用例没在别处 panic 过")
        .len()
}

/// 两张卡上一组设置：点名 `named`（`None` 是不点名）、上限 `memory_limits`（按点名的次序或本机的次序），在外壳上照它从下一批起跑。
fn settings_admitted_on_the_cards(
    background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    named: Option<Vec<GpuDeviceIdentifier>>,
    memory_limits: Vec<u64>,
) -> AdmittedGpuSettings {
    admit_gpu_settings_against_the_inventory(
        &BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: named,
            background_gpu_memory_limit: Some(
                memory_limits.into_iter().map(GpuMemoryInBytes).collect(),
            ),
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        },
        background_gpu,
    )
    .expect("用例给的一组照两张替身卡校验都合法")
}

/// 建一段缓冲时等错误范围用的这张卡的账：等的上限 `wait_limit`，不读调用方在停的标记（卡收进来、核心层交来标记之前的样子）。
/// 绑定宽与一次派发的条目数取到不挡建缓冲的大小，显存不设上限：这几条用例只用它那一份等的期限。
fn budget_of_a_card_waiting_at_most(wait_limit: Duration) -> GpuBudgetUnderTheSettings<u32> {
    GpuBudgetUnderTheSettings::with_nothing_held(
        GpuMemoryInBytes(1 << 30),
        EntryCountOfAChunk(1 << 20),
        None,
        wait_limit,
    )
}
const OUTCOME_BYTES_OF_THE_CHUNK: u64 = 64;

/// 地址空间压住之后这一段自己的那份缓冲要的字节（`noop_chunk_buffer_with_contents_short` 取 128 MiB）：比压住之后留的那一截
/// [`ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE`] 大，noop 后端在主机内存里 `try_reserve_exact` 这么多就失败、wgpu 报显存不足；
/// 又在 `wgpu::Limits::default()` 的单个缓冲上限（256 MiB）之内，wgpu 不先报校验错。
const BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE: u64 = 128 << 20;
/// 压地址空间上限时比此刻的 VmSize 多留的那一截（取 32 MiB）：两份 64 字节的回读暂存、等错误范围、打印与 libtest 收尾用得下。
const ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE: u64 = 32 << 20;
/// 标记「这一次是地址空间压住的那个子进程」的环境变量：外面那一次起子进程时设上。
const VARIABLE_MARKING_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED: &str =
    "SINGLEFS_BACKGROUND_GPU_TEST_WITH_THE_ADDRESS_SPACE_CAPPED";
/// 地址空间压住的那个子进程判过之后打的那一行的开头：外面那一次认它，认不到判红。
const LINE_OF_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED: &str =
    "CHUNK_SHORT_OF_MEMORY_JUDGED_WITH_THE_ADDRESS_SPACE_CAPPED";

/// 压地址空间时设的上限：软上限取「想压到的字节数」与原来的硬上限里小的那个，硬上限照原样。软上限比硬上限大时 setrlimit 报 EINVAL，
/// 所以硬上限比想压到的还小时，软上限就压到硬上限。
fn address_space_limit_capped_at(
    wanted_soft_limit_in_bytes: u64,
    limit_before: &libc::rlimit,
) -> libc::rlimit {
    libc::rlimit {
        rlim_cur: wanted_soft_limit_in_bytes.min(limit_before.rlim_max),
        rlim_max: limit_before.rlim_max,
    }
}

/// 硬上限比想压到的小：软上限等于硬上限；硬上限是无限：软上限就是想压到的那个数。两种情形硬上限都照原样。
#[test]
fn the_capped_soft_limit_of_the_address_space_does_not_exceed_the_hard_limit() {
    let four_gibibytes: u64 = 4 << 30;
    let one_gibibyte: u64 = 1 << 30;
    let limit_with_a_small_hard_limit = libc::rlimit {
        rlim_cur: one_gibibyte,
        rlim_max: one_gibibyte,
    };
    let capped_under_the_small_hard_limit =
        address_space_limit_capped_at(four_gibibytes, &limit_with_a_small_hard_limit);
    assert_eq!(
        capped_under_the_small_hard_limit.rlim_cur, one_gibibyte,
        "硬上限 1 GiB 比想压到的 4 GiB 小，软上限压到硬上限"
    );
    assert_eq!(capped_under_the_small_hard_limit.rlim_max, one_gibibyte);
    let limit_without_a_hard_limit = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let capped_under_no_hard_limit =
        address_space_limit_capped_at(four_gibibytes, &limit_without_a_hard_limit);
    assert_eq!(
        capped_under_no_hard_limit.rlim_cur, four_gibibytes,
        "硬上限无限，软上限就是想压到的那个数"
    );
    assert_eq!(capped_under_no_hard_limit.rlim_max, libc::RLIM_INFINITY);
}

/// 本进程的地址空间上限（`RLIMIT_AS`）压到「此刻的 VmSize 加 `room_beyond_the_present_size` 字节」跑 `run`，跑完放回原来的上限，交回 `run` 的结局。
/// 压的是整个进程：只在地址空间压住的那个子进程里调（那个进程只跑一条用例），不在跑别的用例的进程里调。
///
/// # Panics
/// 读不到 `/proc/self/status` 的 VmSize 一行，或 `getrlimit` / `setrlimit` 报错：Linux 上这几样总在。
#[allow(
    unsafe_code,
    reason = "std 没有 getrlimit/setrlimit，只能调 libc；只在用例自己起的子进程里压本进程的地址空间"
)]
fn with_the_address_space_capped_beyond_the_present_size<Outcome>(
    room_beyond_the_present_size: u64,
    run: impl FnOnce() -> Outcome,
) -> Outcome {
    let status_of_this_process =
        std::fs::read_to_string("/proc/self/status").expect("Linux 上读得到 /proc/self/status");
    let present_size_in_kibibytes: u64 = status_of_this_process
        .lines()
        .find_map(|line| line.strip_prefix("VmSize:"))
        .and_then(|rest_of_the_line| rest_of_the_line.split_whitespace().next())
        .and_then(|kibibytes| kibibytes.parse().ok())
        .expect("/proc/self/status 有 VmSize 一行，单位 kB");
    let mut limit_before = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: `limit_before` 是这个栈帧里初始化过的 `rlimit`，getrlimit 只往它里面写，调用期间没有别的引用。
    let read_status = unsafe { libc::getrlimit(libc::RLIMIT_AS, &raw mut limit_before) };
    assert_eq!(read_status, 0, "getrlimit(RLIMIT_AS) 在 Linux 上总读得到");
    let capped_limit = address_space_limit_capped_at(
        present_size_in_kibibytes * 1024 + room_beyond_the_present_size,
        &limit_before,
    );
    // SAFETY: `capped_limit` 是这个栈帧里初始化过的 `rlimit`，setrlimit 只读它；软上限不超过原来的硬上限（只压软的那一个）。
    let cap_status = unsafe { libc::setrlimit(libc::RLIMIT_AS, &raw const capped_limit) };
    assert_eq!(cap_status, 0, "setrlimit(RLIMIT_AS) 压软上限");
    let outcome = run();
    // SAFETY: `limit_before` 是上面 getrlimit 读回的那一份，setrlimit 只读它。
    let restore_status = unsafe { libc::setrlimit(libc::RLIMIT_AS, &raw const limit_before) };
    assert_eq!(restore_status, 0, "setrlimit(RLIMIT_AS) 放回原来的上限");
    outcome
}

/// 地址空间压住的那个子进程里判的那一件：在 wgpu 30.0.1 的 noop 后端上走产品那一份 `impl DeviceCreatingTheBuffersOfAChunk for wgpu::Device`，
/// 地址空间压到此刻多 [`ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE`]，这一段自己的缓冲照产品的建法（建的时候映射着、映射得上才写）要
/// [`BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE`] 字节——建不成，交回 `OutOfMemoryCreatingTheBuffers`（带这一段的账）、不 panic，
/// 没接住的错一条都没有、写内容的那一步没走到。
fn the_buffer_of_the_chunk_beyond_the_capped_address_space_is_reported_as_out_of_memory() {
    let (device, _queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor {
        label: Some("singlefs-background-gpu-noop-test"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    });
    let uncaptured_errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let uncaptured_errors_of_the_device = Arc::clone(&uncaptured_errors);
    device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
        uncaptured_errors_of_the_device
            .lock()
            .expect("用例没在别处 panic 过")
            .push(format!("{error}"));
    }));
    let contents_written = Cell::new(false);
    let bytes_held_by_the_chunk_on_the_account = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
        .bytes_of_the_copies(GpuMemoryInBytes(
            BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE,
        ))
        .plus(GpuMemoryInBytes(2 * OUTCOME_BYTES_OF_THE_CHUNK));
    let created = with_the_address_space_capped_beyond_the_present_size(
        ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE,
        || {
            buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
                &device,
                &budget_of_a_card_waiting_at_most(Duration::from_secs(10)),
                bytes_held_by_the_chunk_on_the_account,
                GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
                || {
                    [buffer_mapped_at_creation_within_the_scope_of_the_caller(
                        &device,
                        GpuMemoryInBytes(BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE),
                        wgpu::BufferUsages::STORAGE,
                        |_mapped| contents_written.set(true),
                    )]
                },
            )
        },
    );
    assert!(
        matches!(
            created,
            Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                bytes_held_by_the_chunk,
                ..
            }) if bytes_held_by_the_chunk == bytes_held_by_the_chunk_on_the_account
        ),
        "建不成的那一份被这一段的范围接住、报成建缓冲显存不足：{:?}",
        created.as_ref().map(|_buffers| "建成了")
    );
    assert!(!contents_written.get(), "映射不上：不写内容");
    assert_eq!(
        *uncaptured_errors.lock().expect("用例没在别处 panic 过"),
        Vec::<String>::new(),
        "没有一条错落到范围之外（不解开没建成的缓冲的映射）"
    );
}

/// 用 `noop_chunk_buffer_with_contents_short` 的造法：noop 后端在主机内存里建缓冲，要真建不成就得压
/// 进程的地址空间上限；压的是整个进程，所以本进程照原样再起一次自己、只跑这一条，那一次里压上限再建
/// （[`the_buffer_of_the_chunk_beyond_the_capped_address_space_is_reported_as_out_of_memory`]），跑别的用例的这个进程不压。
/// 外面那一次认那一次退 0、打了判过的那一行。起的那一次放进它自己的进程组，它跑完才交回。
/// 改回 `create_buffer_init` 那种建完就对映射区 `expect` 的写法，那一次里 panic、退非 0，这里红。
#[test]
fn the_buffer_of_the_chunk_short_of_memory_on_a_noop_device_is_reported_as_out_of_memory_without_panicking(
) {
    use std::os::unix::process::CommandExt;
    let test_name = "background_gpu::tests::the_buffer_of_the_chunk_short_of_memory_on_a_noop_device_is_reported_as_out_of_memory_without_panicking";
    if std::env::var_os(VARIABLE_MARKING_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED).is_some() {
        the_buffer_of_the_chunk_beyond_the_capped_address_space_is_reported_as_out_of_memory();
        println!("{LINE_OF_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED} test={test_name}");
        return;
    }
    let run_with_the_address_space_capped =
        std::process::Command::new(std::env::current_exe().expect("这个测试程序自己的路径"))
            .args([test_name, "--exact", "--nocapture", "--test-threads", "1"])
            .env(VARIABLE_MARKING_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED, "1")
            .process_group(0)
            .output()
            .expect("起得来地址空间压住的那一次");
    let standard_output = String::from_utf8_lossy(&run_with_the_address_space_capped.stdout);
    assert!(
        run_with_the_address_space_capped.status.success()
            && standard_output.contains(&format!(
                "{LINE_OF_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED} test={test_name}"
            )),
        "地址空间压住的那一次退 0、判过：{:?}\n{standard_output}\n{}",
        run_with_the_address_space_capped.status,
        String::from_utf8_lossy(&run_with_the_address_space_capped.stderr)
    );
}

// ── 开不成之后生效的照旧是切换之前那一组、挂载那一刻开卡 panic 照重开、一张卡一次挂载里连着试满、还没回来的开卡管到整个进程、
// 读得到切换关卡的泄漏、被盖掉的切换（D26（后台整理与放置回收） 已定项 13 第 2、4 条）──

/// 切换开卡那一步：「0000:01:00.0」等用例放开（至多 20 秒）再开成，别的卡每一次都开不成。
fn opening_the_card_at_one_once_released_and_no_other(
    opening_released: std::sync::mpsc::Receiver<()>,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
       + Send
       + 'static {
    move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
        if discrete_gpu.pci_bus_address != "0000:01:00.0" {
            return Err(reason_the_card_does_not_open_at_the_switch());
        }
        opening_released
            .recv_timeout(Duration::from_secs(20))
            .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
        opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
    }
}

fn not_opened_with_the_openings_of_the_card_at_forty_one_used_up() -> LatestSettingsHandedIn {
    LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
        device: card_identifier("0000:41:00.0"),
        pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
            "0000:41:00.0",
        ),
        causes_of_each_opening: vec![GpuFailureCause::OpeningsInThisMountUsedUp {
            failed_openings: 4,
        }],
    })
}

/// 一张替身独显（没开）。
fn discrete_gpu_not_opened_at(pci_bus_address: &str) -> DiscreteGpuOnThisHost {
    DiscreteGpuOnThisHost {
        position_in_the_enumeration: 0,
        identifier: card_identifier(pci_bus_address),
        pci_bus_address: pci_bus_address.to_string(),
        name: format!("cpu card at {pci_bus_address} not opened yet"),
        total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
    }
}

/// 开卡那一步对「0000:41:00.0」被调了几次。
fn openings_of_the_card_at_forty_one(
    cards_handed_to_the_opening: &Arc<Mutex<Vec<String>>>,
) -> usize {
    cards_handed_to_the_opening
        .lock()
        .expect("用例没在别处 panic 过")
        .iter()
        .filter(|address| address.as_str() == "0000:41:00.0")
        .count()
}

/// 「0000:41:00.0」开不成、每一次的原因是 `causes_of_each_opening`。
fn card_at_forty_one_not_opened_with(
    causes_of_each_opening: Vec<GpuFailureCause>,
) -> LatestSettingsHandedIn {
    LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
        device: card_identifier("0000:41:00.0"),
        pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
            "0000:41:00.0",
        ),
        causes_of_each_opening,
    })
}

/// 换上了，这组设置下一批分得到卡上。
fn put_in_effect_on_every_card() -> LatestSettingsHandedIn {
    LatestSettingsHandedIn::PutInEffect {
        not_split_over_the_gpus: None,
    }
}

/// 替身卡等提交时 panic 带的原话（只供测试的钩子 `submission_waits_panicking`）。
const PANIC_SAID_BY_THE_WAIT_OF_THE_TEST_CARD: &str = "替身：等提交时设备丢了";
