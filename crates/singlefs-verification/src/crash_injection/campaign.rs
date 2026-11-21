//! 一批崩溃注入：工作线程数、种子分片、每次调用自己的镜像目录，多线程跑完收成报告。

use super::*;

/// 崩溃注入的工作线程数从这个环境变量取（十进制正整数）；没设就取每台默认重活线程数
/// （`singlefs_verification::heavy_threads::default_heavy_threads`：`SINGLEFS_THREAD_CAP`，再没设取登记值）。
pub const CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_INJECTION_THREADS";

/// 每个工作线程摊到的片数：各段历史的长短差得远（一步就被拒的与连发四十次的），片切得比线程多，先跑完的线程接着领下一片。
const SLICES_PER_WORKER_THREAD: usize = 4;

/// 工作线程数是从哪来的：与实际起的线程数一起打进进度行，「机器多于 1 核却只用了 1 个线程」看得出来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashInjectionWorkerThreads {
    /// 环境变量 [`CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    FromTheEnvironmentVariable(usize),
    /// 没设环境变量，取每台默认重活线程数（`singlefs_verification::heavy_threads::default_heavy_threads`）。
    FromTheHeavyThreadDefault(usize),
    /// 调用方在代码里直接给的（用例拿不同线程数对拍）。
    GivenByCaller(usize),
}

impl CrashInjectionWorkerThreads {
    /// 线程数取环境变量，没设就取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取登记值）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数同样停。
    #[must_use]
    pub fn from_the_environment() -> Self {
        Self::from_the_environment_variable_named(
            CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
        )
    }

    /// 线程数取 `variable_name` 这个环境变量，没设就取每台默认重活线程数。
    /// 坏盘输入（增补 3 第 5 件）拿它配自己那个变量名：判定只有「环境变量 → 正整数」这一件事，
    /// 抄一份出来两边会分叉（`code-discipline.md`「重复要生成，不许手抄」）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数同样停。
    #[must_use]
    pub fn from_the_environment_variable_named(variable_name: &str) -> Self {
        Self::from_the_environment_value(
            variable_name,
            process_environment_variable(variable_name).as_deref(),
            process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        )
    }

    /// [`Self::from_the_environment_variable_named`] 的判定本身：环境变量的值、`SINGLEFS_THREAD_CAP` 的值（`None` 是没设）
    /// 都由调用方给（用例不改进程的环境变量）。解析经数值环境变量共用的那一份，线程数取 `NonZeroUsize`：
    /// 写错、写 0 的口径与仓里别的线程数变量相同。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串）：消息带变量名与原值；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
    #[must_use]
    pub(super) fn from_the_environment_value(
        variable_name: &str,
        value: Option<&str>,
        thread_cap_value: Option<&str>,
    ) -> Self {
        match number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
            variable_name,
            value,
        ) {
            Some(count) => Self::FromTheEnvironmentVariable(count.get()),
            None => Self::FromTheHeavyThreadDefault(
                default_heavy_threads_of_the_cap_value(thread_cap_value).get(),
            ),
        }
    }

    #[must_use]
    pub fn count(self) -> usize {
        match self {
            Self::FromTheEnvironmentVariable(count)
            | Self::FromTheHeavyThreadDefault(count)
            | Self::GivenByCaller(count) => count,
        }
    }

    #[must_use]
    pub fn source_name(self) -> &'static str {
        match self {
            Self::FromTheEnvironmentVariable(_) => "environment_variable",
            Self::FromTheHeavyThreadDefault(_) => "heavy_thread_default",
            Self::GivenByCaller(_) => "given_by_caller",
        }
    }
}

/// 一次崩溃注入跑什么：种子区间、每段几步、怎么摆崩溃状态、比重、历史怎么跑、线程数、判红时镜像留不留，
/// 以及跑完照哪一组断言判（跑的时候不读它，只交给报告的重放行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrashInjectionCampaign {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub draw: CrashPointDraw,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub worker_threads: CrashInjectionWorkerThreads,
    pub image_retention: FailureImageRetention,
    pub assertions: CrashInjectionAssertions,
}

/// 这个进程里建过几次战役的镜像目录（[`create_the_image_directory_of_a_campaign_call`] 每次取一个号，名字里带它）。
static IMAGE_DIRECTORIES_CREATED_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 一次战役调用试几个名字建镜像目录：名字撞上已有的目录（同进程号的旧进程留下的）就换下一个号，试满还建不出就不留现场。
const MOST_IMAGE_DIRECTORY_NAMES_TRIED: u32 = 64;

/// 给一次战役调用新建一个它独占的空镜像目录（临时目录下，名字带进程号、种子基与这个进程里的调用号；`create_dir` 排他新建，撞上已有的换号）。
/// 同一个进程里同一个种子基的几条用例并行跑时各写各的、各删各的：只由进程号与种子基定名时，一条用例收尾删掉的是另一条正在写的目录，
/// 报告里「判红的崩溃镜像曾写在……」那一行也跟着别的用例变。建不出（临时目录写不了、试满 [`MOST_IMAGE_DIRECTORY_NAMES_TRIED`] 个名字）交回
/// `None`：这一次不留现场（留现场是为了好查，不是判据，同 `write_crash_image_files`）。
fn create_the_image_directory_of_a_campaign_call(first_seed: u64) -> Option<PathBuf> {
    // 迭代上界：MOST_IMAGE_DIRECTORY_NAMES_TRIED 次；只有名字撞上已有目录时才试下一个
    for _name_tried in 0..MOST_IMAGE_DIRECTORY_NAMES_TRIED {
        let call_number = IMAGE_DIRECTORIES_CREATED_IN_THIS_PROCESS.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "singlefs-crash-injection-{}-seedbase-{first_seed}-call-{call_number}",
            std::process::id()
        ));
        match std::fs::create_dir(&directory) {
            Ok(()) => return Some(directory),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_unwritable) => return None,
        }
    }
    None
}

/// 一次战役调用跑完收尾它自己那一份镜像目录：里面一个文件都没有（没有判红的崩溃状态落盘）就删掉、交回 `None`；有的照 `retention`
/// 留着或删掉，交回它的路径（报告里那一行「留在」「曾写在」照它写）。只碰 `directory` 这一份，不碰别的调用的目录。
fn settle_the_image_directory_of_the_call(
    directory: &Path,
    retention: FailureImageRetention,
) -> Option<PathBuf> {
    let images_were_written =
        std::fs::read_dir(directory).is_ok_and(|mut entries| entries.next().is_some());
    match (images_were_written, retention) {
        (
            false,
            FailureImageRetention::KeepTheImageFiles
            | FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        ) => {
            let _ = std::fs::remove_dir(directory);
            None
        }
        (true, FailureImageRetention::KeepTheImageFiles) => Some(directory.to_path_buf()),
        (true, FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes) => {
            let _ = std::fs::remove_dir_all(directory);
            Some(directory.to_path_buf())
        }
    }
}

/// 一次战役调用的镜像目录（[`create_the_image_directory_of_a_campaign_call`] 建的那一份）的守卫：照常跑完由
/// [`Self::settle_into_the_report`] 收尾（照 `retention` 留或删、交回报告里点名的路径）；没走到那一步就丢掉（工作线程 panic、
/// 并片的断言失败）时 Drop 照同一份收尾：没写镜像的空目录删掉，写了镜像的照 `retention` 留或删，留下时把路径打到标准错误
/// （`.claude/singlefs-ai-sop/rules/command-safety.md`「测试镜像一律放临时目录」：成功、失败、panic 都删，留现场走显式开关）。
struct ImageDirectoryOfACampaignCall {
    /// 收尾过一次之后是 `None`，Drop 不再碰它。
    directory: Option<PathBuf>,
    retention: FailureImageRetention,
}

impl ImageDirectoryOfACampaignCall {
    fn of(directory: Option<PathBuf>, retention: FailureImageRetention) -> Self {
        Self {
            directory,
            retention,
        }
    }

    fn path(&self) -> Option<&Path> {
        self.directory.as_deref()
    }

    /// 照常跑完的收尾：交回报告里点名的镜像目录（[`settle_the_image_directory_of_the_call`]）。
    fn settle_into_the_report(mut self) -> Option<PathBuf> {
        self.directory.take().and_then(|directory| {
            settle_the_image_directory_of_the_call(&directory, self.retention)
        })
    }
}

impl Drop for ImageDirectoryOfACampaignCall {
    fn drop(&mut self) {
        let Some(directory) = self.directory.take() else {
            return;
        };
        let left_behind = settle_the_image_directory_of_the_call(&directory, self.retention);
        match (left_behind, self.retention) {
            (Some(kept), FailureImageRetention::KeepTheImageFiles) => eprintln!(
                "崩溃注入这一次调用没跑完就停了，判红的崩溃镜像留在 {}",
                kept.display()
            ),
            (Some(_), FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes)
            | (
                None,
                FailureImageRetention::KeepTheImageFiles
                | FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            ) => {}
        }
    }
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，
/// 线程按片号从小到大领片，调用线程收到一片就打一行 `CRASH_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），
/// 再按片号从小到大并计数。计数按片的次序相加、新发现按种子从小到大留第一个，所以
/// [`CrashInjectionReport::render_the_conclusions`] 与线程数、调度次序无关（进度行是按到达次序打的，只报跑到哪了，不带判定；
/// 判红的崩溃镜像每次调用写进自己新建的那一份目录，[`create_the_image_directory_of_a_campaign_call`]，路径不进那一份）。
///
/// # Panics
/// 某个工作线程 panic（历史里的 panic 在执行器里接住，走到这里的是崩溃状态上恢复或 checker 的断言）；有一片领了却没交回；
/// `draw` 是靶向用例那一支 [`CrashPointDraw::ExactlyOneCrashState`]（只经 [`inject_crashes_into_history`] 摆，不进战役）。
#[must_use]
pub fn run_crash_injection_campaign(
    campaign: &CrashInjectionCampaign,
    given_states_judge: &dyn GivenStatesJudge,
) -> CrashInjectionReport {
    run_crash_injection_campaign_judging_the_given_states_with(campaign, given_states_judge)
}

/// [`run_crash_injection_campaign`]，第一截与第三截的给定状态表交 `given_states_judge` 判（崩溃点重放的引擎经它接进来，
/// `crate::crash_amplification::GivenStatesEngine`；各工作线程共用这一个判器）。
///
/// # Panics
/// 同 [`run_crash_injection_campaign`]；`given_states_judge` 判不了某一张表。
#[must_use]
pub fn run_crash_injection_campaign_judging_the_given_states_with(
    campaign: &CrashInjectionCampaign,
    given_states_judge: &dyn GivenStatesJudge,
) -> CrashInjectionReport {
    let CrashInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        draw,
        weights,
        execution,
        worker_threads,
        image_retention,
        assertions,
    } = *campaign;
    assert!(
        !matches!(draw, CrashPointDraw::ExactlyOneCrashState { .. }),
        "{TARGETED_CRASH_STATE_IS_NOT_A_CAMPAIGN_DRAW}"
    );
    let slices = seed_slices(seed_count, worker_threads.count());
    let spawned_worker_threads = worker_threads.count().min(slices.len().max(1));
    let started = Instant::now();
    let image_directory = ImageDirectoryOfACampaignCall::of(
        create_the_image_directory_of_a_campaign_call(first_seed),
        image_retention,
    );
    println!(
        "CRASH_INJECTION_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} images={}",
        first_seed + seed_count,
        slices.len(),
        worker_threads.count(),
        worker_threads.source_name(),
        image_directory
            .path()
            .map_or_else(|| "none".to_string(), |directory| directory.display().to_string())
    );
    let next_slice_index = AtomicUsize::new(0);
    let (finished_slices, merged_slice_count) = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<HistoryCrashInjection>)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            let image_directory = image_directory.path();
            scope.spawn(move || loop {
                // 领一片之前问叫停（同流水线那一侧「每领一块之前先问」）：叫停了就不再领，领了的片照样判完交回
                if given_states_judge.stop_is_requested() {
                    break;
                }
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let injections: Vec<HistoryCrashInjection> = slice
                    .clone()
                    .map(|offset| {
                        let history = generate_history_with_weights(
                            HistorySeed(first_seed.wrapping_add(offset)),
                            operations_per_history,
                            &weights,
                        );
                        inject_crashes_into_history_judging_the_given_states_with(
                            &history,
                            execution,
                            draw,
                            image_directory,
                            given_states_judge,
                        )
                    })
                    .collect();
                if sender.send((slice_index, injections)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut waiting_for_earlier_slices: BTreeMap<usize, Vec<HistoryCrashInjection>> =
            BTreeMap::new();
        let mut in_order: Vec<HistoryCrashInjection> = Vec::new();
        let mut next_slice_to_merge = 0usize;
        let mut finished_histories = 0u64;
        for (finished_slice_count, (slice_index, injections)) in receiver.iter().enumerate() {
            let slice = &slices[slice_index];
            finished_histories += slice.end - slice.start;
            println!(
                "CRASH_INJECTION_PROGRESS slice={}/{} seeds=[{},{}) finished_slices={}/{} finished_histories={finished_histories}/{seed_count} elapsed_seconds={:.1}",
                slice_index + 1,
                slices.len(),
                first_seed.wrapping_add(slice.start),
                first_seed.wrapping_add(slice.end),
                finished_slice_count + 1,
                slices.len(),
                started.elapsed().as_secs_f64()
            );
            waiting_for_earlier_slices.insert(slice_index, injections);
            while let Some(ready) = waiting_for_earlier_slices.remove(&next_slice_to_merge) {
                in_order.extend(ready);
                next_slice_to_merge += 1;
            }
        }
        (in_order, next_slice_to_merge)
    });
    // 领片的下标从 0 起一个接一个发，叫停只让后面的片没人领：领了的片都交回了，就是并进来的那几片
    let slices_taken = next_slice_index.load(Ordering::Relaxed).min(slices.len());
    assert_eq!(
        merged_slice_count, slices_taken,
        "领了的每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    let histories_left_by_the_stop_request: u64 = slices[slices_taken..]
        .iter()
        .map(|slice| slice.end - slice.start)
        .sum();
    println!(
        "CRASH_INJECTION_FINISHED seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} elapsed_seconds={:.1}",
        first_seed + seed_count,
        slices.len(),
        started.elapsed().as_secs_f64()
    );
    let mut tally = CrashInjectionTally::default();
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<CrashPointFinding> = Vec::new();
    let mut signatures_seen: BTreeSet<(CrashStateStage, FailureSignature)> = BTreeSet::new();
    for injection in &finished_slices {
        tally.absorb(&injection.tally);
        known_red_hits.extend(injection.known_red_hits.iter().cloned());
        for finding in &injection.new_findings {
            if signatures_seen.insert((finding.stage, finding.signature.clone())) {
                new_findings.push(finding.clone());
            }
        }
    }
    let image_directory_of_the_report = image_directory.settle_into_the_report();
    CrashInjectionReport {
        first_seed,
        seed_count,
        operations_per_history,
        draw,
        weights,
        execution,
        tally,
        known_red_hits,
        new_findings,
        image_directory: image_directory_of_the_report,
        image_retention,
        assertions,
        histories_left_by_the_stop_request,
    }
}

/// 把 [0, `seed_count`) 切成首尾相接的种子区间：片数取 min(种子数, 4 × 线程数)，各片长度相差至多 1。
/// 坏盘输入（增补 3 第 5 件）与故障注入共用这一份：切法抄一份出来几处会分叉（`code-discipline.md`「重复要生成，不许手抄」）。
#[must_use]
pub fn seed_slices(seed_count: u64, worker_threads: usize) -> Vec<std::ops::Range<u64>> {
    if seed_count == 0 {
        return Vec::new();
    }
    let wanted = u64::try_from(worker_threads.max(1) * SLICES_PER_WORKER_THREAD).expect("片数");
    let slice_count = wanted.min(seed_count);
    // 种子数 × 片号在 u128 里乘：种子数是用户给的，接近 u64 上限时 u64 里乘就溢出
    let bound_of = |slice_index: u64| -> u64 {
        u64::try_from(u128::from(seed_count) * u128::from(slice_index) / u128::from(slice_count))
            .expect("种子数 × 片号 ÷ 片数不超过种子数，装得回 u64")
    };
    (0..slice_count)
        .map(|slice_index| bound_of(slice_index)..bound_of(slice_index + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 同一个进程里同一个种子基的两次战役调用并发建镜像目录（两条用例并行跑）：各得一份不同的、新建的空目录；没写镜像的那一次收尾
    /// 只删自己那一份、报告不点名镜像目录，另一次写下的镜像还在；写了镜像的那一次收尾删掉自己那一份、报告点名它。
    /// 判别力（`crates/mutations.tsv`）：目录名退回只由进程号与种子基定，第二次调用的名字恒撞上第一次的，建不出自己那一份。
    #[test]
    fn two_campaign_calls_with_the_same_seed_base_in_one_process_each_get_and_settle_their_own_image_directory(
    ) {
        use crate::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
        let (directory_of_one_call, directory_of_another_call) = std::thread::scope(|scope| {
            let one_call = scope.spawn(|| {
                create_the_image_directory_of_a_campaign_call(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE)
            });
            let another_call = scope.spawn(|| {
                create_the_image_directory_of_a_campaign_call(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE)
            });
            (
                one_call.join().expect("建镜像目录的线程不 panic"),
                another_call.join().expect("建镜像目录的线程不 panic"),
            )
        });
        let one_call = directory_of_one_call.expect("这一次调用建得出自己的镜像目录");
        let another_call = directory_of_another_call.expect("另一次调用也建得出自己的镜像目录");
        assert_ne!(
            one_call, another_call,
            "同进程、同种子基的两次调用各一份目录"
        );
        for directory in [&one_call, &another_call] {
            assert!(
                std::fs::read_dir(directory)
                    .expect("新建的目录读得了")
                    .next()
                    .is_none(),
                "新建的 {} 是空的",
                directory.display()
            );
        }
        let image_of_one_call = one_call.join("device-0.img");
        std::fs::write(
            &image_of_one_call,
            b"crash image of the call that judged red",
        )
        .expect("写得进这一次调用的镜像目录");
        assert_eq!(
            settle_the_image_directory_of_the_call(
                &another_call,
                FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes
            ),
            None,
            "没写镜像的那一次：报告不点名镜像目录"
        );
        assert!(
            !another_call.exists(),
            "没写镜像的那一次删了自己那一份空目录"
        );
        assert!(
            image_of_one_call.exists(),
            "另一次调用收尾不删这一次写下的镜像"
        );
        assert_eq!(
            settle_the_image_directory_of_the_call(
                &one_call,
                FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes
            ),
            Some(one_call.clone()),
            "写了镜像的那一次：报告点名它自己那一份目录"
        );
        assert!(!one_call.exists(), "写了镜像的那一次跑完删掉自己那一份");
    }

    /// 只为看镜像目录收没收尾的判器替身：`stop_is_requested` 照 `stop` 答（`Panic` 是问它就 panic，造工作线程 panic），一张表都不判。
    struct JudgeOnlyAnsweringTheStopRequest {
        stop: StopRequestAnswer,
    }

    /// 判器替身怎么答叫停。
    enum StopRequestAnswer {
        StopBeforeAnySlice,
        Panic,
    }

    impl GivenStatesJudge for JudgeOnlyAnsweringTheStopRequest {
        fn stop_is_requested(&self) -> bool {
            match self.stop {
                StopRequestAnswer::StopBeforeAnySlice => true,
                StopRequestAnswer::Panic => panic!("判器替身问叫停就 panic：造一次工作线程 panic"),
            }
        }

        fn judge_the_given_states(
            &self,
            table: &GivenStatesOfATable<'_>,
        ) -> Result<GivenStatesConclusionsOfATable, String> {
            Err(format!(
                "判器替身只答叫停、不判表 {}：领片之前就停了，走不到这里",
                table.table_name
            ))
        }
    }

    /// 一次战役：种子基 `first_seed` 起 4 段、每段 1 步，两个工作线程，判红的镜像跑完就删。
    fn campaign_of_four_histories_from(first_seed: u64) -> CrashInjectionCampaign {
        CrashInjectionCampaign {
            first_seed,
            seed_count: 4,
            operations_per_history: 1,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 1,
            },
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(2),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        }
    }

    /// 临时目录里这个进程以种子基 `first_seed` 建的战役镜像目录（名字照 [`create_the_image_directory_of_a_campaign_call`]）。
    fn image_directories_of_this_process_with_the_seed_base(first_seed: u64) -> Vec<String> {
        let prefix = format!(
            "singlefs-crash-injection-{}-seedbase-{first_seed}-call-",
            std::process::id()
        );
        std::fs::read_dir(std::env::temp_dir())
            .expect("临时目录读得了")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(&prefix))
            .collect()
    }

    /// 只给「照常跑完收尾」那一条用的种子基：别的用例不用它，临时目录里带它的镜像目录只会是那一条建的。
    const SEED_BASE_OF_THE_CAMPAIGN_THAT_ENDS_NORMALLY: u64 = 0x7e3a_0d1e_c7a5_e001;

    /// 只给「工作线程 panic」那一条用的种子基。
    const SEED_BASE_OF_THE_CAMPAIGN_WHOSE_WORKER_PANICS: u64 = 0x7e3a_0d1e_c7a5_e002;

    /// 场景：一次战役照常跑完（判器在领第一片之前就叫停，一段历史都不跑）。
    /// 预期：报告不点名镜像目录，这次调用建的那一份镜像目录不在临时目录里了。
    /// 判别力（`crates/mutations.tsv`）：照常跑完那一步收尾不删目录，目录还在，这条红。
    #[test]
    fn campaign_call_that_ends_normally_leaves_no_image_directory_behind() {
        let first_seed = SEED_BASE_OF_THE_CAMPAIGN_THAT_ENDS_NORMALLY;
        let report = run_crash_injection_campaign(
            &campaign_of_four_histories_from(first_seed),
            &JudgeOnlyAnsweringTheStopRequest {
                stop: StopRequestAnswer::StopBeforeAnySlice,
            },
        );
        assert_eq!(
            report.histories_left_by_the_stop_request, 4,
            "四段都被叫停留下"
        );
        assert_eq!(report.image_directory, None, "没写镜像：报告不点名镜像目录");
        assert_eq!(
            image_directories_of_this_process_with_the_seed_base(first_seed),
            Vec::<String>::new(),
            "照常跑完之后这次调用的镜像目录不在了"
        );
    }

    /// 场景：一次战役的工作线程 panic（判器替身被问叫停就 panic），战役没走到照常收尾那一步就 panic 着出去。
    /// 预期：panic 接住之后，这次调用建的那一份镜像目录不在临时目录里了。
    /// 判别力（`crates/mutations.tsv`）：镜像目录守卫的 Drop 不收尾，目录还在，这条红。
    #[test]
    fn campaign_call_whose_worker_thread_panics_leaves_no_image_directory_behind() {
        let first_seed = SEED_BASE_OF_THE_CAMPAIGN_WHOSE_WORKER_PANICS;
        let panicked = std::panic::catch_unwind(|| {
            run_crash_injection_campaign(
                &campaign_of_four_histories_from(first_seed),
                &JudgeOnlyAnsweringTheStopRequest {
                    stop: StopRequestAnswer::Panic,
                },
            )
        });
        assert!(panicked.is_err(), "工作线程 panic 之后战役 panic 着出去");
        assert_eq!(
            image_directories_of_this_process_with_the_seed_base(first_seed),
            Vec::<String>::new(),
            "panic 之后这次调用的镜像目录不在了"
        );
    }

    /// 一份落了镜像的报告：一个新发现，这一批的镜像目录与那个新发现的镜像都落在 `image_directory` 底下。
    fn report_whose_images_landed_under(image_directory: &Path) -> CrashInjectionReport {
        use crate::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
        let crash_point = CrashPoint {
            segment_index: 1,
            persisted_within_the_segment: vec![true, false],
            step: StepPosition::Operation(3),
            operation_kind: None,
        };
        let finding = CrashPointFinding {
            signature: FailureSignature::CheckerViolations {
                invariants: vec!["I-3.10"],
            },
            seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
            crash_point,
            stage: CrashStateStage::CrashImage,
            observation: FailureObservation {
                position: StepPosition::Operation(3),
                operation_kind: None,
                violations: vec![("I-3.10", "盘 0 槽 40 的分配代 4，诞生代号 5".to_string())],
                panic: None,
                newest_ring_root_txg: Some(5),
                root_ring_slot_count: Some(64),
                harness_judgement: None,
                model_disagreement: None,
                raised_floor_lands_only_on_abandoned_roots: None,
                record_check: RecordCheck::default(),
            },
            read_back: "第 5 版的文件".to_string(),
            image_files: Some(image_directory.join("seed-crash_image")),
        };
        CrashInjectionReport {
            first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            seed_count: 2,
            operations_per_history: 8,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 2,
            },
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            tally: CrashInjectionTally::default(),
            known_red_hits: Vec::new(),
            new_findings: vec![finding],
            image_directory: Some(image_directory.to_path_buf()),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
            histories_left_by_the_stop_request: 0,
        }
    }

    /// 同一批种子跑两遍、两遍都落了镜像（GPU 判器那一趟与 CPU 判器那一趟，镜像目录各是自己新建的那一份）：路径不同而结论相同的两份报告
    /// 判相同，结构体的 `==` 判不同；结论差一处（新发现读回的不同）的两份判不同，只差有没有落镜像的两份也判不同。
    /// 判别力（`crates/mutations.tsv`）：退回按结构体比，路径不同的那两份判成不同，这条红。
    #[test]
    fn two_reports_whose_images_landed_in_different_directories_have_the_same_conclusions_when_only_the_paths_differ(
    ) {
        let gpu_run = report_whose_images_landed_under(Path::new(
            "/nonexistent/singlefs-crash-injection-gpu-run-call-0",
        ));
        let cpu_run = report_whose_images_landed_under(Path::new(
            "/nonexistent/singlefs-crash-injection-cpu-run-call-1",
        ));
        assert_ne!(
            gpu_run, cpu_run,
            "前提：两份的镜像路径不同，结构体的 == 连路径一起比、判不同"
        );
        assert!(
            gpu_run.has_the_same_conclusions_as(&cpu_run),
            "路径不同而结论相同的两份判相同：\n{}\n{}",
            gpu_run.render_the_conclusions(),
            cpu_run.render_the_conclusions()
        );
        let mut differs_in_what_was_read_back = cpu_run.clone();
        differs_in_what_was_read_back.new_findings[0].read_back = "第 4 版的文件".to_string();
        assert!(
            !gpu_run.has_the_same_conclusions_as(&differs_in_what_was_read_back),
            "新发现读回的不同，判不同"
        );
        let mut no_image_landed = cpu_run.clone();
        no_image_landed.image_directory = None;
        no_image_landed.new_findings[0].image_files = None;
        assert!(
            !gpu_run.has_the_same_conclusions_as(&no_image_landed),
            "一边落了镜像、一边没落，判不同"
        );
    }

    /// 片是首尾相接、不重不漏的：并起来正好是 [0, 种子数)。
    #[test]
    fn seed_slices_tile_the_whole_range() {
        for (seed_count, worker_threads) in [(0u64, 4usize), (1, 4), (7, 3), (96, 16), (5, 32)] {
            let slices = seed_slices(seed_count, worker_threads);
            let covered: Vec<u64> = slices.iter().flat_map(|slice| slice.clone()).collect();
            assert_eq!(
                covered,
                (0..seed_count).collect::<Vec<u64>>(),
                "种子 {seed_count} 个、{worker_threads} 个线程"
            );
            assert!(
                slices.iter().all(|slice| slice.start < slice.end),
                "没有空片：{slices:?}"
            );
        }
    }
}
