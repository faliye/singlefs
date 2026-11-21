use super::*;

fn sample_variant_result() -> VariantResult {
    VariantResult {
        applicable: true,
        class: "K0".to_string(),
        error: "none".to_string(),
        count: Some(1),
        expected_count: 1,
        isolated_by_device: vec![(0, 14), (1, 14)],
        intercepted_all: true,
        writes: 9,
        shared: false,
        exclusive_slots: 28,
        exclusive_by_device: vec![(0, 14), (1, 14)],
        continued_to_txg: Some(31),
        reuse_trajectory: vec![0, 0, 3],
    }
}

/// 从起点 `bootstrap(G0, 1)` 做抛弃步（断链方式 b = unit_first_copy），交回起点节点与造成立的对。
fn constructed_pair_at_the_start_node() -> (SimNode, MakeFilesystemParameters, ConstructedPair) {
    let node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("起点");
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let construction = construct_pair(
        &node,
        &parameters,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
    )
    .expect("抛弃步不停机");
    match construction {
        PairConstruction::Constructed(pair) => (node, parameters, pair),
        PairConstruction::NotApplicable(reason) | PairConstruction::NotConstructed(reason) => {
            panic!("起点节点上 b = unit_first_copy 应当造成立（锚点 A4），实际：{reason}")
        }
    }
}

#[test]
fn step_sequence_text_names_each_step_without_whitespace() {
    assert_eq!(step_sequence_text(&[]), "start");
    let steps = [
        HistoryStep::Overwrite,
        HistoryStep::MountWritable,
        HistoryStep::RollbackTo(2, 17),
    ];
    assert_eq!(step_sequence_text(&steps), "W,M,R(2.17)");
    assert_eq!(last_step_kind(&steps), "R");
    assert_eq!(last_step_kind(&[]), "start");
}

#[test]
fn variant_result_survives_its_text_form_unchanged() {
    let result = sample_variant_result();
    assert_eq!(VariantResult::from_text(&result.to_text()), Ok(result));
    let none = VariantResult::not_applicable();
    assert_eq!(VariantResult::from_text(&none.to_text()), Ok(none));
    let mut mount_refused = sample_variant_result();
    mount_refused.class = "K3".to_string();
    mount_refused.error = "StillUnreadableAfterOneReread".to_string();
    mount_refused.count = None;
    mount_refused.isolated_by_device = Vec::new();
    mount_refused.continued_to_txg = None;
    mount_refused.reuse_trajectory = Vec::new();
    assert_eq!(
        VariantResult::from_text(&mount_refused.to_text()),
        Ok(mount_refused)
    );
}

#[test]
fn pair_outcome_survives_its_text_form_unchanged() {
    for outcome in [
        PairOutcome::Constructed {
            abandoned: vec![(1, 13), (1, 14)],
            readable_abandoned: vec![(1, 13)],
        },
        PairOutcome::Constructed {
            abandoned: vec![(1, 13)],
            readable_abandoned: Vec::new(),
        },
        PairOutcome::NotApplicable("slot_generation_below_two".to_string()),
        PairOutcome::NotConstructed("abandoned_set_empty".to_string()),
    ] {
        assert_eq!(PairOutcome::from_text(&outcome.to_text()), Ok(outcome));
    }
}

#[test]
fn the_variant_table_has_fifteen_distinctly_named_variants_in_a_fixed_order() {
    let specifications = variant_specifications();
    assert_eq!(specifications.len(), 15);
    assert_eq!(specifications[0], VariantSpecification::Twin);
    let names: BTreeSet<String> = specifications
        .iter()
        .map(|specification| specification.name())
        .collect();
    assert_eq!(names.len(), 15);
    assert_eq!(
        specifications[1].name(),
        "allocation_record_tree.both_copies.read_fails"
    );
    assert_eq!(specifications[14].name(), "root_slot.reads_zeros");
}

/// 锚点 A1：同一根槽的下一次发布在 txg + 3S，S = 4、8、16 时偏移 12、24、48。
#[test]
fn the_next_publish_on_the_same_root_slot_comes_three_times_the_ring_size_later() {
    for (slots_per_region, expected_offset) in [(4u64, 12u64), (8, 24), (16, 48)] {
        assert_eq!(
            continuation_anchor_of(slots_per_region),
            (true, vec![expected_offset])
        );
    }
}

/// 锚点 A2 的松上界：登记 5.3 的 54837、382653、124113。
#[test]
fn the_node_count_upper_bounds_are_the_registered_ones() {
    let domain_of = |geometry: Geometry, step_limit: u64| FifthRunDomain {
        geometry,
        step_limit,
        starting_overwrites: FIFTH_RUN_STARTING_OVERWRITES.to_vec(),
    };
    assert_eq!(
        node_count_upper_bound(&domain_of(GEOMETRY_PRIMARY, 3)),
        54837
    );
    assert_eq!(
        node_count_upper_bound(&domain_of(GEOMETRY_LARGER_ROOT_RING, 3)),
        382_653
    );
    assert_eq!(
        node_count_upper_bound(&domain_of(GEOMETRY_SMALLER_ROOT_RING, 4)),
        124_113
    );
}

#[test]
fn the_registered_continuation_end_is_the_txg_before_the_next_publish_on_the_root_slot() {
    assert_eq!(
        continuation_end_txg((1, 13), &GEOMETRY_PRIMARY),
        13 + 24 - 1
    );
    assert_eq!(
        continuation_end_txg((2, 7), &GEOMETRY_LARGER_ROOT_RING),
        7 + 48 - 1
    );
    assert_eq!(
        continuation_end_txg((1, 5), &GEOMETRY_SMALLER_ROOT_RING),
        5 + 12 - 1
    );
}

/// 锚点 A4：起点节点上 b = unit_first_copy 抛弃最新根，被抛弃集合恰是时间线尾；b = records 时新实例第一次发布的 txg 与被藏根相同、
/// 落进同一个根槽，被抛弃集合为空（未造出）。
#[test]
fn the_start_node_abandons_the_newest_root_for_the_first_copy_and_nothing_for_every_record() {
    let (node, parameters, pair) = constructed_pair_at_the_start_node();
    assert_eq!(
        pair.abandoned,
        BTreeSet::from([*node.timeline.last().expect("时间线非空")])
    );
    let every_record =
        construct_pair(&node, &parameters, ReplayBreak::EveryRecord).expect("抛弃步不停机");
    assert!(matches!(
        every_record,
        PairConstruction::NotConstructed(ref reason) if reason == "abandoned_set_empty"
    ));
}

/// 世代号不足 2 的节点记「不适用」，不 panic（停机 S5 的那一形）。
#[test]
fn node_whose_newest_system_configuration_generation_is_below_two_is_not_applicable() {
    let mut node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("起点");
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    let unmodified = node.pool.clone();
    for (device, device_image) in &mut node.pool.devices {
        for offset in [0u64, slot_spacing] {
            let bytes = unmodified
                .read(device.0, offset, slot_bytes)
                .expect("两槽都在盘上");
            let mut configuration =
                singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes)
                    .expect("两槽都自证");
            configuration.quantities.slot_generation = 1;
            device_image.write(DeviceOffsetInBytes(offset), &configuration.to_slot());
        }
    }
    for replay_break in FIFTH_RUN_REPLAY_BREAKS {
        let construction =
            construct_pair(&node, &parameters, replay_break).expect("不适用不是停机");
        assert!(matches!(
            construction,
            PairConstruction::NotApplicable(ref reason) if reason == "slot_generation_below_two"
        ));
    }
}

/// 非测试版的「轮换没落盘」造法与测试模块里那份逐字节相同。
#[test]
fn the_unwritten_rotation_pool_matches_the_one_the_earlier_runs_build() {
    let node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("起点");
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let ours = pool_with_the_system_configuration_rotation_unwritten(&node.pool, slot_spacing)
        .expect("世代号够");
    let earlier = crate::tests::with_the_rotation_after_the_newest_publish_not_written(&node.pool);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    for device in [0u32, 1] {
        for offset in [0u64, slot_spacing] {
            assert_eq!(
                ours.read(device, offset, slot_bytes),
                earlier.read(device, offset, slot_bytes)
            );
        }
    }
}

/// 孪生不带故障：计数 0；树表与分配记录树两份读不出：计数等于装置数的 through，且至少 1；单份坏：计数 0。
#[test]
fn the_twin_counts_zero_and_the_both_copies_variants_count_through() {
    let (node, parameters, pair) = constructed_pair_at_the_start_node();
    let root = *node.timeline.last().expect("时间线非空");
    let root_context = build_abandoned_root_context(&pair, root).expect("根读得出");
    let context = VariantContext {
        pair: &pair,
        root,
        root_context: &root_context,
        parameters: &parameters,
        geometry: &GEOMETRY_PRIMARY,
    };
    let twin = judge_variant(&context, VariantSpecification::Twin).expect("孪生");
    assert_eq!(twin.count, Some(0));
    assert!(twin.mounted_ok());
    for aspect in [LedgerAspect::TreeTable, LedgerAspect::AllocationRecordTree] {
        let both = judge_variant(
            &context,
            VariantSpecification::Account {
                aspect,
                severity: FaultSeverity::Both,
                form: UnreadableForm::ReadFails,
            },
        )
        .expect("两份坏");
        assert!(both.expected_count >= 1, "{aspect:?}");
        assert_eq!(both.count, Some(both.expected_count), "{aspect:?}");
        let locations =
            account_locations(aspect, &pair.plain_devices, &root_context.record).expect("位置");
        assert_eq!(
            both.shared,
            is_unit_shared(
                &pair.plain_devices,
                &pair.pool_after_abandonment,
                &pair.pool_geometry,
                root,
                &location_keys(&locations)
            ),
            "{aspect:?}"
        );
        let single = judge_variant(
            &context,
            VariantSpecification::Account {
                aspect,
                severity: FaultSeverity::Disk0,
                form: UnreadableForm::ReadFails,
            },
        )
        .expect("单份坏");
        assert_eq!(single.expected_count, 0);
        assert_eq!(single.count, Some(0), "{aspect:?}");
    }
}

/// 续写发布到 txg_a + 3S − 1 为止，每发布一次记一次累计复用数。
#[test]
fn the_twin_continuation_ends_at_the_txg_before_the_next_publish_on_the_root_slot() {
    let (node, parameters, pair) = constructed_pair_at_the_start_node();
    let root = *node.timeline.last().expect("时间线非空");
    let root_context = build_abandoned_root_context(&pair, root).expect("根读得出");
    let context = VariantContext {
        pair: &pair,
        root,
        root_context: &root_context,
        parameters: &parameters,
        geometry: &GEOMETRY_PRIMARY,
    };
    let twin = judge_variant(&context, VariantSpecification::Twin).expect("孪生");
    assert_eq!(twin.continued_to_txg, Some(root.1 + 3 * 8 - 1));
    assert!(!twin.reuse_trajectory.is_empty());
    assert!(twin
        .reuse_trajectory
        .windows(2)
        .all(|neighbours| neighbours[0] <= neighbours[1]));
}

fn small_domain(step_limit: u64) -> FifthRunDomain {
    FifthRunDomain {
        geometry: GEOMETRY_PRIMARY,
        step_limit,
        starting_overwrites: vec![1],
    }
}

fn lines_of_a_run(
    domain: &FifthRunDomain,
    threads: usize,
    cut_depth: u64,
    judgements: &ReusableJudgements,
) -> FifthRunReport {
    run_fifth_run_domain(domain, "today", threads, Some(cut_depth), judgements)
        .expect("小域上的一趟不停机")
}

fn count_of_lines_containing(lines: &[String], head: &str, needle: &str) -> u64 {
    u64::try_from(
        lines
            .iter()
            .filter(|line| line.starts_with(head) && line.contains(needle))
            .count(),
    )
    .expect("行数装得进 u64")
}

/// 一个单位里走到的节点数（沿用 `enumerate` 的计数，不做判定）。
fn nodes_of_a_unit(domain: &FifthRunDomain, unit: &EnumerationUnit) -> u64 {
    let parameters = parameters_for(&domain.geometry);
    let start = replay_steps(&domain.geometry, unit.starting_overwrites, &unit.prefix)
        .expect("前缀重放得出");
    let geometry_view = independent_geometry(&start.pool).expect("几何");
    let depth = u64::try_from(unit.prefix.len()).expect("深度");
    let remaining = if unit.whole_subtree {
        domain.step_limit - depth
    } else {
        0
    };
    let mut summary = EnumerationSummary::default();
    enumerate(
        &start,
        remaining,
        &parameters,
        &geometry_view,
        &mut summary,
        &mut |_, _, _| {},
    );
    summary.histories
}

/// 单位恰好划分整棵枚举树：各切深下单位里节点数之和等于不切时 `enumerate` 走到的节点数。
#[test]
fn the_units_of_every_cut_depth_partition_the_enumeration_exactly() {
    let domain = small_domain(2);
    let parameters = parameters_for(&domain.geometry);
    let root = bootstrap(&domain.geometry, 1).expect("起点");
    let geometry_view = independent_geometry(&root.pool).expect("几何");
    let mut whole = EnumerationSummary::default();
    enumerate(
        &root,
        domain.step_limit,
        &parameters,
        &geometry_view,
        &mut whole,
        &mut |_, _, _| {},
    );
    for cut_depth in 0..=domain.step_limit {
        let units = list_enumeration_units(&domain, cut_depth).expect("列单位");
        let total: u64 = units
            .iter()
            .map(|unit| nodes_of_a_unit(&domain, unit))
            .sum();
        assert_eq!(total, whole.histories, "切深 {cut_depth}");
    }
}

/// 并行不改产物（登记 5.7 第 7 条 (i)(iii)）：线程 1、2、7、31 与两个不同的切深，整份结果行逐字节相同；
/// 构造成立的对数 ≥ 10 才算有分辨力；单位数小于线程数的趟（线程 31）也在内。
#[test]
fn the_result_lines_do_not_depend_on_the_thread_count_or_the_cut_depth() {
    let domain = small_domain(2);
    let library = DisconnectedJudgementLibrary;
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let reference = lines_of_a_run(&domain, 1, 0, &judgements);
    assert!(
        count_of_lines_containing(&reference.lines, "name=r5_pair", "status=constructed") >= 10,
        "构造成立的对不足 10，小域没有分辨力：把步序列上限加到 3"
    );
    for (threads, cut_depth) in [(2usize, 1u64), (7, 1), (7, 2), (31, 1), (31, 2)] {
        let run = lines_of_a_run(&domain, threads, cut_depth, &judgements);
        assert_eq!(
            run.lines, reference.lines,
            "线程 {threads}、切深 {cut_depth} 的结果行与单线程不同"
        );
    }
}

/// 复用三问之一：同一份输入跑两趟，第二趟现判 0，复用数等于第一趟现判数，结果行逐字节相同。
#[test]
fn second_run_on_the_same_inputs_judges_nothing_fresh_and_prints_the_same_lines() {
    let domain = small_domain(1);
    let library = MemoryJudgementLibrary::default();
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let first = lines_of_a_run(&domain, 2, 0, &judgements);
    assert!(first.fresh > 0);
    assert_eq!(first.reused, 0);
    let second = lines_of_a_run(&domain, 3, 1, &judgements);
    assert_eq!(second.fresh, 0);
    assert_eq!(second.reused, first.fresh);
    assert_eq!(second.lines, first.lines);
}

/// 复用三问之二：改一处不在任何判定闭包里的输入（线程数、切深）再跑，现判 0。
#[test]
fn changing_only_the_thread_count_and_the_cut_depth_judges_nothing_fresh() {
    let domain = small_domain(1);
    let library = MemoryJudgementLibrary::default();
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let first = lines_of_a_run(&domain, 1, 1, &judgements);
    let again = lines_of_a_run(&domain, 5, 0, &judgements);
    assert_eq!(again.fresh, 0);
    assert_eq!(again.reused, first.fresh);
}

/// 复用三问之三：改闭包里一组判法的代码键再跑，现判的恰好是那一组的判定单位，其余复用。
#[test]
fn changing_the_root_slot_group_judges_exactly_the_root_slot_variants_fresh() {
    let domain = small_domain(1);
    let library = MemoryJudgementLibrary::default();
    let first_judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let first = lines_of_a_run(&domain, 2, 0, &first_judgements);
    let expected = expected_root_slot_units(&first.lines);
    let mut changed = JudgementCodeKeys::unconnected();
    changed.root_slot_unreadable = "changed".to_string();
    let second_judgements = ReusableJudgements {
        library: &library,
        code_keys: changed,
    };
    let second = lines_of_a_run(&domain, 2, 0, &second_judgements);
    assert!(
        expected > 0,
        "小域里没有根槽读不出的变体，三问的第三问没有分辨力"
    );
    assert_eq!(second.fresh, expected);
    assert_eq!(second.reused, first.fresh - second.fresh);
    assert_eq!(second.lines, first.lines);
}

/// 每个被抛弃根的根槽变体个数（两种形态各一个）：行里 `:root_slot.` 每出现一次是一个变体。
fn expected_root_slot_units(lines: &[String]) -> u64 {
    u64::try_from(
        lines
            .iter()
            .filter(|line| line.starts_with("name=r5_pair"))
            .map(|line| line.matches(":root_slot.").count())
            .sum::<usize>(),
    )
    .expect("个数装得进 u64")
}

/// 行里 `name=数` 那个字段的数。
fn field_of_a_line(line: &str, name: &str) -> u64 {
    let head = format!("{name}=");
    line.split(' ')
        .find_map(|token| token.strip_prefix(head.as_str()))
        .expect("行里有这个字段")
        .parse()
        .expect("字段是数")
}

fn arguments_after_the_mode(tail: &[&str]) -> Vec<String> {
    ["program", "r5-enum-s4"]
        .iter()
        .chain(tail.iter())
        .map(|word| (*word).to_string())
        .collect()
}

/// 每个单位恰好归一片，各片分到的单位数几乎相等（分片不偏深度优先序的周期）。
#[test]
fn every_unit_belongs_to_exactly_one_shard_and_the_shards_are_even() {
    for count in [2u64, 3, 5, 10] {
        let mut sizes = vec![0usize; usize::try_from(count).expect("片数装得进 usize")];
        for unit_index in 0..1000usize {
            let holders: Vec<u64> = (0..count)
                .filter(|index| {
                    UnitShard {
                        index: *index,
                        count,
                    }
                    .holds(unit_index)
                })
                .collect();
            assert_eq!(
                holders.len(),
                1,
                "片数 {count} 的单位 {unit_index} 归 {holders:?}"
            );
            sizes[usize::try_from(holders[0]).expect("片号装得进 usize")] += 1;
        }
        let largest = sizes.iter().max().expect("片非空");
        let smallest = sizes.iter().min().expect("片非空");
        assert!(
            largest - smallest <= 8,
            "片数 {count} 各片的单位数 {sizes:?} 不均匀"
        );
    }
}

/// 片都跑完之后的合并：一个判定也不现判，结果行与整趟一次跑的逐字相同；各片的单位数、对数加起来等于整趟的。
#[test]
fn the_merge_after_every_shard_prints_the_unsharded_lines_and_judges_nothing_fresh() {
    let domain = small_domain(2);
    let disconnected = DisconnectedJudgementLibrary;
    let unsharded = lines_of_a_run(
        &domain,
        1,
        0,
        &ReusableJudgements {
            library: &disconnected,
            code_keys: JudgementCodeKeys::unconnected(),
        },
    );
    let library = MemoryJudgementLibrary::default();
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let units_total = list_enumeration_units(&domain, 2).expect("列单位").len();
    let shard_count = 3u64;
    let mut units_in_the_shards = 0usize;
    let mut pairs_in_the_shards = 0u64;
    let mut fresh_in_the_shards = 0u64;
    for index in 0..shard_count {
        let shard = UnitShard {
            index,
            count: shard_count,
        };
        let report = run_fifth_run_shard(&domain, "today", 2, Some(2), shard, &judgements)
            .expect("片不停机");
        assert_eq!(report.units_total, units_total);
        assert_eq!(report.reused, 0, "各片的单位两两不交，判定键不重");
        units_in_the_shards += report.units_in_shard;
        fresh_in_the_shards += report.fresh;
        pairs_in_the_shards += field_of_a_line(&report.lines[0], "pairs");
    }
    assert_eq!(units_in_the_shards, units_total);
    assert_eq!(
        pairs_in_the_shards,
        u64::try_from(
            unsharded
                .lines
                .iter()
                .filter(|line| line.starts_with("name=r5_pair "))
                .count()
        )
        .expect("行数装得进 u64")
    );
    let reuse_only = ReuseOnlyJudgementLibrary::over(&library, 0);
    let merging = ReusableJudgements {
        library: &reuse_only,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let merged = lines_of_a_run(&domain, 3, 1, &merging);
    assert_eq!(merged.fresh, 0);
    assert_eq!(merged.reused, fresh_in_the_shards);
    assert_eq!(merged.lines, unsharded.lines);
}

/// 缺一片的合并当场停并点名缺的判定，不悄悄现判。
#[test]
fn merge_with_a_shard_missing_stops_and_names_the_missing_judgement() {
    let domain = small_domain(2);
    let library = MemoryJudgementLibrary::default();
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let shard_count = 3u64;
    for index in 0..2 {
        let shard = UnitShard {
            index,
            count: shard_count,
        };
        run_fifth_run_shard(&domain, "today", 2, Some(2), shard, &judgements).expect("片不停机");
    }
    let units_total = list_enumeration_units(&domain, 2).expect("列单位").len();
    let missing_shard = UnitShard {
        index: 2,
        count: shard_count,
    };
    assert!(
        (0..units_total).any(|unit_index| missing_shard.holds(unit_index)),
        "缺的那一片没分到单位，缺片的合并没有分辨力"
    );
    let reuse_only = ReuseOnlyJudgementLibrary::over(&library, 0);
    let merging = ReusableJudgements {
        library: &reuse_only,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let error = run_fifth_run_domain(&domain, "today", 2, Some(2), &merging)
        .err()
        .expect("缺一片的合并该停");
    assert!(error.contains("库里没有判定"), "{error}");
}

/// 合并许现判的例外只有给定的个数：缺一个判定时现判它（转交哨兵让回「不在」的那一条就是这样），缺两个就停。
#[test]
fn the_merge_judges_fresh_only_as_many_missing_judgements_as_it_is_allowed() {
    let domain = small_domain(2);
    let library = MemoryJudgementLibrary::default();
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let complete = lines_of_a_run(&domain, 2, 2, &judgements);
    let one_removed = library.remove_first_judgement();
    let one_missing = ReuseOnlyJudgementLibrary::over(&library, 1);
    let merging = ReusableJudgements {
        library: &one_missing,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let merged = lines_of_a_run(&domain, 2, 2, &merging);
    assert_eq!(merged.fresh, 1, "缺的 {one_removed} 现判一个");
    assert_eq!(merged.lines, complete.lines);
    library.remove_first_judgement();
    library.remove_first_judgement();
    let two_missing = ReuseOnlyJudgementLibrary::over(&library, 1);
    let refusing = ReusableJudgements {
        library: &two_missing,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let error = run_fifth_run_domain(&domain, "today", 2, Some(2), &refusing)
        .err()
        .expect("缺两个、只许现判一个的合并该停");
    assert!(error.contains("库里没有判定"), "{error}");
}

/// 片与合并的命令行参数：成对、范围、互斥、单线程对拍趟不许。
#[test]
fn the_assembly_arguments_are_checked() {
    let assembly =
        |tail: &[&str]| fifth_run_assembly_of("r5-enum-s4", &arguments_after_the_mode(tail));
    assert_eq!(assembly(&[]), Ok(FifthRunAssembly::Whole));
    assert_eq!(
        assembly(&["--shard-index", "1", "--shard-count", "4"]),
        Ok(FifthRunAssembly::Shard(UnitShard { index: 1, count: 4 }))
    );
    assert_eq!(
        assembly(&[REUSE_ONLY_FLAG]),
        Ok(FifthRunAssembly::ReuseOnlyFromTheLibrary)
    );
    let refused: [&[&str]; 8] = [
        &["--shard-index", "4", "--shard-count", "4"],
        &["--shard-index", "0", "--shard-count", "0"],
        &["--shard-index", "0", "--shard-count", "65537"],
        &["--shard-index", "0"],
        &["--shard-count", "4"],
        &["--shard-index", "x", "--shard-count", "4"],
        &["--shard-count", "4", "--shard-index"],
        &["--shard-index", "0", "--shard-count", "4", REUSE_ONLY_FLAG],
    ];
    for tail in refused {
        assert!(assembly(tail).is_err(), "{tail:?} 该拒");
    }
    let one_thread = |tail: &[&str]| {
        fifth_run_assembly_of("r5-enum-g0-one-thread", &arguments_after_the_mode(tail))
    };
    assert_eq!(one_thread(&[]), Ok(FifthRunAssembly::Whole));
    assert!(one_thread(&[REUSE_ONLY_FLAG]).is_err());
    assert!(one_thread(&["--shard-index", "0", "--shard-count", "2"]).is_err());
}

/// 模式入口里三种拼法各走各的路：库是空的，合并停（退出码 1）；两片各自跑成（0），再合并成（0）。
#[test]
fn the_mode_runs_a_merge_only_after_every_shard_has_filled_the_library() {
    let judging = FifthRunJudging {
        library: Box::new(MemoryJudgementLibrary::default()),
        code_keys: JudgementCodeKeys::unconnected(),
        description: "judgement_library=test".to_string(),
    };
    let domain = small_domain(2);
    let run = |assembly: FifthRunAssembly| {
        run_fifth_run_enumeration("r5-enum-g0", "today", &domain, 2, assembly, &judging)
    };
    assert_eq!(run(FifthRunAssembly::ReuseOnlyFromTheLibrary), 1);
    for index in 0..2 {
        let shard = UnitShard { index, count: 2 };
        assert_eq!(run(FifthRunAssembly::Shard(shard)), 0, "片 {index}");
    }
    assert_eq!(run(FifthRunAssembly::ReuseOnlyFromTheLibrary), 0);
}

/// 片与合并的参数行带对账用的字段：各片的切深与单位总数、这片的单位数；合并那一行带 `assembly=`。
#[test]
fn the_parameter_lines_carry_the_fields_the_shards_are_reconciled_by() {
    let domain = small_domain(2);
    let library = MemoryJudgementLibrary::default();
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let shard = UnitShard { index: 1, count: 2 };
    let report =
        run_fifth_run_shard(&domain, "today", 2, Some(2), shard, &judgements).expect("片不停机");
    let line = report.parameters_line(
        "r5-enum-s4",
        "today",
        &domain,
        2,
        shard,
        "judgement_library=test",
    );
    for expected in [
        format!(" cut_depth={} ", report.cut_depth),
        format!(" units_total={} ", report.units_total),
        " shard_index=1 shard_count=2 ".to_string(),
        format!(" units_in_shard={} ", report.units_in_shard),
        format!(" judgements_fresh={} ", report.fresh),
    ] {
        assert!(line.contains(&expected), "{expected:?} 不在 {line}");
    }
    assert!(report.lines[0].starts_with("name=r5_shard_summary "));
    for field in [
        " nodes=",
        " pairs=",
        " transition_failures=",
        " pair_rows_sha256=",
    ] {
        assert!(
            report.lines[0].contains(field),
            "{field} 不在 {}",
            report.lines[0]
        );
    }
    let whole = lines_of_a_run(&domain, 2, 2, &judgements);
    let merged_line = run_parameters_line(
        "r5-enum-s4",
        "today",
        &domain,
        2,
        FifthRunAssembly::ReuseOnlyFromTheLibrary,
        &whole,
        "judgement_library=test",
    );
    assert!(
        merged_line.ends_with(" assembly=reuse_only_from_the_library"),
        "{merged_line}"
    );
}

/// 判定库口子的读写计时：每读一次、每成批写一次各记一次；不计时的口子交 None。
#[test]
fn the_library_timing_counts_every_read_and_every_batched_write() {
    let library = DeviceJudgementLibrary {
        store: DeviceLibrary::in_memory(),
        timing: LibraryTiming::default(),
    };
    library
        .write_judgements(&JudgementsAwaitingWrite::from([(
            "k".to_string(),
            "v".to_string(),
        )]))
        .expect("写得进");
    assert_eq!(
        library.read_judgement("k").expect("读得回"),
        Some("v".to_string())
    );
    assert_eq!(library.read_judgement("absent").expect("不在不是错"), None);
    let summary = library.timing_summary().expect("连库的口子计时");
    assert!(summary.starts_with("library_read_calls=2 "), "{summary}");
    assert!(summary.contains(" library_write_calls=1 "), "{summary}");
    assert_eq!(DisconnectedJudgementLibrary.timing_summary(), None);
}

fn trivial_units(count: u64) -> Vec<EnumerationUnit> {
    (0..count)
        .map(|index| EnumerationUnit {
            starting_overwrites: index,
            prefix: Vec::new(),
            whole_subtree: false,
        })
        .collect()
}

/// 合并要确定：先做完的单位排在后面（单位 0 睡得最久），交回的次序仍是单位序。
#[test]
fn outputs_come_back_in_unit_order_whatever_order_the_workers_finish_in() {
    let units = trivial_units(8);
    let process = |unit: &EnumerationUnit| -> Result<UnitOutput, String> {
        std::thread::sleep(std::time::Duration::from_millis(
            (8 - unit.starting_overwrites) * 25,
        ));
        Ok(UnitOutput {
            nodes_visited: unit.starting_overwrites,
            ..UnitOutput::default()
        })
    };
    let outputs = run_units_in_parallel(&units, 4, &process).expect("全部收回");
    let order: Vec<u64> = outputs.iter().map(|output| output.nodes_visited).collect();
    assert_eq!(order, (0..8).collect::<Vec<u64>>());
}

/// 派出去的单位数等于收回的单位数（A6）：每个单位恰好做一次。
#[test]
fn every_unit_is_dispatched_once_and_collected() {
    let units = trivial_units(5);
    let process = |unit: &EnumerationUnit| -> Result<UnitOutput, String> {
        Ok(UnitOutput {
            nodes_visited: 1 + unit.starting_overwrites,
            ..UnitOutput::default()
        })
    };
    let outputs = run_units_in_parallel(&units, 3, &process).expect("全部收回");
    let visited: Vec<u64> = outputs.iter().map(|output| output.nodes_visited).collect();
    assert_eq!(visited, vec![1, 2, 3, 4, 5]);
}

#[test]
fn failed_unit_makes_the_whole_run_fail_and_names_the_unit() {
    let units = trivial_units(4);
    let process = |unit: &EnumerationUnit| -> Result<UnitOutput, String> {
        if unit.starting_overwrites == 2 {
            Err("故意失败".to_string())
        } else {
            Ok(UnitOutput::default())
        }
    };
    let error = run_units_in_parallel(&units, 2, &process).expect_err("单位 2 失败");
    assert!(error.contains("reason=unit_failed"), "{error}");
    assert!(error.contains("unit=2"), "{error}");
}

#[test]
fn panicking_worker_makes_the_whole_run_fail() {
    let units = trivial_units(4);
    let process = |unit: &EnumerationUnit| -> Result<UnitOutput, String> {
        assert_ne!(unit.starting_overwrites, 1, "故意 panic");
        Ok(UnitOutput::default())
    };
    let error = run_units_in_parallel(&units, 2, &process).expect_err("单位 1 panic");
    assert!(error.contains("reason=worker_panic"), "{error}");
}

/// 续写的复用数按续写开始时的镜像比：续写发布了几次，累计数不减、末态上直接重算得同一个数。
#[test]
fn the_continuation_counts_the_watched_slots_written_after_its_baseline() {
    let node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("起点");
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let InstrumentedMount {
        result, pool_after, ..
    } = run_instrumented_mount_writable(&node.pool, &parameters, &[], None, None);
    let mounted = result.expect("可写挂载做成");
    let start_txg = mounted.current.root().checkpoint_txg.0;
    let session = Session {
        allocator: mounted.allocator,
        current: mounted.current,
        instance: mounted.output.instance,
    };
    let watched: BTreeSet<(u32, u64, u64)> = [0u32, 1]
        .into_iter()
        .flat_map(|device| (1200..1700u64).map(move |slot| (device, slot, 1)))
        .collect();
    let (reached, trajectory, end_state) = continue_with_overwrites(
        &pool_after,
        session,
        &parameters,
        &pool_after,
        &watched,
        start_txg + 5,
    )
    .expect("续写");
    assert_eq!(reached, start_txg + 5);
    assert_eq!(trajectory.len(), 5);
    assert!(trajectory.windows(2).all(|window| window[0] <= window[1]));
    let last = trajectory.last().copied().expect("发布过五次");
    assert!(last > 0, "盯的槽里没有一个被续写写到，这个用例没有分辨力");
    assert_eq!(
        placements_overwritten_between(&pool_after, &end_state, &watched),
        last
    );
}

/// 被抛弃根独占的落点（U_a）非空，且与其余每条可读根引用的槽不相交。
#[test]
fn the_exclusive_placements_of_the_abandoned_root_are_referenced_by_no_other_readable_root() {
    let (node, _parameters, pair) = constructed_pair_at_the_start_node();
    let root = *node.timeline.last().expect("时间线非空");
    let root_context = build_abandoned_root_context(&pair, root).expect("根读得出");
    assert!(!root_context.exclusive_placements.is_empty());
    let exclusive = slots_of_placements(&root_context.exclusive_placements);
    let mut others_seen = 0;
    for other in readable_roots_independent(&pair.pool_after_abandonment, &pair.pool_geometry) {
        if other == root {
            continue;
        }
        let record = root_record_of(&pair.pool_after_abandonment, &pair.pool_geometry, other)
            .expect("可读根的根记录读得出");
        let referenced = slots_of_placements(
            &placements_referenced_by_root_on(&pair.pool_after_abandonment, &record)
                .expect("可读根的账读得出"),
        );
        assert!(exclusive.is_disjoint(&referenced), "{other:?}");
        others_seen += 1;
    }
    assert!(others_seen > 0, "只有一条可读根，这个用例没有分辨力");
}

/// 开跑检查（停机 S1）在今天这份 `crates/` 上全过。
#[test]
fn the_fifth_run_constants_and_anchors_all_pass() {
    assert!(run_fifth_run_constants_and_anchors("today"));
}

/// 一趟的产物行里有每个量的汇总行、六个阳性对照、判决行与摘要；孪生、复用检测与空域三个对照在今天的代码上过。
#[test]
fn small_run_prints_every_quantity_every_positive_control_and_the_digest() {
    let library = DisconnectedJudgementLibrary;
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let report = lines_of_a_run(&small_domain(1), 2, 1, &judgements);
    for name in [
        "r5_q0",
        "r5_q1",
        "r5_q2a",
        "r5_q2b",
        "r5_q2c",
        "r5_q3a",
        "r5_q3b",
        "r5_q3c",
        "r5_q4",
        "r5_q5",
        "r5_q6",
        "r5_q7",
        "r5_anchors",
        "verdict",
        "r5_digest",
    ] {
        assert_eq!(
            count_of_lines_containing(&report.lines, &format!("name={name} "), "arm=today"),
            1,
            "{name}"
        );
    }
    for control in [
        "PC1-a",
        "PC1-b",
        "PC1-c",
        "PC-twin",
        "PC-reuse-detection",
        "PC-empty-domain",
    ] {
        assert_eq!(
            count_of_lines_containing(
                &report.lines,
                "name=r5_positive_control ",
                &format!("control={control}")
            ),
            1,
            "{control}"
        );
    }
    for control in ["PC-twin", "PC-reuse-detection", "PC-empty-domain"] {
        let line = report
            .lines
            .iter()
            .find(|line| {
                line.starts_with("name=r5_positive_control ")
                    && line.contains(&format!("control={control}"))
            })
            .expect("上面核过恰好一行");
        assert!(line.ends_with("verdict=pass"), "{line}");
    }
    assert!(report
        .lines
        .last()
        .expect("至少有摘要行")
        .starts_with("name=r5_digest "));
}

/// 组成清单的样子（device-code-key.py 文件头「组成清单」）：四个入口各一行 code_key。
fn composition_with(keys: [&str; 4]) -> String {
    let code_key_lines: String = JUDGEMENT_ENTRIES
        .iter()
        .zip(keys)
        .map(|((entry, _), key)| format!("code_key\t{entry}\t{key}\n"))
        .collect();
    format!("{COMPOSITION_FORM_LINE}\njudged\t2026-11-09\nhost\t本机\n{code_key_lines}")
}

fn judging_over(store: &DeviceLibrary, keys: [&str; 4]) -> FifthRunJudging {
    let file = std::env::temp_dir().join(format!(
        "e158-r5-composition-{}-{:?}-{}.txt",
        std::process::id(),
        std::thread::current().id(),
        keys.join("-")
    ));
    std::fs::write(&file, composition_with(keys)).expect("写得进测试的组成清单");
    let arguments = vec![
        "r5-enum-g0".to_string(),
        "--code-key-file".to_string(),
        file.to_string_lossy().to_string(),
    ];
    let connected = connect_judgements(&arguments, Ok(store.clone()));
    std::fs::remove_file(&file).expect("清掉测试的组成清单");
    connected.expect("清单与管道都对，接得上")
}

/// 接线：判定值经真口子（单测里是 DeviceLibrary 的内存那一种，与转交那一种同一套读写）存成 items/<键>.out，
/// 同一输入第二趟现判 0、结果行逐字节相同；组成清单按每把代码键存进 codes/。
#[test]
fn the_connected_library_serves_the_second_run_and_stores_the_composition() {
    let domain = small_domain(1);
    let store = DeviceLibrary::in_memory();
    let keys = ["aa01", "bb02", "cc03", "dd04"];
    let first_judging = judging_over(&store, keys);
    let first_judgements = ReusableJudgements {
        library: first_judging.library.as_ref(),
        code_keys: first_judging.code_keys.clone(),
    };
    let first = lines_of_a_run(&domain, 2, 0, &first_judgements);
    assert!(first.fresh > 0);
    assert_eq!(
        u64::try_from(store.count_entries("items/", ".out")).expect("条数装得进"),
        first.fresh,
        "每个现判的单位在库里恰好一条 items/<键>.out"
    );
    assert_eq!(store.count_entries("codes/", ".txt"), 4);
    let second_judging = judging_over(&store, keys);
    let second_judgements = ReusableJudgements {
        library: second_judging.library.as_ref(),
        code_keys: second_judging.code_keys.clone(),
    };
    let second = lines_of_a_run(&domain, 3, 1, &second_judgements);
    assert_eq!(second.fresh, 0);
    assert_eq!(second.reused, first.fresh);
    assert_eq!(second.lines, first.lines);
}

/// 接线后改一组的代码键：现判恰好是那一组的单位（经真口子走一遍，不止内存桩）。
#[test]
fn the_connected_library_rejudges_only_the_group_whose_code_key_changed() {
    let domain = small_domain(1);
    let store = DeviceLibrary::in_memory();
    let first_judging = judging_over(&store, ["aa01", "bb02", "cc03", "dd04"]);
    let first_judgements = ReusableJudgements {
        library: first_judging.library.as_ref(),
        code_keys: first_judging.code_keys.clone(),
    };
    let first = lines_of_a_run(&domain, 2, 0, &first_judgements);
    let expected = expected_root_slot_units(&first.lines);
    assert!(expected > 0);
    let second_judging = judging_over(&store, ["aa01", "bb02", "cc03", "dd05"]);
    let second_judgements = ReusableJudgements {
        library: second_judging.library.as_ref(),
        code_keys: second_judging.code_keys.clone(),
    };
    let second = lines_of_a_run(&domain, 2, 0, &second_judgements);
    assert_eq!(second.fresh, expected);
}

/// 读回的头对不上这把键（库里的项被换过、截断）：交 Err，不当「库里没有」、不当命中。
#[test]
fn an_item_whose_header_names_another_key_is_an_error_not_a_hit() {
    let store = DeviceLibrary::in_memory();
    store.write("items/k1.out", "key judgement_key=other\nbody");
    let library = DeviceJudgementLibrary {
        store,
        timing: LibraryTiming::default(),
    };
    assert!(library.read_judgement("k1").is_err());
    assert_eq!(library.read_judgement("absent").expect("不在不是错"), None);
    library
        .write_judgements(&JudgementsAwaitingWrite::from([(
            "k2".to_string(),
            "a\nb".to_string(),
        )]))
        .expect("写得进");
    assert_eq!(
        library.read_judgement("k2").expect("读得回"),
        Some("a\nb".to_string()),
        "值里带换行也原样读回"
    );
}

/// 一个工作单位只发一次批量写（数 store 的成批写次数）：头一趟每个单位各一次、条数等于现判数；
/// 第二趟全复用，一次都不发，结果行逐字节相同。切深 1 的小域单位数大于 1，才分得开「每单位一次」与「整趟一次」。
#[test]
fn one_unit_sends_one_batched_write_and_a_fully_reused_run_sends_none() {
    let domain = small_domain(1);
    let store = DeviceLibrary::in_memory();
    let library = DeviceJudgementLibrary {
        store: store.clone(),
        timing: LibraryTiming::default(),
    };
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let first = lines_of_a_run(&domain, 2, 1, &judgements);
    assert!(first.units > 1, "单位数 {} 分不开每单位一次", first.units);
    assert_eq!(
        store.count_write_many_calls(),
        first.units,
        "每个单位都判了东西，各发恰好一次成批写"
    );
    assert_eq!(
        u64::try_from(store.count_entries("items/", ".out")).expect("条数装得进"),
        first.fresh,
        "批里的条数加起来等于现判数"
    );
    let second = lines_of_a_run(&domain, 3, 0, &judgements);
    assert_eq!(second.fresh, 0);
    assert_eq!(second.lines, first.lines);
    assert_eq!(
        store.count_write_many_calls(),
        first.units,
        "全复用的一趟没有东西要写，一次成批写都不发"
    );
}

/// 同一单位里同一把键：只判一次、算一次复用、待写的那一批里只一条；单位判完之前库里一条都没有。
#[test]
fn key_judged_twice_in_one_unit_is_judged_once_and_awaits_one_write() {
    let library = MemoryJudgementLibrary::default();
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let mut awaiting_write = JudgementsAwaitingWrite::new();
    let judge_calls = std::cell::Cell::new(0u32);
    let judge = || {
        judge_calls.set(judge_calls.get() + 1);
        Ok("value".to_string())
    };
    let first = judgements
        .judge_or_reuse(&mut awaiting_write, JudgementGroup::Twin, "same-key", judge)
        .expect("判得出");
    let second = judgements
        .judge_or_reuse(&mut awaiting_write, JudgementGroup::Twin, "same-key", judge)
        .expect("判得出");
    assert_eq!(first, ("value".to_string(), false));
    assert_eq!(second, ("value".to_string(), true), "同一把键第二次算复用");
    assert_eq!(judge_calls.get(), 1, "同一单位里同一把键只判一次");
    assert_eq!(awaiting_write.len(), 1, "待写的那一批里只一条");
    assert!(
        library.rows.lock().expect("内存判定库的锁").is_empty(),
        "单位判完之前不写库"
    );
}

/// 读到第 `reads_answered` 次之后读都交 Err，写只记次数：造一个判到一半出错的单位。
struct LibraryFailingAfterSomeReads {
    reads_answered: u64,
    reads_seen: std::sync::atomic::AtomicU64,
    write_calls: std::sync::atomic::AtomicU64,
}

impl JudgementLibrary for LibraryFailingAfterSomeReads {
    fn read_judgement(&self, _key: &str) -> Result<Option<String>, String> {
        let reads_before = self
            .reads_seen
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if reads_before < self.reads_answered {
            Ok(None)
        } else {
            Err("这一次读没成".to_string())
        }
    }

    fn write_judgements(&self, _judgements: &JudgementsAwaitingWrite) -> Result<(), String> {
        self.write_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

/// 单位判到一半出错（头一个对已现判、下一个读没成）：交 Err，这一单位一条都不写，重跑时它整个现判。
#[test]
fn unit_that_fails_midway_writes_nothing() {
    let domain = small_domain(1);
    let library = LibraryFailingAfterSomeReads {
        reads_answered: 1,
        reads_seen: std::sync::atomic::AtomicU64::new(0),
        write_calls: std::sync::atomic::AtomicU64::new(0),
    };
    let judgements = ReusableJudgements {
        library: &library,
        code_keys: JudgementCodeKeys::unconnected(),
    };
    let units = list_enumeration_units(&domain, 0).expect("列单位");
    let error = process_unit(&units[0], &domain, "today", &judgements)
        .expect_err("第二次读没成，这一单位判不完");
    assert_eq!(error, "这一次读没成");
    assert_eq!(
        library
            .write_calls
            .load(std::sync::atomic::Ordering::SeqCst),
        0,
        "判到一半出错的单位不写库"
    );
}

/// 组成清单读法：四个入口各恰好一行；缺一个、多一行、认不得的入口、不是十六进制、第一行不是 form，都报错。
#[test]
fn the_composition_must_name_each_of_the_four_entries_exactly_once() {
    let good = composition_with(["aa11", "bb22", "cc33", "dd44"]);
    let keys = JudgementCodeKeys::from_composition(&good).expect("四行齐全");
    assert_eq!(
        (
            keys.abandonment.as_str(),
            keys.twin.as_str(),
            keys.account_unreadable.as_str(),
            keys.root_slot_unreadable.as_str()
        ),
        ("aa11", "bb22", "cc33", "dd44")
    );
    let without_last = good
        .lines()
        .filter(|line| !line.contains("root_slot_unreadable_judgement_entry"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(JudgementCodeKeys::from_composition(&without_last).is_err());
    let doubled = format!("{good}code_key\t{}\tff\n", JUDGEMENT_ENTRIES[0].0);
    assert!(JudgementCodeKeys::from_composition(&doubled).is_err());
    let foreign = format!("{good}code_key\tsinglefs_verification::other\tff\n");
    assert!(JudgementCodeKeys::from_composition(&foreign).is_err());
    assert!(
        JudgementCodeKeys::from_composition(&composition_with(["aa11", "zz", "cc33", "dd44"]))
            .is_err()
    );
    assert!(JudgementCodeKeys::from_composition(&good.replacen("form", "forn", 1)).is_err());
}

/// 不退回本地：没给 --code-key-file、给了没跟文件、管道没有，都报错，不交一个「没连库」的口子。
#[test]
fn missing_code_key_file_or_relay_is_refused_not_replaced_by_a_local_judging() {
    let relay_missing = || Err("环境里没有管道".to_string());
    assert!(connect_judgements(&["r5-enum-g0".to_string()], relay_missing()).is_err());
    let dangling = vec!["r5-enum-g0".to_string(), "--code-key-file".to_string()];
    assert!(connect_judgements(&dangling, relay_missing()).is_err());
    let file = std::env::temp_dir().join(format!("e158-r5-norelay-{}.txt", std::process::id()));
    std::fs::write(&file, composition_with(["a1", "b2", "c3", "d4"])).expect("写得进");
    let arguments = vec![
        "r5-enum-g0".to_string(),
        "--code-key-file".to_string(),
        file.to_string_lossy().to_string(),
    ];
    let refused = connect_judgements(&arguments, relay_missing());
    std::fs::remove_file(&file).expect("清掉");
    assert!(refused.is_err(), "清单对而管道没有，也不退回本地");
}

/// 四个入口在源码里各自有一个同名函数（门禁与 device-code-key.py 按入口名取闭包；名字写错取键当场拒）。
#[test]
fn every_registered_entry_names_a_function_in_this_file() {
    let source = include_str!("../main.rs");
    for (entry, _) in JUDGEMENT_ENTRIES {
        let name = entry
            .strip_prefix("singlefs_implementation_experiments::")
            .expect("入口头一段是包名");
        assert!(
            source.contains(&format!("\nfn {name}(")),
            "入口 {entry} 在源码里没有顶层函数"
        );
    }
}
