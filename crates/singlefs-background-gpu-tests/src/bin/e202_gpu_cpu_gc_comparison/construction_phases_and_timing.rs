//! 造池分阶段计时与进度行：阶段、常驻内存、聚簇段、造池钟与造池里的发布。

use super::*;

// ---- 造池分阶段计时与进度行 ----

/// 造池的进度间隔（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」一节：进度间隔 5 分钟）。
pub(super) const CONSTRUCTION_PROGRESS_INTERVAL: Duration = Duration::from_secs(300);

/// 造池的阶段，按造的先后（分阶段计时与进度行按这一份报）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ConstructionPhase {
    MakeFilesystemAndTheFirstFile,
    DataMount,
    CreatingTheInodes,
    SequentialHalf,
    AlignmentSpacer,
    InterleavedHalf,
    FragmentMount,
    UnlinkingTheSpacerFiles,
    ReleasingTheSpacerSlots,
    ReadingTheGeometry,
}

impl ConstructionPhase {
    #[cfg(test)]
    const EVERY_PHASE: [ConstructionPhase; 10] = [
        ConstructionPhase::MakeFilesystemAndTheFirstFile,
        ConstructionPhase::DataMount,
        ConstructionPhase::CreatingTheInodes,
        ConstructionPhase::SequentialHalf,
        ConstructionPhase::AlignmentSpacer,
        ConstructionPhase::InterleavedHalf,
        ConstructionPhase::FragmentMount,
        ConstructionPhase::UnlinkingTheSpacerFiles,
        ConstructionPhase::ReleasingTheSpacerSlots,
        ConstructionPhase::ReadingTheGeometry,
    ];

    pub(super) const fn name(self) -> &'static str {
        match self {
            ConstructionPhase::MakeFilesystemAndTheFirstFile => {
                "make_filesystem_and_the_first_file"
            }
            ConstructionPhase::DataMount => "data_mount",
            ConstructionPhase::CreatingTheInodes => "creating_the_inodes",
            ConstructionPhase::SequentialHalf => "sequential_half",
            ConstructionPhase::AlignmentSpacer => "alignment_spacer",
            ConstructionPhase::InterleavedHalf => "interleaved_half",
            ConstructionPhase::FragmentMount => "fragment_mount",
            ConstructionPhase::UnlinkingTheSpacerFiles => "unlinking_the_spacer_files",
            ConstructionPhase::ReleasingTheSpacerSlots => "releasing_the_spacer_slots",
            ConstructionPhase::ReadingTheGeometry => "reading_the_geometry",
        }
    }
}

/// 本进程此刻的常驻内存（`/proc/self/status` 的 `VmRSS`）；读不到交 `None`。
pub(super) fn resident_bytes_of_this_process() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    resident_bytes_of_the_status(&status)
}

/// `/proc/<pid>/status` 正文里 `VmRSS:   <千字节> kB` 那一行换成字节。
fn resident_bytes_of_the_status(status: &str) -> Option<u64> {
    bytes_of_the_kibibyte_line_of_the_status(status, "VmRSS:")
}

/// 本进程到此刻为止常驻内存的峰（`/proc/self/status` 的 `VmHWM`）；读不到交 `None`。
pub(super) fn peak_resident_bytes_of_this_process() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    peak_resident_bytes_of_the_status(&status)
}

/// `/proc/<pid>/status` 正文里 `VmHWM:   <千字节> kB` 那一行换成字节。
fn peak_resident_bytes_of_the_status(status: &str) -> Option<u64> {
    bytes_of_the_kibibyte_line_of_the_status(status, "VmHWM:")
}

/// `/proc/<pid>/status` 正文里以 `field` 起头、以 `kB` 收尾的那一行换成字节；没有这一行或读不成数交 `None`。
fn bytes_of_the_kibibyte_line_of_the_status(status: &str, field: &str) -> Option<u64> {
    let line = status.lines().find(|line| line.starts_with(field))?;
    let kibibytes: u64 = line
        .strip_prefix(field)?
        .trim()
        .strip_suffix("kB")?
        .trim()
        .parse()
        .ok()?;
    kibibytes.checked_mul(1024)
}

/// 正在造的那一阶段：起点、做完的步数与它至多几步、每一步的挂钟（秒）、开始时已发成的版数。
struct PhaseInProgress {
    pub(super) phase: ConstructionPhase,
    pub(super) started: Instant,
    last_step_ended: Instant,
    planned_steps_at_most: u64,
    step_seconds: Vec<f64>,
    publishes_before_the_phase: u64,
}

/// 一个阶段量到的：挂钟、步数（一步是一次挂载、一次建 inode、写满一个文件的一次发布、一个删掉的文件、一轮抬 F，见 [`ConstructionPhase`]）、
/// 这一阶段发成的版数、每一步的挂钟（头八分之一、末八分之一的平均与最长那一步：看一步的用时随池里的东西多起来涨不涨），阶段末了现行那一版的大小、
/// 本进程常驻内存、内存盘的块字节，与阶段末了这次挂载开过的聚簇段数（没有挂载的那一阶段是 `None`）。
#[derive(Clone, Debug)]
pub(super) struct TimingOfAConstructionPhase {
    pub(super) phase: ConstructionPhase,
    seconds: f64,
    pub(super) steps: u64,
    planned_steps_at_most: u64,
    pub(super) publishes: u64,
    step_milliseconds_mean_of_the_earliest_eighth: f64,
    step_milliseconds_mean_of_the_latest_eighth: f64,
    step_milliseconds_maximum: f64,
    units_in_the_version: Option<u64>,
    allocation_records_in_the_version: Option<u64>,
    resident_bytes: Option<u64>,
    chunk_bytes: ChunkBytesOfThePool,
    cluster_segments: Option<ClusterSegmentsOfTheMount>,
}

/// 这次挂载开过的聚簇段数（核心层分配器 `cluster_segments` 那一份，整次挂载只进不出，D3（空间分配） 已定项 8 第 2 条）与盘 0 单元区的整段数
/// （登记第十二节修订第 14 条：每个 `name=construction_phase` 行带这两个数）。两块盘同槽，聚簇段的起点是两块盘共用的一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ClusterSegmentsOfTheMount {
    pub(super) opened_by_this_mount: u64,
    pub(super) unit_area_segments: u64,
}

pub(super) fn cluster_segments_of_the_mount(session: &MountedSession) -> ClusterSegmentsOfTheMount {
    ClusterSegmentsOfTheMount {
        opened_by_this_mount: u64::try_from(session.allocator.cluster_segments().len())
            .expect("段数装得进 u64"),
        unit_area_segments: session.allocator.devices[0].unit_area_slots()
            / LOCAL_CLUSTER_SEGMENT_SLOTS,
    }
}

/// 几步的平均毫秒；一步都没有交 0。
fn mean_milliseconds_of(seconds: &[f64]) -> f64 {
    if seconds.is_empty() {
        return 0.0;
    }
    seconds.iter().sum::<f64>() * 1000.0
        / count_as_float(u64::try_from(seconds.len()).expect("步数装得进 u64"))
}

impl TimingOfAConstructionPhase {
    fn of_the_phase(
        phase: PhaseInProgress,
        devices: &PoolDevices,
        version: Option<&TransactionOutput>,
        publishes_after_the_phase: u64,
        cluster_segments: Option<ClusterSegmentsOfTheMount>,
    ) -> TimingOfAConstructionPhase {
        let steps = phase.step_seconds.len();
        // 头尾各取八分之一，至少一步；一步都没有就是空的。
        let eighth = if steps == 0 { 0 } else { (steps / 8).max(1) };
        TimingOfAConstructionPhase {
            phase: phase.phase,
            seconds: phase.started.elapsed().as_secs_f64(),
            steps: u64::try_from(steps).expect("步数装得进 u64"),
            planned_steps_at_most: phase.planned_steps_at_most,
            publishes: publishes_after_the_phase - phase.publishes_before_the_phase,
            step_milliseconds_mean_of_the_earliest_eighth: mean_milliseconds_of(
                &phase.step_seconds[..eighth],
            ),
            step_milliseconds_mean_of_the_latest_eighth: mean_milliseconds_of(
                &phase.step_seconds[steps - eighth..],
            ),
            step_milliseconds_maximum: phase.step_seconds.iter().copied().fold(0.0, f64::max)
                * 1000.0,
            units_in_the_version: version
                .map(|at_the_end| u64::try_from(at_the_end.units.len()).expect("条数装得进 u64")),
            allocation_records_in_the_version: version.map(|at_the_end| {
                u64::try_from(at_the_end.allocation_records.len()).expect("条数装得进 u64")
            }),
            resident_bytes: resident_bytes_of_this_process(),
            chunk_bytes: chunk_bytes_of_the_pool(devices),
            cluster_segments,
        }
    }

    /// 结果行（`name=construction_phase`）的字段。
    pub(super) fn fields(&self) -> String {
        let optional = |value: Option<u64>| {
            value.map_or_else(|| "none".to_string(), |count| count.to_string())
        };
        format!(
            "phase={} seconds={:.3} steps={} planned_steps_at_most={} publishes={} \
             step_milliseconds_mean_of_the_earliest_eighth={:.3} step_milliseconds_mean_of_the_latest_eighth={:.3} \
             step_milliseconds_maximum={:.3} units_in_the_version={} allocation_records_in_the_version={} \
             resident_bytes={} chunk_bytes_held_by_the_devices={} chunk_bytes_stored={} cluster_segments={} \
             unit_area_segments={}",
            self.phase.name(),
            self.seconds,
            self.steps,
            self.planned_steps_at_most,
            self.publishes,
            self.step_milliseconds_mean_of_the_earliest_eighth,
            self.step_milliseconds_mean_of_the_latest_eighth,
            self.step_milliseconds_maximum,
            optional(self.units_in_the_version),
            optional(self.allocation_records_in_the_version),
            optional(self.resident_bytes),
            self.chunk_bytes.held_by_the_devices,
            self.chunk_bytes.stored,
            optional(
                self.cluster_segments
                    .map(|cluster_segments| cluster_segments.opened_by_this_mount)
            ),
            optional(
                self.cluster_segments
                    .map(|cluster_segments| cluster_segments.unit_area_segments)
            ),
        )
    }
}

/// 造池的钟：分阶段计时（每阶段一项 [`TimingOfAConstructionPhase`]），每过一个进度间隔往 `progress` 打一行进度
/// （阶段、已做/至多几步、开造起已用秒数、此刻常驻内存；`.claude/rules/test-script-discipline.md`「编译与算法都要快」那一节的进度行）。
/// 只在阶段开头、每做完一步与阶段末了查一次到没到间隔；头一次查就打。计时都在被测进程里、造池那条线程上读单调钟。
pub(super) struct ConstructionClock<'progress> {
    pool: String,
    pub(super) started: Instant,
    progress_interval: Duration,
    last_progress: Option<Instant>,
    progress: &'progress mut dyn FnMut(&str),
    phase_in_progress: Option<PhaseInProgress>,
    pub(super) timings: Vec<TimingOfAConstructionPhase>,
}

impl<'progress> ConstructionClock<'progress> {
    pub(super) fn new(
        pool: String,
        progress_interval: Duration,
        progress: &'progress mut dyn FnMut(&str),
    ) -> ConstructionClock<'progress> {
        ConstructionClock {
            pool,
            started: Instant::now(),
            progress_interval,
            last_progress: None,
            progress,
            phase_in_progress: None,
            timings: Vec::new(),
        }
    }

    /// 开一个阶段：它至多几步、开始时已发成几版（造池那一串的累计）。
    pub(super) fn begin(
        &mut self,
        phase: ConstructionPhase,
        planned_steps_at_most: u64,
        publishes: u64,
    ) {
        let now = Instant::now();
        self.phase_in_progress = Some(PhaseInProgress {
            phase,
            started: now,
            last_step_ended: now,
            planned_steps_at_most,
            step_seconds: Vec::new(),
            publishes_before_the_phase: publishes,
        });
        self.report_if_the_interval_passed();
    }

    /// 做完一步：这一步的挂钟从上一步做完（或阶段开头）量起。
    pub(super) fn step_done(&mut self) {
        let now = Instant::now();
        if let Some(phase) = &mut self.phase_in_progress {
            phase
                .step_seconds
                .push(now.duration_since(phase.last_step_ended).as_secs_f64());
            phase.last_step_ended = now;
        }
        self.report_if_the_interval_passed();
    }

    /// 正在造的那一阶段收尾：量阶段末了的版本大小、内存与块字节，记成一项。`publishes` 是此刻已发成的版数（造池那一串的累计），
    /// `cluster_segments` 是此刻这次挂载开过的聚簇段（这一阶段没有挂载的给 `None`）。
    pub(super) fn finish(
        &mut self,
        devices: &PoolDevices,
        version: Option<&TransactionOutput>,
        publishes: u64,
        cluster_segments: Option<ClusterSegmentsOfTheMount>,
    ) {
        self.report_if_the_interval_passed();
        if let Some(phase) = self.phase_in_progress.take() {
            let timing = TimingOfAConstructionPhase::of_the_phase(
                phase,
                devices,
                version,
                publishes,
                cluster_segments,
            );
            self.timings.push(timing);
        }
    }

    /// 正在造的那一阶段与它正在做的那一步（做完的步数加一）：造池停下的那一刻报它（[`MomentOfTheConstruction`]）。
    pub(super) fn phase_and_step_in_progress(&self) -> (ConstructionPhase, u64) {
        let phase = self
            .phase_in_progress
            .as_ref()
            .expect("造池里每一次发布都在 begin 与 finish 之间：发布之前开了它所在的阶段");
        (
            phase.phase,
            u64::try_from(phase.step_seconds.len()).expect("步数装得进 u64") + 1,
        )
    }

    fn report_if_the_interval_passed(&mut self) {
        let now = Instant::now();
        let is_due = self
            .last_progress
            .is_none_or(|last| now.duration_since(last) >= self.progress_interval);
        if is_due {
            let (phase_name, steps, planned_steps_at_most) = self
                .phase_in_progress
                .as_ref()
                .map_or(("none", 0, 0), |phase| {
                    (
                        phase.phase.name(),
                        phase.step_seconds.len(),
                        phase.planned_steps_at_most,
                    )
                });
            let line = format!(
                "{RESULT_LINE_PREFIX} 造池进度 pool={} phase={phase_name} done={steps} planned_at_most={planned_steps_at_most} \
                 elapsed_seconds={:.1} resident_bytes={}",
                self.pool,
                now.duration_since(self.started).as_secs_f64(),
                resident_bytes_of_this_process()
                    .map_or_else(|| "unread".to_string(), |bytes| bytes.to_string()),
            );
            (self.progress)(&line);
            self.last_progress = Some(now);
        }
    }
}

/// 建池会话发一次用户改动发成了什么：推的抬 F 之外发成的版数、推的抬 F 串数，与这次调用发成的各版真正写出的数据单元按 inode 号数的个数
/// （[`TransactionOutput::rewritten`] 里第一个文件之外的数据单元那一族）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PublishedWhileConstructing {
    pub(super) versions_published: u64,
    pub(super) floor_raises: u64,
    pub(super) data_units_written: BTreeMap<u64, u64>,
}

/// 建池会话发一次用户改动；核心层拒了原样交回它的拒绝（盘上与会话照核心层那一成员的文档）。
pub(super) fn publish_while_constructing(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    change: UserChange<'_>,
) -> Result<PublishedWhileConstructing, UserChangeRefused> {
    let published = session.publish_user_change(devices, change)?;
    let mut data_units_written: BTreeMap<u64, u64> = BTreeMap::new();
    for version in &published.versions_published {
        for role in &version.rewritten {
            if let TransactionUnit::DataOfAFileOtherThanTheFirst { inode, .. } = role {
                *data_units_written.entry(inode.0).or_insert(0) += 1;
            }
        }
    }
    Ok(PublishedWhileConstructing {
        versions_published: u64::try_from(published.versions_published.len()).expect("版数"),
        floor_raises: u64::try_from(published.floor_raises.len()).expect("抬 F 串数"),
        data_units_written,
    })
}

/// 抬 F 到准入上限（会话的分配器与现行那一版，同 E185（GC一批搬迁的挂钟拆分） 装置的做法）。
pub(super) fn raise_the_floor_to_the_admission_ceiling(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
) {
    let mut current = version_of_trees(session).clone();
    let _raised = raise_rollback_floor_to_the_admission_ceiling(
        &make_filesystem_parameters_of_the_pool(),
        devices,
        &mut session.allocator,
        &mut current,
        session.shadow_ledger,
    )
    .expect("抬 F 到准入上限");
    session.current = PoolVersion::of_a_version_of_trees(current);
}

#[cfg(test)]
mod tests;
