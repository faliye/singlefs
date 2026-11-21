//! 一次挂载的后台计算（[`BackgroundComputeOfThisMount`]）：此刻生效、在开卡与挂着请求的几组设置，每批的路线，
//! 一批出错之后停着的后台任务；连同一批中间的只供测试钩子（[`InTheMiddleOfEachBackgroundBatch`]）。

use super::*;

/// 后台计算在这一次挂载里的状态：此刻生效的设置；已交给实现、实现还在会话锁外开卡而没生效的那一组；挂着时请求、从下一批开头起交给实现的那一组；
/// 因一批出错停着的后台任务。生命周期一次挂载，只住内存。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundComputeOfThisMount {
    /// 此刻生效的那一组：一批照它定走哪条路。选 `gpu` 的一组在实现答换上了（[`LatestGpuSettingsHandedIn::PutInEffect`]）那一刻才成为生效的；
    /// 实现答开不成（[`LatestGpuSettingsHandedIn::NotOpened`]）的那一组不生效，生效的照旧是切换之前那一组（实现照旧用的就是它）；
    /// 选 `cpu` 的一组交给实现那一刻就生效。
    in_effect: AdmittedBackgroundComputeSettings,
    /// 已交给实现、实现答还在为它开卡的那一组（选 `gpu`）：这期间照旧照 `in_effect` 跑；这期间一批出错，后台任务停下、报警，
    /// 实现答它换上了的那一刻解除（[`Self::stop_the_task_after_a_compute_error`]）。
    handed_in_while_the_implementation_opens_its_cards: Option<AdmittedBackgroundComputeSettings>,
    /// 实现答开不成的那一组请求给的补跑阈值：那一组不生效而阈值随切换请求从下一批起生效（第 4 条），补跑那一判照它；
    /// 生效的那一组换掉时清掉。没有是 `None`（照 `in_effect` 带的）。
    gc_cpu_compensation_threshold_of_the_switch_not_opened:
        Option<BackgroundGcCpuCompensationThreshold>,
    requested_from_the_next_batch: Option<AdmittedBackgroundComputeSettings>,
    stopped_after_a_compute_error: Vec<BackgroundTaskStoppedAfterAComputeError>,
    /// 停着的后台任务停下以来叫起了几次：头一次叫起时记下（一次叫起里一批出错停下的，停下那一刻就记下 0），停着的那几条换掉或清掉时一起清掉
    /// （[`Self::replace_the_tasks_stopped`]）。
    /// 一种后台任务只经换掉、清掉不再停着，所以这里没有不停着的任务的那一条。
    wakes_of_the_tasks_stopped: Vec<WakesOfAStoppedTask>,
    /// 此刻有没有一次叫起在做：挂载那一刻与可写挂载续做时没有。
    wake_under_way: WakeOfTheBackgroundTasks,
    /// 只供测试的钩子在这一批里注入的错，这一批读完它就清掉（[`Self::take_the_compute_error_of_the_batch_under_way`]）。
    compute_error_of_the_batch_under_way: Option<BackgroundComputeError>,
    /// 补跑那一判照什么判：产品恒是照读数判，只供测试的开关能改。
    gc_cpu_compensation_judgement: GcCpuCompensationJudgement,
}

impl BackgroundComputeOfThisMount {
    /// 挂载那一刻：挂载给的那一组（校验过）生效，没有待生效的。当场把这一组交给它带着的实现（[`AdmittedBackgroundComputeSettings`] 选 `gpu`
    /// 时交这一组，选 `cpu` 而带着实现时告诉它一张卡都不参与）、选 `gpu` 时当场问一次（[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]
    /// 里的同一判）：实现答换上了而照这一组一张卡都用不上时，有 GPU 路径的后台任务当场记成出错停着
    /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0），挂载照常做成；别的情形没有停着的后台任务。
    #[must_use]
    pub fn of_a_mount(admitted_at_the_mount: AdmittedBackgroundComputeSettings) -> Self {
        let mut background_compute = Self {
            in_effect: admitted_at_the_mount.clone(),
            handed_in_while_the_implementation_opens_its_cards: None,
            gc_cpu_compensation_threshold_of_the_switch_not_opened: None,
            requested_from_the_next_batch: None,
            stopped_after_a_compute_error: Vec::new(),
            wakes_of_the_tasks_stopped: Vec::new(),
            wake_under_way: WakeOfTheBackgroundTasks::NoneUnderWay,
            compute_error_of_the_batch_under_way: None,
            gc_cpu_compensation_judgement: GcCpuCompensationJudgement::ByTheReading,
        };
        background_compute.hand_the_settings_in(admitted_at_the_mount);
        background_compute.take_the_answer_on_the_settings_handed_in();
        background_compute
    }

    /// 只供测试（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：从这一刻起，停着的后台任务每一批开头的补跑那一判都当占用不超过阈值
    /// （[`WhetherTheTaskRunsTheNextBatch::StaysStopped`]，读数照常读），一直到这次挂载结束。在一批读完设置、开算之前的钩子里调，
    /// 强制进入「这一批在 CPU 上补跑了，下一批开头不补跑」那一格：下一批不开、撤 R 排他，停在先前出错停着上。没停着的后台任务不受影响。
    pub fn judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests(&mut self) {
        self.gc_cpu_compensation_judgement =
            GcCpuCompensationJudgement::NotAboveTheThresholdForcedForTests;
    }

    /// 因一批出错停着的后台任务（第 4 条），按停下的先后。
    #[must_use]
    pub fn stopped_after_a_compute_error(&self) -> &[BackgroundTaskStoppedAfterAComputeError] {
        &self.stopped_after_a_compute_error
    }

    /// `task` 此刻因一批出错停着。
    #[must_use]
    pub fn is_stopped_after_a_compute_error(&self, task: BackgroundTask) -> bool {
        self.stopped_after_a_compute_error
            .iter()
            .any(|stopped| stopped.task == task)
    }

    /// 只供测试（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：在一批读完设置、开算之前的钩子里调，让这一批照「算着出了错」走
    /// （第 4 条那一支；产品路径上 GPU 实现交回出错时也走它，[`BackgroundComputeError::GpuSegmentFailed`]）。
    pub fn report_a_compute_error_in_the_batch_under_way_for_tests(&mut self) {
        self.compute_error_of_the_batch_under_way =
            Some(BackgroundComputeError::InjectedByTheTestOnlyHook);
    }

    /// 这一批出了错没有（取走就清掉）：一批读完设置、钩子调过之后读一次。
    pub(crate) fn take_the_compute_error_of_the_batch_under_way(
        &mut self,
    ) -> Option<BackgroundComputeError> {
        self.compute_error_of_the_batch_under_way.take()
    }

    /// 一批出错之后（第 4 条）：一批里同一个条目键出现两次的只放弃这一批（[`ReachOfTheComputeError::AbandonsOnlyTheBatch`]），`task` 不停、不报警；
    /// 出错之前就挂着请求了、还没交给实现的切换，就当场解除——那一次切换算停着时收到的请求，`task` 不记成停着，之后照常等它生效；
    /// 别的把 `task` 记成停着：交给了实现而实现还在为它开卡的，那一组换上的那一刻解除（[`Self::take_the_answer_on_the_settings_handed_in`]），
    /// 都等下一次通过校验的切换请求。停在一次叫起里的（[`Self::begin_a_wake`] 开过、还没结束），
    /// 那一次叫起结束时报警不数这一趟（[`Self::alarm_at_the_end_of_a_wake`] 报 0）；停在可写挂载续做里的不是叫起，停下之后头一次叫起照数 1。不读写盘。
    pub(crate) fn stop_the_task_after_a_compute_error(
        &mut self,
        task: BackgroundTask,
        error: BackgroundComputeError,
    ) -> BackgroundTaskAfterTheComputeError {
        match error.reach_of_the_error() {
            ReachOfTheComputeError::AbandonsOnlyTheBatch => {
                return BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned;
            }
            ReachOfTheComputeError::StopsTheTask => {}
        }
        if self.requested_from_the_next_batch.is_some() {
            return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;
        }
        let task_after_the_error = match self.handed_in_while_the_implementation_opens_its_cards {
            Some(_) => BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards,
            None => BackgroundTaskAfterTheComputeError::LeftStopped,
        };
        if !self.is_stopped_after_a_compute_error(task) {
            assert!(
                self.wakes_of_the_tasks_stopped
                    .iter()
                    .all(|wakes| wakes.task != task),
                "停下以来叫起的次数只记停着的任务：没停着的这一种不该有一条"
            );
            self.stopped_after_a_compute_error
                .push(BackgroundTaskStoppedAfterAComputeError {
                    task,
                    error,
                    batches_compensated_on_the_cpu: 0,
                });
            match self.wake_under_way {
                WakeOfTheBackgroundTasks::UnderWay => {
                    self.wakes_of_the_tasks_stopped.push(WakesOfAStoppedTask {
                        task,
                        wakes_while_stopped: 0,
                        is_stopped_in_the_wake_under_way: true,
                    });
                }
                WakeOfTheBackgroundTasks::NoneUnderWay => {}
            }
        }
        task_after_the_error
    }

    /// 一次叫起开头（整理的显式入口每一趟、后台入口每一次判起跑，入口在任何读写之前就拒的那几判之后）：记下此刻有一次叫起在做，
    /// 这一次叫起里一批出错停下的，到它结束时不数（[`Self::stop_the_task_after_a_compute_error`]）。不读写盘。
    pub(crate) fn begin_a_wake(&mut self) {
        self.wake_under_way = WakeOfTheBackgroundTasks::UnderWay;
    }

    /// 换掉停着的那几条（`tasks_stopped` 可以是空：解除全部停下），停下以来叫起的次数一起清掉：换上的每一条都从这一刻起重新数。
    fn replace_the_tasks_stopped(
        &mut self,
        tasks_stopped: Vec<BackgroundTaskStoppedAfterAComputeError>,
    ) {
        self.stopped_after_a_compute_error = tasks_stopped;
        self.wakes_of_the_tasks_stopped.clear();
    }

    /// 一次叫起结束时（整理的显式入口每一趟、后台入口每一次判起跑，不论起没起跑、补没补跑；入口在任何读写之前就拒的那一次不算）：这一次叫起到此结束；
    /// `task` 此刻停着，就把它停下以来叫起的次数加一（这一次叫起里一批出错停下的不加，报 0）、交回这一次的报警
    /// （[`AlarmOfABackgroundTaskStoppedAfterAComputeError`]）；没停着交回 `None`。不读写盘。
    #[must_use = "停着的后台任务每次叫起都要报警：交回的报警要交到调用方手里"]
    pub(crate) fn alarm_at_the_end_of_a_wake(
        &mut self,
        task: BackgroundTask,
    ) -> Option<AlarmOfABackgroundTaskStoppedAfterAComputeError> {
        self.wake_under_way = WakeOfTheBackgroundTasks::NoneUnderWay;
        let stopped = self
            .stopped_after_a_compute_error
            .iter()
            .find(|stopped| stopped.task == task)?
            .clone();
        let wakes_while_stopped = match self
            .wakes_of_the_tasks_stopped
            .iter_mut()
            .find(|wakes| wakes.task == task)
        {
            Some(wakes) if wakes.is_stopped_in_the_wake_under_way => {
                wakes.is_stopped_in_the_wake_under_way = false;
                wakes.wakes_while_stopped
            }
            Some(wakes) => {
                wakes.wakes_while_stopped += 1;
                wakes.wakes_while_stopped
            }
            None => {
                self.wakes_of_the_tasks_stopped.push(WakesOfAStoppedTask {
                    task,
                    wakes_while_stopped: 1,
                    is_stopped_in_the_wake_under_way: false,
                });
                1
            }
        };
        Some(AlarmOfABackgroundTaskStoppedAfterAComputeError {
            task,
            gpu_devices_named_by_the_error: stopped.error.gpu_devices_named(),
            error_that_stopped_the_task: stopped.error,
            wakes_while_stopped,
            batches_compensated_on_the_cpu: stopped.batches_compensated_on_the_cpu,
        })
    }

    /// 补跑那一判此刻的读数：`fullest_device` 是调用方从分配器取的池子此刻最满的那一块盘
    /// （[`AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator`]），阈值取下一批读的那一组
    /// （[`Self::gc_cpu_compensation_threshold_of_the_next_batch`]）。
    #[must_use]
    pub fn gc_cpu_compensation_reading(
        &self,
        fullest_device: AllocatedBytesAndCapacityOfADevice,
    ) -> GcCpuCompensationReading {
        GcCpuCompensationReading {
            fullest_device,
            threshold: self.gc_cpu_compensation_threshold_of_the_next_batch(),
        }
    }

    /// 下一批读的那一组的补跑阈值：有请求了还没交给实现的取它，再看交给了实现而实现还在开卡的那一组，再看实现答开不成的那一组请求给的，
    /// 都没有取生效的（第 4 条：阈值随切换请求一起换、从下一批起生效，不等开卡，开不成也照它）。
    fn gc_cpu_compensation_threshold_of_the_next_batch(
        &self,
    ) -> BackgroundGcCpuCompensationThreshold {
        self.requested_from_the_next_batch
            .as_ref()
            .or(self
                .handed_in_while_the_implementation_opens_its_cards
                .as_ref())
            .map(|requested| requested.gc_cpu_compensation_threshold)
            .or(self.gc_cpu_compensation_threshold_of_the_switch_not_opened)
            .unwrap_or(self.in_effect.gc_cpu_compensation_threshold)
    }

    /// `task` 下一批跑不跑、怎么跑（第 4 条与）：没停着照设置走；GC 停着时池子最满的那一块盘的已分配占它的容量严格大于阈值
    /// （有一块盘超过）就在 CPU 上补跑，不超过就照旧停着。只读，不改状态。
    #[must_use]
    pub fn whether_the_task_runs_the_next_batch(
        &self,
        task: BackgroundTask,
        fullest_device: AllocatedBytesAndCapacityOfADevice,
    ) -> WhetherTheTaskRunsTheNextBatch {
        if !self.is_stopped_after_a_compute_error(task) {
            return WhetherTheTaskRunsTheNextBatch::AsTheSettingsSay;
        }
        let reading = self.gc_cpu_compensation_reading(fullest_device);
        match self.gc_cpu_compensation_judgement {
            GcCpuCompensationJudgement::ByTheReading => {}
            GcCpuCompensationJudgement::NotAboveTheThresholdForcedForTests => {
                return WhetherTheTaskRunsTheNextBatch::StaysStopped(reading);
            }
        }
        match task {
            // 只管 GC：里程碑六的打包搬迁加成员时这里加一臂，照旧停着。
            BackgroundTask::CompactionRelocation => {
                if reading.is_the_allocated_share_above_the_threshold() {
                    WhetherTheTaskRunsTheNextBatch::OnTheCpuToCompensateWhileStopped(reading)
                } else {
                    WhetherTheTaskRunsTheNextBatch::StaysStopped(reading)
                }
            }
        }
    }

    /// 此刻生效的那一组：正在算的这一批（有的话）用的就是它。
    #[must_use]
    pub fn in_effect(&self) -> &AdmittedBackgroundComputeSettings {
        &self.in_effect
    }

    /// 已交给实现、实现还在会话锁外为它开卡、还没生效的那一组（选 `gpu`）；没有是 `None`（`.claude/rules/fs-design.md` 五条硬要求第 4 条：
    /// 运行时看得出这一刻跑的是切换之前那一组）。
    #[must_use]
    pub fn handed_in_while_the_implementation_opens_its_cards(
        &self,
    ) -> Option<&AdmittedBackgroundComputeSettings> {
        self.handed_in_while_the_implementation_opens_its_cards
            .as_ref()
    }

    /// 挂着时请求过、还没到下一批开头的那一组。
    #[must_use]
    pub fn requested_from_the_next_batch(&self) -> Option<&AdmittedBackgroundComputeSettings> {
        self.requested_from_the_next_batch.as_ref()
    }

    /// 同 [`Self::request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch`]，GC 在 CPU 上补跑的阈值沿用下一批读的那一组的
    /// （有待生效的取它，没有取生效的）：不给阈值的切换不动阈值。
    ///
    /// # Errors
    /// 同 [`Self::request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch`]。
    pub fn request_switch_from_the_next_batch(
        &mut self,
        settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
    ) -> Result<(), BackgroundComputeSettingsRefused> {
        let threshold_of_the_next_batch = self.gc_cpu_compensation_threshold_of_the_next_batch();
        self.request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
            settings,
            gpu_compute,
            threshold_of_the_next_batch,
        )
    }

    /// 挂着切换（第 1 条，一次切全部后台任务）：与挂载同一套校验
    /// （[`admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold`]）；过了就记成待生效，
    /// 下一批开头起用，正在算的这一批照旧；同一批开头之前请求了几次，用最后一次。切到 `gpu` 要有实现：这一次递进来的，没递就沿用这次挂载
    /// 已有的那一份（待生效的那一组带的，没有再看生效的那一组带的）；都没有就是 GPU 不在，拒。有后台任务因一批出错停着时，过了校验的这一次当场解除
    /// 全部停下，从下一批起用新设置。
    ///
    /// # Errors
    /// 校验不过（第 4 条）：拒绝切换，生效的与待生效的都留着原样，停着的照旧停着。
    pub fn request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
        &mut self,
        settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
        gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
    ) -> Result<(), BackgroundComputeSettingsRefused> {
        let gpu_compute_of_this_mount = gpu_compute
            .or_else(|| {
                self.requested_from_the_next_batch
                    .as_ref()
                    .and_then(|requested| requested.gpu_compute.clone())
            })
            .or_else(|| self.in_effect.gpu_compute.clone());
        let admitted =
            admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
                settings,
                gpu_compute_of_this_mount,
                gc_cpu_compensation_threshold,
            )?;
        self.requested_from_the_next_batch = Some(admitted);
        self.replace_the_tasks_stopped(Vec::new());
        Ok(())
    }

    /// 两批之间：挂着请求、待生效的那一组有的话交给实现（[`Self::hand_the_settings_in`]），再问实现最近交给它的那一组此刻怎样了
    /// （[`Self::take_the_answer_on_the_settings_handed_in`]）：选 `gpu` 的一组要等实现答换上了才生效，实现还在会话锁外开卡的这段时间照旧照切换之前那一组跑
    /// （这期间一批出错，后台任务停下、报警，[`Self::stop_the_task_after_a_compute_error`]）；答换上了就照挂载那一刻的停法判
    /// （照它一张卡都用不上就把有 GPU 路径的后台任务当场记成出错停着，[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；用得上就不停，
    /// 开卡期间出错停着的也在这一刻解除）；答开不成，那一组不生效、生效的照旧是切换之前那一组，有 GPU 路径的后台任务照「一张卡都用不上」一样停着、报那个错
    /// （第 4 条「3 次都不成，GC 停下，报警……不自动再试」），直到下一次通过校验的切换。
    /// 选 `cpu` 的一组交给实现那一刻就生效。
    /// 一批开头（[`Self::begin_a_batch`]）调；整理的两个入口在判「整理停没停」之前也调（显式入口一趟开头、后台入口判起不起跑之前），
    /// 切到一张卡都用不上的那一组之后，下一次进来在任何一个写之前就判出停着：不走删除续做、不取走「该跑了」，与挂载那一刻同。
    /// 只在两批之间调：一批之内的切换只记成待生效。不读写盘。
    pub(crate) fn put_the_switch_requested_from_the_next_batch_in_effect(&mut self) {
        if let Some(requested) = self.requested_from_the_next_batch.take() {
            self.hand_the_settings_in(requested);
        }
        self.take_the_answer_on_the_settings_handed_in();
    }

    /// 把 `settings` 交给它带着的实现（[`AdmittedBackgroundComputeSettings::hand_the_settings_to_the_implementation`]）：选 `cpu` 的当场生效
    /// （之前交给实现、还在开卡的那一组作废；带着的实现那把锁中毒、告诉不了它一张卡都不参与也照样生效，CPU 上算不借它）；选 `gpu` 的记成交给了实现、
    /// 还没生效，等实现答（[`Self::take_the_answer_on_the_settings_handed_in`]）——实现那把锁中毒、交不进去时这一组不生效，有 GPU 路径的后台任务照这个错停着
    /// （[`BackgroundComputeError::GpuImplementationPanickedWhileLent`]），等下一次通过校验的切换（换一份实现）。
    fn hand_the_settings_in(&mut self, settings: AdmittedBackgroundComputeSettings) {
        let handed_to_the_implementation = settings.hand_the_settings_to_the_implementation();
        match (settings.gpu_settings(), handed_to_the_implementation) {
            (None, Ok(()) | Err(GpuImplementationPanickedWhileLent)) => {
                self.put_the_settings_in_effect(settings);
                self.handed_in_while_the_implementation_opens_its_cards = None;
            }
            (Some(_), Ok(())) => {
                self.handed_in_while_the_implementation_opens_its_cards = Some(settings);
            }
            (Some(_), Err(GpuImplementationPanickedWhileLent)) => {
                self.handed_in_while_the_implementation_opens_its_cards = None;
                self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by_the_error(
                    &BackgroundComputeError::GpuImplementationPanickedWhileLent,
                ));
            }
        }
    }

    /// 换上生效的那一组（实现答换上了、或选 `cpu` 的一组交给实现那一刻）：开不成那一组留下的补跑阈值跟着清掉。
    fn put_the_settings_in_effect(&mut self, settings: AdmittedBackgroundComputeSettings) {
        self.in_effect = settings;
        self.gc_cpu_compensation_threshold_of_the_switch_not_opened = None;
    }

    /// 问实现最近交给它的那一组此刻怎样了（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]）：
    /// 有交给了实现、还没生效的那一组就问它带着的实现，照答案办——还在开卡：不动；换上了：它生效，停着的那几条换成照它判出的
    /// （[`tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect`]；开卡期间出错停着的就此解除）；开不成：它不生效、清掉，
    /// 生效的照旧是切换之前那一组，有 GPU 路径的后台任务记成出错停着、报那个错，补跑阈值照它请求给的（[`Self::gc_cpu_compensation_threshold_of_the_next_batch`]）。
    /// 没有还没生效的那一组而生效的选 `gpu` 时也问生效那一组带着的实现（让它收下为更早几次交设置开完的卡），答案不用。
    fn take_the_answer_on_the_settings_handed_in(&mut self) {
        let Some(handed_in) = self
            .handed_in_while_the_implementation_opens_its_cards
            .take()
        else {
            let _answer_with_no_settings_waiting = self
                .in_effect
                .latest_gpu_settings_handed_in_asked_of_the_implementation();
            return;
        };
        match handed_in.latest_gpu_settings_handed_in_asked_of_the_implementation() {
            Some(Err(GpuImplementationPanickedWhileLent)) => {
                // 实现那把锁中毒、问不出那一组怎样了：那一组不生效，有 GPU 路径的后台任务照这个错停着。
                self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by_the_error(
                    &BackgroundComputeError::GpuImplementationPanickedWhileLent,
                ));
            }
            None | Some(Ok(LatestGpuSettingsHandedIn::StillOpeningTheirCards)) => {
                self.handed_in_while_the_implementation_opens_its_cards = Some(handed_in);
            }
            Some(Ok(LatestGpuSettingsHandedIn::PutInEffect {
                failure_under_the_settings,
            })) => {
                self.put_the_settings_in_effect(handed_in);
                self.replace_the_tasks_stopped(
                    tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect(
                        failure_under_the_settings.as_ref(),
                    ),
                );
            }
            Some(Ok(LatestGpuSettingsHandedIn::NotOpened { failure })) => {
                self.gc_cpu_compensation_threshold_of_the_switch_not_opened =
                    Some(handed_in.gc_cpu_compensation_threshold);
                self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));
            }
        }
    }

    /// 一批开头（每批读一次设置的那一处）：先把待生效的交给实现、问实现最近交给它的那一组此刻怎样了
    /// （[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]；一批之内不再交、不再问），
    /// 实现答换上了而照它一张卡都用不上、或答开不成，就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着
    /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；切换照收）；再判 `task` 这一批跑不跑（[`Self::whether_the_task_runs_the_next_batch`]，`fullest_device`
    /// 是调用方这一刻从分配器取的池子最满的那一块盘）——停着而不补跑就不开这一批；最后定这一批走哪条路：补跑的走 CPU、停下以来补跑的批数加一，
    /// 别的按生效的那一组定（实现还在为交给它的那一组开卡、或答了开不成时，生效的仍是切换之前那一组）。先问再判：停着的那几条照这一刻的答案换过
    /// （换上了的那一刻解除开卡期间出错的停下，开不成的那一刻照开不成停着）。
    ///
    /// # Errors
    /// `task` 停着、这一批不补跑 ⇒ [`BatchNotBegunWhileTheTaskStaysStopped`]：没有还没生效的切换、实现也没答出新结局时状态一样都没动；
    /// 有的话那一组已交给了实现、实现照它答过，停着的是这一批开头照答案记下的那几条。
    ///
    /// # Panics
    /// 上一批注入的错没被读走：注入只在一批的钩子里，那一批读完设置之后就读走。
    pub(crate) fn begin_a_batch(
        &mut self,
        task: BackgroundTask,
        fullest_device: AllocatedBytesAndCapacityOfADevice,
    ) -> Result<BackgroundBatchUnderWay, BatchNotBegunWhileTheTaskStaysStopped> {
        assert!(
            self.compute_error_of_the_batch_under_way.is_none(),
            "只供测试注入的错只在一批的钩子里注入、那一批就读走：一批开头不该还挂着上一批的"
        );
        self.put_the_switch_requested_from_the_next_batch_in_effect();
        let compensation = match self.whether_the_task_runs_the_next_batch(task, fullest_device) {
            WhetherTheTaskRunsTheNextBatch::StaysStopped(_) => {
                return Err(BatchNotBegunWhileTheTaskStaysStopped);
            }
            WhetherTheTaskRunsTheNextBatch::OnTheCpuToCompensateWhileStopped(reading) => {
                Some(reading)
            }
            WhetherTheTaskRunsTheNextBatch::AsTheSettingsSay => None,
        };
        let requested_backend = self.in_effect.settings.background_compute;
        let route = match compensation {
            None => route_of_the_task(task, requested_backend),
            Some(reading) => {
                let stopped = self
                    .stopped_after_a_compute_error
                    .iter_mut()
                    .find(|stopped| stopped.task == task)
                    .expect("补跑只在 task 停着时判出：上面那一判读的就是这张表，其间没改过它");
                stopped.batches_compensated_on_the_cpu += 1;
                BackgroundComputeRoute::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading),
                )
            }
        };
        Ok(BackgroundBatchUnderWay {
            settings_read_at_the_start_of_the_batch: self.in_effect.clone(),
            task,
            requested_backend,
            route,
        })
    }

    /// 一批结束：交回这一批的记账。`gpu_work` 是这一批在 GPU 上算了什么：走 GPU 的一批恒有（出错放弃的那一批是出错之前算完的几段），
    /// 走 CPU 的恒无。`compute_error_that_abandoned_the_batch` 是这一批算着出错放弃时的那个错。
    ///
    /// # Panics
    /// 生效的设置在这一批里换了（分支变量在操作中途改变，`.claude/rules/fs-design.md` 五条硬要求第 5 条）：
    /// 挂着的切换只记成待生效，换上只在 [`Self::begin_a_batch`]。`gpu_work` 有没有与这一批开头定下的走法不符。
    pub(crate) fn end_the_batch(
        &self,
        under_way: BackgroundBatchUnderWay,
        gpu_work: Option<GpuWorkOfTheBatch>,
        compute_error_that_abandoned_the_batch: Option<BackgroundComputeError>,
    ) -> BackgroundComputeOfOneBatch {
        assert_eq!(
            self.in_effect, under_way.settings_read_at_the_start_of_the_batch,
            "后台计算的设置一批之内不换：挂着的切换只记成待生效，下一批开头才换上"
        );
        let path_taken = match (under_way.route, gpu_work) {
            (BackgroundComputeRoute::Cpu(reason), None) => BackgroundComputePathTaken::Cpu(reason),
            (BackgroundComputeRoute::Gpu, Some(work)) => BackgroundComputePathTaken::Gpu(work),
            (BackgroundComputeRoute::Cpu(_), Some(_)) | (BackgroundComputeRoute::Gpu, None) => {
                panic!(
                    "一批在 GPU 上算了什么只在走 GPU 的那一批有：这一批开头定的是 {:?}",
                    under_way.route
                )
            }
        };
        BackgroundComputeOfOneBatch {
            task: under_way.task,
            requested_backend: under_way.requested_backend,
            path_taken,
            compute_error_that_abandoned_the_batch,
        }
    }
}

/// 一批读完设置、开算之前做什么（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。产品路径什么都不做；只供测试的那一臂调用例给的钩子，
/// 钩子拿到这次挂载的后台计算状态，在这一刻请求切换，强制进入「一批跑到一半时请求切换」那一格。
pub enum InTheMiddleOfEachBackgroundBatch<'hook> {
    NothingHappens,
    CallTheTestOnlyHook(&'hook mut dyn FnMut(&mut BackgroundComputeOfThisMount)),
}

impl InTheMiddleOfEachBackgroundBatch<'_> {
    pub(crate) fn in_the_middle_of_the_batch(
        &mut self,
        background_compute: &mut BackgroundComputeOfThisMount,
    ) {
        match self {
            InTheMiddleOfEachBackgroundBatch::NothingHappens => {}
            InTheMiddleOfEachBackgroundBatch::CallTheTestOnlyHook(hook) => hook(background_compute),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 切换点到没开的卡：交给实现之后、实现开完卡之前，切换还没生效（第 4 条「开成了，下一批起换成新设置」「出错之前已经收到、还没生效的切换请求，
    // 在停下那一刻也算数」「报的是挂载那一刻同一个成员」；第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」）──

    fn card_for_the_switch_tests(address: &[u8]) -> GpuDeviceIdentifier {
        GpuDeviceIdentifier::of_opaque_bytes(address.to_vec())
    }

    /// 替身照 [`BackgroundGpuCompute`] 文档写的约定：交来的一组点的卡都开着就当场换上；点到没开的卡就在会话锁外开（用例把 `opening_ends` 摆成 `true` 就算开完了），
    /// 开着的这段时间两批之间那一问答还在开卡，开完了换上、答换上了连同这一组用不用得上（`failure_once_put_in_effect`）。告诉它一张卡都不参与时开着的卡全部关掉。
    struct CardsOpenedByTheTestDoubleState {
        cards_opened: Vec<GpuDeviceIdentifier>,
        cards_waiting_for_the_opening: Option<Vec<GpuDeviceIdentifier>>,
        opening_ends: bool,
        failure_under_the_settings_in_use: Option<BackgroundGpuComputeFailed>,
        failure_once_put_in_effect: Option<BackgroundGpuComputeFailed>,
        times_told_no_gpu_takes_part: u32,
    }

    struct GpuComputeOpeningCardsWhenTheTestSays {
        state: Arc<Mutex<CardsOpenedByTheTestDoubleState>>,
    }

    impl GpuInventory for GpuComputeOpeningCardsWhenTheTestSays {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            vec![
                (
                    card_for_the_switch_tests(b"0000:01:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
                (
                    card_for_the_switch_tests(b"0000:41:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
            ]
        }
    }

    impl BackgroundGpuCompute for GpuComputeOpeningCardsWhenTheTestSays {
        fn verify_unit_checksums(
            &mut self,
            _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
        {
            Ok(Vec::new())
        }

        fn classify_region_records(
            &mut self,
            _region: CompactionRegion,
            _records: &[RegionRecordToClassifyOnTheGpu<'_>],
            _mapping_entries: &[CentralMappingEntry],
        ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn count_live_slots_per_cluster_segment(
            &mut self,
            _allocation_records: &[AllocationRecord],
            _unit_area: UnitAreaOfOneDevice,
            _segments: &[ClusterSegmentEntryKey],
        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
            let mut state = self.state.lock().expect("用例没在别处 panic 过");
            let cards_named: Vec<GpuDeviceIdentifier> = settings
                .participating_devices()
                .iter()
                .map(|participating| participating.device.clone())
                .collect();
            if cards_named
                .iter()
                .all(|card| state.cards_opened.contains(card))
            {
                state.cards_waiting_for_the_opening = None;
                state.failure_under_the_settings_in_use = state.failure_once_put_in_effect.clone();
            } else {
                state.cards_waiting_for_the_opening = Some(cards_named);
            }
        }

        fn use_no_gpu_from_the_next_batch(&mut self) {
            let mut state = self.state.lock().expect("用例没在别处 panic 过");
            state.cards_opened.clear();
            state.cards_waiting_for_the_opening = None;
            state.times_told_no_gpu_takes_part += 1;
        }

        fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
            &mut self,
        ) -> LatestGpuSettingsHandedIn {
            let mut state = self.state.lock().expect("用例没在别处 panic 过");
            if state.cards_waiting_for_the_opening.is_some() {
                if !state.opening_ends {
                    return LatestGpuSettingsHandedIn::StillOpeningTheirCards;
                }
                state.cards_opened = state
                    .cards_waiting_for_the_opening
                    .take()
                    .expect("上面判过在等");
                state.failure_under_the_settings_in_use = state.failure_once_put_in_effect.clone();
            }
            LatestGpuSettingsHandedIn::PutInEffect {
                failure_under_the_settings: state.failure_under_the_settings_in_use.clone(),
            }
        }
    }

    fn gpu_settings_naming_the_card(
        address: &[u8],
        memory_limit: Option<u64>,
    ) -> BackgroundComputeSettings {
        BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![card_for_the_switch_tests(address)]),
            background_gpu_memory_limit: memory_limit.map(|limit| vec![GpuMemoryInBytes(limit)]),
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        }
    }

    /// 池子最满的那一块盘远没到补跑阈值：停着就照旧停着。
    fn fullest_device_far_below_the_threshold() -> AllocatedBytesAndCapacityOfADevice {
        AllocatedBytesAndCapacityOfADevice {
            device: DeviceIdentity(0),
            allocated_bytes: 0,
            capacity_bytes: 1 << 30,
        }
    }

    fn every_gpu_left_out_on_the_card(address: &[u8]) -> BackgroundGpuComputeFailed {
        BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
            gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
                device: card_for_the_switch_tests(address),
                memory_limit: GpuMemoryInBytes(64),
                smallest_piece_bytes: GpuMemoryInBytes(972),
                bytes_already_held_on_the_device: GpuMemoryInBytes(0),
            }],
        }
    }

    /// 替身开着「0000:01:00.0」，两个出错照 `failure_under_the_settings_in_use`、`failure_once_put_in_effect` 摆。
    fn state_with_the_card_at_one_opened(
        failure_under_the_settings_in_use: Option<BackgroundGpuComputeFailed>,
        failure_once_put_in_effect: Option<BackgroundGpuComputeFailed>,
    ) -> Arc<Mutex<CardsOpenedByTheTestDoubleState>> {
        Arc::new(Mutex::new(CardsOpenedByTheTestDoubleState {
            cards_opened: vec![card_for_the_switch_tests(b"0000:01:00.0")],
            cards_waiting_for_the_opening: None,
            opening_ends: false,
            failure_under_the_settings_in_use,
            failure_once_put_in_effect,
            times_told_no_gpu_takes_part: 0,
        }))
    }

    /// 选 `gpu`、点「0000:01:00.0」的一次挂载，替身状态照 `state`。
    fn mount_on_the_card_at_one(
        state: &Arc<Mutex<CardsOpenedByTheTestDoubleState>>,
    ) -> BackgroundComputeOfThisMount {
        let handle = BackgroundGpuComputeHandle::of_an_implementation(
            GpuComputeOpeningCardsWhenTheTestSays {
                state: Arc::clone(state),
            },
        );
        BackgroundComputeOfThisMount::of_a_mount(
            admit_background_compute_settings_with_the_gpu_compute(
                gpu_settings_naming_the_card(b"0000:01:00.0", None),
                Some(handle),
            )
            .expect("卡清单列着「0000:01:00.0」"),
        )
    }

    fn failure_on_the_card_at_one() -> BackgroundComputeError {
        BackgroundComputeError::GpuSegmentFailed {
            segment: GpuComputedSegment::UnitChecksumVerification,
            failure: BackgroundGpuComputeFailed::GpuFailed {
                device: card_for_the_switch_tests(b"0000:01:00.0"),
                cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                    reason: "替身：切换之前那张卡上设备丢了".to_string(),
                },
                pci_state_when_judged_unusable: Box::new(
                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                ),
            },
        }
    }

    fn route_of_the_batch(
        batch: &Result<BackgroundBatchUnderWay, BatchNotBegunWhileTheTaskStaysStopped>,
    ) -> Option<BackgroundComputeRoute> {
        batch.as_ref().ok().map(|under_way| under_way.route)
    }

    /// 交给了实现、实现还在开卡的期间一批出错：挂载点「0000:01:00.0」，请求切到没开的「0000:41:00.0」；下一批开头交给实现（实现开卡、照切换之前那一组跑），
    /// 这一批在「0000:01:00.0」上出错——停下（不照还没生效的那一组解除）；卡开成、实现答换上了的那一批开头解除，那一批照常在 GPU 上开，
    /// 生效的是点「0000:41:00.0」的那一组。
    #[test]
    fn a_batch_failing_on_the_card_before_the_switch_while_the_new_card_opens_stops_until_that_switch_is_put_in_effect(
    ) {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_while_opening),
            Some(BackgroundComputeRoute::Gpu),
            "开卡的这段时间照切换之前那一组在 GPU 上算"
        );
        assert!(
            background_compute
                .handed_in_while_the_implementation_opens_its_cards()
                .is_some(),
            "交给了实现、还在开卡：切换还没生效"
        );
        assert_eq!(
            background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                failure_on_the_card_at_one(),
            ),
            BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards,
            "开卡期间旧卡出错：停下"
        );
        assert!(background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_once_opened),
            Some(BackgroundComputeRoute::Gpu),
            "换上的那一刻解除：那一批照常开"
        );
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
        assert_eq!(
            background_compute
                .in_effect()
                .settings()
                .background_gpu_devices,
            Some(vec![card_for_the_switch_tests(b"0000:41:00.0")]),
            "实现答换上了：生效的是切换点的那一组"
        );
    }

    /// 同一段历史的对照：同一个错落在核心层把切换交给实现之前（请求还挂着）——照那一次请求当场解除。
    #[test]
    fn the_same_error_before_the_handing_in_is_lifted_by_the_switch_request() {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        assert_eq!(
            background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                failure_on_the_card_at_one(),
            ),
            BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError
        );
    }

    /// 第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」：挂载那一刻照挂载那一组一张卡都用不上、停着；
    /// 请求切到没开的「0000:41:00.0」（装得下）——请求那一刻解除；交给实现那一批开头实现还在开卡，不读切换之前那一组的「用不上」，不再停；
    /// 卡开成、换上之后照它判，用得上，照常开批。
    #[test]
    fn a_mount_stopped_under_its_settings_is_not_stopped_again_while_the_switched_card_opens_nor_once_it_is_put_in_effect(
    ) {
        let state = state_with_the_card_at_one_opened(
            None,
            Some(every_gpu_left_out_on_the_card(b"0000:01:00.0")),
        );
        let mut background_compute = mount_on_the_card_at_one(&state);
        assert!(
            background_compute
                .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
            "挂载那一组一张卡都用不上：挂载就停着"
        );
        state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_once_put_in_effect = None;
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(
            batch_while_opening.is_ok(),
            "交给实现那一刻实现还在开卡：不读切换之前那一组的用不上、不再停"
        );
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(batch_once_opened.is_ok(), "换上之后用得上：照常开批");
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
    }

    /// 第 4 条「挂着切换到参与的卡一张都不剩的设置……报的是挂载那一刻同一个成员（GpuUnusableUnderTheSettingsHandedIn）」用于点没开的卡：
    /// 请求切到没开的「0000:41:00.0」、给它装不下最小的一片的上限——开卡的这段时间照旧开批；卡开成、实现答换上了而一张卡都用不上，
    /// 那一批开头按这个成员停着、不开批。
    #[test]
    fn the_switch_put_in_effect_after_the_opening_with_every_card_left_out_stops_as_unusable_and_begins_no_batch(
    ) {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_once_put_in_effect = Some(every_gpu_left_out_on_the_card(b"0000:41:00.0"));
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", Some(64)),
                None,
            )
            .expect("上限不超过显存总量：照收");
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(
            batch_while_opening.is_ok(),
            "开卡的这段时间照切换之前那一组开批"
        );
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(
            batch_once_opened.is_err(),
            "换上的那一组一张卡都用不上：不开批"
        );
        assert_eq!(
            background_compute
                .stopped_after_a_compute_error()
                .iter()
                .map(|stopped| stopped.error.clone())
                .collect::<Vec<_>>(),
            vec![
                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                    failure: every_gpu_left_out_on_the_card(b"0000:41:00.0"),
                }
            ],
            "报的是挂载那一刻同一个成员"
        );
    }

    /// 对照：同样一张卡都用不上的设置，点的是挂载那一刻开着的「0000:01:00.0」——交给实现那一刻就换上、当场按同一个成员停着、不开批。
    #[test]
    fn the_switch_to_every_card_left_out_on_a_card_already_open_stops_at_the_handing_in() {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_once_put_in_effect = Some(every_gpu_left_out_on_the_card(b"0000:01:00.0"));
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:01:00.0", Some(64)),
                None,
            )
            .expect("上限不超过显存总量：照收");
        let batch_at_the_handing_in = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(batch_at_the_handing_in.is_err());
        assert!(background_compute
            .stopped_after_a_compute_error()
            .iter()
            .any(|stopped| matches!(
                stopped.error,
                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { .. }
            )));
    }

    /// 第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」：挂载点「0000:01:00.0」，切到 `cpu`——那一批开头告诉实现一张卡都不参与（替身关掉开着的卡），那一批在 CPU 上算。
    #[test]
    fn the_switch_to_the_cpu_tells_the_implementation_that_no_gpu_takes_part() {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
            .expect("选 cpu 照收");
        let batch_on_the_cpu = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_on_the_cpu),
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
            ))
        );
        let state_after_the_switch = state.lock().expect("用例没在别处 panic 过");
        assert_eq!(state_after_the_switch.times_told_no_gpu_takes_part, 1);
        assert!(
            state_after_the_switch.cards_opened.is_empty(),
            "开着的卡全部关掉"
        );
    }

    /// 切到 `cpu`、开着的卡全部关掉之后切回 `gpu`：点的卡都当没开的卡开——开卡的这段时间生效的仍是 `cpu`，那一批在 CPU 上算；
    /// 开成、实现答换上了之后，下一批起在 GPU 上算。
    #[test]
    fn the_switch_back_to_the_gpu_runs_on_the_cpu_while_the_cards_open_and_on_the_gpu_once_put_in_effect(
    ) {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
            .expect("选 cpu 照收");
        let _batch_on_the_cpu = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:01:00.0", None),
                None,
            )
            .expect("挂载时递进来的实现还在：切回 gpu 照收");
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_while_opening),
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
            )),
            "开卡的这段时间生效的仍是 cpu：在 CPU 上算"
        );
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_once_opened),
            Some(BackgroundComputeRoute::Gpu),
            "开成之后下一批起在 GPU 上算"
        );
    }

    // ── 实现照用例逐批摆的答法（两批之间那一问答什么由用例定），钉核心层在开不成、开卡期间出错、重复键三处的结局 ──

    /// 替身的脚本：两批之间那一问答 `answer`；告知一张卡都不参与、被问那一问各记一次。
    struct ScriptOfTheImplementation {
        answer: LatestGpuSettingsHandedIn,
        times_told_no_gpu_takes_part: u32,
        times_asked_between_two_batches: u32,
    }

    /// 卡清单同 [`GpuComputeOpeningCardsWhenTheTestSays`]（「0000:01:00.0」「0000:41:00.0」），三段交回空，两批之间那一问照脚本答。
    struct GpuComputeAnsweringAsTheScriptSays {
        script: Arc<Mutex<ScriptOfTheImplementation>>,
    }

    impl GpuInventory for GpuComputeAnsweringAsTheScriptSays {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            vec![
                (
                    card_for_the_switch_tests(b"0000:01:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
                (
                    card_for_the_switch_tests(b"0000:41:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
            ]
        }
    }

    impl BackgroundGpuCompute for GpuComputeAnsweringAsTheScriptSays {
        fn verify_unit_checksums(
            &mut self,
            _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
        {
            Ok(Vec::new())
        }

        fn classify_region_records(
            &mut self,
            _region: CompactionRegion,
            _records: &[RegionRecordToClassifyOnTheGpu<'_>],
            _mapping_entries: &[CentralMappingEntry],
        ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn count_live_slots_per_cluster_segment(
            &mut self,
            _allocation_records: &[AllocationRecord],
            _unit_area: UnitAreaOfOneDevice,
            _segments: &[ClusterSegmentEntryKey],
        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}

        fn use_no_gpu_from_the_next_batch(&mut self) {
            self.script
                .lock()
                .expect("用例没在别处 panic 过")
                .times_told_no_gpu_takes_part += 1;
        }

        fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
            &mut self,
        ) -> LatestGpuSettingsHandedIn {
            let mut script = self.script.lock().expect("用例没在别处 panic 过");
            script.times_asked_between_two_batches += 1;
            script.answer.clone()
        }
    }

    fn script_answering(
        answer: LatestGpuSettingsHandedIn,
    ) -> Arc<Mutex<ScriptOfTheImplementation>> {
        Arc::new(Mutex::new(ScriptOfTheImplementation {
            answer,
            times_told_no_gpu_takes_part: 0,
            times_asked_between_two_batches: 0,
        }))
    }

    fn answer_from_now_on(
        script: &Arc<Mutex<ScriptOfTheImplementation>>,
        answer: LatestGpuSettingsHandedIn,
    ) {
        script.lock().expect("用例没在别处 panic 过").answer = answer;
    }

    fn put_in_effect_and_usable() -> LatestGpuSettingsHandedIn {
        LatestGpuSettingsHandedIn::PutInEffect {
            failure_under_the_settings: None,
        }
    }

    /// 选 `gpu`、点「0000:01:00.0」的一次挂载，替身照 `script` 答。
    fn mount_on_the_card_at_one_answering_as_the_script_says(
        script: &Arc<Mutex<ScriptOfTheImplementation>>,
    ) -> BackgroundComputeOfThisMount {
        let handle =
            BackgroundGpuComputeHandle::of_an_implementation(GpuComputeAnsweringAsTheScriptSays {
                script: Arc::clone(script),
            });
        BackgroundComputeOfThisMount::of_a_mount(
            admit_background_compute_settings_with_the_gpu_compute(
                gpu_settings_naming_the_card(b"0000:01:00.0", None),
                Some(handle),
            )
            .expect("卡清单列着「0000:01:00.0」"),
        )
    }

    fn not_opened_on_the_card_at_forty_one() -> BackgroundGpuComputeFailed {
        BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
            device: card_for_the_switch_tests(b"0000:41:00.0"),
            failures_of_each_opening: Vec::new(),
            pci_state_when_judged_unusable: Box::new(
                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
            ),
        }
    }

    /// 池子最满的那一块盘 95%，过了默认 85% 的补跑阈值：停着就在 CPU 上补跑。
    fn fullest_device_above_the_default_threshold() -> AllocatedBytesAndCapacityOfADevice {
        AllocatedBytesAndCapacityOfADevice {
            device: DeviceIdentity(0),
            allocated_bytes: 95,
            capacity_bytes: 100,
        }
    }

    fn route_of_the_next_batch(
        background_compute: &mut BackgroundComputeOfThisMount,
        fullest_device: AllocatedBytesAndCapacityOfADevice,
    ) -> Option<BackgroundComputeRoute> {
        route_of_the_batch(
            &background_compute.begin_a_batch(BackgroundTask::CompactionRelocation, fullest_device),
        )
    }

    /// 一段历史在开不成之后的读数：开不成那一批走哪条路、开不成之后生效的那一组点哪张卡、再请求同一组之后那一批走哪条路。
    struct ReadingsOfTheRetryAfterNotOpened {
        times_told_no_gpu_takes_part: u32,
        route_once_not_opened: Option<BackgroundComputeRoute>,
        devices_in_effect_once_not_opened: Option<Vec<GpuDeviceIdentifier>>,
        route_at_the_retry: Option<BackgroundComputeRoute>,
        handed_in_at_the_retry: bool,
    }

    /// 挂载点「0000:01:00.0」；`switch_to_the_cpu_first` 时先切到 `cpu`；请求切到没开的「0000:41:00.0」，实现答还在开卡、之后答开不成；
    /// 再请求同一组（停着时的切换：当场解除），实现答还在开卡。
    fn readings_of_the_retry_after_not_opened(
        switch_to_the_cpu_first: bool,
    ) -> ReadingsOfTheRetryAfterNotOpened {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        if switch_to_the_cpu_first {
            background_compute
                .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
                .expect("选 cpu 照收");
            let _route_on_the_cpu = route_of_the_next_batch(
                &mut background_compute,
                fullest_device_far_below_the_threshold(),
            );
        }
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let _route_while_opening = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: not_opened_on_the_card_at_forty_one(),
            },
        );
        let route_once_not_opened = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let devices_in_effect_once_not_opened = background_compute
            .in_effect()
            .settings()
            .background_gpu_devices
            .clone();
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let route_at_the_retry = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let times_told_no_gpu_takes_part = script
            .lock()
            .expect("用例没在别处 panic 过")
            .times_told_no_gpu_takes_part;
        ReadingsOfTheRetryAfterNotOpened {
            times_told_no_gpu_takes_part,
            route_once_not_opened,
            devices_in_effect_once_not_opened,
            route_at_the_retry,
            handed_in_at_the_retry: background_compute
                .handed_in_while_the_implementation_opens_its_cards()
                .is_some(),
        }
    }

    /// 实现答开不成时核心层不把那一组记成生效：先切到 `cpu`、切回的卡开不成、再请求同一组——开不成那一批停着、生效的仍是 `cpu`，
    /// 再请求之后实现还在开卡，那一批在 CPU 上算（不交给一张卡都没开的实现）。
    #[test]
    fn a_switch_not_opened_after_the_cpu_leaves_the_cpu_in_effect_and_the_retry_runs_on_the_cpu() {
        let readings = readings_of_the_retry_after_not_opened(true);
        assert_eq!(readings.times_told_no_gpu_takes_part, 1);
        assert_eq!(readings.route_once_not_opened, None, "开不成：停着、不开批");
        assert_eq!(
            readings.devices_in_effect_once_not_opened, None,
            "开不成的那一组不生效：生效的仍是 cpu"
        );
        assert_eq!(
            readings.route_at_the_retry,
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
            )),
            "再请求之后实现还在开卡：照生效的 cpu 算"
        );
        assert!(readings.handed_in_at_the_retry);
    }

    /// 同一段历史不先切到 `cpu`：开不成之后核心层生效的仍是切换之前那张「0000:01:00.0」——与实现照旧参与的是同一组；再请求之后那一批照它在 GPU 上算。
    #[test]
    fn a_switch_not_opened_on_the_gpu_leaves_the_card_before_the_switch_in_effect() {
        let readings = readings_of_the_retry_after_not_opened(false);
        assert_eq!(readings.times_told_no_gpu_takes_part, 0);
        assert_eq!(readings.route_once_not_opened, None, "开不成：停着、不开批");
        assert_eq!(
            readings.devices_in_effect_once_not_opened,
            Some(vec![card_for_the_switch_tests(b"0000:01:00.0")]),
            "开不成的那一组不生效：生效的仍是切换之前那一组"
        );
        assert_eq!(
            readings.route_at_the_retry,
            Some(BackgroundComputeRoute::Gpu)
        );
        assert!(readings.handed_in_at_the_retry);
    }

    /// 实现答开不成之后，补跑那一判照那一次切换请求给的阈值：挂载取默认 85%，请求切到「0000:41:00.0」给 50%，开不成——
    /// 池子最满的那一块盘 60%，下一批在 CPU 上补跑。
    #[test]
    fn the_gc_cpu_compensation_threshold_of_a_switch_not_opened_is_the_one_the_request_gave() {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
                BackgroundGcCpuCompensationThreshold {
                    allocated_percent_of_the_capacity: 50,
                },
            )
            .expect("卡清单列着「0000:41:00.0」、阈值不超过 100");
        answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: not_opened_on_the_card_at_forty_one(),
            },
        );
        let fullest_device_at_sixty_percent = AllocatedBytesAndCapacityOfADevice {
            device: DeviceIdentity(0),
            allocated_bytes: 60,
            capacity_bytes: 100,
        };
        let route =
            route_of_the_next_batch(&mut background_compute, fullest_device_at_sixty_percent);
        assert!(
            matches!(
                route,
                Some(BackgroundComputeRoute::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                ))
            ),
            "照请求给的 50% 判：60% 超过，补跑：{route:?}"
        );
    }

    /// 开不成那一组请求给的阈值只管到生效的那一组换掉为止：请求切到「0000:41:00.0」给 50%、开不成；再请求切到 `cpu` 给 90%，它交给实现那一刻生效——
    /// 之后补跑那一判照 90%，不再照开不成那一组留下的 50%。
    #[test]
    fn the_gc_cpu_compensation_threshold_left_by_a_switch_not_opened_is_dropped_once_another_switch_is_put_in_effect(
    ) {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
                BackgroundGcCpuCompensationThreshold {
                    allocated_percent_of_the_capacity: 50,
                },
            )
            .expect("卡清单列着「0000:41:00.0」、阈值不超过 100");
        answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: not_opened_on_the_card_at_forty_one(),
            },
        );
        let _route_once_not_opened = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        background_compute
            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                BackgroundComputeSettings::PRODUCT,
                None,
                BackgroundGcCpuCompensationThreshold {
                    allocated_percent_of_the_capacity: 90,
                },
            )
            .expect("选 cpu、阈值不超过 100：照收");
        let _route_on_the_cpu = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            background_compute
                .gc_cpu_compensation_reading(fullest_device_far_below_the_threshold())
                .threshold,
            BackgroundGcCpuCompensationThreshold {
                allocated_percent_of_the_capacity: 90,
            },
            "此刻生效的是 cpu 那一组：照它的 90%"
        );
    }

    /// 交给了实现、实现还在开卡的期间每一批都在切换之前那张卡上出错，池子最满的那一块盘过了补跑阈值：头一批出错停下，之后每次叫起报警、
    /// 每一批在 CPU 上补跑；实现答换上了的那一刻解除，那一批在 GPU 上算。
    #[test]
    fn batches_failing_while_the_switched_card_opens_stop_alarm_and_compensate_until_it_is_put_in_effect(
    ) {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        background_compute.begin_a_wake();
        let route_of_the_failing_batch = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        let _alarm_of_the_failing_wake =
            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
        background_compute.begin_a_wake();
        let route_while_stopped = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        let alarm_while_stopped =
            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
        assert_eq!(
            route_of_the_failing_batch,
            Some(BackgroundComputeRoute::Gpu)
        );
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards
        );
        assert!(
            matches!(
                route_while_stopped,
                Some(BackgroundComputeRoute::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                ))
            ),
            "停着、过了阈值：在 CPU 上补跑：{route_while_stopped:?}"
        );
        assert_eq!(
            alarm_while_stopped
                .map(|alarm| (alarm.wakes_while_stopped, alarm.error_that_stopped_the_task)),
            Some((1, failure_on_the_card_at_one())),
            "停下之后的下一次叫起报警"
        );
        answer_from_now_on(&script, put_in_effect_and_usable());
        let route_once_put_in_effect = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        assert_eq!(
            route_once_put_in_effect,
            Some(BackgroundComputeRoute::Gpu),
            "实现答换上了：解除，照新的那一组在 GPU 上算"
        );
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
    }

    /// 开卡期间出错停着、之后实现答开不成：照开不成停着，报的是开不成那一个错。
    #[test]
    fn a_batch_failing_while_the_switched_card_opens_stays_stopped_by_the_not_opened_failure_once_not_opened(
    ) {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let _route_of_the_failing_batch = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let _after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: not_opened_on_the_card_at_forty_one(),
            },
        );
        let route_once_not_opened = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(route_once_not_opened, None, "照开不成停着");
        assert_eq!(
            background_compute
                .stopped_after_a_compute_error()
                .iter()
                .map(|stopped| stopped.error.clone())
                .collect::<Vec<_>>(),
            vec![
                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                    failure: not_opened_on_the_card_at_forty_one(),
                }
            ]
        );
    }

    /// 对照：同一个错、同样过了补跑阈值，没有切换——停下（`LeftStopped`），下一批在 CPU 上补跑。
    #[test]
    fn the_same_error_without_a_switch_is_left_stopped_and_compensates_on_the_cpu() {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        let _route_of_the_failing_batch = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        let route_of_the_next_batch_after_the_error = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        assert!(matches!(
            route_of_the_next_batch_after_the_error,
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
            ))
        ));
    }

    /// 一批里同一个条目键出现两次只放弃这一批：那个后台任务不停、叫起不报警，下一批照常在 GPU 上算。
    #[test]
    fn the_same_entry_key_twice_in_one_batch_abandons_only_the_batch_without_stopping_or_alarming()
    {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute.begin_a_wake();
        let _route_of_the_abandoned_batch = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            BackgroundComputeError::GpuSegmentFailed {
                segment: GpuComputedSegment::LiveSlotsPerClusterSegment,
                failure: BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
                    position_of_the_first_occurrence: 1,
                    position_of_the_second_occurrence: 3,
                },
            },
        );
        let alarm =
            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
        let route_of_the_next_batch_after_the_error = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned
        );
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
        assert_eq!(alarm, None, "不报警");
        assert_eq!(
            route_of_the_next_batch_after_the_error,
            Some(BackgroundComputeRoute::Gpu),
            "下一批照常算"
        );
    }

    /// 交给了实现、还在开卡的那一组被一次 `cpu` 的切换盖掉：交 `cpu` 那一刻作废还在开卡的那一格，之后不再为它问实现。
    #[test]
    fn a_switch_to_the_cpu_drops_the_settings_whose_cards_are_still_opening() {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let _route_while_opening = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        background_compute
            .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
            .expect("选 cpu 照收");
        let route_on_the_cpu = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_on_the_cpu,
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
            ))
        );
        assert!(
            background_compute
                .handed_in_while_the_implementation_opens_its_cards()
                .is_none(),
            "交 cpu 那一刻作废还在开卡的那一组"
        );
    }

    /// 开卡期间出错停着时，补跑那一判照交给了实现、还在开卡的那一组请求给的阈值：挂载取默认 85%，请求切到「0000:41:00.0」给 50%，开卡期间出错——
    /// 池子最满的那一块盘 60%，下一批在 CPU 上补跑。
    #[test]
    fn the_gc_cpu_compensation_threshold_while_the_switch_opens_its_cards_is_the_one_the_request_gave(
    ) {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
                BackgroundGcCpuCompensationThreshold {
                    allocated_percent_of_the_capacity: 50,
                },
            )
            .expect("卡清单列着「0000:41:00.0」、阈值不超过 100");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let fullest_device_at_sixty_percent = AllocatedBytesAndCapacityOfADevice {
            device: DeviceIdentity(0),
            allocated_bytes: 60,
            capacity_bytes: 100,
        };
        let _route_of_the_failing_batch =
            route_of_the_next_batch(&mut background_compute, fullest_device_at_sixty_percent);
        let _after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        let route =
            route_of_the_next_batch(&mut background_compute, fullest_device_at_sixty_percent);
        assert!(
            matches!(
                route,
                Some(BackgroundComputeRoute::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                ))
            ),
            "照还在开卡的那一组给的 50% 判：60% 超过，补跑：{route:?}"
        );
    }

    /// 没有交给了实现、还没生效的那一组时，一批开头照样问一次生效那一组带着的实现（让它收下为更早几次交设置开完的卡），答案不用：
    /// 挂载那一刻问 1 次，下一批开头再问 1 次。
    #[test]
    fn a_batch_begun_with_no_settings_waiting_still_asks_the_implementation_once() {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        let asked_at_the_mount = script
            .lock()
            .expect("用例没在别处 panic 过")
            .times_asked_between_two_batches;
        let _route = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            (
                asked_at_the_mount,
                script
                    .lock()
                    .expect("用例没在别处 panic 过")
                    .times_asked_between_two_batches
            ),
            (1, 2)
        );
    }

    /// 第 4 条「停着之后每次叫起交回的报警带停下那一刻读的同一份读数，不重读」：一批因「0000:01:00.0」重交用完停下（错里带判它用不了那一刻读的一份），
    /// 停下那一趟与之后三次叫起的报警都带那一份，与停下记录里那个错带的相同。
    // harness-test-granularity:one-scenario 同一份停着的状态上按次序叫起几次，钉的是每一次报警带的都是停下那一刻的同一份
    #[test]
    fn the_alarm_at_every_wake_while_stopped_carries_the_pci_state_read_when_the_gpu_was_judged_unusable(
    ) {
        let pci_state_read =
            PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::Present,
                PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 1 }),
            )));
        let error = BackgroundComputeError::GpuSegmentFailed {
            segment: GpuComputedSegment::UnitChecksumVerification,
            failure: BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
                device: card_for_the_switch_tests(b"0000:01:00.0"),
                resubmissions: 3,
                wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                last_submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
                pci_state_when_judged_unusable: Box::new(pci_state_read.clone()),
            },
        };
        let mut background_compute =
            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
        background_compute.begin_a_wake();
        assert_eq!(
            background_compute
                .stop_the_task_after_a_compute_error(BackgroundTask::CompactionRelocation, error),
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        let pci_states_of_the_alarms: Vec<Option<PciStateOfTheGpuReadWhenJudgedUnusable>> = (0..4)
            .map(|wake| {
                if wake > 0 {
                    background_compute.begin_a_wake();
                }
                background_compute
                    .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
                    .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable())
            })
            .collect();
        assert_eq!(
            pci_states_of_the_alarms,
            vec![Some(pci_state_read.clone()); 4],
            "停下那一趟与之后每次叫起的报警都带停下那一刻读的那一份"
        );
        assert_eq!(
            background_compute.stopped_after_a_compute_error()[0]
                .error
                .pci_state_when_judged_unusable(),
            pci_state_read,
            "停下记录里的那个错带的是同一份"
        );
    }

    /// 第 4 条：不是判一张卡用不了的那三种时刻、或不归哪张卡的错不读，报的是没读那个显式成员：只供测试的钩子注入的停下之后报警照报、读数那一格是没读；
    /// 一批里同一个条目键出现两次、一次等超过上限（还没用完重交）、参与的卡一张都不剩的错带的也是没读。
    #[test]
    fn errors_not_judging_a_gpu_unusable_carry_no_pci_state_read() {
        let mut background_compute =
            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
        assert_eq!(
            background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                BackgroundComputeError::InjectedByTheTestOnlyHook,
            ),
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        background_compute.begin_a_wake();
        assert_eq!(
            background_compute
                .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
                .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable()),
            Some(PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError)
        );
        let failures_not_judging_a_gpu_unusable = [
            BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
                position_of_the_first_occurrence: 0,
                position_of_the_second_occurrence: 1,
            },
            BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
                device: card_for_the_switch_tests(b"0000:01:00.0"),
                wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
            },
            every_gpu_left_out_on_the_card(b"0000:01:00.0"),
        ];
        assert_eq!(
            failures_not_judging_a_gpu_unusable
                .iter()
                .map(BackgroundGpuComputeFailed::pci_state_when_judged_unusable)
                .collect::<Vec<_>>(),
            vec![PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError; 3]
        );
    }

    /// 点「0000:01:00.0」的那一份替身，调用方自己也留着一份（用例在别的线程上借它、让它中毒）。
    fn shared_implementation_with_the_card_at_one_opened(
    ) -> Arc<Mutex<dyn BackgroundGpuCompute + Send>> {
        Arc::new(Mutex::new(GpuComputeOpeningCardsWhenTheTestSays {
            state: state_with_the_card_at_one_opened(None, None),
        }))
    }

    /// 切到 `gpu` 的校验照把手带的卡清单快照判，不取 GPU 实现那把锁：替身那把锁被另一条线程借着时（锁外算一批的样子），
    /// 挂着的切换校验立刻返回、照收，不等那条线程放锁。
    #[test]
    fn a_switch_to_the_gpu_is_admitted_without_waiting_for_the_gpu_implementation_lent_to_another_thread(
    ) {
        let shared = shared_implementation_with_the_card_at_one_opened();
        let mut background_compute = BackgroundComputeOfThisMount::of_a_mount(
            admit_background_compute_settings_with_the_gpu_compute(
                gpu_settings_naming_the_card(b"0000:01:00.0", None),
                Some(BackgroundGpuComputeHandle::of_a_shared_implementation(
                    Arc::clone(&shared),
                )),
            )
            .expect("卡清单列着「0000:01:00.0」"),
        );
        let (lent_sender, lent_receiver) = std::sync::mpsc::channel::<()>();
        let (release_sender, release_receiver) = std::sync::mpsc::channel::<()>();
        let lending_thread = std::thread::spawn({
            let shared = Arc::clone(&shared);
            move || {
                let _lent = shared.lock().expect("替身没在别处 panic 过");
                lent_sender.send(()).expect("用例在等借出");
                release_receiver.recv().expect("用例叫放锁");
            }
        });
        lent_receiver.recv().expect("另一条线程借着实现");
        let (switched_sender, switched_receiver) = std::sync::mpsc::channel::<bool>();
        let switching_thread = std::thread::spawn(move || {
            let is_admitted = background_compute
                .request_switch_from_the_next_batch(
                    gpu_settings_naming_the_card(b"0000:41:00.0", None),
                    None,
                )
                .is_ok();
            switched_sender
                .send(is_admitted)
                .expect("用例在等切换的结局");
            background_compute
        });
        let switched_while_lent = switched_receiver.recv_timeout(Duration::from_secs(5));
        release_sender.send(()).expect("借着的那条线程在等放锁");
        lending_thread.join().expect("借着的那条线程不 panic");
        let background_compute_after_the_switch =
            switching_thread.join().expect("切换那条线程不 panic");
        assert_eq!(
            switched_while_lent,
            Ok(true),
            "实现那把锁被借着时切换校验照快照判、立刻照收"
        );
        assert!(
            background_compute_after_the_switch
                .requested_from_the_next_batch()
                .is_some(),
            "切换记成待生效"
        );
    }

    /// GPU 实现上一次借出时 panic 过（那把锁中毒）：借出交回错误、不 panic；挂载那一刻把设置交给它时交不进去，有 GPU 路径的后台任务照这个错停着。
    #[test]
    fn a_gpu_implementation_poisoned_while_lent_is_reported_as_an_error_instead_of_panicking() {
        let shared = shared_implementation_with_the_card_at_one_opened();
        let handle = BackgroundGpuComputeHandle::of_a_shared_implementation(Arc::clone(&shared));
        let admitted = admit_background_compute_settings_with_the_gpu_compute(
            gpu_settings_naming_the_card(b"0000:01:00.0", None),
            Some(handle.clone()),
        )
        .expect("卡清单列着「0000:01:00.0」");
        let poisoning_thread = std::thread::spawn({
            let shared = Arc::clone(&shared);
            move || {
                let _lent = shared.lock().expect("替身还没中毒");
                panic!("替身：借着实现时 panic，让那把锁中毒");
            }
        });
        assert!(
            poisoning_thread.join().is_err(),
            "前提：借着的那条线程 panic 了"
        );
        assert!(
            matches!(handle.lock(), Err(GpuImplementationPanickedWhileLent)),
            "借出中毒的实现交回错误"
        );
        let background_compute = BackgroundComputeOfThisMount::of_a_mount(admitted);
        assert_eq!(
            background_compute.stopped_after_a_compute_error(),
            &[BackgroundTaskStoppedAfterAComputeError {
                task: BackgroundTask::CompactionRelocation,
                error: BackgroundComputeError::GpuImplementationPanickedWhileLent,
                batches_compensated_on_the_cpu: 0,
            }],
            "交设置时实现那把锁中毒：有 GPU 路径的后台任务照这个错停着"
        );
    }
}
