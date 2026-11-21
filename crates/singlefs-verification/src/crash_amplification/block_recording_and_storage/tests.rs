use super::*;

/// 场景（一块里绿状态的判定向量只登记一次）：一块逐状态的红项「绿、红甲、绿、红乙、红甲、绿」，一份空库里照一块登记
/// （[`verdict_vector_numbers_of_the_red_items_of_each_state`]），另一份空库里逐状态各登记一次。
/// 预期：两份交回的编号逐个相同；三个绿状态同一个编号、两个红甲同一个编号，绿、红甲、红乙三个编号两两不同。
/// 判别力：`crates/mutations.tsv` 里「崩溃放量 一块的判定向量编号把红状态也取成绿的那一个」那一行让这一条红。
#[test]
fn red_items_numbered_with_the_green_number_registered_once_equal_registering_each_state() {
    let (_once_directory, mut registering_the_green_once) =
        an_empty_library_for("green-verdict-registered-once");
    let (_each_directory, mut registering_each_state) =
        an_empty_library_for("green-verdict-registered-for-each-state");
    let red = |item: &str| vec![item.to_string()];
    let items_of_each_state = vec![
        Vec::new(),
        red("pool_checker:I-1.1"),
        Vec::new(),
        red("pool_checker:I-2.1"),
        red("pool_checker:I-1.1"),
        Vec::new(),
    ];
    let numbers = verdict_vector_numbers_of_the_red_items_of_each_state(
        &mut registering_the_green_once,
        &items_of_each_state,
    )
    .expect("登记得了");
    let numbers_registered_for_each_state: Vec<VerdictVectorNumber> = items_of_each_state
        .iter()
        .map(|items| {
            registering_each_state
                .register_verdict_vector(&encoded_verdict_of_the_red_items(items))
                .expect("登记得了")
        })
        .collect();
    assert_eq!(
        numbers, numbers_registered_for_each_state,
        "照一块登记与逐状态登记交回的编号逐个相同"
    );
    assert_eq!(
        (
            [numbers[2], numbers[5]],
            numbers[4],
            BTreeSet::from([numbers[0], numbers[1], numbers[3]]).len()
        ),
        ([numbers[0], numbers[0]], numbers[1], 3),
        "三个绿状态同一个编号、两个红甲同一个编号，三种两两不同：{numbers:?}"
    );
}

/// 场景：那条两个扇区的小流录进库，每一块的判器结论先以红的（一项单测造的红项、头一个状态违例）存在一把版本下；再由 CPU 判器照这一把真判一趟；
/// 另一份空库里同一条流由 CPU 判器照同一把判一趟当对照。
/// 预期：记着红的块不复用：判之前每一块都在要判的块里；CPU 判器每一块都现判了（现判数等于块数）；判完库里每一块的结论照新判的整份换上
/// （与空库那一趟逐块是同一个结论，单测造的红项与它的发布名不在了）；判完之后仍红的块下一趟照旧在要判的块里，判绿的不在。
/// 两份库各自按登记先后给判定向量编号（这份库里单测造的红项先登记），逐块比的是判定向量本身
/// （[`ConclusionSpelledOutInVerdictVectors::of_the_stored_block`]），不是编号。
/// 「记红、撤绿、下趟现判」。
/// 判别力（`crates/mutations.tsv`）：复用改回「有结论就复用」（不看红不红），这条红；存块那一处库里有结论就不写（记红现判之后新结论没换上），这条红。
#[test]
fn a_block_stored_red_is_judged_again_and_the_new_conclusion_replaces_it() {
    let (_directory, mut store) = an_empty_library_for("red-block-judged-again");
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let read_back = recorded.with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut store, flow).expect("流写得进库、读得回")
    });
    let flow = read_back.flow();
    let plan = plan_crash_points(&flow);
    let total_blocks: usize = plan.points.iter().map(|point| point.blocks().len()).sum();
    let versions = vec![
        ConclusionVersion(sha256_digest(b"version of the red block unit test\n"));
        plan.points.len()
    ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            store_a_judge_conclusion_of_the_block(
                &mut store,
                &plan,
                &conclusions,
                point_index,
                &local,
                Some("unit_test_red_item"),
                "unit_test_publish",
            );
        }
    }
    assert_eq!(
        pending_blocks(&store, &plan, &conclusions, &|_| true)
            .expect("读得了库")
            .len(),
        total_blocks,
        "记着红的块一块都不复用，每一块都要现判"
    );
    let checking =
        check_blocks_where_until(&mut store, &flow, &plan, &|_| true, &conclusions, &|| false)
            .expect("CPU 判器判得完（红块换成新判的结论）");
    assert_eq!(
        usize::try_from(checking.blocks_checked).expect("装得进 usize"),
        total_blocks,
        "每一块都现判了，现判数计入"
    );
    let (_reference_directory, mut reference_store) =
        an_empty_library_for("red-block-judged-again-reference");
    let reference_read_back =
        a_flow_of_two_writes_to_two_sectors().with_the_flow(|reference_recorded_flow| {
            record_the_flow_and_read_it_back(&mut reference_store, reference_recorded_flow)
                .expect("流写得进库、读得回")
        });
    let reference_flow = reference_read_back.flow();
    check_blocks_where_until(
        &mut reference_store,
        &reference_flow,
        &plan,
        &|_| true,
        &conclusions,
        &|| false,
    )
    .expect("空库那一趟判得完");
    let pending_after_judging: Vec<(usize, Range<u64>)> =
        pending_blocks(&store, &plan, &conclusions, &|_| true).expect("读得了库");
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let key = point.block_key(&local);
            let replaced = store
                .read_conclusion_block(
                    &key,
                    &ConclusionKind::ImageJudgement,
                    &versions[point_index],
                )
                .expect("读得了库")
                .expect("判完每一块都有结论");
            let reference = reference_store
                .read_conclusion_block(
                    &key,
                    &ConclusionKind::ImageJudgement,
                    &versions[point_index],
                )
                .expect("读得了库")
                .expect("空库那一趟每一块都有结论");
            assert_eq!(
                ConclusionSpelledOutInVerdictVectors::of_the_stored_block(&store, &replaced),
                ConclusionSpelledOutInVerdictVectors::of_the_stored_block(
                    &reference_store,
                    &reference
                ),
                "点 {point_index} 块 {local:?}：库里那一份照新判的整份换上"
            );
            assert!(
                !replaced
                    .count_vector
                    .states_by_publish
                    .contains_key("unit_test_publish"),
                "单测造的红结论不在了"
            );
            assert_eq!(
                pending_after_judging.contains(&(point_index, local.clone())),
                !replaced.violations.is_empty(),
                "点 {point_index} 块 {local:?}：判完仍红的下一趟照旧现判，判绿的复用"
            );
        }
    }
}

/// 存块那一处：库里一份不红的结论，同一把键再存一份红的（判红写红记录、撤同一把键的绿），再存回不红的（记着红的现判之后照新结论写回），
/// 两份都不红而不同的照旧拒。
/// 预期：绿换红、红换绿两次都交「换上」、读回是新的那一份（红换绿之后违例行一行不剩）；两份都不红而不同交「同键不同结论」的错、库里那一份不动。
/// 判别力（`crates/mutations.tsv`）：存块那一处不换红（两份不同一律拒）、换上时不删旧的违例行，各让这一条红。
#[test]
fn storing_a_red_conclusion_over_a_green_one_replaces_it_and_back_but_two_different_green_ones_are_refused(
) {
    let (plan, _block_count) = plan_of_the_two_sector_flow_with_its_block_count();
    let (_directory, mut store) = an_empty_library_for("red-replaces-green-and-back");
    let versions =
        vec![
            ConclusionVersion(sha256_digest(b"version of the red replacement unit test\n"));
            plan.points.len()
        ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let local = plan.points[0].blocks()[0].clone();
    let key = plan.points[0].block_key(&local);
    let read = |the_library: &VerdictStore| {
        the_library
            .read_conclusion_block(&key, &ConclusionKind::ImageJudgement, &versions[0])
            .expect("读得了库")
            .expect("存过")
    };
    store_a_judge_conclusion_of_the_block(
        &mut store,
        &plan,
        &conclusions,
        0,
        &local,
        None,
        "green_publish",
    );
    assert!(read(&store).violations.is_empty(), "前提：先存的那一份不红");
    store_a_judge_conclusion_of_the_block(
        &mut store,
        &plan,
        &conclusions,
        0,
        &local,
        Some("unit_test_red_item"),
        "red_publish",
    );
    let red = read(&store);
    assert_eq!(red.violations.len(), 1, "绿换红：读回是红的那一份");
    assert!(red
        .count_vector
        .states_by_publish
        .contains_key("red_publish"));
    assert!(
        conclusions.is_red(&store, 0, &key).expect("读得了库"),
        "记红：这一块在库里是红的"
    );
    store_a_judge_conclusion_of_the_block(
        &mut store,
        &plan,
        &conclusions,
        0,
        &local,
        None,
        "green_again_publish",
    );
    let green_again = read(&store);
    assert!(green_again.violations.is_empty(), "红换绿：违例行一行不剩");
    assert!(green_again
        .count_vector
        .states_by_publish
        .contains_key("green_again_publish"));
    assert!(
        !conclusions.is_red(&store, 0, &key).expect("读得了库"),
        "红换绿之后这一块不红"
    );
    let clean = store
        .register_verdict_vector(&encoded_verdict_of_the_red_items(&[]))
        .expect("登记得了");
    let states = local.end - local.start;
    let refused = store.store_conclusion_block(
        &key,
        &ConclusionKind::ImageJudgement,
        &versions[0],
        &ConclusionBlockRecord {
            block: VerdictBlock::new(vec![
                clean;
                usize::try_from(states).expect("装得进 usize")
            ])
            .expect("一块至少一个状态"),
            violating_states: Vec::new(),
            count_vector: BlockCountVector {
                per_invariant: BTreeMap::new(),
                per_observer: BTreeMap::new(),
                states_by_publish: BTreeMap::from([("another_green_publish".to_string(), states)]),
            },
            judged_on: JudgingDevice::Cpu,
            recovery_words: None,
        },
    );
    assert!(
        matches!(
            refused,
            Err(
                VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion { .. }
            )
        ),
        "两份都不红而不同照旧拒：{refused:?}"
    );
    assert!(
        read(&store).is_the_same_conclusion_as(&green_again),
        "拒了之后库里那一份不动"
    );
}

/// 场景：那条两个扇区的小流，五把版本各当一处结论，每一块在各处落一种情形：判过、带槽数、不红；判过、带槽数、红；没判；
/// 判过不红而没有槽数；判过、带槽数、不红而在强制现判表里。
/// 预期：整份计划一次读回的那一份（[`ConclusionsStoredUnder::reuse_read_of_the_plan`]）照 [`ReuseOfTheBlocksReadOnce::is_stored_for_reuse`] 答的，
/// 与逐块问 [`ConclusionsStoredUnder::is_stored_for_reuse`] 答的逐块相同；只有头一种答复用得上。录入时数复用、数已判的态数、抽哨兵都照这一份答。
/// 判别力（`crates/mutations.tsv`）：成批读不看槽数那一行（判过不红就算复用得上），这条红。
#[test]
fn the_reuse_read_of_the_plan_answers_as_each_block_asked_alone() {
    let (plan, block_count) = plan_of_the_two_sector_flow_with_its_block_count();
    let (_directory, mut store) = an_empty_library_for("reuse-read-of-the-plan");
    let reusable_of_each_case = [true, false, false, false, false];
    let versions_of_each_case: Vec<Vec<ConclusionVersion>> = (0..reusable_of_each_case.len())
        .map(|case| {
            vec![
                ConclusionVersion(sha256_digest(
                    format!("reuse read of the plan case {case}\n").as_bytes()
                ));
                plan.points.len()
            ]
        })
        .collect();
    for (case, versions) in versions_of_each_case.iter().enumerate() {
        let conclusions = ConclusionsStoredUnder::TheConclusionKind {
            kind: ConclusionKind::ImageJudgement,
            version_of_each_point: versions,
        };
        for (point_index, point) in plan.points.iter().enumerate() {
            for local in point.blocks() {
                let key = point.block_key(&local);
                let red_item_of_the_conclusion_with_the_slots = match case {
                    0 | 4 => Some(None),
                    1 => Some(Some("unit_test_red_item")),
                    _ => None,
                };
                if let Some(red_item) = red_item_of_the_conclusion_with_the_slots {
                    store_a_judge_conclusion_of_the_block(
                        &mut store,
                        &plan,
                        &conclusions,
                        point_index,
                        &local,
                        red_item,
                        "unit_test_publish",
                    );
                }
                if case == 3 {
                    store_a_judge_conclusion_without_the_slots(
                        &mut store,
                        &key,
                        &versions[point_index],
                        &local,
                    );
                }
                if case == 4 {
                    store.also_force_these_conclusion_blocks_to_be_judged_again([(
                        key,
                        ConclusionKind::ImageJudgement,
                        versions[point_index],
                    )]);
                }
            }
        }
    }
    for (case, versions) in versions_of_each_case.iter().enumerate() {
        let conclusions = ConclusionsStoredUnder::TheConclusionKind {
            kind: ConclusionKind::ImageJudgement,
            version_of_each_point: versions,
        };
        let reuse_read = conclusions
            .reuse_read_of_the_plan(&store, &plan, &|_| true)
            .expect("读得了库");
        let mut block_sequence = 0u64;
        for (point_index, point) in plan.points.iter().enumerate() {
            for local in point.blocks() {
                let key = point.block_key(&local);
                assert_eq!(
                    (
                        reuse_read.is_stored_for_reuse(
                            &store,
                            &conclusions,
                            point_index,
                            &key,
                            block_sequence
                        ),
                        conclusions
                            .is_stored_for_reuse(&store, point_index, &key)
                            .expect("读得了库")
                    ),
                    (reusable_of_each_case[case], reusable_of_each_case[case]),
                    "情形 {case} 第 {block_sequence} 块：读回的那一份与逐块问的相同、照情形钉死"
                );
                block_sequence += 1;
            }
        }
        assert_eq!(
            block_sequence,
            u64::try_from(block_count).expect("块数装得进 u64"),
            "情形 {case}：计划里每一块都问到"
        );
    }
}

/// 判器交回的逐盘自证槽数存进库的字节读回来逐项相同；连着相同的状态并成一段，拼两段时接缝处相同的也并上。
#[test]
fn the_self_verified_slots_handed_back_for_a_block_read_back_from_their_bytes_and_merge_equal_neighbours(
) {
    let mut earlier = SelfVerifiedSystemConfigurationSlotsOfTheStates::default();
    earlier.push_a_state(vec![(0, 2), (1, 2)]);
    earlier.push_a_state(vec![(0, 2), (1, 2)]);
    earlier.push_a_state(vec![(0, 0), (1, 2)]);
    let mut later = SelfVerifiedSystemConfigurationSlotsOfTheStates::default();
    later.push_a_state(vec![(0, 0), (1, 2)]);
    later.push_a_state(vec![(0, 2), (1, 1)]);
    earlier.absorb(later);
    assert_eq!(
        earlier
            .runs
            .iter()
            .map(|run| run.states)
            .collect::<Vec<u64>>(),
        vec![2, 2, 1],
        "连着相同的并成一段，接缝处相同的也并上"
    );
    let handed_back = earlier
        .handed_back_for_the_block(5, &[(0, 2), (1, 2)])
        .expect("五个状态都记了");
    assert_eq!(
        SelfVerifiedSystemConfigurationSlotsHandedBackForABlock::from_bytes(
            &handed_back.to_bytes()
        ),
        Some(handed_back.clone()),
        "字节读回来逐项相同"
    );
    let mut bytes = handed_back.to_bytes();
    bytes.push(0);
    assert_eq!(
        SelfVerifiedSystemConfigurationSlotsHandedBackForABlock::from_bytes(&bytes),
        None,
        "末尾多一个字节读不回"
    );
    assert_eq!(
        SelfVerifiedSystemConfigurationSlotsOfTheStates::default()
            .handed_back_for_the_block(5, &[(0, 2)]),
        None,
        "一个状态都没记（单测里假的判一段）不存"
    );
}
