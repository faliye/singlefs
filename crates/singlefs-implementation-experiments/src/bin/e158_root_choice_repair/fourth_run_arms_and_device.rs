//! 第 4 次跑的臂、装置自己的块设备与被测挂载。

use super::*;

// ============================================================================
// 十二、第 4 次跑（`research/prompts/e158-r4-prereg.md`）：C554 怎么修（岔路单 `research/prompts/c554-fix-forks.md` #1「挂载时有一条根（或它引用的账）暂时读不出」）。
//      装置按登记 5.7 的五段写，模式 `r4-seg1` … `r4-seg5`，跨臂的量在 `r4-compare`。第 2、3 次跑的函数只调用、不改语义；
//      要改的一律复制一份给 `r4-*` 用（登记文件头「装置写在哪」）。
// ============================================================================

/// 候选（岔路单第 1 行：甲 拒可写、乙 重读后再判、丙 从记录重建、对照 今天；乙-窄读 是主 agent 认定第 1 项另立的那一条乙）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FourthRunCandidate {
    Today,
    Jia,
    Yi,
    YiNarrow,
    Bing,
}

/// 臂的判据 N 取哪一样（登记 5.1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FourthRunWitness {
    NotJudged,
    SystemConfiguration,
    SystemConfigurationCarried,
    UnreadableRootRingSlot,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FourthRunArm {
    pub(super) name: &'static str,
    pub(super) candidate: FourthRunCandidate,
    pub(super) witness: FourthRunWitness,
    /// 乙、乙-窄读 的重读轮数 R（别的臂 0）。
    pub(super) reread_rounds: u64,
}

/// 第 4 次跑的臂（文件名拼写 = 臂表 r4 行里的名字）：前十二条是 G0 主族的臂；后六条是第五段敏感性点 R = 3 的 乙 与 乙-窄读。
pub(super) const FOURTH_RUN_ARMS: [FourthRunArm; 18] = [
    FourthRunArm {
        name: "today",
        candidate: FourthRunCandidate::Today,
        witness: FourthRunWitness::NotJudged,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "jia-cfg",
        candidate: FourthRunCandidate::Jia,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "jia-cfg-carry",
        candidate: FourthRunCandidate::Jia,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "jia-slot",
        candidate: FourthRunCandidate::Jia,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "yi-cfg",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-cfg-carry",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-slot",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-narrow-cfg",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-narrow-cfg-carry",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "yi-narrow-slot",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 1,
    },
    FourthRunArm {
        name: "bing-cfg",
        candidate: FourthRunCandidate::Bing,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "bing-cfg-carry",
        candidate: FourthRunCandidate::Bing,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 0,
    },
    FourthRunArm {
        name: "yi-cfg-three-rounds",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-cfg-carry-three-rounds",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-slot-three-rounds",
        candidate: FourthRunCandidate::Yi,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-narrow-cfg-three-rounds",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::SystemConfiguration,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-narrow-cfg-carry-three-rounds",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::SystemConfigurationCarried,
        reread_rounds: 3,
    },
    FourthRunArm {
        name: "yi-narrow-slot-three-rounds",
        candidate: FourthRunCandidate::YiNarrow,
        witness: FourthRunWitness::UnreadableRootRingSlot,
        reread_rounds: 3,
    },
];

pub(super) fn fourth_run_arm_named(name: &str) -> Option<FourthRunArm> {
    FOURTH_RUN_ARMS.iter().copied().find(|arm| arm.name == name)
}

impl FourthRunArm {
    pub(super) fn is_today(self) -> bool {
        self.candidate == FourthRunCandidate::Today
    }

    pub(super) fn rereads(self) -> bool {
        match self.candidate {
            FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow => true,
            FourthRunCandidate::Today | FourthRunCandidate::Jia | FourthRunCandidate::Bing => false,
        }
    }

    /// N 判真之后这一臂交回的拒绝成员名（臂表 r4 行加进 `MountError` 的成员）；今天那一臂没有。
    pub(super) fn refusal_member(self) -> &'static str {
        match (self.candidate, self.witness) {
            (FourthRunCandidate::Today, _) | (_, FourthRunWitness::NotJudged) => "none",
            (FourthRunCandidate::Bing, _) => "R3RebuildFromRecordsStoppedBelowTheWitness",
            (
                FourthRunCandidate::Jia | FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow,
                FourthRunWitness::UnreadableRootRingSlot,
            ) => "R3UnreadableRootRingSlot",
            (
                FourthRunCandidate::Jia | FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow,
                FourthRunWitness::SystemConfiguration
                | FourthRunWitness::SystemConfigurationCarried,
            ) => "R3NewerRootWitnessedByTheSystemConfiguration",
        }
    }

    /// 同一种 N 的甲臂（PC-多读 (iii) 与 Q5-拒 的参照）。
    pub(super) fn jia_arm_with_the_same_witness(self) -> Option<&'static str> {
        match self.witness {
            FourthRunWitness::NotJudged => None,
            FourthRunWitness::SystemConfiguration => Some("jia-cfg"),
            FourthRunWitness::SystemConfigurationCarried => Some("jia-cfg-carry"),
            FourthRunWitness::UnreadableRootRingSlot => Some("jia-slot"),
        }
    }

    pub(super) fn witness_name(self) -> &'static str {
        match self.witness {
            FourthRunWitness::NotJudged => "none",
            FourthRunWitness::SystemConfiguration => "cfg",
            FourthRunWitness::SystemConfigurationCarried => "cfg-carry",
            FourthRunWitness::UnreadableRootRingSlot => "slot",
        }
    }

    pub(super) fn candidate_name(self) -> &'static str {
        match self.candidate {
            FourthRunCandidate::Today => "today",
            FourthRunCandidate::Jia => "jia",
            FourthRunCandidate::Yi => "yi",
            FourthRunCandidate::YiNarrow => "yi-narrow",
            FourthRunCandidate::Bing => "bing",
        }
    }
}

/// H1d 与各族 A 的时长（登记 5.3：各臂 W 与 O(1 只)；乙、乙-窄读 另加 T(δ)、T(2δ)，R = 3 的另加到 T(4δ)（8.2）；
/// 今天、甲、丙 另跑 T(δ) 只跑不断那一格，给 7.3 第三行 V5 对拍）。交回 (时长, 只跑不断那一格)。
pub(super) fn fourth_run_held_once_durations(arm: FourthRunArm) -> Vec<(FaultDuration, bool)> {
    let mut durations = vec![
        (FaultDuration::WholeCall, false),
        (FaultDuration::OnlyTheNthRead(1), false),
    ];
    if arm.rereads() {
        for intervals in 1..=(arm.reread_rounds + 1) {
            durations.push((
                FaultDuration::UntilVirtualTime(intervals * THIRD_RUN_VIRTUAL_INTERVAL),
                false,
            ));
        }
    } else {
        durations.push((
            FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
            true,
        ));
    }
    durations
}

/// H1e 与接 H1e 尾巴的族的时长（登记 5.3：W；乙、乙-窄读 另加 T(δ)，R = 3 的另加到 T(4δ)（8.2））。
pub(super) fn fourth_run_tail_durations(arm: FourthRunArm) -> Vec<FaultDuration> {
    let mut durations = vec![FaultDuration::WholeCall];
    if arm.rereads() {
        let longest = if arm.reread_rounds > 1 {
            arm.reread_rounds + 1
        } else {
            1
        };
        for intervals in 1..=longest {
            durations.push(FaultDuration::UntilVirtualTime(
                intervals * THIRD_RUN_VIRTUAL_INTERVAL,
            ));
        }
    }
    durations
}

// ---------------------------------------------------------------------------
// 12.1 受观测的入口调用（第 4 次跑）：第 3 次跑的故障设备外面再包一层，数「第一个等待标记之前 / 之后」打到盘上的读、
//      标记之后读回全零的读（登记 5.0 第三条、8.1 末），与第一个标记那一刻已被拦下过的落点个数 f（5.4 PC-多读 (iii)）。
// ---------------------------------------------------------------------------

#[derive(Default)]
pub(super) struct FourthRunReadMarker {
    pub(super) read_calls: u64,
    pub(super) reads_before_the_first_wait: Option<u64>,
    faults_intercepted_before_the_first_wait: Option<u64>,
    all_zero_reads_after_the_first_wait: u64,
}

pub(super) struct FourthRunDevice {
    pub(super) device: ThirdRunDevice,
    marker: Rc<RefCell<FourthRunReadMarker>>,
}

impl BlockDevice for FourthRunDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let result = self.device.read_at(offset, buffer);
        let mut marker = self.marker.borrow_mut();
        marker.read_calls += 1;
        if marker.reads_before_the_first_wait.is_some()
            && result.is_ok()
            && buffer.iter().all(|byte| *byte == 0)
        {
            marker.all_zero_reads_after_the_first_wait += 1;
        }
        result
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.device.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.device.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.device.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.device.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.device.size_in_bytes()
    }
}

/// 臂的观测里第 4 次跑新加的两样（臂表 r4 行的观测字段）：重读轮数上限 R、读缓存收不收 journal 空槽（乙-窄读 收）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct FourthRunArmExtras {
    pub(super) reread_rounds_limit: u32,
    pub(super) zeroed_journal_ring_slots_are_cached: bool,
}

/// 一次受观测的入口调用（第 4 次跑）：第 3 次跑那一份交回的东西，加标记前后的读。
pub(super) struct FourthRunCall<Outcome> {
    pub(super) call: ThirdRunCall<Outcome>,
    pub(super) extras: Option<FourthRunArmExtras>,
    pub(super) reads_before_the_first_wait: Option<u64>,
    faults_intercepted_before_the_first_wait: Option<u64>,
    all_zero_reads_after_the_first_wait: u64,
    /// 每个点了名的落点在这次调用里被读了几次（坏没坏都数；L5 的 M 取今天那一臂孪生镜像上这个数）。
    pub(super) reads_of_the_targets: Vec<u64>,
}

impl<Outcome> FourthRunCall<Outcome> {
    /// 第一个等待标记之后打到盘上的读调用数（没有标记 = 0）。
    fn reads_after_the_first_wait(&self) -> u64 {
        self.reads_before_the_first_wait
            .map_or(0, |before| self.call.read_calls - before)
    }

    pub(super) fn render_marker(&self, prefix: &str) -> String {
        format!(
            "{prefix}_reads_before_first_wait={} {prefix}_reads_after_first_wait={} {prefix}_faults_intercepted_before_first_wait={} {prefix}_zero_reads_after_first_wait={} {prefix}_arm_rounds_limit={} {prefix}_arm_caches_zeroed_journal_slots={}",
            self.reads_before_the_first_wait
                .map_or("none".to_string(), |reads| reads.to_string()),
            self.reads_after_the_first_wait(),
            self.faults_intercepted_before_the_first_wait
                .map_or("none".to_string(), |count| count.to_string()),
            self.all_zero_reads_after_the_first_wait,
            self.extras
                .map_or("none".to_string(), |extras| extras.reread_rounds_limit.to_string()),
            self.extras.map_or("none".to_string(), |extras| {
                extras.zeroed_journal_ring_slots_are_cached.to_string()
            })
        )
    }
}

/// 等待钩子被调时记第一个等待标记：那一刻打到盘上的读调用数，与那一刻已被拦下过的落点个数 f（之后的标记不改它们）。
fn mark_the_first_wait(
    marker: &RefCell<FourthRunReadMarker>,
    state: &RefCell<ThirdRunDeviceState>,
) {
    let mut marker = marker.borrow_mut();
    if marker.reads_before_the_first_wait.is_none() {
        marker.reads_before_the_first_wait = Some(marker.read_calls);
        marker.faults_intercepted_before_the_first_wait = Some(
            state
                .borrow()
                .intercepted_reads
                .iter()
                .filter(|intercepted| **intercepted > 0)
                .count() as u64,
        );
    }
}

pub(super) fn fourth_run_call<Outcome>(
    base: &MemoryPool,
    faults: &[ThirdRunFault],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
    call: impl FnOnce(&mut Vec<(DeviceIdentity, FourthRunDevice)>) -> Outcome,
) -> FourthRunCall<Outcome> {
    let state = Rc::new(RefCell::new(ThirdRunDeviceState::new(
        faults.to_vec(),
        failing_write_call,
        failing_barrier_call,
    )));
    let marker = Rc::new(RefCell::new(FourthRunReadMarker::default()));
    let mut devices: Vec<(DeviceIdentity, FourthRunDevice)> = devices_from_pool(base, IMAGE_BYTES)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                FourthRunDevice {
                    device: ThirdRunDevice {
                        identity,
                        inner,
                        state: Rc::clone(&state),
                    },
                    marker: Rc::clone(&marker),
                },
            )
        })
        .collect();
    third_run_arm_hooks::reset_the_virtual_clock();
    let marker_for_the_hook = Rc::clone(&marker);
    let state_for_the_hook = Rc::clone(&state);
    fourth_run_arm_hooks::install_the_marking_wait_hook(
        THIRD_RUN_VIRTUAL_INTERVAL,
        Box::new(move || mark_the_first_wait(&marker_for_the_hook, &state_for_the_hook)),
    );
    let outcome = call(&mut devices);
    let taken = fourth_run_arm_hooks::take_the_arm_observation_and_its_extras();
    let waits = third_run_arm_hooks::waits();
    let plain: Vec<(DeviceIdentity, SparseBlockDevice)> = devices
        .into_iter()
        .map(|(identity, device)| (identity, device.device.inner))
        .collect();
    let pool_after = memory_pool_of(&plain, IMAGE_BYTES);
    let state = state.borrow();
    let marker = marker.borrow();
    FourthRunCall {
        call: ThirdRunCall {
            outcome,
            write_calls: state.write_calls,
            read_calls: state.read_calls,
            read_bytes: state.read_bytes,
            applied: state.applied.clone(),
            intercepted_reads: state.intercepted_reads.clone(),
            injected_failure_fired: state.injected_failure_fired,
            pool_after,
            waits,
            observation: taken.as_ref().map(|(observation, _)| observation.clone()),
        },
        extras: taken.map(|(_, extras)| extras),
        reads_before_the_first_wait: marker.reads_before_the_first_wait,
        faults_intercepted_before_the_first_wait: marker.faults_intercepted_before_the_first_wait,
        all_zero_reads_after_the_first_wait: marker.all_zero_reads_after_the_first_wait,
        reads_of_the_targets: state.reads_of_the_target.clone(),
    }
}

pub(super) fn run_fourth_run_mount_writable(
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    failing_write_call: Option<u64>,
    failing_barrier_call: Option<u64>,
) -> FourthRunCall<Result<Mounted, singlefs_core::mount::MountError>> {
    fourth_run_call(
        base,
        faults,
        failing_write_call,
        failing_barrier_call,
        |devices| mount_writable(parameters, devices),
    )
}

/// 一次被测挂载（第 4 次跑）：同第 3 次跑 `run_tested_mount`，调用走 `fourth_run_call`；今天那一臂照做 7.3 第一行与 F13。
pub(super) struct FourthRunTestedMount {
    pub(super) call: FourthRunCall<Result<Mounted, singlefs_core::mount::MountError>>,
    pub(super) summary: TestedMountSummary,
    pub(super) judgement: DeviceJudgement,
    pub(super) unintercepted: Vec<String>,
}

pub(super) fn run_fourth_run_tested_mount(
    arm: FourthRunArm,
    base: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    label: &str,
    family_summary: &mut ThirdRunFamilySummary,
) -> Result<FourthRunTestedMount, String> {
    let geometry = independent_geometry(base)?;
    let call = run_fourth_run_mount_writable(base, parameters, faults, None, None);
    let judgement = device_judgement(base, &geometry, faults);
    let summary = TestedMountSummary::of(&call.call);
    let unintercepted = call.call.unintercepted(faults);
    if arm.is_today() {
        today_cross_checks(base, &judgement, &call.call, label, family_summary);
        if let Ok(mounted) = &call.call.outcome {
            family_summary.failure_clause_thirteen_checks += 1;
            let expected = first_txg_expected_by_the_rule_of_today(base, &geometry, faults);
            let got = mounted.output.row_publish.root().checkpoint_txg.0;
            if got != expected {
                family_summary
                    .failure_clause_thirteen_findings
                    .push(format!(
                        "{label}: 写行那次发布 txg {got}，按今天的规则 {expected}"
                    ));
            }
        }
    }
    Ok(FourthRunTestedMount {
        call,
        summary,
        judgement,
        unintercepted,
    })
}

pub(super) fn render_fourth_run_tested_mount(tested: &FourthRunTestedMount) -> String {
    format!(
        "{} {} {}",
        tested.summary.render("a"),
        tested.call.render_marker("a"),
        render_device_judgement(&tested.judgement)
    )
}

#[cfg(test)]
mod fourth_run_tests;
