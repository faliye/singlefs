use super::*;

/// 放量不设缩域变量（守卫清掉环境）时取引擎常量，就是全量域：30 次写的段展开全部真子集、31 次写的段不展开、整条流按 30 判全域。
/// 判别力（`crates/mutations.tsv`）：常量改成 6，这条红。
#[test]
fn the_expand_up_to_of_the_amplification_is_thirty_writes_without_any_variable_set() {
    let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
    let limit = ExpandUpToWritesPerSegment::of_the_amplification_from_the_environment();
    assert_eq!(
        limit,
        ExpandUpToWritesPerSegment(EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION)
    );
    assert_eq!(
        limit.expansion_of(&[0; 30]),
        CrashPointReplaySegmentExpansion::EveryProperSubset
    );
    assert_eq!(
        limit.expansion_of(&[0; 31]),
        CrashPointReplaySegmentExpansion::NotExpanded
    );
    assert!(limit.covers_every_segment(&[vec![0; 2], vec![0; 30]]));
    assert!(!limit.covers_every_segment(&[vec![0; 2], vec![0; 31]]));
    assert_eq!(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment(),
        None,
        "没设就是不缩"
    );
}

/// 缩域变量设了就照它缩、只往小缩：放量取它与引擎常量里小的那个（写 6 取 6，两头的空白不算；写 40 照旧 30）。
/// 判别力（`crates/mutations.tsv`）：放量不取两者里小的那个（照变量写多少取多少），这条红。
#[test]
fn the_expand_up_to_variable_narrows_the_amplification_and_never_widens_it() {
    assert_eq!(
        ExpandUpToWritesPerSegment::of_the_amplification_and_the_variable_value(Some(" 6\n")),
        ExpandUpToWritesPerSegment(6)
    );
    assert_eq!(
        ExpandUpToWritesPerSegment::of_the_amplification_and_the_variable_value(Some("40")),
        ExpandUpToWritesPerSegment(EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION),
        "比常量大的不放宽"
    );
    assert_eq!(
        ExpandUpToWritesPerSegment::of_the_amplification_and_the_variable_value(None),
        ExpandUpToWritesPerSegment(EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION)
    );
}

/// 展开上限的变量写坏了（不是非负整数）：panic，消息带变量名与原值，不悄悄退回全量域。
#[test]
#[should_panic(expected = "SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=\"2B\" 认不出")]
fn an_unreadable_expand_up_to_value_panics_naming_the_variable_and_the_value() {
    let _ = ExpandUpToWritesPerSegment::of_the_amplification_and_the_variable_value(Some("2B"));
}

/// 崩溃枚举用例缩域：写数超过上限的段不展开，没超过的照用例自己的展开（全部真子集、甲二、不展开都照留，缩不放宽）。
/// 判别力（`crates/mutations.tsv`）：超过上限的段照用例自己的展开（不缩），这条红。
#[test]
fn a_narrowed_case_expansion_drops_segments_past_the_limit_and_keeps_its_own_expansion_below_it() {
    let limit = ExpandUpToWritesPerSegment(4);
    for own_expansion in [
        CrashPointReplaySegmentExpansion::EveryProperSubset,
        CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll,
        CrashPointReplaySegmentExpansion::NotExpanded,
    ] {
        assert_eq!(
            limit.narrowed_expansion(own_expansion, &[0; 4]),
            own_expansion,
            "4 次写不超过上限：照用例自己的 {own_expansion:?}"
        );
        assert_eq!(
            limit.narrowed_expansion(own_expansion, &[0; 5]),
            CrashPointReplaySegmentExpansion::NotExpanded,
            "5 次写超过上限：不展开（用例自己的是 {own_expansion:?}）"
        );
    }
}

#[test]
fn blocks_of_a_point_cover_its_states_without_gap() {
    let point = PlannedCrashPoint {
        name: "p".to_string(),
        ignored: false,
        segments: Vec::new(),
        ordinals: 10..10 + STATES_PER_BLOCK * 2 + 5,
        reuse_key: StepAttributes {
            write_table_with_barriers: [0; 32],
            parent_output: [0; 32],
        }
        .reuse_key(),
        path_number: "a".to_string(),
        identity: "f::p::0".to_string(),
        judge_inputs: [0; 32],
        plan_hash: [0; 32],
        pieces_around_the_slices: Vec::new(),
    };
    let blocks = point.blocks();
    assert_eq!(
        blocks,
        vec![
            0..STATES_PER_BLOCK,
            STATES_PER_BLOCK..2 * STATES_PER_BLOCK,
            2 * STATES_PER_BLOCK..2 * STATES_PER_BLOCK + 5
        ]
    );
}

/// L3：块键只由内容定——同一份流在两个设点文件、两个点名（崩溃枚举用例的项名就是它那个点的点名）下算出同一把块键；
/// 起步镜像改一个字节、改一段的展开方式，块键都变。判别力（`crates/mutations.tsv`）：把身份（带设点文件路径与点名）拼回块键，这条红。
#[test]
fn the_block_keys_of_a_flow_follow_its_content_and_not_its_setting_file_or_its_names() {
    let (writes, segments) = flow_parts_of_unit_and_journal_writes(&[(2, 1), (1, 1)]);
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("造的流有一次根槽写");
    let device = singlefs_core::address::DeviceIdentity(1);
    let base_with_the_byte = |byte: u8| {
        let mut base = MemoryPool::with_devices(&[device], 1 << 30);
        base.apply_writes(&[RetainedWrite {
            device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(1 << 20),
            contents: WrittenContents::Bytes(vec![byte; 512]),
        }]);
        base
    };
    let every_proper_subset = |_segment_index: usize, _segment: &[usize]| {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    };
    let first_segment_in_place_only = |segment_index: usize, _segment: &[usize]| {
        if segment_index == 0 {
            CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
        } else {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        }
    };
    let keys_of = |base: &MemoryPool,
                   code_file: &str,
                   point_name: &str,
                   expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion
                         + Sync)| {
        let flow = CrashFlow {
            base,
            writes: &writes,
            segments: &segments,
            judged_root_index,
            versions: &[],
            versions_without_a_file: &[],
            expansion,
            crash_points: vec![CrashPointSpan {
                name: point_name.to_string(),
                code_file_in_repository: code_file.to_string(),
                writes: 0..writes.len(),
                ignored: false,
            }],
        };
        let plan = plan_crash_points(&flow);
        plan.points
            .iter()
            .flat_map(|point| {
                point
                    .blocks()
                    .into_iter()
                    .map(|local| point.block_key(&local))
                    .collect::<Vec<VerdictBlockKey>>()
            })
            .collect::<Vec<VerdictBlockKey>>()
    };
    let base = base_with_the_byte(5);
    let keys = keys_of(
        &base,
        "crates/singlefs-verification/tests/one_setting_file.rs",
        "one_item",
        &every_proper_subset,
    );
    assert!(!keys.is_empty());
    assert_eq!(
        keys_of(
            &base,
            "crates/singlefs-verification/tests/another_setting_file.rs",
            "one_item",
            &every_proper_subset
        ),
        keys,
        "设点文件换了块键不变"
    );
    assert_eq!(
        keys_of(
            &base,
            "crates/singlefs-verification/tests/one_setting_file.rs",
            "another_item",
            &every_proper_subset
        ),
        keys,
        "点名（项名）换了块键不变"
    );
    let changed_base = base_with_the_byte(6);
    assert!(
        keys_of(
            &changed_base,
            "crates/singlefs-verification/tests/one_setting_file.rs",
            "one_item",
            &every_proper_subset
        )
        .iter()
        .all(|key| !keys.contains(key)),
        "起步镜像改一个字节，每一块的键都变"
    );
    assert_ne!(
        keys_of(
            &base,
            "crates/singlefs-verification/tests/one_setting_file.rs",
            "one_item",
            &first_segment_in_place_only
        ),
        keys,
        "改一段的展开方式，块键变"
    );
}

/// 场景：那条两次写落在两个扇区的小流，崩溃点那个点三个状态、整点一块（[0, 3)）；空库里只给同一块起点、块长 2 的那一块（[0, 2)，
/// 像块宽是 2 时判过的那一块）存一份判定。
/// 预期：同一块起点、块长不同的两块块键不同；[0, 3) 那一块问复用答不上（块长进块键：块宽改了，没判过的那几个状态不算作复用）。
/// 判别力（`crates/mutations.tsv`）：「块键：不分片的块块长不进块键」这一行让这一条红。
#[test]
fn a_block_of_another_length_from_the_same_start_is_not_reused() {
    let (_directory, mut store) = an_empty_library_for("block-of-another-length");
    a_flow_of_two_writes_to_two_sectors().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let point = &plan.points[0];
        let the_whole_point_as_one_block: Vec<Range<u64>> = std::iter::once(0..3).collect();
        assert_eq!(
            point.blocks(),
            the_whole_point_as_one_block,
            "前提：崩溃点那个点三个状态、整点一块"
        );
        let shorter = 0..2;
        let whole = 0..3;
        assert_ne!(
            point.block_key(&shorter),
            point.block_key(&whole),
            "同一块起点、块长不同的两块块键不同"
        );
        let version = JudgingUnitsFingerprint([9; DIGEST_BYTES]);
        let clean = store
            .register_verdict_vector(&EncodedVerdictVector(b"clean".to_vec()))
            .expect("登记判定向量");
        store
            .store_verdict_block(
                &point.block_key(&shorter),
                &version,
                &VerdictBlock::new(vec![clean; 2]).expect("一块至少一个状态"),
                &[],
            )
            .expect("存块");
        let conclusions = ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version);
        assert!(
            conclusions
                .is_stored_for_reuse(&store, 0, &point.block_key(&shorter))
                .expect("读库"),
            "前提：块长 2 的那一块存进去了、问得到"
        );
        assert!(
            !conclusions
                .is_stored_for_reuse(&store, 0, &point.block_key(&whole))
                .expect("读库"),
            "块长 3 的那一块问复用答不上：库里只有块长 2 的那一块"
        );
    });
}

/// 场景：那条两次写落在两个扇区的小流，崩溃点那个点改成管分片的段的点的样子：它的三个状态是一截不分片的段（截号 0）。
/// 预期：这一截里同一块起点、块长不同的两块（[0, 2) 与 [0, 3)）块键不同（不分片的那几截块长同样进块键）。
/// 判别力（`crates/mutations.tsv`）：「块键：管分片的段的点里不分片的那几截块长不进块键」这一行让这一条红。
#[test]
fn a_block_of_another_length_in_a_piece_not_sliced_has_another_block_key() {
    a_flow_of_two_writes_to_two_sectors().with_the_flow(|flow| {
        let mut plan = plan_crash_points(flow);
        let mut point = plan.points.swap_remove(0);
        let the_whole_point_as_one_block: Vec<Range<u64>> = std::iter::once(0..3).collect();
        assert_eq!(
            point.blocks(),
            the_whole_point_as_one_block,
            "前提：崩溃点那个点三个状态、整点一块"
        );
        point.pieces_around_the_slices = vec![PieceOfAPointWithASlice {
            local_states: 0..3,
            kind: KindOfAPiece::SegmentsNotSliced { piece_index: 0 },
        }];
        assert_ne!(
            point.block_key(&(0..2)),
            point.block_key(&(0..3)),
            "不分片的那一截里同一块起点、块长不同的两块块键不同"
        );
    });
}
