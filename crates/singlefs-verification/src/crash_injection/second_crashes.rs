//! 二次崩溃：挂载途中再崩一次的点按持久子集的指纹摆，每个二次崩溃镜像上判、再挂一次判重放幂等与拒绝合不合法。

use super::*;

/// 摆挂载途中的二次崩溃的随机源，与上面两路再岔开（再按第一次崩溃的段号与段内子集岔开：同一段历史上各个崩溃状态抽的不是同一串，
/// [`draw_second_crash_points`]）。
const SECOND_CRASH_POINT_SEED_SALT: u64 = 0x73_65_63_6f_6e_64_63_72;

/// 第一次崩溃那个段内子集的指纹（[`fingerprint_of_the_persisted_subset`]）取 64 位 FNV-1a：起始值。
const PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// 64 位 FNV-1a 的乘数。
const PERSISTED_SUBSET_FINGERPRINT_PRIME: u64 = 0x0000_0100_0000_01b3;

/// 挂载途中的二次崩溃怎么摆（[`judge_second_crashes_inside_the_writable_mount`]）。
#[derive(Clone, Copy)]
pub enum SecondCrashPointDraw {
    /// 挂载的每一段各按种子摆一个（[`draw_second_crash_points`]）：[`inject_crashes_into_history`] 与战役都走这一支。
    OneInEachMountPhaseBySeed,
    /// 每个第一次崩溃态上都只摆这一个二次崩溃（靶向用例用，攻方改法 F2，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：
    /// 从挂载那一段写表切出的第 `segment_index` 段的第一个写起数 `writes_from_the_segment_on` 个写，其中下标（从这一段的第一个写数起）
    /// 在 `withheld_among_them` 里的扣下、别的持久；更早的段整段持久，这几个写之后的一个都没持久。这几个写可以越过这一段的末尾，
    /// 越过的那几道屏障当设备没守（同 [`CrashPointDraw::ExactlyOneCrashState`]：今天的持久顺序下根槽 FUA 先于轮换、各成一段，
    /// 「轮换落了、根没落」在段内真子集里摆不出来）。崩在挂载的哪一段按第一个被扣下的写取。
    /// 段号不在挂载写出的段里、这几个写越过挂载写表末尾、`writes_from_the_segment_on` 是 0、扣下的下标越界或一个都没有时 panic——
    /// 写死的输入写错了，不悄悄摆别的状态（[`exactly_one_second_crash_point`]）。
    ExactlyOneSecondCrashState {
        segment_index: usize,
        writes_from_the_segment_on: usize,
        withheld_among_them: &'static [usize],
    },
}

/// 靶向用例那一个二次崩溃（[`SecondCrashPointDraw::ExactlyOneSecondCrashState`]）：持久集合从挂载第 `segment_index` 段的第一个写起数，
/// 可以越过段尾；崩在挂载的哪一段取第一个被扣下的写的（`phase_of_each_write` 与挂载写表逐条对应）。
///
/// # Panics
/// 段号不在 `mount_segments` 里、`writes_from_the_segment_on` 是 0 或越过挂载写表末尾、扣下的下标一个都没有或越界。
fn exactly_one_second_crash_point(
    mount_segments: &[Vec<usize>],
    phase_of_each_write: &[WritableMountPhase],
    segment_index: usize,
    writes_from_the_segment_on: usize,
    withheld_among_them: &[usize],
) -> SecondCrashPoint {
    let first_write_of_the_segment = mount_segments
        .get(segment_index)
        .and_then(|segment| segment.first().copied())
        .unwrap_or_else(|| {
            panic!(
                "靶向用例写死的挂载第 {segment_index} 段不在挂载写出的 {} 段里",
                mount_segments.len()
            )
        });
    assert!(
        writes_from_the_segment_on > 0
            && first_write_of_the_segment + writes_from_the_segment_on
                <= phase_of_each_write.len(),
        "靶向用例从挂载第 {segment_index} 段起数 {writes_from_the_segment_on} 个写：要至少一个、不越过挂载写表末尾（{} 个写）",
        phase_of_each_write.len()
    );
    let first_withheld = withheld_among_them
        .iter()
        .min()
        .copied()
        .filter(|_| {
            withheld_among_them
                .iter()
                .all(|withheld| *withheld < writes_from_the_segment_on)
        })
        .unwrap_or_else(|| {
            panic!(
                "靶向用例扣下的写 {withheld_among_them:?} 要至少一个、都落在从挂载第 {segment_index} 段起数的 {writes_from_the_segment_on} 个写里"
            )
        });
    SecondCrashPoint {
        segment_index,
        persisted_within_the_segment: (0..writes_from_the_segment_on)
            .map(|within_the_state| !withheld_among_them.contains(&within_the_state))
            .collect(),
        phase: phase_of_each_write[first_write_of_the_segment + first_withheld],
    }
}

/// 第一次崩溃那个段内子集的指纹：段内第 k 个写持久了折 1、没持久折 0，逐个按 64 位 FNV-1a 折进去。只用来岔开二次崩溃的随机源，
/// 不进任何判定。
pub(super) fn fingerprint_of_the_persisted_subset(persisted_within_the_segment: &[bool]) -> u64 {
    persisted_within_the_segment.iter().fold(
        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,
        |fingerprint, is_persisted| {
            (fingerprint ^ u64::from(*is_persisted))
                .wrapping_mul(PERSISTED_SUBSET_FINGERPRINT_PRIME)
        },
    )
}

/// 一个第一次崩溃态上按种子摆二次崩溃（[`draw_second_crash_points`]）用的随机源种子。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SecondCrashPointSeed(u64);

/// 历史种子、盐、第一次崩溃的段号与它段内子集的指纹（[`fingerprint_of_the_persisted_subset`]）四样异或。
fn seed_of_the_second_crash_points(
    seed: HistorySeed,
    crash_point: &CrashPoint,
) -> SecondCrashPointSeed {
    SecondCrashPointSeed(
        seed.0
            ^ SECOND_CRASH_POINT_SEED_SALT
            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")
            ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),
    )
}

/// 挂载的每一段各摆一个二次崩溃：在那一段的写里按种子挑一个 w，崩在 w 上——w 所在那一段里 w 之前的写都落了、w 没落、
/// w 之后的写各按种子抽；更早的段整段持久，更晚的段一个都没持久（与第一次崩溃同一个枚举域）。挂载没写出某一段的写就不摆那一段。
/// 种子是历史种子、盐、第一次崩溃的段号与它段内子集的指纹四样异或（[`seed_of_the_second_crash_points`]）：同一段里不同子集的
/// 第一次崩溃态各抽各的（攻方改法 F3，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第五节第 1 行；
/// 只混段号时同一段里的各个第一次崩溃态用同一串随机数，挂载写表相同就摆到同一组二次崩溃）。
pub(super) fn draw_second_crash_points(
    seed: HistorySeed,
    crash_point: &CrashPoint,
    segments: &[Vec<usize>],
    phase_of_each_write: &[WritableMountPhase],
) -> Vec<SecondCrashPoint> {
    let mut source =
        SeededRandomSource::from_seed(seed_of_the_second_crash_points(seed, crash_point).0);
    let mut chosen: BTreeSet<SecondCrashPoint> = BTreeSet::new();
    for phase in WritableMountPhase::ALL {
        let writes_of_the_phase: Vec<(usize, usize)> = segments
            .iter()
            .enumerate()
            .flat_map(|(segment_index, segment)| {
                segment
                    .iter()
                    .enumerate()
                    .filter(|(_, write_index)| phase_of_each_write[**write_index] == phase)
                    .map(move |(within_the_segment, _)| (segment_index, within_the_segment))
            })
            .collect();
        if writes_of_the_phase.is_empty() {
            continue;
        }
        let picked = usize::try_from(
            source.below(u64::try_from(writes_of_the_phase.len()).expect("写数装得进 u64")),
        )
        .expect("下标装得进 usize");
        let (segment_index, withheld) = writes_of_the_phase[picked];
        let persisted_within_the_segment: Vec<bool> = (0..segments[segment_index].len())
            .map(
                |within_the_segment| match within_the_segment.cmp(&withheld) {
                    std::cmp::Ordering::Less => true,
                    std::cmp::Ordering::Equal => false,
                    std::cmp::Ordering::Greater => source.below(2) == 1,
                },
            )
            .collect();
        chosen.insert(SecondCrashPoint {
            segment_index,
            persisted_within_the_segment,
            phase,
        });
    }
    chosen.into_iter().collect()
}

/// 第三截：挂载途中的二次崩溃。每个二次崩溃状态上：只读恢复（看 journal）、问模型——允许的版本是模型提交过的每一版，加上这次挂载写出的根
/// （写行那次与暖机照抄第一次崩溃之后恢复到的那一版的文件，所以内容取那一次读回的；那一次恢复失败时一条都不加）；
/// 盘上已持久的最新根槽取第一次崩溃与挂载写表两边里更新的那条——、池级 checker、记录核对器。
/// 池级 checker、两遍恢复与记录核对器在一张给定状态表上判（[`judge_the_second_crashes_on_a_given_states_table`]，配了 GPU 走 GPU）：
/// 记录核对器核的是 mkfs 之后的整条历史录制流接上整条挂载流，历史那几次发布在二次崩溃镜像上还在不在也核；恢复落到由记录重建的
/// 写行那一版时，它的根槽写在更晚的段里，也在表里。要录下可写挂载那一趟 Rust 实现发出的写、二次崩溃之后再挂，上不了 GPU。
// gpu-exempt: second-crash-during-mount
pub(super) fn judge_second_crashes_inside_the_writable_mount(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    inputs: SecondCrashInputs<'_>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    let SecondCrashInputs {
        crash_image_materialized,
        records_across_the_crash,
        mount_segments,
        committed_versions,
        content_read_back_after_the_crash,
        newest_persisted_before_the_mount,
        content_after_the_uninterrupted_mount,
        publishes_before_the_deletion_queue_continuation,
        publishes_before_the_instance_table_sweep,
        fields_read_after_the_first_crash,
        floor_raises_in_the_uninterrupted_mount_writes,
        continuation_of_the_uninterrupted_mount,
        parameters,
        space_admission,
        instance_table_sweep,
        rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
        reference_filesystem_at_the_first_crash,
        read_faults_in_the_first_crash_state,
        second_crash_draw,
        given_states,
    } = inputs;
    let mount_writes = records_across_the_crash.mount_writes();
    let phase_of_each_write = writable_mount_phase_of_each_write(
        mount_writes,
        publishes_before_the_deletion_queue_continuation,
        publishes_before_the_instance_table_sweep,
    );
    let second_crash_points = match second_crash_draw {
        SecondCrashPointDraw::OneInEachMountPhaseBySeed => {
            draw_second_crash_points(seed, crash_point, mount_segments, &phase_of_each_write)
        }
        SecondCrashPointDraw::ExactlyOneSecondCrashState {
            segment_index,
            writes_from_the_segment_on,
            withheld_among_them,
        } => vec![exactly_one_second_crash_point(
            mount_segments,
            &phase_of_each_write,
            segment_index,
            writes_from_the_segment_on,
            withheld_among_them,
        )],
    };
    let mut versions_allowed = committed_versions.clone();
    if let Some(content) = &content_read_back_after_the_crash {
        for write in mount_writes {
            if write.kind != StepKind::RootRecordFua {
                continue;
            }
            let (instance, checkpoint_txg) =
                root_identity_written_by(write.bytes().expect("根槽 FUA 写是普通写，带着字节"));
            versions_allowed.insert(model_root_key(instance, checkpoint_txg), content.clone());
        }
    }
    let mut second_base = crash_image_materialized.clone();
    let mut writes_applied_to_the_second_base = 0usize;
    let mut ordered = second_crash_points;
    ordered.sort_by_key(|second| second.segment_index);
    // 与整条挂载流逐条对应：更早的段整段持久、当前段按二次崩溃的子集、更晚的段一个都没持久。
    let persisted_in_the_mount_of_each_second_crash: Vec<Vec<bool>> = ordered
        .iter()
        .map(|second| {
            persisted_over_the_whole_write_table(
                mount_writes.len(),
                mount_segments[second.segment_index][0],
                &second.persisted_within_the_segment,
            )
        })
        .collect();
    let second_crash_conclusions = judge_the_second_crashes_on_a_given_states_table(
        &given_states,
        mount_writes,
        mount_segments,
        &persisted_in_the_mount_of_each_second_crash,
        format!(
            "seed={}/second-crash/segment={}/first-crash-fingerprint={:016x}",
            seed.0,
            crash_point.segment_index,
            fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment)
        ),
    );
    count_a_table_left_to_another_host(tally, second_crash_conclusions.source);
    for ((second, persisted_in_the_mount), second_crash_conclusion) in ordered
        .iter()
        .zip(&persisted_in_the_mount_of_each_second_crash)
        .zip(&second_crash_conclusions.states)
    {
        let segment = &mount_segments[second.segment_index];
        let first_write_of_the_segment = segment[0];
        let writes_in_the_segment = second.persisted_within_the_segment.len();
        second_base.apply_writes(
            &mount_writes[writes_applied_to_the_second_base..first_write_of_the_segment],
        );
        writes_applied_to_the_second_base = first_write_of_the_segment;
        let image = CrashImage {
            base: &second_base,
            writes: &mount_writes
                [first_write_of_the_segment..first_write_of_the_segment + writes_in_the_segment],
            persisted: second.persisted_within_the_segment.clone(),
        };
        tally.second_crash_points += 1;
        *tally
            .second_crash_points_by_phase
            .entry(second.phase)
            .or_insert(0) += 1;
        let fields_read_after_the_second_crash =
            read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);
        let report = &fields_read_after_the_second_crash.report;
        match &report.outcome {
            RecoveryOutcome::FileRead { .. } => tally.second_crash_recoveries_reading_a_file += 1,
            RecoveryOutcome::NoFile { .. } => tally.second_crash_recoveries_without_a_file += 1,
            RecoveryOutcome::Failed { .. } => tally.second_crash_recoveries_failed += 1,
        }
        let read_back_after_the_second_crash = observed_read_back_after_a_crash(report);
        let newest_persisted_by_the_mount =
            newest_persisted_root(mount_writes, persisted_in_the_mount)
                .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
        let newest_persisted = newest_persisted_before_the_mount.max(newest_persisted_by_the_mount);
        tally.second_crash_model_judgements += 1;
        let disagreement = disagreement_with_the_ideal_model(
            &versions_allowed,
            newest_persisted,
            &read_back_after_the_second_crash,
        );
        let reference_filesystem_vote = reference_filesystem_vote_on_a_second_crash(
            reference_filesystem_at_the_first_crash,
            &read_back_after_the_second_crash,
        );
        tally.note_the_reference_filesystem_vote(
            StageTheReferenceFileSystemJudges::SecondCrashInsideTheWritableMount,
            &reference_filesystem_vote,
            disagreement.as_ref(),
        );
        tally.second_crash_record_checks += 1;
        let record_check = second_crash_conclusion.record_check();
        let second_crash_image_materialized = materialized_crash_image(&image);
        judge_the_remount_after_a_second_crash(
            tally,
            findings,
            &content_after_the_uninterrupted_mount,
            &RefusalOnTheCrashImage {
                crash_image_materialized: &second_crash_image_materialized,
                version_landed_on_after_the_crash: report.effective_root,
                filesystem_identifier_of_the_pool: &parameters.filesystem_identifier,
                read_only_read_back: if disagreement.is_none() {
                    ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
                } else {
                    ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
                },
                read_faults: read_faults_in_the_first_crash_state,
            },
            parameters,
            space_admission,
            instance_table_sweep,
            second.phase,
            seed,
            crash_point,
            image_directory,
        );
        tally.second_crash_checker_runs += 1;
        let mut violations = second_crash_conclusion.pool_checker_violations();
        violations.extend(reference_filesystem_vote.violation());
        // 挂载透明的射程只管不带故障的状态（D13（验证路线） 已定项 1）：第一次崩溃那个态注入过读故障的，它上面那次挂载途中的二次崩溃
        // 也不判，与第二截同一个理由记一次没判（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
        // 「二次崩溃镜像在注入态下照判（J2-d）」那一行，改法 G5）。
        violations.extend(match read_faults_in_the_first_crash_state {
            ReadFaultsInTheCrashState::NoneInjected => {
                mount_transparency_violations_of_a_second_crash(
                    tally,
                    fields_read_after_the_first_crash,
                    &fields_read_after_the_second_crash,
                    floor_raises_in_the_uninterrupted_mount_writes,
                    continuation_of_the_uninterrupted_mount,
                    rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
                    persisted_in_the_mount,
                    second.phase,
                )
            }
            ReadFaultsInTheCrashState::Injected => {
                *tally
                    .mount_transparency_not_judged
                    .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
                    .or_insert(0) += 1;
                Vec::new()
            }
        });
        if violations.is_empty() && disagreement.is_none() && record_check == RecordCheck::default()
        {
            continue;
        }
        let observation =
            crash_state_observation(&image, crash_point, violations, disagreement, record_check);
        findings.settle(
            tally,
            FailureAtAStage {
                seed,
                crash_point,
                stage: CrashStateStage::SecondCrashInsideTheWritableMount(second.phase),
                observation,
                read_back: format!("{read_back_after_the_second_crash:?}"),
                image: &image,
            },
            image_directory,
        );
    }
}

/// I-8.4 的 R-换：二次崩溃镜像（`second_crash_image.crash_image_materialized`）上再起一次可写挂载，「恢复加写行那一整段」于是做了两次；
/// 再挂之后只读恢复读出的内容要与没崩过的那一次挂载之后读出的（`once`）相同。不同判红（违例记在 [`REPLAY_IDEMPOTENCE_INVARIANT`] 名下）；
/// 条款没写的情形（[`RemountIdempotenceNotJudged`]）只计数。再挂被拒时 I-8.4 照旧不判，另判 C670 与改法 G4 那两条
/// （[`judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults`]）；没崩过的那一次挂载被拒（`once` 是没判的理由）时不再挂，两样都不判。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：两次挂载的输入（连同清扫的配置）、计数与发现、这一个二次崩溃落在哪"
)]
fn judge_the_remount_after_a_second_crash(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    once: &Result<ContentReadByRecovery, RemountIdempotenceNotJudged>,
    second_crash_image: &RefusalOnTheCrashImage<'_>,
    parameters: &MakeFilesystemParameters,
    space_admission: SpaceAdmission,
    instance_table_sweep: InstanceTableSweepConfiguration,
    phase: WritableMountPhase,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    tally.remounts_after_a_second_crash += 1;
    let once_content = match once {
        Err(reason) => {
            *tally
                .remount_idempotence_not_judged
                .entry(*reason)
                .or_insert(0) += 1;
            return;
        }
        Ok(content) => content,
    };
    let remount = writable_mount_alone_with_the_instance_table_sweep_configuration_on(
        second_crash_image.crash_image_materialized,
        parameters,
        space_admission,
        instance_table_sweep,
    );
    let pool_after_the_remount = match &remount {
        WritableMountAlone::Refused {
            member,
            refusal,
            pool_after: pool_after_the_refusal,
            ..
        } => {
            *tally
                .remount_idempotence_not_judged
                .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)
                .or_insert(0) += 1;
            judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults(
                tally,
                findings,
                RemountRefusedAfterASecondCrash {
                    member,
                    refusal,
                    pool_after_the_refusal,
                    second_crash_image,
                    phase,
                },
                seed,
                crash_point,
                image_directory,
            );
            return;
        }
        WritableMountAlone::Mounted { pool_after, .. } => pool_after,
    };
    let (differing_content_read_once, differing_content_read_twice) =
        match compare_content_read_once_and_twice(
            once_content,
            &content_read_by_recovery(pool_after_the_remount),
        ) {
            RemountIdempotence::SameContent => {
                *tally
                    .remount_idempotence_judged_by_phase
                    .entry(phase)
                    .or_insert(0) += 1;
                tally.remounts_after_a_second_crash_reading_the_same_content += 1;
                return;
            }
            RemountIdempotence::NotJudged(reason) => {
                *tally
                    .remount_idempotence_not_judged
                    .entry(reason)
                    .or_insert(0) += 1;
                return;
            }
            RemountIdempotence::DifferentContent {
                once: read_once,
                twice: read_twice,
            } => (read_once, read_twice),
        };
    *tally
        .remount_idempotence_judged_by_phase
        .entry(phase)
        .or_insert(0) += 1;
    tally.remounts_after_a_second_crash_reading_different_content += 1;
    let image = CrashImage {
        base: pool_after_the_remount,
        writes: &[],
        persisted: Vec::new(),
    };
    let detail = format!(
        "恢复加写行那一整段做一次读出 {}，崩在挂载的 {} 里再做一次读出 {}",
        differing_content_read_once.render(),
        phase.name(),
        differing_content_read_twice.render()
    );
    let observation = crash_state_observation(
        &image,
        crash_point,
        vec![(REPLAY_IDEMPOTENCE_INVARIANT, detail.clone())],
        None,
        RecordCheck::default(),
    );
    findings.settle(
        tally,
        FailureAtAStage {
            seed,
            crash_point,
            stage: CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(phase),
            observation,
            read_back: detail,
            image: &image,
        },
        image_directory,
    );
}

/// 第三截判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那一条（攻方改法 P1，`research/prompts/m3-formal-code-b16-r1-main-verification.md`
/// 第二节 K2-b）：二次崩溃镜像同样是一个崩溃态，再挂的拒因是判据 N-配置、崩溃态没注入读故障、判据在二次崩溃镜像上（撕裂的系统配置槽跳过）
/// 复算仍为真 ⇒ 拒得不合法，报红（记在 [`CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount`] 那一截）。判官一并判的 U6 那四条
/// 条款只管第二截，这一截不拿它们定红绿：只差那几条的不报红、不计不合法，判官列出的没满足的条件只随 C670 那一条的红带出来。
/// 改法 G4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「新拒因少一条 C670 同形的判法（J2-b）」那一行）同一个接法：
/// 再挂的拒因是「读阶段判完之后系统配置又读不出」、崩溃态没注入读故障、那一拒的判据在二次崩溃镜像上复算为真 ⇒ 拒得不合法，报红
/// （判官的 [`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）。
/// Q3（判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）同一个接法：再挂的拒因是「重建分配器读根环
/// 整环重读一次仍一条不被抛弃的自证有效根都没有」、崩溃态没注入读故障、那一拒的判据在二次崩溃镜像上复算为真 ⇒ 拒得不合法，报红
/// （判官的 [`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）。
/// 拒因是实例表读不出的不判。
// gpu-exempt: witness-criterion-w
fn judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    remount: RemountRefusedAfterASecondCrash<'_>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    let RemountRefusedAfterASecondCrash {
        member,
        refusal,
        pool_after_the_refusal,
        second_crash_image,
        phase,
    } = remount;
    let disk = if *pool_after_the_refusal == *second_crash_image.crash_image_materialized {
        DiskAfterTheRefusal::ByteForByteUnchanged
    } else {
        DiskAfterTheRefusal::Changed
    };
    let verdict = judge_writable_mount_refusal_of_a_crash_state_of_the_pool(
        second_crash_image.crash_image_materialized,
        second_crash_image.filesystem_identifier_of_the_pool,
        second_crash_image.version_landed_on_after_the_crash,
        refusal,
        disk,
        second_crash_image.read_only_read_back,
        second_crash_image.read_faults,
        READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,
    );
    let Some((reason, conditions_not_met)) =
        reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
            tally, verdict,
        )
    else {
        return;
    };
    let detail = format!(
        "崩在挂载的 {} 里、二次崩溃镜像上再挂被拒，理由 {reason:?}；崩溃态没注入读故障，判据在二次崩溃镜像上复算仍为真，拒得不合法（C670 / G4；判官列出的没满足的条件 {conditions_not_met:?}）：{member}",
        phase.name()
    );
    let model_disagreement = ModelDisagreement::new(
        ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
        "没注入读故障的崩溃态上，见证判据与「读阶段判完之后系统配置又读不出」那一拒的判据都不许为真：二次崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）都该成"
            .to_string(),
        detail.clone(),
    );
    let image = CrashImage {
        base: second_crash_image.crash_image_materialized,
        writes: &[],
        persisted: Vec::new(),
    };
    let observation = crash_state_observation(
        &image,
        crash_point,
        Vec::new(),
        Some(model_disagreement),
        RecordCheck::default(),
    );
    findings.settle(
        tally,
        FailureAtAStage {
            seed,
            crash_point,
            stage: CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(phase),
            observation,
            read_back: detail,
            image: &image,
        },
        image_directory,
    );
}

/// 第三截那一判的计数与取舍：拒因是判据 N-配置、「读阶段判完之后系统配置又读不出」或「重建分配器读根环整环重读一次仍一条有效根都没有」的，
/// 各记一次判过；判官列出的没满足的条件里有
/// 没注入读故障时判据复算为真的那一条（N-配置 的 C670：[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]；
/// 另一拒的改法 G4：[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]；
/// Q3：[`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）时
/// 再各记一次不合法，交回拒因与全部没满足的条件（报红用）。拒因是「记录有缺口时拒可写」（F1 乙）或 C 读一次没施加的发布的账读不出而记录不齐
/// （`C_MAPPING`）的只记一次判过：判官对它们不列那一类条件。
/// 拒因是实例表读不出的、那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
    tally: &mut CrashInjectionTally,
    verdict: WritableMountRefusalVerdict,
) -> Option<(
    RefusalReasonOnTheLegalRefusalList,
    Vec<RefusalConditionNotMet>,
)> {
    let (reason, conditions_not_met) = match verdict {
        WritableMountRefusalVerdict::ReasonNotOnTheLegalRefusalList
        | WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::InstanceTableOfTheNewestRootUnreadable,
        )
        | WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheLegalRefusalList::InstanceTableOfTheNewestRootUnreadable,
            ..
        } => return None,
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration,
        ) => {
            tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration,
            conditions_not_met,
        } => {
            tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration += 1;
            (
                RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration,
                conditions_not_met,
            )
        }
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::SystemConfigurationUnreadableAfterTheReadStage,
        ) => {
            tally.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheLegalRefusalList::SystemConfigurationUnreadableAfterTheReadStage,
            conditions_not_met,
        } => {
            tally.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage += 1;
            (
                RefusalReasonOnTheLegalRefusalList::SystemConfigurationUnreadableAfterTheReadStage,
                conditions_not_met,
            )
        }
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator,
        ) => {
            tally.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator,
            conditions_not_met,
        } => {
            tally.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator += 1;
            (
                RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator,
                conditions_not_met,
            )
        }
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
        ) => {
            tally.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason:
                RefusalReasonOnTheLegalRefusalList::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
            conditions_not_met,
        } => {
            tally.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify += 1;
            (
                RefusalReasonOnTheLegalRefusalList::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
                conditions_not_met,
            )
        }
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                ..
            },
        ) => {
            tally.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason:
                reason @ RefusalReasonOnTheLegalRefusalList::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                    ..
                },
            conditions_not_met,
        } => {
            tally.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records += 1;
            (reason, conditions_not_met)
        }
    };
    let mut criterion_true_without_read_faults = false;
    // 迭代上界是判官列出的条件数（至多五条）；跨轮携带的只有上面那一个布尔；没有提前出口。
    for condition in &conditions_not_met {
        match condition {
            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_) => {
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults += 1;
                criterion_true_without_read_faults = true;
            }
            RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {
                tally.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults += 1;
                criterion_true_without_read_faults = true;
            }
            RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults => {
                tally.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults += 1;
                criterion_true_without_read_faults = true;
            }
            RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            | RefusalConditionNotMet::DiskChangedByTheRefusedMount
            | RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet => {}
        }
    }
    criterion_true_without_read_faults.then_some((reason, conditions_not_met))
}

/// 这个崩溃状态在段内扣下的写各是什么种类（同一种只记一次）。段是写表里连续的一截
/// （`writes_and_segments_with_stream_indexes` 按次序切），第 k 个写的下标是段首加 k——靶向用例摆的持久集合越过段尾时也按这个数。
pub(super) fn withheld_write_kinds(
    writes: &[RetainedWrite],
    first_write_of_the_segment: usize,
    crash_point: &CrashPoint,
) -> Vec<StepKind> {
    let mut kinds: BTreeSet<StepKind> = BTreeSet::new();
    for (within_the_segment, is_persisted) in
        crash_point.persisted_within_the_segment.iter().enumerate()
    {
        if !is_persisted {
            kinds.insert(writes[first_write_of_the_segment + within_the_segment].kind);
        }
    }
    kinds.into_iter().collect()
}

#[cfg(test)]
mod tests;
