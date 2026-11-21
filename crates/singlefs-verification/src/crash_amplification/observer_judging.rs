//! 判观察者：判段里读回观察者录下的块、判它们的盘、拼观察者结论（`judge_the_observers_of_the_flow`），之后判复用哨兵。

use super::*;

/// 观察者判段与判复用哨兵的次序，两处跑流的判段（崩溃放量流水线 [`run_from_environment_with_observers`]、崩溃枚举用例的判段）共用这一处：
/// 观察者结论在判器结论之后判（观察者判段读判器交回的那一份），哨兵要连观察者结论一起核（强制现判那一块上的观察者结论这一趟要交来存过），
/// 所以先判观察者（[`judge_the_observers_of_the_flow`]）、再判哨兵（[`ReuseSentinelsOfTheRun::problems_after_judging`]，它清掉强制现判表）。
/// 判观察者之前先补录照判不入库的观察者（[`record_the_observers_kept_out_of_the_library_again`]：录入段在别的进程里跑的那一趟，
/// 它们的录入块不在这个进程读得到的库里）；补录被叫停就不判观察者、撤掉这一趟的强制现判，交回叫停那一句。
/// 不判观察者的那一路（流水线没挂观察者）交 `None`。交回观察者判段的结果、哨兵判出的红（一句一处；没抽哨兵是空的）与补录的账
/// （不判观察者的那一路是全零的账）。
///
/// # Errors
/// 补录与观察者判段的错；补录被叫停；判哨兵读库失败。
pub(crate) fn judge_the_observers_then_the_reuse_sentinels(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    observers_with_their_versions_and_pool_checker: Option<(
        &[ObserverOnTheNodes<'_>],
        &[ObserverVersions],
        &PoolCheckerOfTheImagesAfterTheObservers,
    )>,
    is_mine: &dyn Fn(u64) -> bool,
    reuse_sentinels: Option<&ReuseSentinelsOfTheRun>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<
    (
        Option<ObservingOfTheRun>,
        Vec<String>,
        ObserverRecordingTally,
    ),
    String,
> {
    let mut recorded_again = ObserverRecordingTally::default();
    let observing = match observers_with_their_versions_and_pool_checker {
        Some((observers, versions, pool_checker)) => {
            recorded_again = record_the_observers_kept_out_of_the_library_again(
                library,
                flow,
                plan,
                observers,
                versions,
                is_mine,
                stop_is_requested,
            )?;
            if recorded_again.stopped_early {
                if let Some(sentinels) = reuse_sentinels {
                    sentinels.stop_forcing_the_forced_block(&mut library.take());
                }
                return Err(
                    "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE）：判段补录照判不入库的观察者停在块边上，观察者这一趟没判，重跑同一条命令接着录、接着判"
                        .to_string(),
                );
            }
            Some(judge_the_observers_of_the_flow(
                library,
                plan,
                observers,
                versions,
                pool_checker,
                is_mine,
            )?)
        }
        None => None,
    };
    let problems = match reuse_sentinels {
        Some(sentinels) => sentinels
            .problems_after_judging(&mut library.take())
            .map_err(|error| format!("判复用哨兵读库失败：{error:?}"))?,
        None => Vec::new(),
    };
    Ok((observing, problems, recorded_again))
}

/// 观察者的判段：每个观察者、计划里落在它节点上的每个点、这台机器领的（`is_mine`）每一块，库里这一把结论版本存着的
/// 照复用；没有的读录入段那一块（没有就报「先跑录入段」；这个观察者那一把版本照判不入库的，报它为什么照判不入库）、读它引的看过的恢复镜像，观察者交回的盘里这一把判器版本还没判过的
/// 交 `pool_checker` 判（同一张盘只判一次）、结论写进库，再逐状态拼回一块存：判定向量（观察者 panic 的那一句、盘上池级 checker 的违例与判不了）、
/// 违例、计数向量（观察者计数加判干净那一个计数、按发布分的态数；不变量那一节空着）、在哪判的（盘上的池级 checker 在哪判的；只记计数的观察者
/// 是 CPU）。存完把这条流挂了观察者的每一块（这一趟拼的与复用的）读回来，计数按观察者合起来交回，调用方照它判观察者计数（J7，每一趟都判）。
/// GPU 判不了的盘（建不出判表、表越过内核容量）照样判红，它的那一句与带着它的那一块都不存进库（不是判器的结论，下一趟重判，
/// [`ImageJudgementRecord::is_a_conclusion_of_the_judge`]）；库里早先存着的「判不了」也不复用。
/// 这里不跑恢复、不调观察者、不收用例内存里的流（照判不入库的观察者在判段进程里补录，在调这里之前，[`record_the_observers_kept_out_of_the_library_again`]）。
///
/// # Errors
/// 库读写失败；录入段没录某一块、或它引的恢复镜像与基底不在库里（先跑录入段）；判定向量超过 2^32 种；配了 GPU 而卡用不上。
///
/// # Panics
/// `versions` 与 `observers` 不一样长。
#[allow(
    clippy::too_many_lines,
    reason = "读录入块与镜像、判盘、写判定、逐状态拼块、读回合计是判段的全部步骤，拆开要把几张表与账来回传"
)]
pub(crate) fn judge_the_observers_of_the_flow(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    plan: &CrashPlan,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
    pool_checker: &PoolCheckerOfTheImagesAfterTheObservers,
    is_mine: &dyn Fn(u64) -> bool,
) -> Result<ObservingOfTheRun, String> {
    assert_eq!(
        observers.len(),
        versions.len(),
        "每个观察者各一组版本、同序"
    );
    let library_error = |error: VerdictStoreError| format!("观察者判段读写库失败：{error:?}");
    let mut image_judgements_judged_in_this_run: BTreeMap<
        [u8; DIGEST_BYTES],
        ImageJudgementRecord,
    > = BTreeMap::new();
    let mut observing = ObservingOfTheRun::default();
    // 这台机器领的块落在哪几个点上（不看挂没挂观察者）：调用方照它分「这一片里没有某个观察者节点上的块」与「有块它却一个状态都没看」
    let mut block_sequence_of_the_plan = 0u64;
    for point in &plan.points {
        let mut blocks = 0u64;
        for _local in point.blocks() {
            if is_mine(block_sequence_of_the_plan) {
                blocks += 1;
            }
            block_sequence_of_the_plan += 1;
        }
        if blocks > 0 {
            observing
                .blocks_selected_here_on_each_point
                .push(BlocksSelectedHereOnAPoint {
                    code_file: point_code_file(point).to_string(),
                    point_name: point.name.clone(),
                    blocks,
                });
        }
    }
    let observing_started = std::time::Instant::now();
    let mut progress_last_printed = std::time::Instant::now();
    let mut foundations: BTreeMap<[u8; DIGEST_BYTES], ObserverFoundation> = BTreeMap::new();
    for (observer, observer_versions) in observers.iter().zip(versions) {
        assert_eq!(
            observer_versions.pool_checker,
            pool_checker.judged_by(),
            "观察者那一把池级 checker 的版本按这一趟判盘的那一路取"
        );
        let kind = ConclusionKind::Observer;
        let pool_checker_kind = observer_versions.pool_checker.conclusion_kind();
        let name = observer.observer_name.as_str().to_string();
        let observer_is_kept_out_of_the_library = library
            .take()
            .versions_kept_out_of_the_library()
            .contains(&observer_versions.observer.0);
        let mut block_sequence = 0u64;
        for point in &plan.points {
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
                let key = point.block_key(&local);
                let already_stored = observer_conclusion_is_reused(
                    &library.take(),
                    &key,
                    &observer_versions.conclusion,
                )
                .map_err(library_error)?;
                // 复用哨兵抽中强制现判的这一块：块里各张盘上的池级 checker 判定也一起现判、不取库里的
                let is_forced_to_be_judged_again = library
                    .take()
                    .conclusion_block_is_forced_to_be_judged_again(
                        &key,
                        &kind,
                        &observer_versions.conclusion,
                    );
                // 这一趟拼出的那一块（存没存进库都一样）：收尾照它记，不从库里读回刚存的那一份
                let mut composed_here: Option<ConclusionBlockRecord> = None;
                if !already_stored {
                    let block_record = match &observer.observe {
                        ObserverCode::CountsOnly(_) | ObserverCode::HandsBackAnImage { .. } => {
                            let block_record = library
                                .take()
                                .read_observer_block(&key, &kind, &observer_versions.observer)
                                .map_err(library_error)?
                                .ok_or_else(|| {
                                    if observer_is_kept_out_of_the_library {
                                        format!(
                                            "观察者 {name} 在点 {} 块 {} 的录入段那一块不在库里：{}，录入块照判不入库、只在录它的那个进程里，判段补录过仍读不到",
                                            point.name,
                                            local.start,
                                            why_the_observer_is_kept_out_of_the_library(observer)
                                        )
                                    } else {
                                        format!(
                                            "观察者 {name} 在点 {} 块 {} 的录入段那一块不在库里：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
                                            point.name, local.start
                                        )
                                    }
                                })?;
                            ObserverBlockRecord::from_bytes(&block_record).ok_or_else(|| {
                                format!(
                                    "观察者 {name} 在点 {} 块 {} 的录入块解不出",
                                    point.name, local.start
                                )
                            })?
                        }
                        ObserverCode::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack(
                            observe,
                        ) => observer_block_from_the_self_verified_slots_the_judge_handed_back(
                            &library.take(),
                            &key,
                            local.end - local.start,
                            &observer.observer_name,
                            *observe,
                            &pool_checker_kind,
                            &observer_versions.pool_checker_version,
                        )
                        .map_err(|reason| {
                            format!(
                                "观察者 {name} 在点 {} 块 {}：{reason}",
                                point.name, local.start
                            )
                        })?,
                    };
                    // 块里每一份观察结果读回：只记计数的照块里存的；交回镜像的读看过的恢复镜像那一份（这一块引的几张一帧成批读）
                    let keys_of_the_observed_images: Vec<[u8; DIGEST_BYTES]> = block_record
                        .observations
                        .iter()
                        .filter_map(|observation| match observation {
                            ObservationOfStates::Counted { .. } => None,
                            ObservationOfStates::ImageObserved { deduplication_key } => {
                                Some(*deduplication_key)
                            }
                        })
                        .collect();
                    let mut observed_images_read = library
                        .take()
                        .read_observed_images_in_one_read(&keys_of_the_observed_images)
                        .map_err(library_error)?
                        .into_iter();
                    let mut image_records: Vec<Option<ObservedImageRecord>> =
                        Vec::with_capacity(block_record.observations.len());
                    for observation in &block_record.observations {
                        image_records.push(match observation {
                            ObservationOfStates::Counted { .. } => None,
                            ObservationOfStates::ImageObserved { deduplication_key } => {
                                let bytes = observed_images_read
                                    .next()
                                    .expect("每张交回镜像的观察结果成批读里各问了一键、同序交回")
                                    .ok_or_else(|| {
                                        format!(
                                            "观察者 {name} 的录入块引的恢复镜像 {} 不在库里：先跑录入段",
                                            hexadecimal_text(&deduplication_key[..8])
                                        )
                                    })?;
                                Some(ObservedImageRecord::from_bytes(&bytes).ok_or_else(|| {
                                    format!(
                                        "观察者 {name} 看过的恢复镜像 {} 解不出",
                                        hexadecimal_text(&deduplication_key[..8])
                                    )
                                })?)
                            }
                        });
                    }
                    for record in image_records.iter().flatten() {
                        if let Some(image) = &record.image_to_judge {
                            if let std::collections::btree_map::Entry::Vacant(vacant) =
                                foundations.entry(image.foundation_digest)
                            {
                                let bytes = library
                                    .take()
                                    .read_observer_foundation(&image.foundation_digest)
                                    .map_err(library_error)?
                                    .ok_or_else(|| "观察者基底不在库里：先跑录入段".to_string())?;
                                vacant.insert(
                                    ObserverFoundation::from_bytes(&bytes)
                                        .ok_or_else(|| "观察者基底解不出".to_string())?,
                                );
                            }
                        }
                    }
                    // 盘上的池级 checker：这一把判器版本判过的取库里的（这一块要判的几张一帧成批读），没判过的一批交 pool_checker 判、写进库
                    let mut judgement_of_each_observation: Vec<Option<ImageJudgementRecord>> =
                        vec![None; block_record.observations.len()];
                    let mut to_judge: Vec<(usize, [u8; DIGEST_BYTES])> = Vec::new();
                    let images_with_a_disk_to_judge: Vec<(usize, [u8; DIGEST_BYTES])> =
                        block_record
                            .observations
                            .iter()
                            .zip(&image_records)
                            .enumerate()
                            .filter_map(|(position, (observation, record))| {
                                match (observation, record) {
                                    (
                                        ObservationOfStates::ImageObserved { deduplication_key },
                                        Some(record),
                                    ) if record.image_to_judge.is_some() => {
                                        Some((position, *deduplication_key))
                                    }
                                    (ObservationOfStates::ImageObserved { .. }, _)
                                    | (ObservationOfStates::Counted { .. }, _) => None,
                                }
                            })
                            .collect();
                    let stored_judgements_read = library
                        .take()
                        .read_observed_image_judgements_in_one_read(
                            &images_with_a_disk_to_judge
                                .iter()
                                .map(|(_position, deduplication_key)| *deduplication_key)
                                .collect::<Vec<_>>(),
                            &pool_checker_kind,
                            &observer_versions.pool_checker_version,
                        )
                        .map_err(library_error)?;
                    for ((position, deduplication_key), stored_judgement_bytes) in
                        images_with_a_disk_to_judge
                            .iter()
                            .zip(stored_judgements_read)
                    {
                        let (position, deduplication_key) = (*position, deduplication_key);
                        match stored_judgement_bytes {
                            Some(bytes) => {
                                let stored_judgement = ImageJudgementRecord::from_bytes(&bytes)
                                    .ok_or_else(|| {
                                        "观察者交回的盘上池级 checker 的结论解不出".to_string()
                                    })?;
                                // 早先存进库的「GPU 判不了」不是判器的结论：不复用，这一趟重判。记着红的（判出违例）不复用：
                                // 这一趟现判（「记红、撤绿、下趟现判」），这一趟已经现判过的同一张镜像照这一趟的结论。
                                // 强制现判那一块上的盘同样不取库里的
                                if !is_forced_to_be_judged_again
                                    && stored_judgement.is_a_conclusion_of_the_judge()
                                    && !stored_judgement.is_red()
                                {
                                    judgement_of_each_observation[position] =
                                        Some(stored_judgement);
                                    observing.image_judgements_reused += 1;
                                } else if let Some(judged_in_this_run) =
                                    image_judgements_judged_in_this_run.get(deduplication_key)
                                {
                                    judgement_of_each_observation[position] =
                                        Some(judged_in_this_run.clone());
                                } else {
                                    to_judge.push((position, *deduplication_key));
                                }
                            }
                            None => to_judge.push((position, *deduplication_key)),
                        }
                    }
                    if !to_judge.is_empty() {
                        let images: Vec<ImageAfterTheObserverToJudge<'_>> = to_judge
                            .iter()
                            .map(|(position, deduplication_key)| {
                                let image = image_records[*position]
                                    .as_ref()
                                    .and_then(|record| record.image_to_judge.as_ref())
                                    .expect("要判的盘挑的都是带盘的那几份");
                                ImageAfterTheObserverToJudge {
                                    deduplication_key: *deduplication_key,
                                    foundation: foundations
                                        .get(&image.foundation_digest)
                                        .expect("要判的盘引的基底上面读进来了"),
                                    image,
                                }
                            })
                            .collect();
                        let judged = pool_checker.judge(
                            library,
                            &images,
                            if is_forced_to_be_judged_again {
                                crate::verdict_store::ForcedRejudging::Forced
                            } else {
                                crate::verdict_store::ForcedRejudging::NotForced
                            },
                        )?;
                        let records = image_judgements_to_store(&images, &judged);
                        library
                            .take()
                            .store_observed_image_judgements(
                                &pool_checker_kind,
                                &observer_versions.pool_checker_version,
                                &records,
                            )
                            .map_err(library_error)?;
                        observing.images_judged_here +=
                            u64::try_from(judged.len()).expect("装得进 u64");
                        observing.images_judged_on_the_gpu_here += u64::try_from(
                            judged
                                .iter()
                                .filter(|judgement| {
                                    matches!(judgement.judged_on, JudgingDevice::Gpu { .. })
                                })
                                .count(),
                        )
                        .expect("装得进 u64");
                        // 这一趟现判过的每一张记下：记着红而现判的那几张，几块共用一张时不在一趟里重判
                        for ((position, deduplication_key), judgement) in
                            to_judge.iter().zip(judged)
                        {
                            image_judgements_judged_in_this_run
                                .insert(*deduplication_key, judgement.clone());
                            judgement_of_each_observation[*position] = Some(judgement);
                        }
                    }
                    // 每一份观察结果的红项与计数
                    let mut items_of_each_observation: Vec<Vec<String>> =
                        Vec::with_capacity(block_record.observations.len());
                    let mut counts_of_each_observation: Vec<Vec<(String, u64)>> =
                        Vec::with_capacity(block_record.observations.len());
                    for ((observation, record), judgement) in block_record
                        .observations
                        .iter()
                        .zip(&image_records)
                        .zip(&judgement_of_each_observation)
                    {
                        let (red_item, mut counts) = match (observation, record) {
                            (ObservationOfStates::Counted { red_item, counts }, _) => {
                                (red_item.clone(), counts.clone())
                            }
                            (ObservationOfStates::ImageObserved { .. }, Some(record)) => {
                                (record.red_item.clone(), record.counts.clone())
                            }
                            (ObservationOfStates::ImageObserved { .. }, None) => {
                                unreachable!("交回镜像的那几份上面都读回了它看过的恢复镜像")
                            }
                        };
                        let mut items: Vec<String> = red_item.into_iter().collect();
                        if let (Some(judgement), Some(image)) = (
                            judgement,
                            record
                                .as_ref()
                                .and_then(|record| record.image_to_judge.as_ref()),
                        ) {
                            let judgement_items = judgement.red_items(&observer.observer_name);
                            if judgement_items.is_empty() {
                                counts.push((
                                    image.counter_when_the_pool_checker_is_clean.clone(),
                                    1,
                                ));
                            }
                            items.extend(judgement_items);
                        }
                        items_of_each_observation.push(items);
                        counts_of_each_observation.push(counts);
                    }
                    let judged_on = judgement_of_each_observation
                        .iter()
                        .flatten()
                        .map(|judgement| judgement.judged_on.clone())
                        .next()
                        .unwrap_or(JudgingDevice::Cpu);
                    let encoded_verdict_of_each_observation: Vec<EncodedVerdictVector> =
                        items_of_each_observation
                            .iter()
                            .map(|items| encoded_verdict_of_the_red_items(items))
                            .collect();
                    let number_of_each_observation: Vec<VerdictVectorNumber> = {
                        let mut store = library.take();
                        encoded_verdict_of_each_observation
                            .iter()
                            .map(|encoded_verdict| {
                                store
                                    .register_verdict_vector(encoded_verdict)
                                    .map_err(library_error)
                            })
                            .collect::<Result<_, _>>()?
                    };
                    let mut numbers =
                        Vec::with_capacity(block_record.observation_of_each_state.len());
                    let mut violating = Vec::new();
                    let mut states_of_each_observation =
                        vec![0u64; block_record.observations.len()];
                    for (offset, index) in block_record.observation_of_each_state.iter().enumerate()
                    {
                        let index = usize::try_from(*index).expect("下标装得进 usize");
                        numbers.push(number_of_each_observation[index]);
                        states_of_each_observation[index] += 1;
                        if !items_of_each_observation[index].is_empty() {
                            let offset = u64::try_from(offset).expect("块内序号装得进 u64");
                            violating.push(StateOffsetInBlock(offset));
                            if observing.red_texts.len() < RED_TEXTS_KEPT {
                                observing.red_texts.push((
                                    point.path_number.clone(),
                                    local.start + offset,
                                    items_of_each_observation[index].join("；"),
                                ));
                            }
                        }
                    }
                    assert_eq!(
                        u64::try_from(numbers.len()).expect("装得进 u64"),
                        local.end - local.start,
                        "录入块的状态数与这一块的状态数相同"
                    );
                    let mut count_vector = block_record.count_vector.clone();
                    count_vector.per_invariant = BTreeMap::new();
                    let mut per_observer: BTreeMap<String, u64> = BTreeMap::new();
                    for (counts, states) in counts_of_each_observation
                        .iter()
                        .zip(&states_of_each_observation)
                    {
                        for (counter, count) in counts {
                            *per_observer.entry(counter.clone()).or_insert(0) += count * states;
                        }
                    }
                    count_vector.per_observer = per_observer;
                    let composed = ConclusionBlockRecord {
                        block: VerdictBlock::new(numbers).expect("一块至少一个状态"),
                        violating_states: violating,
                        count_vector,
                        judged_on,
                        recovery_words: None,
                    };
                    // 这一块有盘 GPU 判不了：拼出的这一块照样判红、照样计数，不存进库（下一趟重判）
                    if block_conclusion_is_kept_in_the_library(&judgement_of_each_observation) {
                        library
                            .take()
                            .store_conclusion_block(
                                &key,
                                &kind,
                                &observer_versions.conclusion,
                                &composed,
                            )
                            .map_err(library_error)?;
                    }
                    composed_here = Some(composed);
                    observing.blocks_observed_here += 1;
                    observing.states_observed_here += local.end - local.start;
                    if progress_last_printed.elapsed()
                        >= std::time::Duration::from_secs(
                            crate::crash::PROGRESS_LINE_INTERVAL_IN_SECONDS,
                        )
                    {
                        progress_last_printed = std::time::Instant::now();
                        eprintln!(
                            "CRASH_AMPLIFICATION_OBSERVING observer={name} point={} block_start={} blocks_observed_here={} states_observed_here={} images_judged_here={} image_judgements_reused={} seconds_observing={:.0}",
                            point.name,
                            local.start,
                            observing.blocks_observed_here,
                            observing.states_observed_here,
                            observing.images_judged_here,
                            observing.image_judgements_reused,
                            observing_started.elapsed().as_secs_f64()
                        );
                    }
                }
                // 存进库的与拼出的那一份逐项相同（存的那一处比过：已存着的逐项相同才算已存，不同而有一份红的照这一份换上），
                // 违例数与观察者计数照拼出的记；复用的块读库里那一份
                let (red_states, per_observer) = match composed_here {
                    Some(composed) => (
                        composed.violating_states.len(),
                        composed.count_vector.per_observer,
                    ),
                    None => {
                        let stored = library
                            .take()
                            .read_conclusion_block(&key, &kind, &observer_versions.conclusion)
                            .map_err(library_error)?
                            .ok_or_else(|| {
                                format!(
                                    "观察者 {name} 在点 {} 块 {} 的结论复用得上、库里却读不回来",
                                    point.name, local.start
                                )
                            })?;
                        (stored.violations.len(), stored.count_vector.per_observer)
                    }
                };
                *observing
                    .states_of_each_observer
                    .entry(name.clone())
                    .or_insert(0) += local.end - local.start;
                *observing
                    .red_states_of_each_observer
                    .entry(name.clone())
                    .or_insert(0) += u64::try_from(red_states).expect("违例数装得进 u64");
                let counts = observing
                    .counts_of_each_observer
                    .entry(name.clone())
                    .or_default();
                for (counter, count) in per_observer {
                    *counts.entry(counter).or_insert(0) += count;
                }
            }
        }
    }
    Ok(observing)
}

#[cfg(test)]
mod tests;
