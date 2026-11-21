//! 把失败的历史收缩到最小的复现，跑一批历史出战役报告。

use super::*;

/// 收缩时回退目标与抬 F 目标试的小选择子个数：根环至多 24 条可读根，F 的可选个数是现行 txg − F + 3，默认批的一段历史里不过几十。
const SMALL_SELECTORS_TRIED_WHEN_SHRINKING: u64 = 32;

/// 一步操作的几种更简单的写法，只往简单的方向换（换过去的不会再换回来，收缩因此有界）：内容按 0 字节 < 1 字节 < 别的排，只试排在
/// 现在这个前面的；回退目标与抬 F 目标只试比现在小的选择子（选择子执行时按条数取模：一个很大的选择子与某个小数落到同一条根、同一个 F，
/// 从小到大试，第一个还失败的就是它的小写法）。
fn simpler_variants(operation: HistoryOperation) -> Vec<HistoryOperation> {
    const EMPTY_CONTENT: ContentChoice = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    const SHORTEST_NON_EMPTY_CONTENT: ContentChoice = ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 0 },
        fill_seed: 0,
    };
    let simplicity_rank = |content: ContentChoice| {
        if content == EMPTY_CONTENT {
            0
        } else if content == SHORTEST_NON_EMPTY_CONTENT {
            1
        } else {
            2
        }
    };
    let simpler_contents = move |content: ContentChoice| {
        [EMPTY_CONTENT, SHORTEST_NON_EMPTY_CONTENT]
            .into_iter()
            .filter(move |simpler| simplicity_rank(*simpler) < simplicity_rank(content))
    };
    let smaller_selectors = |selector: u64| 0..selector.min(SMALL_SELECTORS_TRIED_WHEN_SHRINKING);
    match operation {
        HistoryOperation::PublishFirstFile(content) => simpler_contents(content)
            .map(HistoryOperation::PublishFirstFile)
            .collect(),
        HistoryOperation::PublishOverwrite(content) => simpler_contents(content)
            .map(HistoryOperation::PublishOverwrite)
            .collect(),
        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(content) => {
            simpler_contents(content)
                .map(HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable)
                .collect()
        }
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
            index_from_newest,
        }) => smaller_selectors(index_from_newest)
            .map(|smaller| {
                HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                    index_from_newest: smaller,
                })
            })
            .collect(),
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::BeyondNewestRoot {
            txg_beyond_newest,
        }) => smaller_selectors(txg_beyond_newest)
            .map(|smaller| {
                HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::BeyondNewestRoot {
                    txg_beyond_newest: smaller,
                })
            })
            .collect(),
        HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
            steps_above_current_floor,
        }) => smaller_selectors(steps_above_current_floor)
            .map(|smaller| {
                HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                    steps_above_current_floor: smaller,
                })
            })
            .collect(),
        // 候选集的下沿按那一刻的盘面现解，没有更简单的写法。
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRootAtTheNewestFloor)
        | HistoryOperation::PublishWithoutUnits
        | HistoryOperation::CloseAndMountWritable
        | HistoryOperation::ColdStartRecover
        | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
        | HistoryOperation::Unlink
        | HistoryOperation::CreateInode
        | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
        | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros
        | HistoryOperation::CreateSnapshot => Vec::new(),
        // 建几个往 1 那一头换。
        HistoryOperation::CreateInodes(choice) => (1..choice.count_drawn.min(SMALL_SELECTORS_TRIED_WHEN_SHRINKING))
            .map(|fewer| HistoryOperation::CreateInodes(InodeCountChoice { count_drawn: fewer }))
            .collect(),
        // 按 inode 号写：内容往简单那一头换，选择子往小里试（同回退目标）。
        HistoryOperation::WriteOfAnInode(choice) => simpler_contents(choice.content)
            .map(|simpler| WriteOfAnInodeChoice {
                inode_selector: choice.inode_selector,
                content: simpler,
            })
            .chain(
                smaller_selectors(choice.inode_selector).map(|smaller| WriteOfAnInodeChoice {
                    inode_selector: smaller,
                    content: choice.content,
                }),
            )
            .map(HistoryOperation::WriteOfAnInode)
            .collect(),
        // 按偏移写：格不换，号与偏移的选择子往小里试（同回退目标）。
        HistoryOperation::WriteAtOffset(choice) => smaller_selectors(choice.selector)
            .map(|smaller| PartialWriteChoice {
                inode_selector: choice.inode_selector,
                cell: choice.cell,
                selector: smaller,
                fill_seed: choice.fill_seed,
            })
            .chain(
                smaller_selectors(choice.inode_selector).map(|smaller| PartialWriteChoice {
                    inode_selector: smaller,
                    cell: choice.cell,
                    selector: choice.selector,
                    fill_seed: choice.fill_seed,
                }),
            )
            .map(HistoryOperation::WriteAtOffset)
            .collect(),
        HistoryOperation::ReadSnapshot(SnapshotChoice {
            index_among_the_snapshots,
        }) => smaller_selectors(index_among_the_snapshots)
            .map(|smaller| {
                HistoryOperation::ReadSnapshot(SnapshotChoice {
                    index_among_the_snapshots: smaller,
                })
            })
            .collect(),
        HistoryOperation::DestroySnapshot(SnapshotChoice {
            index_among_the_snapshots,
        }) => smaller_selectors(index_among_the_snapshots)
            .map(|smaller| {
                HistoryOperation::DestroySnapshot(SnapshotChoice {
                    index_among_the_snapshots: smaller,
                })
            })
            .collect(),
        // 选段的选择子执行时按候选段数取模，同回退目标那样从小到大试；一批的单元数往 1 那一头换。
        HistoryOperation::CompactWhileMounted(choice) => smaller_selectors(choice.region_selector)
            .map(|smaller| CompactionChoice {
                batch_units: choice.batch_units,
                region_selector: smaller,
                ending: choice.ending,
            })
            .chain((1..choice.batch_units).map(|fewer_units| CompactionChoice {
                batch_units: fewer_units,
                region_selector: choice.region_selector,
                ending: choice.ending,
            }))
            .map(HistoryOperation::CompactWhileMounted)
            .collect(),
        // 截断往两头换（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8「往 0、往当前 size 两个方向缩」）：缩到 0、
        // 改成现在的 size（只改记录）；换过去的这两类不再往别处换，收缩因此有界。选择子往小里试，同回退目标。
        HistoryOperation::Truncate(choice) => match choice.target {
            TruncationTarget::ToZero | TruncationTarget::ToTheCurrentSize => {
                smaller_selectors(choice.selector)
                    .map(|smaller| TruncationChoice {
                        target: choice.target,
                        selector: smaller,
                    })
                    .map(HistoryOperation::Truncate)
                    .collect()
            }
            TruncationTarget::ToADataUnitBoundary
            | TruncationTarget::IntoTheMiddleOfADataUnit
            | TruncationTarget::GrowingWithinTheLastDataUnit
            | TruncationTarget::GrowingAcrossDataUnits
            | TruncationTarget::BackIntoTheHole => [
                TruncationTarget::ToZero,
                TruncationTarget::ToTheCurrentSize,
            ]
            .into_iter()
            .map(|target| TruncationChoice {
                target,
                selector: choice.selector,
            })
            .chain(
                smaller_selectors(choice.selector).map(|smaller| TruncationChoice {
                    target: choice.target,
                    selector: smaller,
                }),
            )
            .map(HistoryOperation::Truncate)
            .collect(),
        },
    }
}

/// 收缩到最短：先按块删（块长从一半起、删不动就减半，到 1 为止），再逐步换成更简单的写法；`still_fails` 为真的删法与换法才留下。
/// 次数有界：每一轮要么少一步，要么块长减半，块长为 1 且一步都删不动就停。
///
/// 删法按窗口并行试：从当前块起连着 `worker_threads` 块各删一块、同时跑，取块号最小的那个还失败的——与一块一块按次序试的结果逐项相同
/// （按次序试时排在它前面的那几块在同一份序列上试、同样不失败），只是多跑了窗口里排在它后面的几个。
pub fn shrink_operations(
    operations: &[HistoryOperation],
    still_fails: &(dyn Fn(&[HistoryOperation]) -> bool + Sync),
    worker_threads: usize,
) -> Vec<HistoryOperation> {
    let window = worker_threads.max(1);
    let mut kept = operations.to_vec();
    let mut chunk_length = (kept.len() / 2).max(1);
    loop {
        let mut removed_this_pass = false;
        let mut chunk_start = 0;
        while chunk_start < kept.len() {
            let window_starts: Vec<usize> = (0..window)
                .map(|offset| chunk_start + offset * chunk_length)
                .take_while(|start| *start < kept.len())
                .collect();
            let candidates: Vec<Vec<HistoryOperation>> = window_starts
                .iter()
                .map(|start| {
                    let end = (start + chunk_length).min(kept.len());
                    kept[..*start].iter().chain(&kept[end..]).copied().collect()
                })
                .collect();
            let verdicts: Vec<bool> = std::thread::scope(|scope| {
                let evaluations: Vec<_> = candidates
                    .iter()
                    .map(|candidate| scope.spawn(move || still_fails(candidate)))
                    .collect();
                evaluations
                    .into_iter()
                    .map(|evaluation| {
                        evaluation.join().expect(
                            "still_fails 自己不 panic：历史里的 panic 在 execute_history_with 里接住",
                        )
                    })
                    .collect()
            });
            match verdicts.iter().position(|fails| *fails) {
                Some(first_failing) => {
                    kept = candidates[first_failing].clone();
                    chunk_start = window_starts[first_failing];
                    removed_this_pass = true;
                }
                None => {
                    chunk_start =
                        window_starts.last().expect("窗口里至少有当前这一块") + chunk_length;
                }
            }
        }
        if !removed_this_pass {
            if chunk_length == 1 {
                break;
            }
            chunk_length = (chunk_length / 2).max(1);
        }
    }
    for step_index in 0..kept.len() {
        for simpler in simpler_variants(kept[step_index]) {
            let mut candidate = kept.clone();
            candidate[step_index] = simpler;
            if still_fails(&candidate) {
                kept = candidate;
                break;
            }
        }
    }
    kept
}

/// 一段失败的历史收缩到最短：起点不变，签名不变的删法与换法才留下。先截掉失败那一步之后的操作。每一次重跑的 checker 与盘宽与发现它的
/// 那一次相同（`execution`）。
#[must_use]
pub fn shrink_failing_history(
    history: &GeneratedHistory,
    signature: &FailureSignature,
    execution: HistoryExecution,
    worker_threads: usize,
) -> GeneratedHistory {
    let run = |candidate: &GeneratedHistory| {
        execute_history_with(candidate, execution, &SharedStream::new(), &mut |_| {})
    };
    let still_fails = |operations: &[HistoryOperation]| {
        let candidate = GeneratedHistory {
            seed: history.seed,
            starting_point: history.starting_point,
            operations: operations.to_vec(),
        };
        matches!(
            run(&candidate).ending,
            HistoryEnding::NewFinding { signature: found, .. } if found == *signature
        )
    };
    let failing_length = match run(history).ending {
        HistoryEnding::NewFinding { observation, .. } => match observation.position {
            StepPosition::StartingPoint => 0,
            StepPosition::Operation(step_index) => step_index + 1,
        },
        HistoryEnding::Completed | HistoryEnding::KnownRed { .. } => history.operations.len(),
    };
    let operations = shrink_operations(
        &history.operations[..failing_length],
        &still_fails,
        worker_threads,
    );
    GeneratedHistory {
        seed: history.seed,
        starting_point: history.starting_point,
        operations,
    }
}

/// 收缩之后的最短复现：那段历史与它每一步的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShrunkReproduction {
    pub history: GeneratedHistory,
    pub outcomes: Vec<StepOutcome>,
}

/// 收缩一段失败的历史，连同最短复现每一步的结局。
#[must_use]
pub fn shrink_to_reproduction(
    history: &GeneratedHistory,
    signature: &FailureSignature,
    execution: HistoryExecution,
    worker_threads: usize,
) -> ShrunkReproduction {
    let shrunk_history = shrink_failing_history(history, signature, execution, worker_threads);
    let outcomes = execute_history_with(
        &shrunk_history,
        execution,
        &SharedStream::new(),
        &mut |_| {},
    )
    .outcomes;
    ShrunkReproduction {
        history: shrunk_history,
        outcomes,
    }
}

/// 一类新发现：第一个撞到它的种子、同一个签名的全部种子、收缩之后的最短复现（没收缩的是 None）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewFindingReport {
    pub signature: FailureSignature,
    pub first_seed: HistorySeed,
    pub observation: FailureObservation,
    pub seeds: Vec<HistorySeed>,
    pub shrunk: Option<ShrunkReproduction>,
    /// 发现它的那一段的取样参数：复现行照它写（[`random_history_reproduction_line`]）。
    pub sampling: HistorySampling,
}

/// 新发现收不收缩：收缩一类要把那段历史反复重跑几十次，debug 下每次几秒（门禁 crates-mutation-replay 在变异下跑随机历史默认批时量到过：只收一类也要两分多钟）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindingShrinking {
    /// 每一类都收缩第一个种子（设 `SINGLEFS_RANDOM_HISTORY_SHRINK=every` 的那一批，release 下跑）。
    EveryFindingClass,
    /// 不收缩，只报种子与失败在哪一步（默认批：要的是判红与种子；收缩交给「收缩一个种子」那个 example 或设了收缩变量的那一批）。
    ReportSeedsOnly,
}

impl NewFindingReport {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "新发现 {:?}：第一个种子 {}（同签名的种子 {:?}），在 {:?}（{:?}）之后",
            self.signature,
            self.first_seed.0,
            self.seeds.iter().map(|seed| seed.0).collect::<Vec<_>>(),
            self.observation.position,
            self.observation.operation_kind
        );
        for (invariant, detail) in &self.observation.violations {
            let _ = writeln!(text, "  {invariant}：{detail}");
        }
        if let Some(panic) = &self.observation.panic {
            let _ = writeln!(text, "  panic 在 {}：{}", panic.location, panic.message);
        }
        if let Some(judgement) = &self.observation.harness_judgement {
            let _ = writeln!(text, "  执行器判出：{}：{judgement:?}", judgement.name());
        }
        if let Some(disagreement) = &self.observation.model_disagreement {
            let _ = writeln!(
                text,
                "  模型对不上（{}）：模型答 {}；实现 {}",
                disagreement.aspect.name(),
                disagreement.model_answer,
                disagreement.implementation_answer
            );
        }
        let _ = writeln!(
            text,
            "  复现：{}",
            random_history_reproduction_line(self.first_seed, &self.sampling)
        );
        let Some(shrunk) = &self.shrunk else {
            let _ = writeln!(
                text,
                "  没收缩（这一档只报种子）：照上一行跑「收缩一个种子」那条 #[ignore] 用例"
            );
            return text;
        };
        let _ = writeln!(
            text,
            "  最短复现：起点 {:?}，{} 步",
            shrunk.history.starting_point,
            shrunk.history.operations.len()
        );
        for (step_index, operation) in shrunk.history.operations.iter().enumerate() {
            let outcome = shrunk.outcomes.get(step_index).map_or_else(
                || "（失败在这一步）".to_string(),
                |outcome| format!("{outcome:?}"),
            );
            let _ = writeln!(text, "    {step_index}. {operation:?} → {outcome}");
        }
        text
    }
}

/// 一批种子跑下来的报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampaignReport {
    pub first_seed: u64,
    pub seed_count: u64,
    /// 这一批每段历史的取样参数（每段步数、比重、跑法）：历史就是照它生成、照它跑的，每个新发现的复现行也照它写。
    pub sampling: HistorySampling,
    pub tally: HistoryTally,
    /// 以「已知红」收尾的种子：(种子, 清单第几条, 在哪一步)。
    pub known_red_hits: Vec<(HistorySeed, usize, StepPosition)>,
    /// 按第一个种子从小到大排。
    pub new_findings: Vec<NewFindingReport>,
}

impl CampaignReport {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "种子 [{}, {})，每段 {} 步，比重：{}；{}",
            self.first_seed,
            self.first_seed + self.seed_count,
            self.sampling.operations_per_history,
            self.sampling.weights.name,
            self.sampling.execution.name()
        );
        text.push_str(&self.tally.render());
        for (form_index, form) in KNOWN_RED_FORMS.iter().enumerate() {
            let hits: Vec<(u64, StepPosition)> = self
                .known_red_hits
                .iter()
                .filter(|(_, hit_form, _)| *hit_form == form_index)
                .map(|(seed, _, position)| (seed.0, *position))
                .collect();
            let _ = writeln!(
                text,
                "已知红第 {form_index} 条（{}）：{} 段；前几个种子 {:?}",
                form.closeout_table_row,
                hits.len(),
                hits.iter().take(8).collect::<Vec<_>>()
            );
        }
        for finding in &self.new_findings {
            text.push_str(&finding.render());
        }
        text
    }
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，每段 `operations_per_history` 步、按 `execution` 跑（checker 与盘宽），分给
/// `worker_threads` 个线程（每段历史各用各的盘、各用各的录制流，互不相干）；跑完按种子排好再汇总，结论与线程数、调度次序无关。
/// 新发现按签名归类，按 `shrinking` 收缩各类的第一个种子。
///
/// # Panics
/// 某个线程在历史之外 panic（历史里的 panic 在 `execute_history_with` 里接住，走不到这里）。
#[must_use]
pub fn run_history_campaign(
    first_seed: u64,
    seed_count: u64,
    operations_per_history: usize,
    weights: &GenerationWeights,
    execution: HistoryExecution,
    worker_threads: usize,
    shrinking: FindingShrinking,
) -> CampaignReport {
    // 取样参数只在这里拼一次：历史照它生成、照它跑，报告与每个新发现的复现行也照它写——复现行写出的就是真跑的那一种
    // （C607：生成与复现行各拼一份时，复现行那一份写错了没有用例看得见）。
    let sampling = HistorySampling {
        operations_per_history,
        weights: *weights,
        execution,
    };
    let next_offset = AtomicU64::new(0);
    let finished: Mutex<Vec<(GeneratedHistory, HistoryRun)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..worker_threads.max(1) {
            scope.spawn(|| loop {
                let offset = next_offset.fetch_add(1, Ordering::Relaxed);
                if offset >= seed_count {
                    break;
                }
                let history = sampling.generate(HistorySeed(first_seed + offset));
                let run = execute_history_with(
                    &history,
                    sampling.execution,
                    &SharedStream::new(),
                    &mut |_| {},
                );
                finished
                    .lock()
                    .expect("别的线程拿着这把锁时只做一次 push，不会在锁里 panic")
                    .push((history, run));
            });
        }
    });
    let finished = finished
        .into_inner()
        .expect("线程都已结束；锁里只做 push，没有线程在锁里 panic");
    campaign_report_of_finished_histories(
        first_seed,
        seed_count,
        &sampling,
        finished,
        worker_threads,
        shrinking,
    )
}

/// 一批跑完的历史（次序不限）汇总成报告：按种子排好，计数相加，「已知红」照记，新发现按签名归类、按 `shrinking` 收缩各类的第一个种子；
/// 报告与每个新发现带的取样参数都是 `sampling`（这批历史就是照它生成、照它跑的）。
pub(super) fn campaign_report_of_finished_histories(
    first_seed: u64,
    seed_count: u64,
    sampling: &HistorySampling,
    mut finished: Vec<(GeneratedHistory, HistoryRun)>,
    worker_threads: usize,
    shrinking: FindingShrinking,
) -> CampaignReport {
    finished.sort_by_key(|(history, _)| history.seed);
    let mut tally = HistoryTally::default();
    let mut known_red_hits = Vec::new();
    let mut findings: BTreeMap<
        FailureSignature,
        (GeneratedHistory, FailureObservation, Vec<HistorySeed>),
    > = BTreeMap::new();
    for (history, run) in finished {
        tally.absorb(&run.tally);
        match run.ending {
            HistoryEnding::Completed => {}
            HistoryEnding::KnownRed { form, observation } => {
                known_red_hits.push((history.seed, form, observation.position));
            }
            HistoryEnding::NewFinding {
                signature,
                observation,
            } => {
                findings
                    .entry(signature)
                    .or_insert_with(|| (history.clone(), observation, Vec::new()))
                    .2
                    .push(history.seed);
            }
        }
    }
    let mut by_first_seed: Vec<_> = findings.into_iter().collect();
    by_first_seed.sort_by_key(|(_, (history, _, _))| history.seed);
    let new_findings = by_first_seed
        .into_iter()
        .map(|(signature, (history, observation, seeds))| {
            let shrunk = match shrinking {
                FindingShrinking::EveryFindingClass => Some(shrink_to_reproduction(
                    &history,
                    &signature,
                    sampling.execution,
                    worker_threads,
                )),
                FindingShrinking::ReportSeedsOnly => None,
            };
            NewFindingReport {
                signature,
                first_seed: history.seed,
                observation,
                seeds,
                shrunk,
                sampling: *sampling,
            }
        })
        .collect();
    CampaignReport {
        first_seed,
        seed_count,
        sampling: *sampling,
        tally,
        known_red_hits,
        new_findings,
    }
}

/// 执行器的可写挂载装的整理开关：产品值，只差叫停整理（[`CompactionRunForcing::ForcedToStopForTests`]）。模型不模挂载续做整理的发布，
/// 逐版比那一串里不要有它们；用户改动发成之后也不记「该跑了」，整理只在 `HistoryOperation::CompactWhileMounted` 那一步装上自己的开关之后跑。
pub(super) fn compaction_switches_of_the_mounts_of_the_executor() -> CompactionSwitches {
    let CompactionSwitches {
        batch_size,
        forced_region,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    CompactionSwitches {
        batch_size,
        forced_region,
        run_forcing: CompactionRunForcing::ForcedToStopForTests,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`run_history_campaign`] 报告带的取样参数就是调用方给的那一组，跑的就是照它生成的历史：一段 6 步的历史，计数与照取样参数
    /// 自己生成、自己跑的那一段逐项相同。判别力：同一个种子按默认批的比重生成的是另一段历史，计数也不同（前提断言），
    /// 取样参数装成默认批那一组时这里红。
    #[test]
    fn campaign_runs_and_reports_the_sampling_it_was_given() {
        let sampling = HistorySampling {
            operations_per_history: 6,
            weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
            execution: HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: HistoryDeviceWidth::FourGibibytes,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            },
        };
        let seed = HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE);
        let tally_of = |weights: &GenerationWeights| {
            execute_history_with(
                &generate_history_with_weights(seed, sampling.operations_per_history, weights),
                sampling.execution,
                &SharedStream::new(),
                &mut |_| {},
            )
            .tally
        };
        let tally_under_the_sampling = tally_of(&sampling.weights);
        assert_ne!(
            tally_under_the_sampling,
            tally_of(&GenerationWeights::BROAD),
            "前提：同一个种子按默认批的比重跑出来的计数不同，下面那条断言才分得出跑的是哪一组"
        );
        let mut tally_of_a_one_history_campaign = HistoryTally::default();
        tally_of_a_one_history_campaign.absorb(&tally_under_the_sampling);
        let report = run_history_campaign(
            seed.0,
            1,
            sampling.operations_per_history,
            &sampling.weights,
            sampling.execution,
            1,
            FindingShrinking::ReportSeedsOnly,
        );
        assert_eq!(report.sampling, sampling, "报告带的取样参数不是调用方给的");
        assert_eq!(
            report.tally, tally_of_a_one_history_campaign,
            "跑出来的计数与照取样参数生成的那一段不同：跑的不是这一组取样参数下的历史"
        );
    }

    /// 收缩在一个人造的失败判据上（冷启动之前有抬 F 或零单元发布就算失败）：序列是 [抬 F, 3 × 噪声, 零单元发布, 3 × 噪声, 冷启动]。
    /// 按次序删，先删掉的是前半（抬 F 连三条噪声），收到 [零单元发布, 冷启动]；按窗口并行试要收到同一个结果——窗口里若取的不是块号最小
    /// 的那个，会先删后半、收到 [抬 F, 冷启动]，两条路分得出来。
    #[test]
    fn shrinking_keeps_only_the_steps_the_failure_needs() {
        let noise = HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 7 },
            fill_seed: 9,
        });
        let raise = HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
            steps_above_current_floor: 5,
        });
        let operations: Vec<HistoryOperation> = std::iter::once(raise)
            .chain(std::iter::repeat_n(noise, 3))
            .chain(std::iter::once(HistoryOperation::PublishWithoutUnits))
            .chain(std::iter::repeat_n(noise, 3))
            .chain(std::iter::once(HistoryOperation::ColdStartRecover))
            .collect();
        let runs = AtomicU64::new(0);
        let still_fails = |candidate: &[HistoryOperation]| {
            runs.fetch_add(1, Ordering::Relaxed);
            let cold_start_at = candidate
                .iter()
                .position(|operation| operation.kind() == HistoryOperationKind::ColdStartRecover);
            cold_start_at.is_some_and(|cold_index| {
                candidate[..cold_index].iter().any(|operation| {
                    matches!(
                        operation.kind(),
                        HistoryOperationKind::RaiseRollbackFloor
                            | HistoryOperationKind::PublishWithoutUnits
                    )
                })
            })
        };
        let shrunk_one_at_a_time = shrink_operations(&operations, &still_fails, 1);
        assert_eq!(
            shrunk_one_at_a_time,
            vec![
                HistoryOperation::PublishWithoutUnits,
                HistoryOperation::ColdStartRecover
            ],
            "只剩零单元发布与冷启动两步"
        );
        let runs_one_at_a_time = runs.load(Ordering::Relaxed);
        assert!(
            runs_one_at_a_time > 2,
            "收缩真的试过删法：{runs_one_at_a_time} 次"
        );
        assert_eq!(
            shrink_operations(&operations, &still_fails, 7),
            shrunk_one_at_a_time,
            "按窗口并行试与一块一块按次序试，收到同一个结果"
        );
    }
}
