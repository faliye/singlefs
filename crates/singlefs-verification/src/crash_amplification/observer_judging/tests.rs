use super::*;

/// 场景：那条两个扇区的小流（四个状态：点里三个、末节点一个），判器那一块的结论连同它交回的逐盘自证槽数已在库里（判段第 ① 段写的那一处）：
/// 基镜像上盘 1、盘 3 两槽都在、盘 2 一个都没有；序号 1 的状态盘 1 一个都没有，序号 3 的状态三块盘都一个没有，其余同基镜像
/// （序号 2 的盘 3 只剩一槽）。录入段对这个观察者什么都不跑；判段从库里读回判器交的那一份。
/// 预期：录入段一块都不录；判段四个状态都看过，只有序号 1 判红（新出现的死盘，点名盘 1）；盘 2 基镜像上就死、不记；序号 3 全池皆废，
/// 数进全池皆废那一项、不判红。
/// 判别力：`crates/mutations.tsv` 里「两槽全废盘面观察者判段从判器交回的那一份里取基镜像时取成崩溃状态的」「两槽全废盘面观察者判段恒有槽」
/// 两行各让这一条红。
#[test]
fn the_two_slot_observer_judges_in_the_judge_stage_on_the_slots_the_judge_stored_against_the_base_image(
) {
    let (_directory, mut store) =
        an_empty_library_for("two-slot-observer-judged-in-the-judge-stage");
    let versions = [versions_of_the_unit_test_observer([0; DIGEST_BYTES])];
    let observers = [the_two_slot_observer_on_the_nodes_of_the_unit_test_flow()];
    let slots_on_the_base_image = vec![(1, 2), (2, 0), (3, 2)];
    let slots_of_the_ordinal = |ordinal: u64| match ordinal {
        1 => vec![(1, 0), (2, 0), (3, 2)],
        2 => vec![(1, 2), (2, 0), (3, 1)],
        3 => vec![(1, 0), (2, 0), (3, 0)],
        _ => vec![(1, 2), (2, 0), (3, 2)],
    };
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let observing = recorded.with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        assert_eq!(
            plan.points
                .last()
                .map(|end_of_flow| end_of_flow.ordinals.end),
            Some(4),
            "前提：这条小流四个状态"
        );
        let version_of_each_point = vec![versions[0].pool_checker_version; plan.points.len()];
        store_a_judge_conclusion_for_every_block(
            &mut store,
            &plan,
            &ConclusionsStoredUnder::TheConclusionKind {
                kind: versions[0].pool_checker.conclusion_kind(),
                version_of_each_point: &version_of_each_point,
            },
            &slots_on_the_base_image,
            Some(&slots_of_the_ordinal),
        );
        let recording = record_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            flow,
            &plan,
            &observers,
            &versions,
            &|_| true,
            &|| false,
        )
        .expect("录入段读写得了库");
        assert_eq!(
            (
                recording.blocks_recorded_here,
                recording.states_counted_here
            ),
            (0, 0),
            "录入段对判段看的观察者什么都不跑"
        );
        judge_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            &plan,
            &observers,
            &versions,
            &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
            &|_| true,
        )
        .expect("判段从库里读得到判器交回的逐盘槽数")
    });
    let name = "unit_test_two_slot_observer";
    let counts = observing
        .counts_of_each_observer
        .get(name)
        .cloned()
        .unwrap_or_default();
    assert_eq!(
            (
                observing.states_of_each_observer.get(name).copied(),
                counts.get(crate::crash_point_replay_provenance::SYSTEM_CONFIGURATION_SLOTS_JUDGED_AGAINST_THE_BASE_IMAGE_COUNTER).copied(),
                counts.get(crate::crash_point_replay_provenance::EVERY_DEVICE_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_COUNTER).copied(),
                observing.red_states_of_each_observer.get(name).copied(),
            ),
            (Some(4), Some(3), Some(1), Some(1)),
            "四个状态都看过；不红的三个记了看过（判红那一个的计数随红项丢掉，同录入段的接法）；全池皆废一个；判红一个：{observing:?}"
        );
    assert!(
        observing.red_texts.len() == 1 && observing.red_texts[0].2.contains("盘 [1]"),
        "判红的那一个点名盘 1：{:?}",
        observing.red_texts
    );
}

/// 场景：那条两个扇区的小流，判器那一块的结论在库里、它交回的逐盘自证槽数不在（取样那一路存的、别的机器导进来的那一种）；
/// 再把逐盘槽数补进去；另一把版本下由 CPU 判器真判一遍。
/// 预期：槽数不在的块不算复用得上（每一块都还要判），两槽全废盘面观察者的判段读不到、报错点名先跑判段；补上之后一块都不用判；
/// CPU 判器真判的那一遍每一块都连同结论存下了槽数（每个状态一份、基镜像那一份取 CPU 参照），之后还要判的恰好是判红的那几块：
/// 槽数在、不红的不再判；记着红的下一趟现判（「记红、撤绿、下趟现判」，那一面由
/// `a_block_stored_red_is_judged_again_and_the_new_conclusion_replaces_it` 钉）。这条小流的盘没做过 mkfs，CPU 判器每一块都判红，
/// 槽数在不在只由上面逐块读回槽数那一段判，这里不拿「一块都不用判」顶。
/// 判别力：`crates/mutations.tsv` 里「判器存结论块时不写逐盘自证槽数」「复用不看判器交回的逐盘自证槽数在不在」两行各让这一条红。
#[test]
fn a_judge_conclusion_without_the_self_verified_slots_is_judged_again_and_the_cpu_judge_stores_them_with_every_block(
) {
    let (_directory, mut store) =
        an_empty_library_for("judge-conclusion-without-the-self-verified-slots");
    let versions = [versions_of_the_unit_test_observer([0; DIGEST_BYTES])];
    let observers = [the_two_slot_observer_on_the_nodes_of_the_unit_test_flow()];
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let read_back = recorded.with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut store, flow).expect("流写得进库、读得回")
    });
    let flow = read_back.flow();
    let plan = plan_crash_points(&flow);
    let total_blocks: usize = plan.points.iter().map(|point| point.blocks().len()).sum();
    let version_without_the_slots = vec![versions[0].pool_checker_version; plan.points.len()];
    let without_the_slots = ConclusionsStoredUnder::TheConclusionKind {
        kind: versions[0].pool_checker.conclusion_kind(),
        version_of_each_point: &version_without_the_slots,
    };
    store_a_judge_conclusion_for_every_block(&mut store, &plan, &without_the_slots, &[], None);
    assert_eq!(
        pending_blocks(&store, &plan, &without_the_slots, &|_| true)
            .expect("读得了库")
            .len(),
        total_blocks,
        "结论在、判器交回的逐盘槽数不在：每一块都还要判"
    );
    let refused = judge_the_observers_of_the_flow(
        &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
        &plan,
        &observers,
        &versions,
        &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
        &|_| true,
    )
    .expect_err("判段读不到判器交回的逐盘槽数要报错");
    assert!(
        refused.contains("交回的逐盘自证槽数不在库里") && refused.contains("先跑判段"),
        "报错点名缺的是判器交回的逐盘槽数、出路是先跑判段：{refused}"
    );
    let slots_on_the_base_image =
        self_verified_system_configuration_slots_of_each_device(flow.base);
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let mut slots_of_the_states =
                SelfVerifiedSystemConfigurationSlotsOfTheStates::default();
            for _ in local.start..local.end {
                slots_of_the_states.push_a_state(slots_on_the_base_image.clone());
            }
            let handed_back = slots_of_the_states
                .handed_back_for_the_block(local.end - local.start, &slots_on_the_base_image)
                .expect("每个状态都记了");
            store
                .store_observer_block(
                    &point.block_key(&local),
                    &versions[0].pool_checker.conclusion_kind(),
                    &version_without_the_slots[point_index],
                    &handed_back.to_bytes(),
                )
                .expect("写得进库");
        }
    }
    assert!(
        pending_blocks(&store, &plan, &without_the_slots, &|_| true)
            .expect("读得了库")
            .is_empty(),
        "补上判器交回的逐盘槽数之后一块都不用判"
    );
    let judged_versions = vec![
        ConclusionVersion(sha256_digest(
            b"cpu judge that hands back the slots in the unit test\n"
        ));
        plan.points.len()
    ];
    let judged = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &judged_versions,
    };
    let checking =
        check_blocks_where_until(&mut store, &flow, &plan, &|_| true, &judged, &|| false)
            .expect("CPU 判器判得完");
    assert_eq!(
        usize::try_from(checking.blocks_checked).expect("装得进 usize"),
        total_blocks,
        "CPU 判器每一块都判了"
    );
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let bytes = store
                .read_observer_block(
                    &point.block_key(&local),
                    &ConclusionKind::ImageJudgement,
                    &judged_versions[point_index],
                )
                .expect("读得了库")
                .expect("CPU 判器连同结论块存下了逐盘自证槽数");
            let handed_back =
                SelfVerifiedSystemConfigurationSlotsHandedBackForABlock::from_bytes(&bytes)
                    .expect("解得开");
            assert_eq!(
                (
                    handed_back.slots_of_the_states.state_count(),
                    handed_back.slots_of_each_device_on_the_base_image.clone(),
                ),
                (local.end - local.start, slots_on_the_base_image.clone()),
                "每个状态一份、基镜像那一份取 CPU 参照"
            );
        }
    }
    let pending_after_judging: Vec<(usize, Range<u64>)> =
        pending_blocks(&store, &plan, &judged, &|_| true).expect("读得了库");
    let mut red_blocks = 0usize;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let is_red = !store
                .read_conclusion_block(
                    &point.block_key(&local),
                    &ConclusionKind::ImageJudgement,
                    &judged_versions[point_index],
                )
                .expect("读得了库")
                .expect("CPU 判器每一块都存了结论")
                .violations
                .is_empty();
            red_blocks += usize::from(is_red);
            assert_eq!(
                pending_after_judging.contains(&(point_index, local.clone())),
                is_red,
                "点 {point_index} 块 {local:?}：槽数在，判过而不红的不再判，记着红的下一趟现判"
            );
        }
    }
    assert_eq!(
        pending_after_judging.len(),
        red_blocks,
        "CPU 判器判过的块里还要判的恰好是判红的那几块"
    );
}

/// 场景：那条一个扇区两次写同一份字节的小流，同一份库照键复用录、判两趟。
/// 预期：第二趟录入段一块都不录（两块都算录过）、判段一块都不拼、一张镜像都不判（观察者结论库里有）。
#[test]
fn a_run_reusing_by_key_records_composes_and_judges_no_observer_block_again() {
    let (_directory, mut store) = an_empty_library_for("observer-reused-by-key");
    let first = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    first.observing.expect("头一趟判段判得了");
    let reusing = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    let recording = reusing.recording.expect("录入段跑了");
    assert_eq!(
        (
            recording.blocks_recorded_here,
            recording.blocks_already_recorded
        ),
        (0, 2),
        "照键复用的第二趟：两块都算录过、一块都不录"
    );
    let observing = reusing.observing.expect("判段判得了");
    assert_eq!(
        (observing.blocks_observed_here, observing.images_judged_here),
        (0, 0),
        "照键复用的第二趟：观察者结论库里有，一块都不拼、一张镜像都不判"
    );
}

/// 场景：那条一个扇区两次写同一份字节的小流挂一个交回镜像的观察者（读整个池，按恢复报告与读集合去重），录进一份空库；复用哨兵这一趟抽中
/// 崩溃点那一块强制现判（`VerdictStore::also_force_these_conclusion_blocks_to_be_judged_again`，观察者那一份结论），判段只领那一块，
/// 池级 checker 走 CPU 那一路、每交来一批盘记下交来的强制现判。
/// 预期：交来恰好一批盘，交来的是强制现判。
/// 判别力（`crates/mutations.tsv`）：观察者判段交池级 checker 时一律交不强制（不看这一块是不是强制现判），这条红。
#[test]
fn the_observer_judging_hands_the_forced_rejudging_of_its_block_down_to_the_pool_checker() {
    let (_directory, mut store) = an_empty_library_for("observer-forced-block-handed-down");
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        counts.add("images_seen", 1);
        ImageAfterTheObserver {
            writes_on_the_crash_image: Vec::new(),
            disks_after_the_observer: pool_of_the_crash_image(
                image.base,
                image.writes,
                &image.persisted,
            ),
            ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::whole_pool(image.base),
            invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::Only(Vec::new()),
            counter_when_the_pool_checker_is_clean: "judged_clean".to_string(),
        }
    };
    let versions = versions_of_the_unit_test_observer([11; DIGEST_BYTES]);
    let observers = [ObserverOnTheNodes {
        observer_name: ObserverName::new("unit_test_image_observer").expect("名字合规矩"),
        observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
        extra_inputs_digest: [11; DIGEST_BYTES],
        nodes: vec![
            NodeOfAnObserver {
                setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                entry_name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
            },
            NodeOfAnObserver {
                setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                entry_name: END_OF_FLOW_NODE_NAME.to_string(),
            },
        ],
        observe: ObserverCode::HandsBackAnImage {
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        },
    }];
    a_flow_of_two_writes_of_the_same_bytes_to_one_sector().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        record_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            flow,
            &plan,
            &observers,
            std::slice::from_ref(&versions),
            &|_| true,
            &|| false,
        )
        .expect("录入段写得进库");
        // 崩溃点那一块在计划里的块序号与块键（末节点那一块不领、不强制）
        let mut block_sequence = 0u64;
        let mut crash_point_block = None;
        for point in &plan.points {
            for local in point.blocks() {
                if crash_point_block.is_none() && point.name == OBSERVER_UNIT_TEST_POINT_NAME {
                    crash_point_block = Some((block_sequence, point.block_key(&local)));
                }
                block_sequence += 1;
            }
        }
        let (forced_block_sequence, forced_block_key) =
            crash_point_block.expect("前提：计划里有崩溃点那一块");
        store.also_force_these_conclusion_blocks_to_be_judged_again([(
            forced_block_key,
            ConclusionKind::Observer,
            versions.conclusion,
        )]);
        let pool_checker =
            PoolCheckerOfTheImagesAfterTheObservers::OnTheCpuNotingTheForcedRejudging(
                std::sync::Mutex::new(Vec::new()),
            );
        judge_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            &plan,
            &observers,
            std::slice::from_ref(&versions),
            &pool_checker,
            &|sequence| sequence == forced_block_sequence,
        )
        .expect("判段判得了");
        let PoolCheckerOfTheImagesAfterTheObservers::OnTheCpuNotingTheForcedRejudging(
            forced_rejudging_handed_down,
        ) = pool_checker
        else {
            panic!("建的就是记下强制现判的那一路");
        };
        assert_eq!(
            forced_rejudging_handed_down
                .into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec![crate::verdict_store::ForcedRejudging::Forced],
            "强制现判那一块交来恰好一批盘，交来的是强制现判"
        );
    });
}

/// 那条一个扇区两次写同一份字节的小流里每一块的块键，按块序。
fn block_keys_of_the_two_writes_to_one_sector() -> Vec<VerdictBlockKey> {
    a_flow_of_two_writes_of_the_same_bytes_to_one_sector().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        plan.points
            .iter()
            .flat_map(|point| {
                point
                    .blocks()
                    .into_iter()
                    .map(|local| point.block_key(&local))
                    .collect::<Vec<_>>()
            })
            .collect()
    })
}

/// 把库里一块观察者结论换成记着红的一份（块与计数向量照旧，块内序号 0 记一个违例）：存块那一处两份有一份红就换上。
fn overwrite_the_observer_conclusion_with_a_red_one(
    store: &mut VerdictStore,
    key: &VerdictBlockKey,
    observer_conclusion_version: &ConclusionVersion,
) {
    let stored = store
        .read_conclusion_block(key, &ConclusionKind::Observer, observer_conclusion_version)
        .expect("库读得了")
        .expect("头一趟存过这一块的观察者结论");
    assert!(stored.violations.is_empty(), "前提：头一趟拼出的结论不红");
    let storing = store
        .store_conclusion_block(
            key,
            &ConclusionKind::Observer,
            observer_conclusion_version,
            &ConclusionBlockRecord {
                block: stored.block,
                violating_states: vec![StateOffsetInBlock(0)],
                count_vector: stored.count_vector,
                judged_on: stored.judged_on,
                recovery_words: stored.recovery_words,
            },
        )
        .expect("记着红的一份写得进库");
    assert_eq!(
        storing,
        crate::verdict_store::VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed,
        "前提：红的一份换上了不红的那一份"
    );
}

/// 场景：那条一个扇区两次写同一份字节的小流，同一份库照键复用录、判一趟；把头一块的观察者结论换成记着红的一份，再照键复用录、判一趟。
/// 预期：第二趟录入段重录记着红的那一块、另一块算录过（1 块录、1 块算录过），判段只重拼那一块（另一块照库里的复用），拼出的结论与头一趟
/// 逐状态相同，库里那一块的结论照新结论换回不红（「记红、撤绿、下趟现判」，观察者结论同一个规矩）。
/// 判别力（`crates/mutations.tsv`）：「红观察者结论：判段照库里的复用」「红观察者结论：录入段照算录过」两行各让这一条红。
#[test]
fn an_observer_conclusion_stored_red_is_recorded_and_composed_again_and_the_new_conclusion_replaces_it(
) {
    let (_directory, mut store) = an_empty_library_for("observer-stored-red");
    let first = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    first.observing.expect("头一趟判段判得了");
    let versions = versions_of_the_unit_test_observer([3; DIGEST_BYTES]);
    let keys = block_keys_of_the_two_writes_to_one_sector();
    assert_eq!(keys.len(), 2, "前提：小流两块（段内一块、末节点一块）");
    overwrite_the_observer_conclusion_with_a_red_one(&mut store, &keys[0], &versions.conclusion);
    let after_red = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    let recording = after_red.recording.expect("录入段跑了");
    assert_eq!(
        (
            recording.blocks_recorded_here,
            recording.blocks_already_recorded
        ),
        (1, 1),
        "记着红的那一块重录，另一块算录过"
    );
    let observing = after_red.observing.expect("判段判得了");
    assert_eq!(
        observing.blocks_observed_here, 1,
        "记着红的那一块重拼，另一块照库里的复用"
    );
    assert_eq!(
        after_red.items_of_each_state, first.items_of_each_state,
        "重拼出的结论与头一趟逐状态相同"
    );
    assert!(
        store
            .read_conclusion_block(&keys[0], &ConclusionKind::Observer, &versions.conclusion)
            .expect("库读得了")
            .expect("那一块的结论在库里")
            .violations
            .is_empty(),
        "库里那一块的结论照新结论换回不红"
    );
}

/// 场景：同一条小流照键复用录、判一趟；把两块的观察者结论都换成记着红的，并把交回镜像上池级 checker 的结论都换成判出违例的，再照键复用录、判一趟。
/// 预期：两块都重拼，交回镜像上的池级 checker 记着红的那两张在头一块都现判（库里记着红的一张都不复用），末节点那一块的镜像是这一趟头一块
/// 刚现判、写回库里的那一张，照库里的复用（复用 1）；拼出的结论与头一趟逐状态相同（「记红、撤绿、下趟现判」）。
/// 判别力（`crates/mutations.tsv`）：「红镜像判定：照库里的复用」这一行让这一条红。
#[test]
fn a_pool_checker_judgement_stored_red_on_an_observed_image_is_judged_again() {
    let (_directory, mut store) = an_empty_library_for("observer-image-judgement-stored-red");
    let first = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    first.observing.expect("头一趟判段判得了");
    let versions = versions_of_the_unit_test_observer([3; DIGEST_BYTES]);
    let mut deduplication_keys: BTreeSet<[u8; DIGEST_BYTES]> = BTreeSet::new();
    for key in block_keys_of_the_two_writes_to_one_sector() {
        overwrite_the_observer_conclusion_with_a_red_one(&mut store, &key, &versions.conclusion);
        let block_bytes = store
            .read_observer_block(&key, &ConclusionKind::Observer, &versions.observer)
            .expect("库读得了")
            .expect("头一趟录过这一块");
        for observation in ObserverBlockRecord::from_bytes(&block_bytes)
            .expect("录入块解得开")
            .observations
        {
            if let ObservationOfStates::ImageObserved { deduplication_key } = observation {
                deduplication_keys.insert(deduplication_key);
            }
        }
    }
    assert_eq!(deduplication_keys.len(), 2, "前提：两张不同的镜像");
    let red_judgement = ImageJudgementRecord {
        violated_invariants: vec!["I-1.1".to_string()],
        could_not_judge: None,
        judged_on: JudgingDevice::Cpu,
    }
    .to_bytes();
    store
        .store_observed_image_judgements(
            &versions.pool_checker.conclusion_kind(),
            &versions.pool_checker_version,
            &deduplication_keys
                .iter()
                .map(|deduplication_key| (*deduplication_key, red_judgement.clone()))
                .collect::<Vec<_>>(),
        )
        .expect("记着红的镜像判定写得进库");
    let after_red = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    let observing = after_red.observing.expect("判段判得了");
    assert_eq!(
            (
                observing.blocks_observed_here,
                observing.images_judged_here,
                observing.image_judgements_reused
            ),
            (2, 2, 1),
            "两块都重拼，记着红的两张镜像在头一块都现判；末节点那一块的那一张是这一趟头一块刚现判、写回库里的，照库里的复用"
        );
    assert_eq!(
        after_red.items_of_each_state, first.items_of_each_state,
        "重拼出的结论与头一趟逐状态相同"
    );
}

/// 场景：那条一个扇区两次写同一份字节的小流，交回镜像的观察者照键复用录、判一趟；复用哨兵抽中崩溃点那一块强制现判（它的观察者结论登进
/// 强制现判表），再照键复用录、判一趟。
/// 预期：录入段一块都不录（两块录入段那一行都在）；判段只重拼强制现判那一块，块里两张盘上的池级 checker 判定都现判、一张都不取库里的
/// （「强制现判那一块，块里各张盘的池级 checker 判定也一起现判」）；拼出的结论与头一趟逐状态相同。
/// 判别力（`crates/mutations.tsv`）：「复用哨兵：强制现判那一块上的盘照取库里的池级 checker 判定」这一行让这一条红。
#[test]
fn the_pool_checker_judgements_on_the_images_of_the_forced_block_are_judged_again() {
    let (_directory, mut store) = an_empty_library_for("forced-block-image-judgements");
    let first = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    first.observing.expect("头一趟判段判得了");
    let versions = versions_of_the_unit_test_observer([3; DIGEST_BYTES]);
    let keys = block_keys_of_the_two_writes_to_one_sector();
    assert_eq!(keys.len(), 2, "前提：小流两块（段内一块、末节点一块）");
    let forced_observer_conclusion = (
        keys[0].clone(),
        ConclusionKind::Observer,
        versions.conclusion,
    );
    store.also_force_these_conclusion_blocks_to_be_judged_again([
        forced_observer_conclusion.clone()
    ]);
    let forced_run = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    store.stop_forcing_these_conclusion_blocks_to_be_judged_again([forced_observer_conclusion]);
    let recording = forced_run.recording.expect("录入段跑了");
    assert_eq!(
        (
            recording.blocks_recorded_here,
            recording.blocks_already_recorded
        ),
        (0, 2),
        "两块录入段那一行都在：一块都不录"
    );
    let observing = forced_run.observing.expect("判段判得了");
    assert_eq!(
        (
            observing.blocks_observed_here,
            observing.images_judged_here,
            observing.image_judgements_reused
        ),
        (1, 2, 0),
        "只重拼强制现判那一块，块里两张盘都现判、一张都不取库里的"
    );
    assert_eq!(
        forced_run.items_of_each_state, first.items_of_each_state,
        "重拼出的结论与头一趟逐状态相同"
    );
}

/// 观察者两段：只录那一趟库里只有录入块、没有观察者的结论；没录过就只判，判段报「先跑录入段」；录过之后只判那一趟从库里拼出结论、不调观察者。
#[test]
fn the_observer_record_stage_does_not_judge_and_the_judge_stage_only_reads_the_library() {
    let (_directory, mut store) = an_empty_library_for("observer-two-stages");
    let judged_before_recording = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::JudgeOnly,
    );
    assert!(
        judged_before_recording
            .observing
            .as_ref()
            .is_err_and(|refusal| refusal.contains("先跑录入段")),
        "没录过就只判：{:?}",
        judged_before_recording.observing
    );
    let recorded_only = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordOnly,
    );
    assert_eq!(recorded_only.observer_calls, 2, "录入段看了两张镜像");
    assert!(
        recorded_only.items_of_each_state.is_empty(),
        "只录那一趟一块结论都不存"
    );
    assert_eq!(recorded_only.observations_of_each_block, vec![2, 1]);
    let judged_only = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::JudgeOnly,
    );
    assert!(judged_only.observing.is_ok(), "{:?}", judged_only.observing);
    assert_eq!(judged_only.observer_calls, 0, "只判那一趟不调观察者");
    assert_eq!(
        judged_only.items_of_each_state.len(),
        4,
        "只判那一趟从库里拼出每个状态的结论"
    );
}
