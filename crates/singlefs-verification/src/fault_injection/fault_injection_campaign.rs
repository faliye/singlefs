//! 故障注入的整趟：工作线程数从哪来，种子区间切成片分给工作线程跑，按片的次序并计数、出报告。

use super::*;

/// 工作线程数是从哪来的：与实际起的线程数一起打进进度行，「机器多于 1 核却只用了 1 个线程」看得出来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultInjectionWorkerThreads {
    /// 环境变量 [`FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    FromTheEnvironmentVariable(usize),
    /// 没设环境变量，取每台默认重活线程数（[`crate::heavy_threads::default_heavy_threads`]）。
    FromTheHeavyThreadDefault(usize),
    /// 调用方在代码里直接给的（用例拿不同线程数对拍）。
    GivenByCaller(usize),
}

impl FaultInjectionWorkerThreads {
    /// 线程数取环境变量，没设就取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取登记值）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数同样停。
    #[must_use]
    pub fn from_the_environment() -> Self {
        Self::from_the_environment_value(
            std::env::var(FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE),
            process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        )
    }

    /// [`Self::from_the_environment`] 的判定本身：环境变量读到什么、`SINGLEFS_THREAD_CAP` 的值（`None` 是没设）都由调用方给
    /// （用例不改进程的环境变量）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串；解析经数值环境变量共用的那一份，消息带变量名与原值），或不是 UTF-8；
    /// 落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
    #[must_use]
    fn from_the_environment_value(
        variable: Result<String, std::env::VarError>,
        thread_cap_value: Option<&str>,
    ) -> Self {
        let value = match variable {
            Ok(text) => Some(text),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE} 不是 UTF-8：{raw:?}")
            }
        };
        match number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
            FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
            value.as_deref(),
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

/// 一次故障注入跑什么：种子区间、每段几步、每段注入几次、比重、历史怎么跑、线程数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaultInjectionCampaign {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub faults_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub worker_threads: FaultInjectionWorkerThreads,
}

/// 一批种子跑下来的故障注入报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultInjectionReport {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub faults_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub tally: FaultInjectionTally,
    pub known_red_hits: Vec<KnownRedAtAFault>,
    pub new_findings: Vec<FaultFinding>,
}

impl FaultInjectionReport {
    /// 给人看的一整块：跑了什么、计数、「已知红」命中、新发现。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!(
            "故障注入：种子 [{}, {})（种子基是这个测试周期写死的那一个）、每段 {} 步、每段注入 {} 次、比重 {}、{}\n",
            self.first_seed,
            self.first_seed + self.seed_count,
            self.operations_per_history,
            self.faults_per_history,
            self.weights.name,
            self.execution.name()
        );
        text.push_str(&self.tally.render());
        let mut known_red_by_form: BTreeMap<KnownRedFormAtAFault, Vec<&KnownRedAtAFault>> =
            BTreeMap::new();
        for hit in &self.known_red_hits {
            known_red_by_form.entry(hit.form).or_default().push(hit);
        }
        for (form, hits) in &known_red_by_form {
            let first = hits.first().expect("这一条至少有一次命中");
            let _ = writeln!(
                text,
                "「已知红」{}：{} 次，第一次在种子 {}（{}）",
                form.render(),
                hits.len(),
                first.seed.0,
                first.drawn.render()
            );
        }
        for finding in &self.new_findings {
            text.push_str(&finding.render());
        }
        text
    }
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，
/// 线程按片号从小到大领片，调用线程收到一片就打一行 `FAULT_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），
/// 再按片号从小到大并计数。计数按片的次序相加、新发现按种子从小到大留第一个，所以
/// [`FaultInjectionReport::render`] 与线程数、调度次序无关（进度行是按到达次序打的，只报跑到哪了，不带判定）。
///
/// # Panics
/// 有一片领了却没交回。
#[must_use]
pub fn run_fault_injection_campaign(campaign: &FaultInjectionCampaign) -> FaultInjectionReport {
    run_fault_injection_campaign_handing_out_the_images(campaign, &|_| {})
}

/// 同 [`run_fault_injection_campaign`]，另把每个注入点重开的盘交给 `hand_out`（[`inject_faults_into_history_handing_out_the_images`]）：
/// 工作线程各自调它，交出去的次序按线程领片的次序、不按种子（调用方要按种子归并就照 [`ImagesAfterAFault::seed`] 与注入点排）。
///
/// # Panics
/// 同 [`run_fault_injection_campaign`]。
#[must_use]
pub fn run_fault_injection_campaign_handing_out_the_images(
    campaign: &FaultInjectionCampaign,
    hand_out: &(dyn Fn(ImagesAfterAFault) + Sync),
) -> FaultInjectionReport {
    let FaultInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        faults_per_history,
        weights,
        execution,
        worker_threads,
    } = *campaign;
    let slices = seed_slices(seed_count, worker_threads.count());
    let spawned_worker_threads = worker_threads.count().min(slices.len().max(1));
    let started = Instant::now();
    println!(
        "FAULT_INJECTION_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={}",
        first_seed.wrapping_add(seed_count),
        slices.len(),
        worker_threads.count(),
        worker_threads.source_name()
    );
    let next_slice_index = AtomicUsize::new(0);
    let (finished_slices, merged_slice_count) = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<HistoryFaultInjection>)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            scope.spawn(move || loop {
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let injections: Vec<HistoryFaultInjection> = slice
                    .clone()
                    .map(|offset| {
                        let history = generate_history_with_weights(
                            HistorySeed(first_seed.wrapping_add(offset)),
                            operations_per_history,
                            &weights,
                        );
                        inject_faults_into_history_handing_out_the_images(
                            &history,
                            execution,
                            faults_per_history,
                            &mut |images| hand_out(images),
                        )
                    })
                    .collect();
                if sender.send((slice_index, injections)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut waiting_for_earlier_slices: BTreeMap<usize, Vec<HistoryFaultInjection>> =
            BTreeMap::new();
        let mut in_order: Vec<HistoryFaultInjection> = Vec::new();
        let mut next_slice_to_merge = 0usize;
        let mut finished_histories = 0u64;
        for (finished_slice_count, (slice_index, injections)) in receiver.iter().enumerate() {
            let slice = &slices[slice_index];
            finished_histories += slice.end - slice.start;
            println!(
                "FAULT_INJECTION_PROGRESS slice={}/{} seeds=[{},{}) finished_slices={}/{} finished_histories={finished_histories}/{seed_count} elapsed_seconds={:.1}",
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
    assert_eq!(
        merged_slice_count,
        slices.len(),
        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "FAULT_INJECTION_FINISHED seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} elapsed_seconds={:.1}",
        first_seed.wrapping_add(seed_count),
        slices.len(),
        started.elapsed().as_secs_f64()
    );
    let mut tally = FaultInjectionTally::default();
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<FaultFinding> = Vec::new();
    let mut signatures_seen: BTreeSet<FailureSignature> = BTreeSet::new();
    for injection in &finished_slices {
        tally.absorb(&injection.tally);
        known_red_hits.extend(injection.known_red_hits.iter().cloned());
        for finding in &injection.new_findings {
            if signatures_seen.insert(finding.signature.clone()) {
                new_findings.push(finding.clone());
            }
        }
    }
    FaultInjectionReport {
        first_seed,
        seed_count,
        operations_per_history,
        faults_per_history,
        weights,
        execution,
        tally,
        known_red_hits,
        new_findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 线程数从环境变量取，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取登记值，不看本机核数）；
    /// 设了不是正整数就停，不悄悄退回单线程。
    #[test]
    fn fault_injection_worker_thread_count_comes_from_the_environment_variable_or_the_heavy_thread_default(
    ) {
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(Ok("3".to_string()), Some("5")),
            FaultInjectionWorkerThreads::FromTheEnvironmentVariable(3),
            "自己的变量先读"
        );
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                Some("5")
            ),
            FaultInjectionWorkerThreads::FromTheHeavyThreadDefault(5),
            "没设自己的变量读 SINGLEFS_THREAD_CAP"
        );
        let registered = crate::heavy_threads::registered_heavy_threads().get();
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                None
            ),
            FaultInjectionWorkerThreads::FromTheHeavyThreadDefault(registered),
            "都没设取 .claude/heavy-thread-default 的登记值"
        );
        assert_eq!(
            FaultInjectionWorkerThreads::FromTheHeavyThreadDefault(registered).source_name(),
            "heavy_thread_default"
        );
    }

    /// 线程数变量写成认不出的字样：经数值环境变量共用的那一份解析 panic，消息带变量名与原值，不悄悄退回默认。
    #[test]
    #[should_panic(expected = "SINGLEFS_FAULT_INJECTION_WORKER_THREADS=\"twelve\" 认不出")]
    fn an_unreadable_fault_injection_worker_thread_count_panics_naming_the_variable_and_the_value()
    {
        let _threads =
            FaultInjectionWorkerThreads::from_the_environment_value(Ok("twelve".to_string()), None);
    }
}
