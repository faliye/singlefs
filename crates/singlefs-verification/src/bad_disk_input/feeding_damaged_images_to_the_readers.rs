//! 把历史里抽到的合法镜像逐个坏掉、喂给恢复、可写挂载与池级 checker，收它们各自的结局、记进计数与发现。

use super::*;

/// 一档基线的蓄水池：合格的镜像见过几份、手里留的是哪一份。
struct BaseImageReservoir {
    tier: BaseImageTier,
    qualifying_images_seen: u64,
    chosen: Option<MemoryPool>,
    random: SeededRandomSource,
}

/// 跑一段历史，按 [`EVERY_BASE_IMAGE_TIER`] 每一档各抽一份合法镜像，逐个坏法坏一遍，每份坏镜像喂给恢复、可写挂载与池级 checker。
///
/// 每一档的基线镜像取**按种子抽中的那一步**之后的那一份（蓄水池抽样，每档只留一份：一段历史几十步，全留下几十份稀疏镜像太占内存）：
/// - 「三棵树都在」那一档只在最新那条根下面该有的那几棵树都在的那几步里抽（[`newest_root_has_the_trees_the_targeted_damages_need`]）：
///   树表 0 条的盘面上指着普查那几族的坏法一样也做不出来，抽中它这一档就只跑了三个盲坏法。一步都不合格时退回最后那一步的镜像。
/// - 「树表 0 条」那一档只在最新那条根的树表 0 条的那几步里抽（C481（坏盘输入的基线镜像取不到树表 0 条的盘面））：
///   只做过 mkfs 的池、第一个文件版本之前的写行与暖机、回退到这样一版——这些盘面从前一档都没打过。一步都没有就这一段不抽。
///
/// 两档的判据相同（不许读回没提交过的内容、不许 panic），坏法照旧全部跑，做不出来的照实记「不适用」。
#[must_use]
pub fn feed_bad_disk_inputs_from_history(
    seed: HistorySeed,
    operations_per_history: usize,
    weights: &GenerationWeights,
    execution: HistoryExecution,
) -> HistoryBadDiskInput {
    feed_bad_disk_inputs_from_history_handing_out_the_damaged_images(
        seed,
        operations_per_history,
        weights,
        execution,
        &mut |_, _| {},
    )
}

/// 同 [`feed_bad_disk_inputs_from_history`]，另把每一份坏镜像（连同它坏的是哪一档的基线）喂给三个读者之前交给 `hand_out`，
/// 次序就是喂的次序：GPU 判器对坏输入的对拍拿这同一组坏镜像（经库）判，与 CPU 池级 checker 逐张比
/// （`tests/bad_disk_input_campaign.rs` 的 `the_gpu_judge_agrees_with_the_cpu_pool_checker_on_every_damaged_image_of_the_leading_default_batch_seeds`）。
#[must_use]
pub fn feed_bad_disk_inputs_from_history_handing_out_the_damaged_images(
    seed: HistorySeed,
    operations_per_history: usize,
    weights: &GenerationWeights,
    execution: HistoryExecution,
    hand_out: &mut dyn FnMut(BaseImageTier, &DamagedImage),
) -> HistoryBadDiskInput {
    let history = generate_history_with_weights(seed, operations_per_history, weights);
    let mut committed_contents: BTreeSet<Option<Vec<u8>>> = BTreeSet::new();
    let mut last_image: Option<MemoryPool> = None;
    let mut reservoirs: Vec<BaseImageReservoir> = EVERY_BASE_IMAGE_TIER
        .into_iter()
        .map(|tier| BaseImageReservoir {
            tier,
            qualifying_images_seen: 0,
            chosen: None,
            random: SeededRandomSource::from_seed(seed.0 ^ BAD_DISK_SEED_SALT ^ tier.salt()),
        })
        .collect();
    execute_history_with(
        &history,
        execution,
        &SharedStream::new(),
        &mut |observation| {
            for (_, file) in observation.model.committed_versions() {
                committed_contents.insert(file.map(|content| content.to_vec()));
            }
            last_image = Some(observation.image.clone());
            for reservoir in &mut reservoirs {
                if !reservoir.tier.admits(observation.image) {
                    continue;
                }
                reservoir.qualifying_images_seen += 1;
                // 蓄水池抽样：第 n 份合格镜像以 1/n 的概率换下手里那一份，跑完手里就是合格那几步里均匀抽的一份。
                if reservoir.random.below(reservoir.qualifying_images_seen) == 0 {
                    reservoir.chosen = Some(observation.image.clone());
                }
            }
        },
    );

    let mut tally = BadDiskTally {
        histories: 1,
        ..BadDiskTally::default()
    };
    if last_image.is_none() {
        tally.histories_without_a_base_image = 1;
        return HistoryBadDiskInput {
            seed,
            tally,
            findings: Vec::new(),
        };
    }
    let base_images: Vec<(BaseImageTier, MemoryPool)> = reservoirs
        .into_iter()
        .filter_map(|reservoir| {
            let chosen = match reservoir.tier {
                BaseImageTier::NewestRootHasTheTreesTheTargetedDamagesNeed => {
                    reservoir.chosen.or_else(|| last_image.clone())
                }
                BaseImageTier::TreeTableWithoutEntries => reservoir.chosen,
            };
            chosen.map(|image| (reservoir.tier, image))
        })
        .collect();

    let parameters = execution.device_width.parameters();
    let mut findings = Vec::new();
    for (tier, base_image) in &base_images {
        *tally.base_images_by_tier.entry(tier.name()).or_insert(0) += 1;
        for (kind_index, kind) in EVERY_DAMAGE_KIND.into_iter().enumerate() {
            let mut random = SeededRandomSource::from_seed(
                seed.0
                    .wrapping_mul(u64::try_from(EVERY_DAMAGE_KIND.len()).expect("坏法数"))
                    .wrapping_add(u64::try_from(kind_index).expect("坏法序号"))
                    ^ BAD_DISK_SEED_SALT
                    ^ tier.salt(),
            );
            let Some(damaged) = damage_image(base_image, kind, &mut random) else {
                *tally
                    .damages_not_applicable_by_kind
                    .entry(kind.name())
                    .or_insert(0) += 1;
                continue;
            };
            *tally.damaged_images_by_kind.entry(kind.name()).or_insert(0) += 1;
            *tally.damaged_images_by_tier.entry(tier.name()).or_insert(0) += 1;
            hand_out(*tier, &damaged);
            let observation = feed_a_damaged_image(
                &damaged,
                &committed_contents,
                &parameters,
                execution.device_width,
            );
            absorb_observation(&mut tally, &observation);
            if observation.was_seen_by_a_reader() {
                *tally
                    .damages_seen_by_a_reader_by_kind
                    .entry(kind.name())
                    .or_insert(0) += 1;
            }
            findings.extend(findings_in(seed, *tier, &observation));
        }
    }
    HistoryBadDiskInput {
        seed,
        tally,
        findings,
    }
}

fn absorb_observation(tally: &mut BadDiskTally, observation: &BadDiskObservation) {
    tally.recoveries += 1;
    tally.mounts += 1;
    tally.checker_runs += 1;
    match &observation.recovery {
        ReaderOutcome::Panicked(_) => {}
        ReaderOutcome::Finished { how } => {
            if how.starts_with("FileRead") {
                tally.recoveries_reading_a_file += 1;
            } else if how.starts_with("NoFile") {
                tally.recoveries_without_a_file += 1;
            } else {
                tally.recoveries_failed += 1;
            }
        }
    }
    match &observation.mount {
        ReaderOutcome::Panicked(_) => {}
        ReaderOutcome::Finished { how } => {
            if how.starts_with("Ok") {
                tally.mounts_succeeded += 1;
            } else {
                tally.mounts_refused += 1;
            }
        }
    }
    match &observation.read_back {
        ReadBackVerdict::RecoveryPanicked => {}
        ReadBackVerdict::ReportedAnError { .. } => tally.read_back_errors += 1,
        ReadBackVerdict::ReadBackACommittedVersion => tally.read_back_committed_versions += 1,
        ReadBackVerdict::ReadBackAVersionNeverCommitted { .. } => {
            tally.read_back_versions_never_committed += 1;
        }
    }
    for invariant in &observation.invariants_violated {
        *tally.invariant_violations.entry(invariant).or_insert(0) += 1;
    }
    for (_, captured) in observation.panics() {
        match known_panic_site_of(&captured) {
            Some(site) => {
                *tally
                    .known_panics_by_registry_name
                    .entry(site.registry_name)
                    .or_insert(0) += 1;
            }
            None => tally.panics_outside_the_known_list += 1,
        }
    }
}

fn findings_in(
    seed: HistorySeed,
    tier: BaseImageTier,
    observation: &BadDiskObservation,
) -> Vec<BadDiskFinding> {
    let mut findings = Vec::new();
    for (reader, captured) in observation.panics() {
        if known_panic_site_of(&captured).is_some() {
            continue;
        }
        findings.push(BadDiskFinding {
            seed,
            tier,
            kind: observation.kind,
            what: observation.what.clone(),
            headline: format!(
                "{reader} panic 在 {}：{}（不在 KNOWN_PANIC_SITES 里）",
                captured.location, captured.message
            ),
            observation_text: render_observation(observation),
        });
    }
    if let ReadBackVerdict::ReadBackAVersionNeverCommitted { what } = &observation.read_back {
        findings.push(BadDiskFinding {
            seed,
            tier,
            kind: observation.kind,
            what: observation.what.clone(),
            headline: format!("恢复读回了一版从没提交过的内容：{what}"),
            observation_text: render_observation(observation),
        });
    }
    findings
}

fn render_observation(observation: &BadDiskObservation) -> String {
    format!(
        "恢复 {:?}；可写挂载 {:?}；checker 判红 {:?}",
        observation.recovery, observation.mount, observation.invariants_violated
    )
}

/// 把一份坏掉的镜像喂给恢复、可写挂载与池级 checker，三者都用 [`with_panic_capture`] 兜着——
/// panic 是被测代码的缺口，不是这里该崩的理由；接住之后照实记下位置与消息。
///
/// 三个读者各读一份自己的镜像拷贝：可写挂载会往盘上写（取号、写行、暖机），不能让它改到别人读的那一份。
///
/// 三个读者都在 CPU 上真跑：被测对象就是它们自己在坏输入上不 panic、不读回没提交过的内容（池级 checker 那一截判的是 CPU 那一份 checker 的代码，
/// GPU 判器是照它另写的一份，判不了它）。GPU 判器在同一组坏输入上的结论另由对拍用例经库与 CPU 池级 checker 逐张比。
// gpu-exempt: checker-under-test-on-bad-disk-inputs
// gpu-exempt: implementation-under-test-on-bad-disk-inputs
// gpu-exempt: writable-mount
#[must_use]
pub fn feed_a_damaged_image(
    damaged: &DamagedImage,
    committed_contents: &BTreeSet<Option<Vec<u8>>>,
    parameters: &MakeFilesystemParameters,
    device_width: HistoryDeviceWidth,
) -> BadDiskObservation {
    let recovery_image = damaged.image.clone();
    let recovery = with_panic_capture(|| recover(&recovery_image, JournalPolicy::Consult));
    let (recovery_outcome, read_back) = match recovery {
        Err(captured) => (
            ReaderOutcome::Panicked(captured),
            ReadBackVerdict::RecoveryPanicked,
        ),
        Ok(report) => {
            let read_back = observed_read_back(&report.outcome);
            let verdict = read_back_verdict(&read_back, committed_contents);
            (
                ReaderOutcome::Finished {
                    how: describe_read_back_naming_the_variant(&read_back),
                },
                verdict,
            )
        }
    };

    let mount_image = damaged.image.clone();
    let mount = with_panic_capture(|| {
        let mut devices = block_devices_of(&mount_image, device_width);
        match mount_writable(parameters, &mut devices) {
            Ok(mounted) => format!(
                "Ok（实例 {}，施加前缀之后的根 txg {}）",
                mounted.output.instance.0, mounted.output.effective_root.checkpoint_txg.0
            ),
            Err(error) => format!("Err({error:?})"),
        }
    });
    let mount_outcome = match mount {
        Err(captured) => ReaderOutcome::Panicked(captured),
        Ok(how) => ReaderOutcome::Finished { how },
    };

    let checker_image = damaged.image.clone();
    let checker = with_panic_capture(|| check_pool_image(&checker_image));
    let (checker_outcome, invariants_violated, checker_judged_nothing) = match checker {
        Err(captured) => (ReaderOutcome::Panicked(captured), Vec::new(), false),
        Ok(report) => {
            let violated: Vec<&'static str> = report
                .iter()
                .filter_map(|(invariant, verdict)| match verdict {
                    InvariantVerdict::Violated(_) => Some(*invariant),
                    InvariantVerdict::Holds
                    | InvariantVerdict::NotApplicable(_)
                    | InvariantVerdict::PartiallyJudged(_) => None,
                })
                .collect();
            let judged_nothing = checker_judged_no_invariant(&report);
            (
                ReaderOutcome::Finished {
                    how: format!("判红 {} 条", violated.len()),
                },
                violated,
                judged_nothing,
            )
        }
    };

    BadDiskObservation {
        kind: damaged.kind,
        what: damaged.what.clone(),
        recovery: recovery_outcome,
        mount: mount_outcome,
        checker: checker_outcome,
        read_back,
        invariants_violated,
        checker_judged_nothing,
    }
}

/// 池级 checker 一条都没判出来：每一条都报不适用或部分没判（[`BadDiskObservation::checker_judged_nothing`]）。
/// 部分没判不算判过（`InvariantVerdict::PartiallyJudged`：判到的那一部分对得上、射程里另有一部分这张镜像上判不了，它不是成立，
/// `research/prompts/m3-root-ring-code-r3-main-verification.md` 第三节）。
pub(super) fn checker_judged_no_invariant(report: &[(&'static str, InvariantVerdict)]) -> bool {
    report.iter().all(|(_, verdict)| match verdict {
        InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => true,
        InvariantVerdict::Holds | InvariantVerdict::Violated(_) => false,
    })
}

fn describe_read_back_naming_the_variant(read_back: &ObservedReadBack) -> String {
    match read_back {
        ObservedReadBack::NoFile { root } => format!(
            "NoFile（实例 {} 第 {} 代根）",
            root.instance.0, root.checkpoint_txg.0
        ),
        ObservedReadBack::FileRead { root, content } => format!(
            "FileRead（实例 {} 第 {} 代根，{} 字节）",
            root.instance.0,
            root.checkpoint_txg.0,
            content.len()
        ),
        ObservedReadBack::Failed { what } => format!("Failed（{what}）"),
    }
}

/// 坏盘输入这一侧的读回判据：报错合法；读回的内容必须与模型提交过的某一版逐字节相同。
/// **按内容认、不按根的身份认**：坏盘能把根记录里的 txg 与实例代号一起改掉，按身份认会把「内容没错、身份被改」也判成失败，
/// 而里程碑那一句管的是内容（「不许读回一版从没提交过的内容」）。
#[must_use]
pub fn read_back_verdict(
    read_back: &ObservedReadBack,
    committed_contents: &BTreeSet<Option<Vec<u8>>>,
) -> ReadBackVerdict {
    match read_back {
        ObservedReadBack::Failed { what } => {
            ReadBackVerdict::ReportedAnError { what: what.clone() }
        }
        ObservedReadBack::NoFile { .. } => {
            if committed_contents.contains(&None) {
                ReadBackVerdict::ReadBackACommittedVersion
            } else {
                ReadBackVerdict::ReadBackAVersionNeverCommitted {
                    what: "报了没有文件，而模型提交过的每一版都带着文件".to_string(),
                }
            }
        }
        ObservedReadBack::FileRead { content, .. } => {
            if committed_contents.contains(&Some(content.clone())) {
                ReadBackVerdict::ReadBackACommittedVersion
            } else {
                ReadBackVerdict::ReadBackAVersionNeverCommitted {
                    what: format!(
                        "读回 {} 字节（首字节 {:?}、末字节 {:?}），模型提交过的 {} 版里没有这一份",
                        content.len(),
                        content.first(),
                        content.last(),
                        committed_contents.len()
                    ),
                }
            }
        }
    }
}
