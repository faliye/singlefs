//! 录观察者：每个状态恢复一次、几个观察者共用（`record_the_observers_of_the_flow`），几条线程录块、进度行，
//! 与强制现判那一块、被关在库外的观察者重录。

use super::*;

/// 录入段一块里一个要录的观察者（[`record_the_observer_pieces_recovering_each_state_once`] 逐状态交给它）：观察者、它这一趟的几把版本、
/// 它的录入段上下文（这条流排好的状态、去重摘要器、基底摘要与库各观察者同一份，桶的见证索引与真跑锁是它自己的）。
#[derive(Clone, Copy)]
struct ObserverRecordingOfTheBlock<'block, 'observer, 'store> {
    observer: &'block ObserverOnTheNodes<'observer>,
    versions: &'block ObserverVersions,
    context: &'block ObserverRecordingContext<'block, 'store>,
}

/// 一个观察者在一段状态上录着的：逐状态落在哪一份观察结果上、按发布分的态数、账、读写库失败的那一句（失败之后的状态只占位、不再看），
/// 每个桶上一个状态对上见证的那一组读集（相邻的状态多半读同一组段，下一个状态先查它）。
struct ObserverPieceBeingRecorded {
    observation_of_each_state: Vec<ObservationOfStates>,
    state_counts: StateCountsOfABlock,
    tally: ObserverRecordingTally,
    failure: Option<String>,
    read_set_group_matched_last_in_each_bucket:
        std::collections::HashMap<[u8; DIGEST_BYTES], ReadSetGroupPosition>,
}

/// 一个崩溃状态上几个观察者共用的：看 journal 那一遍恢复的报告与它的摘要、这张镜像相对基础镜像的改动（各在头一个要它的观察者那里算一次，
/// 后面的观察者取同一份；恢复只读不写、报告与改动都只是镜像的函数）。`recoveries` 每恢复一次加一。看一段的那条线程跨状态留着的
/// （[`ReportDigestAndOverlayPrefixKeptAcrossStates`]）：上一个状态的恢复报告与它的摘要、改动的前缀那一截。
struct CrashStateSharedByTheObservers<'state, 'image> {
    image: &'state CrashImage<'image>,
    recoveries: &'state std::sync::atomic::AtomicU64,
    kept_across_the_states: &'state ReportDigestAndOverlayPrefixKeptAcrossStates,
    recovery_report: std::cell::OnceCell<RecoveryReport>,
    recovery_report_digest: std::cell::OnceCell<[u8; DIGEST_BYTES]>,
    overlay: std::cell::OnceCell<ImageOverlayOverAPrefix>,
}

/// 看一段的那条线程跨状态留着的：上一个算过摘要的恢复报告与它的摘要（下一个状态的报告逐字段相同就取这个摘要、不另算；摘要只是报告的函数），
/// 改动的前缀那一截（[`ReadSetWitnessDigester::overlay_of_the_state`]）。
#[derive(Default)]
struct ReportDigestAndOverlayPrefixKeptAcrossStates {
    recovery_report_with_its_digest:
        std::cell::RefCell<Option<(RecoveryReport, [u8; DIGEST_BYTES])>>,
    overlay_prefix: std::cell::RefCell<Option<std::rc::Rc<ImageOverlayOfAPrefix>>>,
}

/// `report` 的摘要（[`recovery_report_digest`]）：`kept` 里留着的那一份报告与它逐字段相同就交留着的摘要，不同就现算、换掉留着的。
fn recovery_report_digest_reusing_the_one_kept(
    report: &RecoveryReport,
    kept: &std::cell::RefCell<Option<(RecoveryReport, [u8; DIGEST_BYTES])>>,
) -> [u8; DIGEST_BYTES] {
    let mut kept = kept.borrow_mut();
    if let Some((kept_report, kept_digest)) = kept.as_ref() {
        if kept_report == report {
            return *kept_digest;
        }
    }
    let digest = recovery_report_digest(report);
    *kept = Some((report.clone(), digest));
    digest
}

impl<'state, 'image> CrashStateSharedByTheObservers<'state, 'image> {
    fn of_the_image(
        image: &'state CrashImage<'image>,
        recoveries: &'state std::sync::atomic::AtomicU64,
        kept_across_the_states: &'state ReportDigestAndOverlayPrefixKeptAcrossStates,
    ) -> Self {
        Self {
            image,
            recoveries,
            kept_across_the_states,
            recovery_report: std::cell::OnceCell::new(),
            recovery_report_digest: std::cell::OnceCell::new(),
            overlay: std::cell::OnceCell::new(),
        }
    }

    /// 看 journal 那一遍恢复的报告：这个状态上头一回问时恢复一次（`recoveries` 加一），之后交同一份。
    fn recovery_report(&self) -> &RecoveryReport {
        self.recovery_report.get_or_init(|| {
            self.recoveries
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            recover(self.image, JournalPolicy::Consult)
        })
    }

    /// 恢复报告的摘要（[`recovery_report_digest`]），这个状态上至多算一次；与上一个状态的报告逐字段相同就取那一个的。
    fn recovery_report_digest(&self) -> [u8; DIGEST_BYTES] {
        *self.recovery_report_digest.get_or_init(|| {
            recovery_report_digest_reusing_the_one_kept(
                self.recovery_report(),
                &self.kept_across_the_states.recovery_report_with_its_digest,
            )
        })
    }

    /// 这张镜像相对基础镜像的改动（[`ReadSetWitnessDigester::overlay_of_the_state`]），这个状态上算一次；前缀那一截与相邻的状态共用。
    fn overlay(&self, digester: &ReadSetWitnessDigester<'_>) -> &ImageOverlayOverAPrefix {
        self.overlay.get_or_init(|| {
            digester.overlay_of_the_state(
                &self.image.persisted,
                &self.kept_across_the_states.overlay_prefix,
            )
        })
    }
}

/// 录入段在 CPU 上看第 `point_index` 个点的块内序号 `local` 那一段，这一块要录的几个观察者（`observers_of_the_block`）一起看：
/// 逐状态造一次崩溃镜像、至多恢复一次（看 journal 那一遍，[`CrashStateSharedByTheObservers`]），按观察者的次序交给每个观察者
/// （[`observe_a_crash_state_for_one_observer`]）。交回每个观察者一段（与 `observers_of_the_block` 同序），各自的观察结果、计数、账与库读写失败
/// 与只给它一个人看这一段时逐项相同：观察者的结果只随镜像与恢复报告变（[`ObserveACrashStateHandingBackAnImage`] 的契约），
/// 一个观察者的桶、见证与真跑锁不碰别的观察者的。
///
/// # Panics
/// `observers_of_the_block` 是空的、各观察者的上下文借的不是同一份排好的状态；这一段的序号越过了整条流的状态数。
fn record_the_observer_pieces_recovering_each_state_once(
    plan: &CrashPlan,
    point_index: usize,
    block: &Range<u64>,
    local: Range<u64>,
    observers_of_the_block: &[ObserverRecordingOfTheBlock<'_, '_, '_>],
    states_recovered_in_the_block: &std::sync::atomic::AtomicU64,
) -> Vec<Result<RecordedObserverPiece, String>> {
    let shared_context = observers_of_the_block
        .first()
        .expect("这一块至少一个要录的观察者才起看各段的线程")
        .context;
    assert!(
        observers_of_the_block.iter().all(|recording| std::ptr::eq(
            recording.context.crash_states_of_the_flow,
            shared_context.crash_states_of_the_flow
        )),
        "这一块各观察者借的是录入段建好的同一份排好的状态"
    );
    let point = &plan.points[point_index];
    let block_key = point.block_key(block);
    let states_of_the_piece = usize::try_from(local.end - local.start).expect("一块装得进 usize");
    let mut pieces: Vec<ObserverPieceBeingRecorded> = observers_of_the_block
        .iter()
        .map(|_recording| ObserverPieceBeingRecorded {
            observation_of_each_state: Vec::with_capacity(states_of_the_piece),
            state_counts: StateCountsOfABlock::default(),
            tally: ObserverRecordingTally::default(),
            failure: None,
            read_set_group_matched_last_in_each_bucket: std::collections::HashMap::new(),
        })
        .collect();
    let kept_across_the_states = ReportDigestAndOverlayPrefixKeptAcrossStates::default();
    let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
    let stream_state_count = shared_context
        .crash_states_of_the_flow
        .visit_the_states_in_the_ordinals(ordinals, &mut |ordinal, publish, image| {
            shared_context
                .states_looked_at_in_the_block
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let state = CrashStateSharedByTheObservers::of_the_image(
                image,
                states_recovered_in_the_block,
                &kept_across_the_states,
            );
            for (recording, piece) in observers_of_the_block.iter().zip(&mut pieces) {
                piece.state_counts.count_the_publish_of_a_state(publish);
                if piece.failure.is_some() {
                    piece
                        .observation_of_each_state
                        .push(ObservationOfStates::Counted {
                            red_item: None,
                            counts: Vec::new(),
                        });
                    continue;
                }
                let observation = observe_a_crash_state_for_one_observer(
                    recording, piece, point, block, &block_key, ordinal, &state,
                );
                piece.observation_of_each_state.push(observation);
            }
        });
    assert!(
        stream_state_count >= point.ordinals.end,
        "这一段的序号越过了整条流的状态数"
    );
    pieces
        .into_iter()
        .map(|piece| match piece.failure {
            Some(failure) => Err(failure),
            None => Ok(RecordedObserverPiece {
                local: local.clone(),
                observation_of_each_state: piece.observation_of_each_state,
                state_counts: piece.state_counts,
                tally: piece.tally,
            }),
        })
        .collect()
}

/// 一个观察者看一个崩溃状态（`state` 是这个状态上几个观察者共用的镜像与恢复报告）：观察者真跑实现（可写挂载、续做；这一截只能在 CPU 上，
/// 豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount、continuation 两行；只记计数的观察者对恢复报告下的断言另提一行，见报告）。
/// 交回镜像的观察者按恢复报告分桶、按读集见证去重（[`ObserverDeduplication`]）：落进的桶里有见证、这个状态在见证那几段上的字节与它相同的，
/// 不再看、取那一次的结果；一个都对不上的看完连同新见证一批原子地写进库。观察者 panic 的状态记成红项（没交回读过的段，按状态单存）。
/// 读写库失败记进 `piece.failure`（这个观察者这一段之后的状态只占位）。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
// gpu-exempt: observer-assertions-on-the-recovery-report
#[allow(
    clippy::too_many_lines,
    reason = "恢复报告分桶、查见证、真跑锁、看、核观察者交回的写、落盘是一个状态的全部步骤，拆开要把桶、见证与账来回传"
)]
fn observe_a_crash_state_for_one_observer(
    recording: &ObserverRecordingOfTheBlock<'_, '_, '_>,
    piece: &mut ObserverPieceBeingRecorded,
    point: &PlannedCrashPoint,
    block: &Range<u64>,
    block_key: &VerdictBlockKey,
    ordinal: u64,
    state: &CrashStateSharedByTheObservers<'_, '_>,
) -> ObservationOfStates {
    let ObserverRecordingOfTheBlock {
        observer,
        versions,
        context,
    } = *recording;
    let image = state.image;
    match &observer.observe {
        ObserverCode::CountsOnly(observe) => {
            let report = state.recovery_report();
            let mut counts_of_this_state = CrashPointReplayObserverCounts::default();
            let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                observe(ordinal, image, report, &mut counts_of_this_state);
            }));
            piece.tally.states_counted_here += 1;
            match observed {
                Ok(()) => ObservationOfStates::Counted {
                    red_item: None,
                    counts: counts_of_this_state
                        .counts_in_name_order()
                        .map(|(name, count)| (name.to_string(), count))
                        .collect(),
                },
                Err(payload) => ObservationOfStates::Counted {
                    red_item: Some(observer_red_item(
                        &observer.observer_name,
                        &text_carried_by_the_panic_payload(payload.as_ref()),
                    )),
                    counts: Vec::new(),
                },
            }
        }
        ObserverCode::HandsBackAnImage {
            observe,
            deduplication,
        } => {
            let digester = context
                .digester
                .expect("有交回镜像的观察者时录入段先建了去重摘要器");
            let offset_in_block = ordinal - point.ordinals.start - block.start;
            // 先恢复、按恢复报告分桶：恢复只读不写，报告是镜像的函数
            let report = state.recovery_report();
            let bucket_key =
                observer_bucket_key(observer, versions, &state.recovery_report_digest());
            let key_without_deduplication = || {
                observed_image_key_without_deduplication(&bucket_key, block_key, offset_in_block)
            };
            let witnesses_of_the_bucket: Option<std::sync::Arc<ReadSetWitnessIndexOfABucket>> =
                match deduplication {
                    ObserverDeduplication::OffForTheSelfProof => None,
                    ObserverDeduplication::ByTheRecoveryReportAndTheReadSet => {
                        match context.witnesses_of_the_bucket(&bucket_key) {
                            Ok(witnesses) => Some(witnesses),
                            Err(error) => {
                                piece.failure = Some(error);
                                Some(std::sync::Arc::new(std::sync::RwLock::new(
                                    ReadSetWitnessIndex::empty(),
                                )))
                            }
                        }
                    }
                };
            let overlay = witnesses_of_the_bucket
                .is_some()
                .then(|| state.overlay(digester));
            // 每组读集只算一次这个状态的见证摘要（[`ReadSetWitnessIndex::find_the_matching_witness`]）；拿这个桶索引的读锁查，
            // 别的线程添见证时等这一次查完，查到的是添之前或添之后的整份
            let mut find_a_matching_witness = |witnesses: &Option<
                std::sync::Arc<ReadSetWitnessIndexOfABucket>,
            >|
             -> Option<[u8; DIGEST_BYTES]> {
                let (witnesses, overlay) = witnesses.as_ref().zip(overlay.as_ref())?;
                let matched = witnesses
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .find_the_matching_witness(
                        piece
                            .read_set_group_matched_last_in_each_bucket
                            .get(&bucket_key)
                            .copied(),
                        |ranges| digester.witness_digest(overlay, ranges),
                    )?;
                piece
                    .read_set_group_matched_last_in_each_bucket
                    .insert(bucket_key, matched.group);
                Some(matched.observed_image_key)
            };
            let mut matched_witness = find_a_matching_witness(&witnesses_of_the_bucket);
            // 一个都对不上：拿这个桶的真跑锁，等正在真跑的线程落完盘再核一次（它刚添的见证多半就对上了），还对不上才自己跑
            let bucket_run_lock = (matched_witness.is_none()
                && witnesses_of_the_bucket.is_some()
                && piece.failure.is_none())
            .then(|| context.run_lock_of_the_bucket(&bucket_key));
            let _bucket_run_held = bucket_run_lock.as_ref().map(|lock| {
                lock.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
            });
            // 手里的就是这个桶唯一那一份索引，别的线程刚添的见证已经在里面，不再去取
            if _bucket_run_held.is_some() {
                matched_witness = find_a_matching_witness(&witnesses_of_the_bucket);
            }
            // 关掉去重时每个状态各一个键：库里存着（上一趟录到一半）的照样不再看
            let stored_without_deduplication = witnesses_of_the_bucket.is_none()
                && match context
                    .store
                    .take()
                    .read_observed_image(&key_without_deduplication())
                {
                    Ok(stored) => stored.is_some(),
                    Err(error) => {
                        piece.failure = Some(observer_library_error(&error));
                        true
                    }
                };
            let deduplication_key = if let Some(key) = matched_witness {
                piece.tally.states_reusing_an_observed_image += 1;
                key
            } else if stored_without_deduplication {
                piece.tally.states_reusing_an_observed_image += 1;
                key_without_deduplication()
            } else if piece.failure.is_some() {
                key_without_deduplication()
            } else {
                if witnesses_of_the_bucket.as_ref().is_some_and(|witnesses| {
                    witnesses
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .witness_count()
                        != 0
                }) {
                    piece.tally.witness_misses += 1;
                }
                let mut counts_of_this_state = CrashPointReplayObserverCounts::default();
                let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    observe(ordinal, image, report, &mut counts_of_this_state)
                }));
                let (record, witness) = match observed {
                    Ok(image_after) => {
                        let witness = overlay.as_ref().map(|overlay| {
                            let read_set = ObservedReadSet::of_the_ranges(normalized_read_ranges(
                                &image_after.ranges_read_on_the_crash_image,
                            ));
                            let digest = digester.witness_digest(overlay, &read_set.ranges);
                            ReadSetWitness {
                                read_set,
                                digest,
                                observed_image_key: observed_image_key_of_the_witness(
                                    &bucket_key,
                                    &digest,
                                ),
                            }
                        });
                        let mut rebuilt =
                            pool_of_the_crash_image(image.base, image.writes, &image.persisted);
                        rebuilt.apply_writes(&image_after.writes_on_the_crash_image);
                        let record = if pools_hold_the_same_bytes(
                            &rebuilt,
                            &image_after.disks_after_the_observer,
                        ) {
                            ObservedImageRecord {
                                red_item: None,
                                counts: counts_of_this_state
                                    .counts_in_name_order()
                                    .map(|(name, count)| (name.to_string(), count))
                                    .collect(),
                                image_to_judge: Some(ImageToJudgeAfterTheObserver {
                                    foundation_digest: context.foundation_digest,
                                    persisted: image.persisted.clone(),
                                    writes_on_the_crash_image: image_after
                                        .writes_on_the_crash_image,
                                    invariants_judged: image_after.invariants_judged,
                                    counter_when_the_pool_checker_is_clean: image_after
                                        .counter_when_the_pool_checker_is_clean,
                                }),
                            }
                        } else {
                            ObservedImageRecord {
                                red_item: Some(observer_red_item(
                                    &observer.observer_name,
                                    "观察者交回的写叠回崩溃镜像，与它停下时的盘字节对不上（录制流漏了写）：判段重建不出它判的那张盘",
                                )),
                                counts: Vec::new(),
                                image_to_judge: None,
                            }
                        };
                        (record, witness)
                    }
                    Err(payload) => (
                        ObservedImageRecord {
                            red_item: Some(observer_red_item(
                                &observer.observer_name,
                                &text_carried_by_the_panic_payload(payload.as_ref()),
                            )),
                            counts: Vec::new(),
                            image_to_judge: None,
                        },
                        None,
                    ),
                };
                let bytes = record.to_bytes();
                let key = witness
                    .as_ref()
                    .map_or_else(key_without_deduplication, |witness| {
                        witness.observed_image_key
                    });
                let stored = match witness {
                    Some(witness) => {
                        context.store_the_image_with_its_witness(&bucket_key, witness, &bytes)
                    }
                    None => context.store_the_image(&key, &bytes),
                };
                if let Err(error) = stored {
                    piece.failure = Some(error);
                }
                piece.tally.images_observed_here += 1;
                key
            };
            ObservationOfStates::ImageObserved { deduplication_key }
        }
        ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack(_) => {
            unreachable!(
                "录入段只录 is_recorded_in_the_recording_stage 为真的那两种：record_the_observers_of_the_flow 逐观察者先挑过，判段看的那一种不交到这里"
            )
        }
    }
}

/// 观察者录入段一块正在录的时候按挂钟那一行要报的（这一块定了就不变的那几样）：哪个观察者、哪个点、块从点内第几个状态起、切成几段。
struct ObserverBlockInProgress<'names> {
    observer_name: &'names str,
    point_name: &'names str,
    block_start: u64,
    piece_count: usize,
}

/// 按挂钟那一行里这一刻现数的：这一块已经看过几个状态、这个观察者这一趟有几个桶、桶里共几条见证、共几组读集。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ObserverRecordingProgress {
    states_looked_at_in_the_block: u64,
    buckets: usize,
    witnesses: usize,
    read_set_groups: usize,
}

/// 现数各桶的见证：桶数、见证条数与读集组数各桶相加（每个桶拿一下读锁）。
fn observer_recording_progress_of_the_buckets(
    witnesses_of_each_bucket: &WitnessIndexesOfEachBucket,
    states_looked_at_in_the_block: u64,
) -> ObserverRecordingProgress {
    let witnesses = witnesses_of_each_bucket
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut progress = ObserverRecordingProgress {
        states_looked_at_in_the_block,
        buckets: witnesses.len(),
        witnesses: 0,
        read_set_groups: 0,
    };
    for index in witnesses.values() {
        let index = index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        progress.witnesses += index.witness_count();
        progress.read_set_groups += index.read_set_group_count();
    }
    progress
}

/// 观察者录入段按挂钟打的那一行：观察者、点、块起点、收齐了几段（共几段）、这一块已看过几个状态、桶数、见证条数、读集组数、开跑多久。
fn observer_recording_wall_clock_line(
    block: &ObserverBlockInProgress<'_>,
    pieces_finished: usize,
    progress: &ObserverRecordingProgress,
    elapsed: std::time::Duration,
) -> String {
    format!(
        "CRASH_AMPLIFICATION_OBSERVER_RECORDING by=wall_clock observer={} point={} block_start={} pieces_finished={}/{} states_looked_at_in_the_block={} buckets={} witnesses={} read_set_groups={} elapsed_seconds={:.1}",
        block.observer_name,
        block.point_name,
        block.block_start,
        pieces_finished,
        block.piece_count,
        progress.states_looked_at_in_the_block,
        progress.buckets,
        progress.witnesses,
        progress.read_set_groups,
        elapsed.as_secs_f64()
    )
}

/// 收齐观察者录入段一块的各段：收到一段就记下（不打行，也不重起间隔）；从上一行按挂钟的（或这一趟开跑）起 `clock` 每走过一个进度间隔，
/// 交 `write_progress_line` 一行 [`observer_recording_wall_clock_line`]（进度现数 `progress_now`）。一块录多久都至少隔一个间隔一行：
/// 段与段到得再密也不推迟这一行。看各段的线程都退出（发送端全丢掉）才交回，交回的是收到的各段（次序是到达的次序；一段是看那一段的线程交的那一份，
/// 录入段里是这一块各观察者各一段）。
fn receive_the_observer_pieces_of_one_block<PiecesOfAThread>(
    receiver: &std::sync::mpsc::Receiver<PiecesOfAThread>,
    wall_clock_lines: &mut WallClockProgressLines,
    clock: &dyn Fn() -> std::time::Duration,
    block: &ObserverBlockInProgress<'_>,
    progress_now: &dyn Fn() -> ObserverRecordingProgress,
    write_progress_line: &mut dyn FnMut(String),
) -> Vec<PiecesOfAThread> {
    let mut arrived: Vec<PiecesOfAThread> = Vec::with_capacity(block.piece_count);
    // 迭代上界：段数次收到，加上这一块录了多少个进度间隔那么多次到期；跨轮只带收到的各段与按挂钟那一行的起点
    loop {
        let wait = wall_clock_lines.wait_before_the_next_line_is_due(clock());
        match receiver.recv_timeout(wait) {
            Ok(piece) => arrived.push(piece),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let now = clock();
                if wall_clock_lines.wall_clock_line_is_due(now) {
                    write_progress_line(observer_recording_wall_clock_line(
                        block,
                        arrived.len(),
                        &progress_now(),
                        now,
                    ));
                    wall_clock_lines
                        .restart_the_interval_at_a_received_message_or_a_wall_clock_line(now);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    arrived
}

/// 观察者的录入段（「观察者改走两段」）：计划里的每个点、这台机器领的（`is_mine`）每一块，落在这个点上的每个观察者里
/// 库里这一把观察者版本录过、或这一把结论版本已经存着的不再录；还要录的几个观察者一起看这一块：切成核对线程数那么多段、一段一个线程，
/// 每个状态造一次镜像、至多恢复一次、按观察者的次序交给每个观察者（[`record_the_observer_pieces_recovering_each_state_once`]），
/// 再逐个观察者拼回一块写进库：去重之后的观察结果清单、每个状态的下标、按发布分的态数（各观察者写下的与一个观察者单独录时逐字节相同）。交回镜像的观察者看过的恢复镜像先于块落盘（块在、它引的镜像必在），
/// 被打断后重跑同一条命令只补没录的块，看过的镜像照库里的复用；每录一块之前问一次 `stop_is_requested`，叫停了就停在块边上交回
/// （`stopped_early`）。录入段不判池级 checker。
///
/// # Errors
/// 库读写失败。
///
/// # Panics
/// `versions` 与 `observers` 不一样长；看一段的核对线程自己 panic（观察者的 panic 在线程里接住，记成红项）。
pub(crate) fn record_the_observers_of_the_flow(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
    is_mine: &dyn Fn(u64) -> bool,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<ObserverRecordingTally, String> {
    let threads = checking_threads();
    record_the_observers_of_the_flow_with_threads(
        library,
        flow,
        plan,
        observers,
        versions,
        is_mine,
        stop_is_requested,
        threads,
    )
}

/// 同 [`record_the_observers_of_the_flow`]，核对线程数由调用方给：要录的一块切成这么多段、一段一个线程。
///
/// # Errors
/// 同 [`record_the_observers_of_the_flow`]。
///
/// # Panics
/// 同 [`record_the_observers_of_the_flow`]；`threads` 是 0。
#[allow(
    clippy::too_many_lines,
    reason = "建去重摘要器与基底、逐观察者逐块切段并行看、拼块写库是录入段的全部步骤，拆开要把摘要器、认领表与账来回传"
)]
#[allow(
    clippy::too_many_arguments,
    reason = "与 record_the_observers_of_the_flow 同一组参数，多一个线程数"
)]
fn record_the_observers_of_the_flow_with_threads(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
    is_mine: &dyn Fn(u64) -> bool,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
    threads: usize,
) -> Result<ObserverRecordingTally, String> {
    assert!(threads > 0, "核对线程数至少一条");
    assert_eq!(
        observers.len(),
        versions.len(),
        "每个观察者各一组版本、同序"
    );
    // 单测进程里从用例共用的那一份额度拿（[`crate::crash_library::InnerThreadsTaken`]），产品构建里照给：拿到几条就切几段
    let threads = crate::crash_library::InnerThreadsTaken::take(threads);
    let library_error = |error: VerdictStoreError| format!("观察者录入段读写库失败：{error:?}");
    let mut tally = ObserverRecordingTally::default();
    if observers.is_empty() {
        return Ok(tally);
    }
    let hands_back_an_image = observers.iter().any(|observer| match observer.observe {
        ObserverCode::CountsOnly(_)
        | ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack(_) => false,
        ObserverCode::HandsBackAnImage { .. } => true,
    });
    // 枚举的表与排好次序的状态：这一趟只建一份（第一次要录一块、或要建基底时才建），各观察者、各块、各段（各核对线程）借同一份；
    // 一份表在用过的盘上重做 mkfs 那一类流上逾 1.5 GiB，逐段各建一份时线程数那么多份同时在（那条流按甲二展开的那一条因此撞过 16G 的内存上限）
    let tables_of_the_flow = std::cell::OnceCell::new();
    let crash_states_of_the_flow = std::cell::OnceCell::new();
    let crash_states_built_once = || {
        crash_states_of_the_flow.get_or_init(|| {
            tables_of_the_flow
                .get_or_init(|| {
                    crate::crash::TornImageTablesOfTheReadBackFlow::of_the_read_back_flow(
                        flow.base,
                        flow.writes,
                        flow.segments,
                    )
                })
                .crash_states_planned_by(
                    flow.base,
                    flow.writes,
                    flow.segments,
                    &|segment_index, segment| plan.segment_expansion(flow, segment_index, segment),
                    plan.segment_slices(),
                )
        })
    };
    // 基底与去重摘要器：枚举写表与判段读回的流照同一套规则建（`crash::TornImageTablesOfTheReadBackFlow`，与展开方式无关），取上面那一份
    let enumerated_writes: &[RetainedWrite] = if hands_back_an_image {
        crash_states_built_once();
        tables_of_the_flow
            .get()
            .expect("上一句建了枚举的表")
            .enumerated_writes()
    } else {
        &[]
    };
    let foundation_digest = if hands_back_an_image {
        let bytes = observer_foundation_bytes(flow.base, enumerated_writes);
        let digest = sha256_digest(&bytes);
        library
            .take()
            .store_observer_foundation(&digest, &bytes)
            .map_err(library_error)?;
        digest
    } else {
        [0; DIGEST_BYTES]
    };
    let digester =
        hands_back_an_image.then(|| ReadSetWitnessDigester::of(flow.base, enumerated_writes));
    // 每过一个进度间隔至少打一行：一块录得比它久时，等着收各段的这条线程照样隔一个间隔打一行按挂钟的（这一块已看过几个状态、几个桶、几条见证、几组读集）
    let recording_started = std::time::Instant::now();
    let clock = || recording_started.elapsed();
    let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
    // 录入段录的观察者（判段从判器交回的逐盘槽数看的那一种录入段什么都不跑：不恢复、不调池级 checker），各一份这一趟的桶与真跑锁、跨块带着
    let recorded_observers: Vec<(&ObserverOnTheNodes<'_>, &ObserverVersions)> = observers
        .iter()
        .zip(versions)
        .filter(|(observer, _observer_versions)| {
            observer.observe.is_recorded_in_the_recording_stage()
        })
        .collect();
    let witnesses_of_each_recorded_observer: Vec<WitnessIndexesOfEachBucket> = recorded_observers
        .iter()
        .map(|_recorded| std::sync::Mutex::new(std::collections::HashMap::new()))
        .collect();
    let run_locks_of_each_recorded_observer: Vec<RunLocksOfEachBucket> = recorded_observers
        .iter()
        .map(|_recorded| std::sync::Mutex::new(std::collections::HashMap::new()))
        .collect();
    let kind = ConclusionKind::Observer;
    // 按块录：一块里每个状态造一次镜像、至多恢复一次，交给这一块上要录的每个观察者（[`record_the_observer_pieces_recovering_each_state_once`]）
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let observers_on_the_point: Vec<usize> = recorded_observers
            .iter()
            .enumerate()
            .filter(|(_position, (observer, _observer_versions))| {
                observer
                    .nodes
                    .iter()
                    .any(|node| node.holds_the_point(point))
            })
            .map(|(position, _recorded)| position)
            .collect();
        let blocks_of_the_point = point.blocks();
        let first_block_of_the_point = block_sequence;
        // 这个点上这台机器领的每一块、挂在这个点上的每个观察者，录入段那一份在不在库里一次成批读回（块序、观察者序），下面逐块逐观察者按次序取
        let read_at_the_start_of_the_point = if observers_on_the_point.is_empty() {
            Vec::new()
        } else {
            let recorded_observers_of_the_run = &recorded_observers;
            let blocks_and_observers: Vec<(VerdictBlockKey, ObserverVersions)> =
                blocks_of_the_point
                    .iter()
                    .enumerate()
                    .filter(|(offset_in_the_point, _local)| {
                        is_mine(
                            first_block_of_the_point
                                + u64::try_from(*offset_in_the_point).expect("块数装得进 u64"),
                        )
                    })
                    .flat_map(|(_offset_in_the_point, local)| {
                        let key = point.block_key(local);
                        observers_on_the_point.iter().map(move |position| {
                            (key.clone(), *recorded_observers_of_the_run[*position].1)
                        })
                    })
                    .collect();
            observer_blocks_recorded_or_concluded(&library.take(), &blocks_and_observers)
                .map_err(library_error)?
        };
        let mut read_at_the_start_of_the_point = read_at_the_start_of_the_point.into_iter();
        for local in blocks_of_the_point {
            let this_block = block_sequence;
            block_sequence += 1;
            if observers_on_the_point.is_empty() || !is_mine(this_block) {
                continue;
            }
            if stop_is_requested() {
                tally.stopped_early = true;
                return Ok(tally);
            }
            let key = point.block_key(&local);
            let mut observers_to_record: Vec<usize> =
                Vec::with_capacity(observers_on_the_point.len());
            for position in &observers_on_the_point {
                let at_the_read = read_at_the_start_of_the_point
                    .next()
                    .expect("这台机器领的每一块、点上每个观察者各读了一项，次序相同");
                let observer_versions = recorded_observers[*position].1;
                let is_forced_now = library
                    .take()
                    .conclusion_block_is_forced_to_be_judged_again(
                        &key,
                        &kind,
                        &observer_versions.conclusion,
                    );
                let is_recorded_or_concluded =
                    if is_forced_now == at_the_read.was_forced_at_the_read {
                        at_the_read.is_recorded_or_concluded
                    } else {
                        observer_block_is_recorded_or_concluded(
                            &library.take(),
                            &key,
                            observer_versions,
                        )
                        .map_err(library_error)?
                    };
                if is_recorded_or_concluded {
                    tally.blocks_already_recorded += 1;
                } else {
                    observers_to_record.push(*position);
                }
            }
            if observers_to_record.is_empty() {
                continue;
            }
            let piece_length = (local.end - local.start)
                .div_ceil(u64::try_from(threads.threads()).expect("段数装得进 u64"))
                .max(1);
            let mut pieces: Vec<Range<u64>> = Vec::new();
            let mut start = local.start;
            while start < local.end {
                let end = (start + piece_length).min(local.end);
                pieces.push(start..end);
                start = end;
            }
            let states_looked_at_in_the_block = std::sync::atomic::AtomicU64::new(0);
            let states_recovered_in_the_block = std::sync::atomic::AtomicU64::new(0);
            let contexts: Vec<ObserverRecordingContext<'_, '_>> = observers_to_record
                .iter()
                .map(|position| ObserverRecordingContext {
                    crash_states_of_the_flow: crash_states_built_once(),
                    digester: digester.as_ref(),
                    foundation_digest,
                    witnesses_of_each_bucket: &witnesses_of_each_recorded_observer[*position],
                    run_locks_of_each_bucket: &run_locks_of_each_recorded_observer[*position],
                    store: library,
                    states_looked_at_in_the_block: &states_looked_at_in_the_block,
                })
                .collect();
            let recordings: Vec<ObserverRecordingOfTheBlock<'_, '_, '_>> = observers_to_record
                .iter()
                .zip(&contexts)
                .map(|(position, context)| ObserverRecordingOfTheBlock {
                    observer: recorded_observers[*position].0,
                    versions: recorded_observers[*position].1,
                    context,
                })
                .collect();
            let names_of_the_observers_recorded: String = observers_to_record
                .iter()
                .map(|position| recorded_observers[*position].0.observer_name.as_str())
                .collect::<Vec<&str>>()
                .join("+");
            let piece_count = pieces.len();
            let recorded: Vec<Vec<Result<RecordedObserverPiece, String>>> =
                std::thread::scope(|scope| {
                    let (piece_sender, piece_receiver) =
                        std::sync::mpsc::channel::<Vec<Result<RecordedObserverPiece, String>>>();
                    let handles: Vec<_> = pieces
                        .iter()
                        .cloned()
                        .map(|piece| {
                            let recordings = &recordings;
                            let block = &local;
                            let states_recovered_in_the_block = &states_recovered_in_the_block;
                            let piece_sender = piece_sender.clone();
                            scope.spawn(move || {
                                let recorded_pieces =
                                    record_the_observer_pieces_recovering_each_state_once(
                                        plan,
                                        point_index,
                                        block,
                                        piece,
                                        recordings,
                                        states_recovered_in_the_block,
                                    );
                                // 发不出去说明收的这条线程已经走了（它 panic 了），这一段的结果没人要
                                let _ = piece_sender.send(recorded_pieces);
                            })
                        })
                        .collect();
                    // 只留看各段的线程手里的发送端：它们都退出之后（录完或 panic），下面的接收才结束
                    drop(piece_sender);
                    let arrived = receive_the_observer_pieces_of_one_block(
                        &piece_receiver,
                        &mut wall_clock_lines,
                        &clock,
                        &ObserverBlockInProgress {
                            observer_name: &names_of_the_observers_recorded,
                            point_name: &point.name,
                            block_start: local.start,
                            piece_count,
                        },
                        &|| {
                            let mut progress = ObserverRecordingProgress {
                                states_looked_at_in_the_block: states_looked_at_in_the_block
                                    .load(std::sync::atomic::Ordering::Relaxed),
                                buckets: 0,
                                witnesses: 0,
                                read_set_groups: 0,
                            };
                            for position in &observers_to_record {
                                let of_the_observer = observer_recording_progress_of_the_buckets(
                                    &witnesses_of_each_recorded_observer[*position],
                                    0,
                                );
                                progress.buckets += of_the_observer.buckets;
                                progress.witnesses += of_the_observer.witnesses;
                                progress.read_set_groups += of_the_observer.read_set_groups;
                            }
                            progress
                        },
                        &mut |line| eprintln!("{line}"),
                    );
                    // 看一段的线程 panic 时把它的原话往外传（观察者自己的 panic 在线程里接住、记成红项，到这里的是录入段的 bug）
                    for handle in handles {
                        handle
                            .join()
                            .unwrap_or_else(|payload| std::panic::resume_unwind(payload));
                    }
                    arrived
                });
            assert_eq!(
                recorded.len(),
                piece_count,
                "这一块的每一段都录完交回了（看一段的线程 panic 时上面已经往外传）"
            );
            tally.states_recovered_here += states_recovered_in_the_block.into_inner();
            // 各段交回的是这一块各观察者各一段（与 observers_to_record 同序）：按观察者拆开，逐个观察者拼块、写库
            let mut pieces_of_each_observer: Vec<Vec<Result<RecordedObserverPiece, String>>> =
                observers_to_record
                    .iter()
                    .map(|_position| Vec::with_capacity(piece_count))
                    .collect();
            for pieces_of_a_thread in recorded {
                assert_eq!(
                    pieces_of_a_thread.len(),
                    observers_to_record.len(),
                    "看一段的线程交回这一块每个要录的观察者各一段"
                );
                for (pieces_of_the_observer, recorded_piece) in
                    pieces_of_each_observer.iter_mut().zip(pieces_of_a_thread)
                {
                    pieces_of_the_observer.push(recorded_piece);
                }
            }
            for (position, pieces_of_the_observer) in
                observers_to_record.iter().zip(pieces_of_each_observer)
            {
                let (observer, observer_versions) = recorded_observers[*position];
                let name = observer.observer_name.as_str();
                let mut pieces_in_block_order: Vec<RecordedObserverPiece> = pieces_of_the_observer
                    .into_iter()
                    .collect::<Result<_, _>>()?;
                pieces_in_block_order.sort_by_key(|piece| piece.local.start);
                let mut observations: Vec<ObservationOfStates> = Vec::new();
                let mut index_of_each_observation: std::collections::HashMap<
                    ObservationOfStates,
                    u32,
                > = std::collections::HashMap::new();
                let mut observation_of_each_state: Vec<u32> = Vec::with_capacity(
                    usize::try_from(local.end - local.start).expect("一块装得进 usize"),
                );
                let mut state_counts = StateCountsOfABlock::default();
                let mut next_expected = local.start;
                for piece in pieces_in_block_order {
                    assert_eq!(piece.local.start, next_expected, "块的段没有接上");
                    next_expected = piece.local.end;
                    for observation in piece.observation_of_each_state {
                        let index = *index_of_each_observation
                            .entry(observation.clone())
                            .or_insert_with(|| {
                                observations.push(observation);
                                u32::try_from(observations.len() - 1)
                                    .expect("一块最多 2^16 个状态，观察结果的清单装得进 u32")
                            });
                        observation_of_each_state.push(index);
                    }
                    state_counts.absorb(piece.state_counts);
                    tally.absorb(&piece.tally);
                }
                assert_eq!(next_expected, local.end, "块的段没有拼满");
                let block_record = ObserverBlockRecord {
                    observations,
                    observation_of_each_state,
                    count_vector: state_counts.into_count_vector(local.end - local.start),
                };
                let block_record_bytes = block_record.to_bytes();
                library
                    .take()
                    .store_observer_block(
                        &key,
                        &kind,
                        &observer_versions.observer,
                        &block_record_bytes,
                    )
                    .map_err(library_error)?;
                tally.blocks_recorded_here += 1;
                tally.states_recorded_here += local.end - local.start;
                let now = clock();
                if wall_clock_lines.wall_clock_line_is_due(now) {
                    wall_clock_lines
                        .restart_the_interval_at_a_received_message_or_a_wall_clock_line(now);
                    // 读集组数（各桶相加）：匹配一个状态最多算这一桶的组数那么多次摘要
                    let progress = observer_recording_progress_of_the_buckets(
                        &witnesses_of_each_recorded_observer[*position],
                        local.end - local.start,
                    );
                    eprintln!(
                        "CRASH_AMPLIFICATION_OBSERVER_RECORDING observer={name} point={} block_start={} blocks_recorded_here={} states_recorded_here={} images_observed_here={} states_reusing_an_observed_image={} buckets={} witnesses={} witness_misses={} seconds_recording={:.0} read_set_groups={} states_recovered_here={}",
                        point.name,
                        local.start,
                        tally.blocks_recorded_here,
                        tally.states_recorded_here,
                        tally.images_observed_here,
                        tally.states_reusing_an_observed_image,
                        progress.buckets,
                        progress.witnesses,
                        tally.witness_misses,
                        now.as_secs_f64(),
                        progress.read_set_groups,
                        tally.states_recovered_here
                    );
                }
            }
        }
    }
    for witnesses_of_each_bucket in &witnesses_of_each_recorded_observer {
        tally.buckets += u64::try_from(
            witnesses_of_each_bucket
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len(),
        )
        .expect("桶数装得进 u64");
    }
    Ok(tally)
}

/// 一个观察者在一块上的录入段那一份在不在库里：这一把观察者版本录过，或这一把结论版本已经存着（结论存着的块判段不再读录入块）。
/// 录入段（[`record_the_observers_of_the_flow`]）照它跳过录过的块，录制复用之前的核对（[`observers_whose_recorded_blocks_are_missing`]）
/// 照它点名缺块的观察者：两处一个判法。这一把结论版本存着而记着红（结论里有违例）的答不在：
/// 观察者那一块重录、判段重拼，照新结论写回（「记红、撤绿、下趟现判」；判段那一半见 [`observer_conclusion_is_reused`]）。
/// 复用哨兵这一趟抽中强制现判的那一份（`VerdictStore::conclusion_block_is_forced_to_be_judged_again`）只认录入段那一份：判段要现拼它，
/// 结论存着不算录过（[`record_the_observers_of_the_forced_block_again`]）。
fn observer_block_is_recorded_or_concluded(
    store: &VerdictStore,
    key: &VerdictBlockKey,
    observer_versions: &ObserverVersions,
) -> Result<bool, VerdictStoreError> {
    let kind = ConclusionKind::Observer;
    let is_forced_to_be_judged_again = store.conclusion_block_is_forced_to_be_judged_again(
        key,
        &kind,
        &observer_versions.conclusion,
    );
    if is_forced_to_be_judged_again {
        return Ok(store
            .read_observer_block(key, &kind, &observer_versions.observer)?
            .is_some());
    }
    match store.read_conclusion_block(key, &kind, &observer_versions.conclusion)? {
        Some(stored) if stored.violations.is_empty() => return Ok(true),
        Some(_stored_red) => return Ok(false),
        None => {}
    }
    Ok(store
        .read_observer_block(key, &kind, &observer_versions.observer)?
        .is_some())
}

/// 一块一个观察者的录入段那一份在不在库里，成批读回的一项（[`observer_blocks_recorded_or_concluded`]）：读的那一刻这一份在不在强制现判表里，
/// 与照那一刻读到的库答的在不在。用的时候强制现判表与读的那一刻不同了（同一条连接上别的趟添了、撤了），照 [`observer_block_is_recorded_or_concluded`] 现问。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ObserverBlockRecordedOrConcludedAtTheRead {
    was_forced_at_the_read: bool,
    is_recorded_or_concluded: bool,
}

/// [`observer_block_is_recorded_or_concluded`] 的成批版：几块几个观察者（`blocks_and_observers`，交回的次序与它相同）一起问，不逐块读结论块。
/// 不在强制现判表里的先成批问观察者结论存没存、红不红（[`VerdictStore::conclusion_blocks_stored_and_red`]：块属性那一行与违例表这一块的前缀），
/// 存着不红答在、存着红答不在；没存的与在强制现判表里的，再成批读观察者录入块那一行（[`VerdictStore::observer_block_rows_stored`]）答在不在。
/// 与逐块问的是同一个谓词：逐块问看结论块那一行在不在、违例表有没有行，块属性与结论块同一批写（`put_conclusion_rows_into_the_batch`），
/// 库里两样同在同不在；只在两样不同在的坏库上答的不同（逐块问报结论块缺行的错，这里照块属性那一行答），判段读这一份结论时照样报那个错。
///
/// # Errors
/// 库读失败；块属性那一行不合布局。
fn observer_blocks_recorded_or_concluded(
    store: &VerdictStore,
    blocks_and_observers: &[(VerdictBlockKey, ObserverVersions)],
) -> Result<Vec<ObserverBlockRecordedOrConcludedAtTheRead>, VerdictStoreError> {
    let kind = ConclusionKind::Observer;
    let was_forced_at_the_read: Vec<bool> = blocks_and_observers
        .iter()
        .map(|(key, observer_versions)| {
            store.conclusion_block_is_forced_to_be_judged_again(
                key,
                &kind,
                &observer_versions.conclusion,
            )
        })
        .collect();
    let conclusions_not_forced: Vec<crate::verdict_store::ConclusionOfABlockUnderAKey> =
        blocks_and_observers
            .iter()
            .zip(&was_forced_at_the_read)
            .filter(|(_block_and_observer, was_forced)| !**was_forced)
            .map(|((key, observer_versions), _was_forced)| {
                (key.clone(), kind.clone(), observer_versions.conclusion)
            })
            .collect();
    let mut conclusions_read = store
        .conclusion_blocks_stored_and_red(&conclusions_not_forced)?
        .into_iter();
    // 每一项：结论定下来的是 Some(答案)，在强制现判表里的与结论没存的是 None（看录入块那一行）
    let mut decided_by_the_conclusion: Vec<Option<bool>> =
        Vec::with_capacity(blocks_and_observers.len());
    for was_forced in &was_forced_at_the_read {
        if *was_forced {
            decided_by_the_conclusion.push(None);
            continue;
        }
        decided_by_the_conclusion.push(
            match conclusions_read
                .next()
                .expect("不在强制现判表里的每一项各问了一次结论")
            {
                ConclusionBlockInTheLibrary::StoredGreen => Some(true),
                ConclusionBlockInTheLibrary::StoredRed => Some(false),
                ConclusionBlockInTheLibrary::NotStored => None,
            },
        );
    }
    let observer_rows_to_read: Vec<crate::verdict_store::ConclusionOfABlockUnderAKey> =
        blocks_and_observers
            .iter()
            .zip(&decided_by_the_conclusion)
            .filter(|(_block_and_observer, decided)| decided.is_none())
            .map(|((key, observer_versions), _decided)| {
                (key.clone(), kind.clone(), observer_versions.observer)
            })
            .collect();
    let mut observer_rows_stored = store
        .observer_block_rows_stored(&observer_rows_to_read)?
        .into_iter();
    Ok(was_forced_at_the_read
        .into_iter()
        .zip(decided_by_the_conclusion)
        .map(
            |(was_forced, decided)| ObserverBlockRecordedOrConcludedAtTheRead {
                was_forced_at_the_read: was_forced,
                is_recorded_or_concluded: decided.unwrap_or_else(|| {
                    observer_rows_stored
                        .next()
                        .expect("结论没定下来的每一项各读了一行录入块")
                }),
            },
        )
        .collect())
}

/// 复用哨兵这一趟抽中强制现判的那一块（[`plan_the_reuse_sentinels`] 照种子抽、不看录入段那一行在不在），判段重拼它上面挂着的观察者结论
/// 要读的那一行（[`ObserverForTheReuseSentinels::input_row`]）有不在库里的（录入段见观察者结论在、不红就不录那一块）：回录入段，只给这一块录
/// （「抽中库里没有录入块的块，回录入段重录，不丢掉再抽」）。录入段照 [`observer_block_is_recorded_or_concluded`]
/// 认这一块没录过（强制现判的那一份只认录入段那一份），看的是从库里读回的流，与判段读的同一份。抽哨兵之后、判段之前调。交回录了没有。
///
/// # Errors
/// 库读写失败。
pub(crate) fn record_the_observers_of_the_forced_block_again(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
    sentinels: &ReuseSentinelsOfTheRun,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<bool, String> {
    let Some((forced, _stored_before_judging)) = &sentinels.forced else {
        return Ok(false);
    };
    if observer_inputs_of_the_block_are_stored(
        &library.take(),
        &observers_for_the_reuse_sentinels(plan, observers, versions),
        forced.point_index,
        &forced.key.0,
    )
    .map_err(|error| format!("复用哨兵：读强制现判那一块的录入段失败：{error:?}"))?
    {
        return Ok(false);
    }
    let forced_block = forced.block_sequence;
    record_the_observers_of_the_flow(
        library,
        flow,
        plan,
        observers,
        versions,
        &|block| block == forced_block,
        stop_is_requested,
    )?;
    Ok(true)
}

/// 判段补录照判不入库的观察者：版本登记成照判不入库的观察者（判法单元拒算，取了这个进程的占位，
/// [`observer_versions_keeping_the_refused_out`]），它录入段的块只留在录它的那个进程的库连接里、不进长期库；录入与判段各在一个进程里跑时
/// （门禁按需领卡那一路先起一个只录的进程、再起一个只判的），判段这个进程照自己的占位读不到。这里在判段进程里核这几个观察者
/// 在这台机器领的块上录入段那一份都在（[`observers_whose_recorded_blocks_are_missing`]），缺块的照这个进程的占位补录
/// （[`record_the_observers_of_the_flow`]，看的是从库里读回的流，与判段读的同一份），录进的行照旧照判不入库。
/// 录入与判段在同一个进程里跑的那一趟块都在，一块都不补；观察者没有照判不入库的那一把，一块都不补。交回补录的账。
///
/// # Errors
/// 库读写失败。
pub(crate) fn record_the_observers_kept_out_of_the_library_again(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
    is_mine: &dyn Fn(u64) -> bool,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<ObserverRecordingTally, String> {
    let versions_kept_out = library.take().versions_kept_out_of_the_library();
    let (observers_kept_out, versions_of_the_observers_kept_out): (
        Vec<ObserverOnTheNodes<'_>>,
        Vec<ObserverVersions>,
    ) = observers
        .iter()
        .zip(versions)
        .filter(|(_observer, observer_versions)| {
            versions_kept_out.contains(&observer_versions.observer.0)
        })
        .map(|(observer, observer_versions)| (observer.clone(), *observer_versions))
        .unzip();
    let missing = observers_whose_recorded_blocks_are_missing(
        &library.take(),
        plan,
        &observers_kept_out,
        &versions_of_the_observers_kept_out,
        is_mine,
    )?;
    let (observers_to_record, versions_to_record): (
        Vec<ObserverOnTheNodes<'_>>,
        Vec<ObserverVersions>,
    ) = missing
        .into_iter()
        .map(|position| {
            (
                observers_kept_out[position].clone(),
                versions_of_the_observers_kept_out[position],
            )
        })
        .unzip();
    record_the_observers_of_the_flow(
        library,
        flow,
        plan,
        &observers_to_record,
        &versions_to_record,
        is_mine,
        stop_is_requested,
    )
}

/// 判段：一个观察者在一块上这一把结论版本的结论照不照库里的复用。库里没有、或记着红（结论里有违例）的不复用：
/// 现拼、照新结论写回（「记红、撤绿、下趟现判」；存块那一处两份有一份红就换上）。复用哨兵这一趟抽中强制现判的
/// 那一份同样不复用（[`plan_the_reuse_sentinels`] 把它登进库的强制现判表）。
pub(super) fn observer_conclusion_is_reused(
    store: &VerdictStore,
    key: &VerdictBlockKey,
    observer_conclusion_version: &ConclusionVersion,
) -> Result<bool, VerdictStoreError> {
    if store.conclusion_block_is_forced_to_be_judged_again(
        key,
        &ConclusionKind::Observer,
        observer_conclusion_version,
    ) {
        return Ok(false);
    }
    Ok(store
        .read_conclusion_block(key, &ConclusionKind::Observer, observer_conclusion_version)?
        .is_some_and(|stored| stored.violations.is_empty()))
}

/// 录制复用之前的核对：流的录入键不罩观察者（观察者的版本不进录入键，录入段命中时录入那一段整段不跑），所以命中之后逐个观察者核
/// 计划里落在它节点上、这台机器领的（`is_mine`）每一块，录入段那一份都在库里（[`observer_block_is_recorded_or_concluded`]）；
/// 交回缺块的那几个观察者在 `observers` 里的下标（从小到大），调用方只给它们补录。
///
/// # Errors
/// 库读失败。
///
/// # Panics
/// `versions` 与 `observers` 不一样长。
pub(crate) fn observers_whose_recorded_blocks_are_missing(
    store: &VerdictStore,
    plan: &CrashPlan,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
    is_mine: &dyn Fn(u64) -> bool,
) -> Result<Vec<usize>, String> {
    assert_eq!(
        observers.len(),
        versions.len(),
        "每个观察者各一组版本、同序"
    );
    let mut missing = Vec::new();
    // 迭代上界：观察者个数；每个观察者里是计划的点数乘每点的块数，遇到第一块缺的就不再往下核这个观察者
    for (position, (observer, observer_versions)) in observers.iter().zip(versions).enumerate() {
        // 录入段不录的那一种（判段从判器交回的逐盘槽数看）没有录入块可缺
        if !observer.observe.is_recorded_in_the_recording_stage() {
            continue;
        }
        let mut block_sequence = 0u64;
        let mut some_block_is_missing = false;
        'points: for point in &plan.points {
            let is_observed = observer
                .nodes
                .iter()
                .any(|node| node.holds_the_point(point));
            for local in point.blocks() {
                let this_block = block_sequence;
                block_sequence += 1;
                if !is_observed || !is_mine(this_block) {
                    continue;
                }
                if !observer_block_is_recorded_or_concluded(
                    store,
                    &point.block_key(&local),
                    observer_versions,
                )
                .map_err(|error| format!("录制复用之前核观察者的块读库失败：{error:?}"))?
                {
                    some_block_is_missing = true;
                    break 'points;
                }
            }
        }
        if some_block_is_missing {
            missing.push(position);
        }
    }
    Ok(missing)
}

#[cfg(test)]
mod tests;
