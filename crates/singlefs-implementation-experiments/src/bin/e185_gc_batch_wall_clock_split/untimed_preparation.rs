use super::*;

// ───────────────────────────── 不计时的准备并行（登记第十二节 12.10「并行与挂钟」） ─────────────────────────────

const MEMORY_CAP_ENVIRONMENT_VARIABLE: &str = "E185_MEMORY_CAP";
/// 环境变量与本进程控制组的 `memory.max` 都没给（没经任务服务起）时的内存上限，取 28G；经服务起的，额度由服务给，落在控制组的 `memory.max` 里。
const DEFAULT_MEMORY_CAP_BYTES: u64 = 28 << 30;
/// 内存盘每写过一个 512 字节扇区占的内存上界（推的）：扇区本身 512 字节、堆分配头 16 字节、`Vec` 头 24 字节、键 8 字节，
/// 再加 B 树节点按半满摊到每个条目的约 64 字节，凑整到 640。
const MEMORY_BYTES_PER_WRITTEN_SECTOR_AT_MOST: u64 = 640;
/// 造池之前估一份池的内存时，元数据（节点、journal、根环）按数据单元字节的同等量留（推的）；造完之后按真写过的扇区数算，不再用它。
const METADATA_BYTES_PER_DATA_BYTE_AT_MOST: u64 = 1;
/// 准备一个取样点时同时攥着几份池：造好的那一份，与几何核时可写挂载用的拷贝（[`predict_the_greedy_choice`]）。
pub(super) const POOLS_HELD_WHILE_PREPARING_A_SAMPLING_POINT: u64 = 2;
/// 重造一份池的峰值按池的 5/4 算：池本身，加建池会话的版本、分配器与树（推的）。
const REBUILD_PEAK_NUMERATOR: u64 = 5;
const REBUILD_PEAK_DENOMINATOR: u64 = 4;
/// 准备之后的串行测量另要的余量，按那一点的池几份算：内存臂每次测量拷一份池，另有重放材料与镜像提示的工作集（推的）。
const POOLS_OF_HEADROOM_FOR_THE_MEASUREMENT: u64 = 2;

/// 内存上限从哪取。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MemoryCapSource {
    Environment,
    ControlGroup,
    Default,
}

impl MemoryCapSource {
    pub(super) const fn label(self) -> &'static str {
        match self {
            MemoryCapSource::Environment => "environment_variable",
            MemoryCapSource::ControlGroup => "control_group_memory_limit",
            MemoryCapSource::Default => "default_twenty_eight_gibibytes",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MemoryCap {
    pub(super) bytes: u64,
    pub(super) source: MemoryCapSource,
}

/// `28G`、`512M` 这种 systemd 的 MemoryMax 写法（1024 进制），或不带单位的字节数（控制组 `memory.max` 的写法）；零与读不懂的交回 `None`。
fn bytes_of_a_memory_size_text(text: &str) -> Option<u64> {
    let trimmed = text.trim();
    let units: [(char, u64); 4] = [
        ('K', 1 << 10),
        ('M', 1 << 20),
        ('G', 1 << 30),
        ('T', 1 << 40),
    ];
    let (digits, bytes_per_unit) = units
        .iter()
        .find_map(|(suffix, bytes_per_unit)| {
            trimmed
                .strip_suffix(*suffix)
                .map(|digits| (digits, *bytes_per_unit))
        })
        .unwrap_or((trimmed, 1));
    digits
        .parse::<u64>()
        .ok()
        .filter(|count| *count > 0)?
        .checked_mul(bytes_per_unit)
}

/// 内存上限的取法：`E185_MEMORY_CAP` 设了照它，没设取本进程控制组的 `memory.max`（`run-with-memory-cap.sh` 给的上限就落在那里），
/// 那里是 `max` 或读不到时取 28G。
///
/// # Panics
/// `E185_MEMORY_CAP` 设了而读不懂。
fn memory_cap_given(
    environment_value: Option<&str>,
    control_group_memory_limit_text: Option<&str>,
) -> MemoryCap {
    if let Some(value) = environment_value {
        let bytes = bytes_of_a_memory_size_text(value).unwrap_or_else(|| {
            panic!("{MEMORY_CAP_ENVIRONMENT_VARIABLE} 要写成正整数加 K / M / G / T，或字节数，拿到「{value}」")
        });
        return MemoryCap {
            bytes,
            source: MemoryCapSource::Environment,
        };
    }
    match control_group_memory_limit_text.and_then(bytes_of_a_memory_size_text) {
        Some(bytes) => MemoryCap {
            bytes,
            source: MemoryCapSource::ControlGroup,
        },
        None => MemoryCap {
            bytes: DEFAULT_MEMORY_CAP_BYTES,
            source: MemoryCapSource::Default,
        },
    }
}

/// 本进程所在控制组（cgroup v2，`/proc/self/cgroup` 里 `0::` 那一行）的 `memory.max` 原文。
fn control_group_memory_limit_of_this_process() -> Option<String> {
    let membership = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let group = membership
        .lines()
        .find_map(|line| line.strip_prefix("0::"))?;
    std::fs::read_to_string(format!("/sys/fs/cgroup{group}/memory.max")).ok()
}

/// 准备阶段的设定：线程数（与饱和吞吐的 N 同一个取法）与内存上限。
pub(super) struct PreparationSettings {
    pub(super) threads: usize,
    pub(super) memory_cap: MemoryCap,
}

impl PreparationSettings {
    pub(super) fn from_the_environment() -> PreparationSettings {
        PreparationSettings {
            threads: worker_threads_from_the_environment(),
            memory_cap: memory_cap_given(
                std::env::var(MEMORY_CAP_ENVIRONMENT_VARIABLE)
                    .ok()
                    .as_deref(),
                control_group_memory_limit_of_this_process().as_deref(),
            ),
        }
    }
}

/// 造池之前估的一份池的内存（字节）：两块盘、每个数据单元至多写两次（建池一次、覆盖写一次），加多建的空 inode，元数据按同等量留，
/// 再按写过的扇区数折成内存。登记的两点：P = 1024 是 346030080，P = 8192 是 2694840320。
pub(super) fn estimated_pool_footprint_bytes_before_construction(point: SamplingPoint) -> u64 {
    let data_bytes = DEVICES_HOLDING_A_COPY
        * (2 * point.pool_data_units + SPARE_INODES_FOR_THE_FRONTIER)
        * LOCAL_DATA_UNIT_BYTES;
    let written_bytes = data_bytes * (1 + METADATA_BYTES_PER_DATA_BYTE_AT_MOST);
    written_bytes / u64::from(PHYSICAL_BLOCK_BYTES) * MEMORY_BYTES_PER_WRITTEN_SECTOR_AT_MOST
}

/// 造好的池的内存（字节）：两块内存盘写过的扇区数 × 每扇区的上界。
fn footprint_bytes_of_the_pool(pool: &ConstructedPool) -> u64 {
    pool.devices
        .iter()
        .map(|(_identity, device)| {
            u64::try_from(device.image.written_sectors().len()).expect("扇区数装得进 u64")
        })
        .sum::<u64>()
        * MEMORY_BYTES_PER_WRITTEN_SECTOR_AT_MOST
}

/// 这一刻还能再派的内存：上限减去已经记在账上的（在跑的各份按峰值、造好还没取走的池按真写过的扇区）与之后串行测量要的余量。
fn preparation_budget_bytes(memory_cap_bytes: u64, held_bytes: u64, headroom_bytes: u64) -> u64 {
    memory_cap_bytes.saturating_sub(held_bytes.saturating_add(headroom_bytes))
}

/// 这一刻能再派几份：按次序往里加，份数不超过空着的线程数、各份峰值之和不超过还能派的内存；装不下的那一份之后的都不派
/// （不跳过它去派后面的：免得后量的点占住内存、先量的点一直等）。各份峰值相同时就是 min(线程数, 预算 ÷ 一份峰值)。
/// 一份都装不下时的保底不在这里，在 [`PreparationQueue::units_to_start`]：什么都没在跑、要的那一份还没好时才保底一份。
pub(super) fn tasks_of_the_next_group(
    peak_bytes_in_order: &[u64],
    threads: usize,
    budget_bytes: u64,
) -> usize {
    let mut total_bytes = 0_u64;
    peak_bytes_in_order
        .iter()
        .take(threads)
        .take_while(|peak_bytes| {
            total_bytes = total_bytes.saturating_add(**peak_bytes);
            total_bytes <= budget_bytes
        })
        .count()
}

/// `/proc/self/status` 的 `Threads:` 那一行：本进程此刻的线程数。
fn threads_in_the_process_status(status_text: &str) -> Option<usize> {
    status_text
        .lines()
        .find_map(|line| line.strip_prefix("Threads:"))
        .and_then(|count| count.trim().parse().ok())
}

/// 本进程只剩量的那一条线程。
fn is_only_the_measuring_thread_left(status_text: &str) -> bool {
    threads_in_the_process_status(status_text) == Some(1)
}

/// 等准备线程从 `/proc/self/task` 里消失至多看几次（每毫秒一次）：join 交回之后，退出的线程消失得比它晚一点。
const LOOKS_FOR_THE_PREPARATION_THREADS_TO_LEAVE_AT_MOST: u32 = 1000;

/// 计时窗口（每次 `run_compaction`、设备顺序写带宽、第二段的阶段重放与 2 ms 对照）开始之前，本进程只剩一条线程（饱和吞吐自己起的线程除外，
/// 它在自己的计时里）。
///
/// # Panics
/// 等满仍不是一条（准备线程漏进了计时窗口），或读不到 `/proc/self/status`。
pub(super) fn assert_this_process_runs_one_thread(moment: &str) {
    let mut status_text = None;
    for _look in 0..LOOKS_FOR_THE_PREPARATION_THREADS_TO_LEAVE_AT_MOST {
        status_text = std::fs::read_to_string("/proc/self/status").ok();
        if status_text
            .as_deref()
            .is_some_and(is_only_the_measuring_thread_left)
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!(
        "计时窗口（{moment}）开始之前本进程应只剩一条线程，读到 {:?} 条",
        status_text
            .as_deref()
            .and_then(threads_in_the_process_status)
    );
}

/// 计时窗口（登记第十二节 12.10 ②）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TimingWindow {
    WallClockCompactionRun,
    DeviceSequentialWrite,
    StageSplitCompactionRun,
    StageReplay,
    StageControls,
    DiskSizeSensitivity,
    /// 第 4 次重跑写法与拆分段：一次调用之后九种写法的计时件、对拍与 PW1、PW2（登记 `research/prompts/e185-r4-prereg.md` 第五节）。
    CpuWritingPieces,
    /// 校准的基线、加倍对照与等常态（第 2 次重跑登记第十四节「串行」：量的是这一条测量线程此刻跑同一份活要多久）。
    Calibration,
}

impl TimingWindow {
    /// 线程核没过时 panic 消息里点名的那一处。
    const fn moment(self) -> &'static str {
        match self {
            TimingWindow::WallClockCompactionRun | TimingWindow::StageSplitCompactionRun => {
                "run_compaction"
            }
            TimingWindow::DeviceSequentialWrite => "device_sequential_write",
            TimingWindow::StageReplay => "stage_replay",
            TimingWindow::StageControls => "stage_controls",
            TimingWindow::DiskSizeSensitivity => "disk_size_sensitivity",
            TimingWindow::CpuWritingPieces => "cpu_writing_pieces",
            TimingWindow::Calibration => "calibration",
        }
    }
}

/// 进一个计时窗口：本进程只剩一条线程（[`assert_this_process_runs_one_thread`]）。
pub(super) fn enter_the_timing_window(window: TimingWindow) {
    assert_this_process_runs_one_thread(window.moment());
}

// ───────────────────────────── 不计时的准备：一个全局队列（登记第十二节 12.11） ─────────────────────────────

/// 一份不计时的准备。封闭集合。派生的 `Ord` 只给记账的表当键用；派活的次序是 [`preparation_units_in_measurement_order`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum PreparationUnit {
    /// 一个取样点的造池与几何核（[`prepare_the_sampling_point`]；同一点的几次改负载重造前后依赖，在一条线程上按次序）。
    PrepareThePoint { point_index: usize },
    /// 一个取样点的真盘镜像（第一段）：起始件与工作件写满零、写进池、核摘要。
    BuildTheRealDiskImages { point_index: usize },
    /// 一个取样点第 `round` 轮（2–5）的内存池重造。
    RebuildForTheRound { point_index: usize, round: usize },
}

impl PreparationUnit {
    const fn point_index(self) -> usize {
        match self {
            PreparationUnit::PrepareThePoint { point_index }
            | PreparationUnit::BuildTheRealDiskImages { point_index }
            | PreparationUnit::RebuildForTheRound { point_index, .. } => point_index,
        }
    }

    /// 产物行里点名这一份：`prepare:<活槽占比>:<池的数据量>`、`real_disk_images:…`、`rebuild:…:<轮>`。
    pub(super) fn label(self, points: &[SamplingPoint]) -> String {
        let point = points[self.point_index()];
        let point_text = format!("{}:{}", point.fraction.label(), point.pool_data_units);
        match self {
            PreparationUnit::PrepareThePoint { .. } => format!("prepare:{point_text}"),
            PreparationUnit::BuildTheRealDiskImages { .. } => {
                format!("real_disk_images:{point_text}")
            }
            PreparationUnit::RebuildForTheRound { round, .. } => {
                format!("rebuild:{point_text}:{round}")
            }
        }
    }
}

/// 一段里全部准备单位，按量的次序：每个取样点先造池与几何核，再建真盘镜像（第一段），再第 2–5 轮重造。
/// 内存紧时按这个次序派（[`tasks_of_the_next_group`]），先量的先造好。
fn preparation_units_in_measurement_order(
    points: usize,
    with_real_disk_images: bool,
) -> Vec<PreparationUnit> {
    (0..points)
        .flat_map(|point_index| {
            std::iter::once(PreparationUnit::PrepareThePoint { point_index })
                .chain(
                    with_real_disk_images
                        .then_some(PreparationUnit::BuildTheRealDiskImages { point_index }),
                )
                .chain(
                    (2..=ROUNDS).map(move |round| PreparationUnit::RebuildForTheRound {
                        point_index,
                        round,
                    }),
                )
        })
        .collect()
}

/// 派给一条准备线程的东西：都是它自有的值或共享只读的，线程之间不共享可变的东西。
pub(super) enum PreparationWork {
    PrepareThePoint(SamplingPoint),
    BuildTheRealDiskImages {
        directory: PathBuf,
        prepared: Arc<PreparedPoint>,
    },
    Rebuild(PoolRecipe),
}

/// 重造的一份池：用的配方（那一点还没造好时是头一次的配方，推测着先造）、池与它的分块摘要。
pub(super) struct RebuiltPool {
    pub(super) recipe: PoolRecipe,
    pub(super) pool: ConstructedPool,
    pub(super) digests: Vec<ChunkDigest>,
}

/// 一份准备的结果。池的内存（[`footprint_bytes_of_the_pool`]）在准备线程上算好带回来。真盘镜像建不成的原因已经写成产物行的字段。
pub(super) enum PreparationOutcome {
    Prepared {
        prepared: PreparedPoint,
        footprint_bytes: u64,
    },
    RealDiskImages(Result<RealDiskImages, String>),
    Rebuilt {
        rebuilt: RebuiltPool,
        footprint_bytes: u64,
    },
}

/// 头一次的配方：只有取样点与量的那一侧用的覆盖写次序（改法甲），没有改负载多覆盖写的 inode。几何核还没定时，重造按它推测着先造。
pub(super) fn first_recipe_of(point: SamplingPoint) -> PoolRecipe {
    PoolRecipe {
        point,
        overwrite_order: OverwriteOrder::HighestFullSegmentLast,
        extra_overwritten_inodes: BTreeSet::new(),
    }
}

/// 真做一份准备（[`PreparationQueue`] 平常派的就是它；单测换成假的）。
pub(super) fn run_the_preparation_work(work: PreparationWork) -> PreparationOutcome {
    match work {
        PreparationWork::PrepareThePoint(point) => {
            let prepared = prepare_the_sampling_point(point);
            let footprint_bytes = footprint_bytes_of_the_pool(&prepared.pool);
            PreparationOutcome::Prepared {
                prepared,
                footprint_bytes,
            }
        }
        PreparationWork::BuildTheRealDiskImages {
            directory,
            prepared,
        } => PreparationOutcome::RealDiskImages(
            RealDiskImages::create(&directory, prepared.recipe.point, &prepared.pool)
                .map_err(|error| field_token(&error.to_string())),
        ),
        PreparationWork::Rebuild(recipe) => {
            // 第 2–5 轮的重造不数落点（改动三只打每个点头一次造池），数与不数池逐块相同
            let pool = construct_the_pool(&recipe, LandingCount::NotCounted);
            let digests = digests_of_the_pool(&pool);
            let footprint_bytes = footprint_bytes_of_the_pool(&pool);
            PreparationOutcome::Rebuilt {
                rebuilt: RebuiltPool {
                    recipe,
                    pool,
                    digests,
                },
                footprint_bytes,
            }
        }
    }
}

/// 派准备的那个函数：平常是 [`run_the_preparation_work`]，单测换成假的。
pub(super) type PreparationWorker<'context> =
    &'context (dyn Fn(PreparationWork) -> PreparationOutcome + Sync);

/// 一份准备走到哪了。
enum UnitProgress<Outcome> {
    Waiting,
    Running,
    Done(Outcome),
    /// 量的那一侧取走了（或那一点量完、放掉了）。
    Taken,
}

/// 队列里一个取样点的几份准备。
pub(super) struct PointInTheQueue {
    pub(super) point: SamplingPoint,
    prepared: UnitProgress<Arc<PreparedPoint>>,
    real_disk_images: UnitProgress<Result<RealDiskImages, String>>,
    rebuilt_by_round: BTreeMap<usize, UnitProgress<RebuiltPool>>,
    is_measured: bool,
    pub(super) speculative_rebuilds_voided: usize,
}

/// 一段里全部不计时的准备：一个队列，按量的次序派到 min(线程数, 内存预算 ÷ 一份峰值) 条线程上（登记第十二节 12.11）。
/// 量的那一侧要哪一份就取哪一份（[`PreparationQueue::prepared_point`]、[`PreparationQueue::take_the_rebuilt_pool`]、
/// [`PreparationQueue::take_the_real_disk_images`]），还没好就跑一次准备（[`run_the_preparation_phase`]）：内存装得下全部就一次全派出去，
/// 装不下就派到装满为止，要的那一份好了就不再派新的，等在跑的都停下交回；之后量的那一侧串行量，下一次缺了再跑一次（批内并行、批间串行量）。
pub(super) struct PreparationQueue<'context> {
    pub(super) segment: &'static str,
    settings: &'context PreparationSettings,
    worker: PreparationWorker<'context>,
    image_directory: Option<PathBuf>,
    pub(super) points: Vec<PointInTheQueue>,
    order: Vec<PreparationUnit>,
    /// 记在账上的内存：在跑的按峰值，造好还没取走的按池的内存（[`footprint_bytes_of_the_pool`]）。
    accounted_bytes_by_unit: BTreeMap<PreparationUnit, u64>,
    footprint_bytes_of_the_prepared: BTreeMap<usize, u64>,
    phases: Vec<PreparationPhaseReading>,
}

impl<'context> PreparationQueue<'context> {
    /// 第一段带 `image_directory`（每点另有一份真盘镜像），第二段不带。
    pub(super) fn new(
        segment: &'static str,
        points: &[SamplingPoint],
        image_directory: Option<&Path>,
        settings: &'context PreparationSettings,
        worker: PreparationWorker<'context>,
    ) -> Self {
        PreparationQueue {
            segment,
            settings,
            worker,
            image_directory: image_directory.map(Path::to_path_buf),
            points: points
                .iter()
                .map(|point| PointInTheQueue {
                    point: *point,
                    prepared: UnitProgress::Waiting,
                    real_disk_images: UnitProgress::Waiting,
                    rebuilt_by_round: (2..=ROUNDS)
                        .map(|round| (round, UnitProgress::Waiting))
                        .collect(),
                    is_measured: false,
                    speculative_rebuilds_voided: 0,
                })
                .collect(),
            order: preparation_units_in_measurement_order(points.len(), image_directory.is_some()),
            accounted_bytes_by_unit: BTreeMap::new(),
            footprint_bytes_of_the_prepared: BTreeMap::new(),
            phases: Vec::new(),
        }
    }

    fn sampling_points(&self) -> Vec<SamplingPoint> {
        self.points
            .iter()
            .map(|point_state| point_state.point)
            .collect()
    }

    fn accounted_bytes(&self) -> u64 {
        self.accounted_bytes_by_unit.values().sum()
    }

    /// 一份池的内存：那一点造好了按真写过的扇区，没造好按估的。
    fn pool_bytes_of(&self, point_index: usize) -> u64 {
        self.footprint_bytes_of_the_prepared
            .get(&point_index)
            .copied()
            .unwrap_or_else(|| {
                estimated_pool_footprint_bytes_before_construction(self.points[point_index].point)
            })
    }

    /// 之后串行测量要留的余量：还没量完的点里最大一份池的几份（内存臂每次测量拷一份池，另有重放材料与镜像提示的工作集）。
    fn headroom_bytes(&self) -> u64 {
        (0..self.points.len())
            .filter(|point_index| !self.points[*point_index].is_measured)
            .map(|point_index| self.pool_bytes_of(point_index))
            .max()
            .unwrap_or(0)
            * POOLS_OF_HEADROOM_FOR_THE_MEASUREMENT
    }

    /// 一份在跑时的峰值内存：造池与几何核 2 份池（造好的与几何核的拷贝），重造 5/4 份，建镜像 1 份（提示盘至多是池那么大）。
    fn peak_bytes_of(&self, unit: PreparationUnit) -> u64 {
        match unit {
            PreparationUnit::PrepareThePoint { point_index } => {
                estimated_pool_footprint_bytes_before_construction(self.points[point_index].point)
                    * POOLS_HELD_WHILE_PREPARING_A_SAMPLING_POINT
            }
            PreparationUnit::BuildTheRealDiskImages { point_index } => {
                self.pool_bytes_of(point_index)
            }
            PreparationUnit::RebuildForTheRound { point_index, .. } => {
                self.pool_bytes_of(point_index) * REBUILD_PEAK_NUMERATOR / REBUILD_PEAK_DENOMINATOR
            }
        }
    }

    fn is_point_prepared(&self, point_index: usize) -> bool {
        matches!(self.points[point_index].prepared, UnitProgress::Done(_))
    }

    /// 这一份好了（造好了、或已被取走）。
    fn is_done(&self, unit: PreparationUnit) -> bool {
        let point_state = &self.points[unit.point_index()];
        match unit {
            PreparationUnit::PrepareThePoint { .. } => matches!(
                point_state.prepared,
                UnitProgress::Done(_) | UnitProgress::Taken
            ),
            PreparationUnit::BuildTheRealDiskImages { .. } => matches!(
                point_state.real_disk_images,
                UnitProgress::Done(_) | UnitProgress::Taken
            ),
            PreparationUnit::RebuildForTheRound { round, .. } => matches!(
                point_state.rebuilt_by_round.get(&round),
                Some(UnitProgress::Done(_) | UnitProgress::Taken)
            ),
        }
    }

    /// 能派了：还在等，且它要的都有了。真盘镜像要那一点已造好、它之前的点都已量完（盘上同一时刻只有正在量的那一点的镜像，
    /// 镜像在量那一点之前建）。重造不等那一点造好（推测着先造，[`PreparationQueue::start`]）。
    fn is_ready(&self, unit: PreparationUnit) -> bool {
        let point_state = &self.points[unit.point_index()];
        match unit {
            PreparationUnit::PrepareThePoint { .. } => {
                matches!(point_state.prepared, UnitProgress::Waiting)
            }
            PreparationUnit::BuildTheRealDiskImages { point_index } => {
                matches!(point_state.real_disk_images, UnitProgress::Waiting)
                    && self.is_point_prepared(point_index)
                    && self.points[..point_index]
                        .iter()
                        .all(|earlier_point| earlier_point.is_measured)
            }
            PreparationUnit::RebuildForTheRound { round, .. } => matches!(
                point_state.rebuilt_by_round.get(&round),
                Some(UnitProgress::Waiting)
            ),
        }
    }

    /// 这一刻派哪几份：能派的按量的次序排。最先量的那一点（能派的里头一份所在的点）取 [`tasks_of_the_next_group`] 份，可以只派一部分；
    /// 它整点派出去了，后面的点要整点装得下（线程与内存）才派，一点装不下就停。整点派：一点的几份一起派、一起做完，不让半点的长活
    /// （例如后量的那一点只派出造池、重造留到下一次）把这一次准备拖长又在下一次再拖一遍。一份都装不下而 `is_guaranteed_one`（什么都没在跑、
    /// 要的那一份还没好）时保底派最前面那一份，照串行那样一次只做一份。交回（派哪几份、是不是保底派的）。
    fn units_to_start(
        &self,
        free_threads: usize,
        is_guaranteed_one: bool,
    ) -> (Vec<PreparationUnit>, bool) {
        let ready: Vec<PreparationUnit> = self
            .order
            .iter()
            .copied()
            .filter(|unit| self.is_ready(*unit))
            .collect();
        let peaks: Vec<u64> = ready.iter().map(|unit| self.peak_bytes_of(*unit)).collect();
        let free_bytes = preparation_budget_bytes(
            self.settings.memory_cap.bytes,
            self.accounted_bytes(),
            self.headroom_bytes(),
        );
        let units_of_the_point_at = |start: usize| {
            ready[start..]
                .iter()
                .take_while(|unit| unit.point_index() == ready[start].point_index())
                .count()
        };
        let leading_units = if ready.is_empty() {
            0
        } else {
            units_of_the_point_at(0)
        };
        let mut fitting =
            tasks_of_the_next_group(&peaks[..leading_units], free_threads, free_bytes);
        if fitting == leading_units {
            let mut threads_left = free_threads.saturating_sub(fitting);
            let mut bytes_left = free_bytes.saturating_sub(peaks[..fitting].iter().sum());
            // 迭代上界：每一圈派出整整一点，至多 ready.len() 圈
            while fitting < ready.len() {
                let group_units = units_of_the_point_at(fitting);
                let group_bytes: u64 = peaks[fitting..fitting + group_units].iter().sum();
                if group_units > threads_left || group_bytes > bytes_left {
                    break;
                }
                threads_left -= group_units;
                bytes_left -= group_bytes;
                fitting += group_units;
            }
        }
        let is_guaranteed = fitting == 0 && is_guaranteed_one && !ready.is_empty();
        let count = if is_guaranteed { 1 } else { fitting };
        (ready.into_iter().take(count).collect(), is_guaranteed)
    }

    /// 记下开派一份（在跑，按峰值记账），交回给线程的东西。重造用这一刻的配方：那一点已造好就是它最后的配方，没造好就是
    /// 头一次的配方（推测着先造；那一点造好而配方改了，[`PreparationQueue::void_the_stale_rebuilds`] 把它作废）。
    pub(super) fn start(&mut self, unit: PreparationUnit) -> PreparationWork {
        let peak_bytes = self.peak_bytes_of(unit);
        self.accounted_bytes_by_unit.insert(unit, peak_bytes);
        let point_state = &mut self.points[unit.point_index()];
        match unit {
            PreparationUnit::PrepareThePoint { .. } => {
                point_state.prepared = UnitProgress::Running;
                PreparationWork::PrepareThePoint(point_state.point)
            }
            PreparationUnit::BuildTheRealDiskImages { .. } => {
                let prepared = match &point_state.prepared {
                    UnitProgress::Done(prepared) => Arc::clone(prepared),
                    UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => {
                        panic!("真盘镜像只在那一点造好之后、量完之前派（is_ready）")
                    }
                };
                point_state.real_disk_images = UnitProgress::Running;
                PreparationWork::BuildTheRealDiskImages {
                    directory: self
                        .image_directory
                        .clone()
                        .expect("只有带镜像目录的队列（第一段）才排真盘镜像这一份"),
                    prepared,
                }
            }
            PreparationUnit::RebuildForTheRound { round, .. } => {
                let recipe = match &point_state.prepared {
                    UnitProgress::Done(prepared) => prepared.recipe.clone(),
                    UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => {
                        first_recipe_of(point_state.point)
                    }
                };
                point_state
                    .rebuilt_by_round
                    .insert(round, UnitProgress::Running);
                PreparationWork::Rebuild(recipe)
            }
        }
    }

    /// 记下做完一份：账从峰值换成池的内存（真盘镜像不留账）；之后作废那一点配方对不上的重造。
    pub(super) fn finish(&mut self, unit: PreparationUnit, outcome: PreparationOutcome) {
        self.accounted_bytes_by_unit.remove(&unit);
        let point_index = unit.point_index();
        match (unit, outcome) {
            (
                PreparationUnit::PrepareThePoint { .. },
                PreparationOutcome::Prepared {
                    prepared,
                    footprint_bytes,
                },
            ) => {
                self.footprint_bytes_of_the_prepared
                    .insert(point_index, footprint_bytes);
                self.accounted_bytes_by_unit.insert(unit, footprint_bytes);
                self.points[point_index].prepared = UnitProgress::Done(Arc::new(prepared));
            }
            (
                PreparationUnit::BuildTheRealDiskImages { .. },
                PreparationOutcome::RealDiskImages(images),
            ) => {
                self.points[point_index].real_disk_images = UnitProgress::Done(images);
            }
            (
                PreparationUnit::RebuildForTheRound { round, .. },
                PreparationOutcome::Rebuilt {
                    rebuilt,
                    footprint_bytes,
                },
            ) => {
                self.accounted_bytes_by_unit.insert(unit, footprint_bytes);
                self.points[point_index]
                    .rebuilt_by_round
                    .insert(round, UnitProgress::Done(rebuilt));
            }
            (
                PreparationUnit::PrepareThePoint { .. },
                PreparationOutcome::RealDiskImages(_) | PreparationOutcome::Rebuilt { .. },
            )
            | (
                PreparationUnit::BuildTheRealDiskImages { .. },
                PreparationOutcome::Prepared { .. } | PreparationOutcome::Rebuilt { .. },
            )
            | (
                PreparationUnit::RebuildForTheRound { .. },
                PreparationOutcome::Prepared { .. } | PreparationOutcome::RealDiskImages(_),
            ) => panic!("准备线程交回的结果与派的那一份对不上"),
        }
        self.void_the_stale_rebuilds(point_index);
    }

    /// 那一点造好了：配方与它不同的重造（头一次的配方推测着先造的，几何核改了负载）作废、回去重排，按新配方再造。
    /// 还在跑的那几份交回时（[`PreparationQueue::finish`]）再判一次。
    fn void_the_stale_rebuilds(&mut self, point_index: usize) {
        let point_state = &mut self.points[point_index];
        let UnitProgress::Done(prepared) = &point_state.prepared else {
            return;
        };
        let stale_rounds: Vec<usize> = point_state
            .rebuilt_by_round
            .iter()
            .filter_map(|(round, progress)| match progress {
                UnitProgress::Done(rebuilt) => {
                    (rebuilt.recipe != prepared.recipe).then_some(*round)
                }
                UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => None,
            })
            .collect();
        for round in stale_rounds {
            point_state
                .rebuilt_by_round
                .insert(round, UnitProgress::Waiting);
            point_state.speculative_rebuilds_voided += 1;
            self.accounted_bytes_by_unit
                .remove(&PreparationUnit::RebuildForTheRound { point_index, round });
        }
    }

    pub(super) fn speculative_rebuilds_voided(&self) -> usize {
        self.points
            .iter()
            .map(|point_state| point_state.speculative_rebuilds_voided)
            .sum()
    }

    /// 要的那一份还没好就跑一次准备（[`run_the_preparation_phase`]）。
    ///
    /// # Panics
    /// 跑完那一次它仍没好（它要的点还没量完就要它的真盘镜像这类次序错）。
    fn make_ready(&mut self, unit: PreparationUnit, emitter: &mut Emitter) {
        if !self.is_done(unit) {
            run_the_preparation_phase(self, emitter, unit);
        }
        assert!(
            self.is_done(unit),
            "一次准备跑完，要的那一份（{}）应已好",
            unit.label(&self.sampling_points())
        );
    }

    /// 一个取样点造好的样子（造池与几何核）；那一点量完之前队列里留着一份（[`PreparationQueue::finish_the_point`] 放掉）。
    pub(super) fn prepared_point(
        &mut self,
        point_index: usize,
        emitter: &mut Emitter,
    ) -> Arc<PreparedPoint> {
        self.make_ready(PreparationUnit::PrepareThePoint { point_index }, emitter);
        match &self.points[point_index].prepared {
            UnitProgress::Done(prepared) => Arc::clone(prepared),
            UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => {
                panic!("那一点量完之后不再取它造好的池")
            }
        }
    }

    /// 取走一个取样点的真盘镜像（第一段，量那一点之前）。
    pub(super) fn take_the_real_disk_images(
        &mut self,
        point_index: usize,
        emitter: &mut Emitter,
    ) -> Result<RealDiskImages, String> {
        let unit = PreparationUnit::BuildTheRealDiskImages { point_index };
        self.make_ready(unit, emitter);
        self.accounted_bytes_by_unit.remove(&unit);
        match std::mem::replace(
            &mut self.points[point_index].real_disk_images,
            UnitProgress::Taken,
        ) {
            UnitProgress::Done(images) => images,
            UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => {
                panic!("一个取样点的真盘镜像只取一次")
            }
        }
    }

    /// 取走一个取样点第 `round` 轮的重造池（量那一轮之前）。
    pub(super) fn take_the_rebuilt_pool(
        &mut self,
        point_index: usize,
        round: usize,
        emitter: &mut Emitter,
    ) -> RebuiltPool {
        let unit = PreparationUnit::RebuildForTheRound { point_index, round };
        self.make_ready(unit, emitter);
        self.accounted_bytes_by_unit.remove(&unit);
        match self.points[point_index]
            .rebuilt_by_round
            .insert(round, UnitProgress::Taken)
        {
            Some(UnitProgress::Done(rebuilt)) => rebuilt,
            Some(UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken) | None => {
                panic!("一个取样点一轮的重造池只取一次，轮号在 2..=ROUNDS 里")
            }
        }
    }

    /// 那一点量完了：放掉它造好的池与没取走的重造（预算不再算它们），它之后的那一点可以建真盘镜像了。
    pub(super) fn finish_the_point(&mut self, point_index: usize) {
        let point_state = &mut self.points[point_index];
        point_state.is_measured = true;
        point_state.prepared = UnitProgress::Taken;
        for progress in point_state.rebuilt_by_round.values_mut() {
            *progress = UnitProgress::Taken;
        }
        self.accounted_bytes_by_unit
            .retain(|unit, _bytes| unit.point_index() != point_index);
    }
}

/// 一次准备的读数（产物 `name=preparation` 一行）。
#[derive(Clone, Debug, PartialEq)]
struct PreparationPhaseReading {
    target: PreparationUnit,
    started_prepare_units: usize,
    started_real_disk_image_units: usize,
    started_rebuild_units: usize,
    started_speculative_rebuild_units: usize,
    speculative_rebuilds_voided: usize,
    guaranteed_units: usize,
    most_threads_at_once: usize,
    distinct_threads: usize,
    memory_headroom_bytes: u64,
    largest_accounted_bytes: u64,
    wall_seconds: f64,
}

/// 一份准备交回主线程的：哪一份、在哪条线程上做的、结果（panic 了带着 panic 的内容，主线程再抛）。
type FinishedPreparation = (
    PreparationUnit,
    std::thread::ThreadId,
    std::thread::Result<PreparationOutcome>,
);

/// 跑一次准备：按 [`PreparationQueue::units_to_start`] 派，每份各起一条线程、各自建状态；收回一份就再看能不能派。
/// `target`（量的那一侧此刻要的那一份）好了就不再派新的，等在跑的都交回、全部线程停下（`std::thread::scope`）才回来，之后的计时窗口里没有准备线程。
/// 打一行 `name=preparation`，读数另记进 [`PreparationQueue::phases`]。
fn run_the_preparation_phase(
    queue: &mut PreparationQueue<'_>,
    emitter: &mut Emitter,
    target: PreparationUnit,
) {
    let started = Instant::now();
    let voided_before = queue.speculative_rebuilds_voided();
    let threads = queue.settings.threads;
    let worker = queue.worker;
    let mut reading = PreparationPhaseReading {
        target,
        started_prepare_units: 0,
        started_real_disk_image_units: 0,
        started_rebuild_units: 0,
        started_speculative_rebuild_units: 0,
        speculative_rebuilds_voided: 0,
        guaranteed_units: 0,
        most_threads_at_once: 0,
        distinct_threads: 0,
        memory_headroom_bytes: queue.headroom_bytes(),
        largest_accounted_bytes: queue.accounted_bytes(),
        wall_seconds: 0.0,
    };
    let mut distinct: HashSet<std::thread::ThreadId> = HashSet::new();
    let (sender, receiver) = std::sync::mpsc::channel::<FinishedPreparation>();
    std::thread::scope(|scope| {
        let mut running = 0_usize;
        // 迭代上界：每一圈至少收回一份（running 为 0 就出去）；要的那一份没好之前派的每一份至多被作废重派一次（作废只发生在那一点
        // 造好的那一刻与之前推测着派的那几份上），所以派出的份数有上界。跨圈的状态：running = 派出去还没收回的份数，不超过 threads。
        loop {
            if !queue.is_done(target) {
                let (units, is_guaranteed) =
                    queue.units_to_start(threads.saturating_sub(running), running == 0);
                if is_guaranteed {
                    reading.guaranteed_units += units.len();
                }
                for unit in units {
                    match unit {
                        PreparationUnit::PrepareThePoint { .. } => {
                            reading.started_prepare_units += 1;
                        }
                        PreparationUnit::BuildTheRealDiskImages { .. } => {
                            reading.started_real_disk_image_units += 1;
                        }
                        PreparationUnit::RebuildForTheRound { point_index, .. } => {
                            reading.started_rebuild_units += 1;
                            if !queue.is_point_prepared(point_index) {
                                reading.started_speculative_rebuild_units += 1;
                            }
                        }
                    }
                    let work = queue.start(unit);
                    let sender_of_this_unit = sender.clone();
                    scope.spawn(move || {
                        let outcome =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker(work)));
                        sender_of_this_unit
                            .send((unit, std::thread::current().id(), outcome))
                            .expect("收的一方等到派出去的每一份都交回");
                    });
                    running += 1;
                }
                reading.most_threads_at_once = reading.most_threads_at_once.max(running);
                reading.largest_accounted_bytes =
                    reading.largest_accounted_bytes.max(queue.accounted_bytes());
            }
            if running == 0 {
                break;
            }
            let (unit, thread, outcome) = receiver
                .recv()
                .expect("还有派出去没收回的，每一份交回之前发的一方都在");
            running -= 1;
            distinct.insert(thread);
            match outcome {
                Ok(outcome) => queue.finish(unit, outcome),
                Err(panic_content) => std::panic::resume_unwind(panic_content),
            }
        }
    });
    reading.distinct_threads = distinct.len();
    reading.speculative_rebuilds_voided = queue.speculative_rebuilds_voided() - voided_before;
    reading.wall_seconds = started.elapsed().as_secs_f64();
    emitter.line(&format!(
        "name=preparation segment={} target={} started_units={} started_prepare_units={} started_real_disk_image_units={} started_rebuild_units={} started_speculative_rebuild_units={} speculative_rebuilds_voided={} guaranteed_units={} most_threads_at_once={} distinct_threads={} thread_limit={threads} memory_cap_bytes={} memory_headroom_bytes={} largest_accounted_bytes={} wall_seconds={:.3}",
        queue.segment,
        target.label(&queue.sampling_points()),
        reading.started_prepare_units + reading.started_real_disk_image_units + reading.started_rebuild_units,
        reading.started_prepare_units,
        reading.started_real_disk_image_units,
        reading.started_rebuild_units,
        reading.started_speculative_rebuild_units,
        reading.speculative_rebuilds_voided,
        reading.guaranteed_units,
        reading.most_threads_at_once,
        reading.distinct_threads,
        queue.settings.memory_cap.bytes,
        reading.memory_headroom_bytes,
        reading.largest_accounted_bytes,
        reading.wall_seconds,
    ));
    queue.phases.push(reading);
}

/// 配置行里准备阶段那几段：线程数、内存上限与它从哪取、一共几份准备、按造池前的估算头一次能同时派几份、由此装不装得下全部
/// （`all_at_once`：除真盘镜像外全部一次派出；`in_batches`：分几批，批内并行、批间串行量）。真盘镜像那几份不在头一次里（要等前一点量完）。
pub(super) fn preparation_configuration_fields(
    points: &[SamplingPoint],
    image_directory: Option<&Path>,
    settings: &PreparationSettings,
) -> String {
    let queue = PreparationQueue::new(
        "configuration",
        points,
        image_directory,
        settings,
        &run_the_preparation_work,
    );
    let (started_at_once, _is_guaranteed) = queue.units_to_start(settings.threads, true);
    let real_disk_image_units = queue
        .order
        .iter()
        .filter(|unit| matches!(unit, PreparationUnit::BuildTheRealDiskImages { .. }))
        .count();
    let is_all_at_once = started_at_once.len() + real_disk_image_units == queue.order.len();
    format!(
        "preparation_threads={} preparation_memory_cap_bytes={} preparation_memory_cap_source={} preparation_units={} preparation_units_started_at_once_by_the_estimates={} preparation_plan_by_the_estimates={}",
        settings.threads,
        settings.memory_cap.bytes,
        settings.memory_cap.source.label(),
        queue.order.len(),
        started_at_once.len(),
        if is_all_at_once { "all_at_once" } else { "in_batches" },
    )
}

#[cfg(test)]
mod tests;
