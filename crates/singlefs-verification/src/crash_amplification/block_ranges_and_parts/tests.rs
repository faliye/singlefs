use super::*;

/// 份的那几条用例的份长：一份 4 块，2^70 − 1 个态的那一段（2^54 块）拆成很多份，第 0 份、第 1 份都整份判得完。
const BLOCKS_PER_PART_OF_THE_PART_CASES: BlocksPerPartAsAPowerOfTwo = BlocksPerPartAsAPowerOfTwo(2);

/// `<份号>/<起块>-<止块>`。
fn part_range_of(part: u64, start: u64, end: u64) -> PartRangeOfASlicedFlow {
    PartRangeOfASlicedFlow {
        part: Unsigned256Bits::from_u64(part),
        blocks: Unsigned256Bits::from_u64(start)..Unsigned256Bits::from_u64(end),
    }
}

/// 照一段区间判一趟（假判段，同 [`store_a_clean_verdict_for_every_unjudged_block`] 只钉块键）：照区间定片、排计划、录入，
/// 区间点到的块里库里还没有判定块的记一个干净判定。交回点到的块的块键与这一趟现判了几块。
fn judge_a_part_range_with_the_fake_judge(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    domains: &[DomainOfARegisteredSegment],
    part_range: &PartRangeOfASlicedFlow,
    version: &JudgingUnitsFingerprint,
) -> (BTreeSet<VerdictBlockKey>, usize) {
    let selection = selection_of_the_part_range_of_the_flow(
        flow,
        domains,
        part_range,
        BLOCKS_PER_PART_OF_THE_PART_CASES,
    )
    .expect("区间在份以内");
    let plan = plan_crash_points_with_slices(flow, &selection.segment_slices);
    let blocks_of_the_plan: usize = plan.points.iter().map(|point| point.blocks().len()).sum();
    assert_eq!(
        u64::try_from(blocks_of_the_plan).expect("小"),
        selection.blocks_in_the_plan,
        "照区间算的块数与照这几片排出的计划的块数相同"
    );
    write_the_recorded_blocks(store, &plan).expect("录入");
    let clean = store
        .register_verdict_vector(&crate::verdict_store::EncodedVerdictVector(
            b"clean".to_vec(),
        ))
        .expect("登记判定向量");
    let mut selected_keys = BTreeSet::new();
    let mut judged_here = 0usize;
    let mut block_sequence = 0u64;
    for point in &plan.points {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !selection.blocks_of_the_plan.contains(&this_block) {
                continue;
            }
            let key = point.block_key(&local);
            if store
                .read_verdict_block(&key, version)
                .expect("读块")
                .is_none()
            {
                let states = usize::try_from(local.end - local.start).expect("一块装得进 usize");
                store
                    .store_verdict_block(
                        &key,
                        version,
                        &VerdictBlock::new(vec![clean; states]).expect("一块至少一个状态"),
                        &[],
                    )
                    .expect("存块");
                judged_here += 1;
            }
            selected_keys.insert(key);
        }
    }
    (selected_keys, judged_here)
}

/// 分片的段（点 0 里第 0 段）里段内块号是 `blocks` 那几块的块键：块从段内编号 0 起按 [`STATES_PER_BLOCK`] 切，块键锚在段内编号上。
fn keys_of_the_sliced_blocks(
    flow: &CrashFlow<'_>,
    domains: &[DomainOfARegisteredSegment],
    blocks: Range<u64>,
) -> BTreeSet<VerdictBlockKey> {
    let plan = plan_crash_points_with_slices(
        flow,
        &placeholder_slices_of_the_sliced_segments(flow.segments.len(), domains),
    );
    blocks
        .map(|block| {
            plan.points[0].block_key_of_a_slice_block(
                0,
                &StateCount1280Bits::from_u64(block * STATES_PER_BLOCK),
            )
        })
        .collect()
}

/// 份：先判后一段区间再判前一段，与反过来，两种次序下每段区间判到的块键相同，都是它自己那几块（第 0 份 2-4 是段内第 2、3 块，0-2 是第 0、1 块）；
/// 先照进度续跑判一片、库里的续点挪了，照区间点到的照旧是这几块。判别力（`crates/mutations.tsv`）：区间起点照止块算，这条红。
#[test]
fn a_part_range_judges_the_same_blocks_whichever_range_is_judged_first() {
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        let version = JudgingUnitsFingerprint([7; DIGEST_BYTES]);
        let later_range = part_range_of(0, 2, 4);
        let earlier_range = part_range_of(0, 0, 2);
        let (later_first_directory, mut later_first) =
            an_empty_library_for("part-range-later-first");
        // 照进度续跑先判一片（一块），续点挪到第 1 块
        let plan_to_find_the_resume_points = plan_crash_points_with_slices(
            flow,
            &placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains),
        );
        let conclusions = ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version);
        let (progress_slices, _resume_points) = segment_slices_of_the_recording(
            &later_first,
            flow.segments.len(),
            &domains,
            &plan_to_find_the_resume_points,
            &conclusions,
            STATES_PER_BLOCK,
        )
        .expect("读库");
        let progress_plan = plan_crash_points_with_slices(flow, &progress_slices);
        write_the_recorded_blocks(&mut later_first, &progress_plan).expect("录入");
        store_a_clean_verdict_for_every_unjudged_block(&mut later_first, &progress_plan, &version);
        advance_the_progress_of_the_sliced_segments(&mut later_first, &progress_plan, &conclusions)
            .expect("推进进度");
        let (later_keys_judged_first, _) = judge_a_part_range_with_the_fake_judge(
            &mut later_first,
            flow,
            &domains,
            &later_range,
            &version,
        );
        let (earlier_keys_judged_second, _) = judge_a_part_range_with_the_fake_judge(
            &mut later_first,
            flow,
            &domains,
            &earlier_range,
            &version,
        );
        let (earlier_first_directory, mut earlier_first) =
            an_empty_library_for("part-range-earlier-first");
        let (earlier_keys_judged_first, _) = judge_a_part_range_with_the_fake_judge(
            &mut earlier_first,
            flow,
            &domains,
            &earlier_range,
            &version,
        );
        let (later_keys_judged_second, _) = judge_a_part_range_with_the_fake_judge(
            &mut earlier_first,
            flow,
            &domains,
            &later_range,
            &version,
        );
        assert_eq!(later_keys_judged_first, later_keys_judged_second);
        assert_eq!(earlier_keys_judged_first, earlier_keys_judged_second);
        assert_eq!(
            later_keys_judged_first,
            keys_of_the_sliced_blocks(flow, &domains, 2..4),
            "第 0 份 2-4 是段内第 2、3 块"
        );
        assert_eq!(
            earlier_keys_judged_first,
            keys_of_the_sliced_blocks(flow, &domains, 0..2),
            "第 0 份 0-2 是段内第 0、1 块"
        );
        drop(later_first);
        drop(later_first_directory);
        drop(earlier_first);
        drop(earlier_first_directory);
    });
}

/// 份：同一份里几段首尾相接的区间（0-1、1-3、3-4）判完，判过的块键并起来等于这一份整份一趟（0-4）判完的，不重不漏；
/// 每段现判的块数就是它的长度。判别力（`crates/mutations.tsv`）：分片的段的片多切一块，这条红。
#[test]
fn ranges_tiling_a_part_judge_exactly_the_blocks_of_the_whole_part() {
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        let version = JudgingUnitsFingerprint([8; DIGEST_BYTES]);
        let (tiled_directory, mut tiled) = an_empty_library_for("part-range-tiled");
        let mut keys_of_the_tiles = BTreeSet::new();
        let mut blocks_of_the_tiles = 0usize;
        for (start, end) in [(0, 1), (1, 3), (3, 4)] {
            let (keys, judged_here) = judge_a_part_range_with_the_fake_judge(
                &mut tiled,
                flow,
                &domains,
                &part_range_of(0, start, end),
                &version,
            );
            assert_eq!(
                judged_here,
                usize::try_from(end - start).expect("小"),
                "区间 0/{start}-{end} 现判的块数就是它的长度"
            );
            blocks_of_the_tiles += keys.len();
            keys_of_the_tiles.extend(keys);
        }
        assert_eq!(
            blocks_of_the_tiles,
            keys_of_the_tiles.len(),
            "几段区间两两不相交"
        );
        let (whole_directory, mut whole) = an_empty_library_for("part-range-whole-part");
        let (keys_of_the_whole_part, judged_in_one_run) = judge_a_part_range_with_the_fake_judge(
            &mut whole,
            flow,
            &domains,
            &part_range_of(0, 0, 4),
            &version,
        );
        assert_eq!(judged_in_one_run, 4);
        assert_eq!(keys_of_the_tiles, keys_of_the_whole_part, "不重不漏");
        drop(tiled);
        drop(tiled_directory);
        drop(whole);
        drop(whole_directory);
    });
}

/// 份：跨两份各判一段（第 0 份 1-3、第 1 份 0-2），两份的块键不相交，各是各自那一段的状态（段内第 1、2 块与第 4、5 块）。
/// 判别力（`crates/mutations.tsv`）：份起点少乘一半份长，这条红。
#[test]
fn ranges_in_two_parts_judge_disjoint_blocks_each_of_its_own_states() {
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        let version = JudgingUnitsFingerprint([9; DIGEST_BYTES]);
        let (directory, mut store) = an_empty_library_for("part-range-two-parts");
        let (keys_of_part_zero, _) = judge_a_part_range_with_the_fake_judge(
            &mut store,
            flow,
            &domains,
            &part_range_of(0, 1, 3),
            &version,
        );
        let (keys_of_part_one, _) = judge_a_part_range_with_the_fake_judge(
            &mut store,
            flow,
            &domains,
            &part_range_of(1, 0, 2),
            &version,
        );
        assert!(keys_of_part_zero.is_disjoint(&keys_of_part_one));
        assert_eq!(
            keys_of_part_zero,
            keys_of_the_sliced_blocks(flow, &domains, 1..3)
        );
        assert_eq!(
            keys_of_part_one,
            keys_of_the_sliced_blocks(flow, &domains, 4..6),
            "第 1 份从段内第 4 块起"
        );
        drop(store);
        drop(directory);
    });
}

/// 份：一段区间判完之后再录入、再判同一段，全部复用、现判 0；同一段区间两趟的录入键相同（只判那一段读到的是自己这一段的清单），
/// 别的区间、不设区间那一份（不带片）的录入键与它不同（清单不会被别的片换掉）。
/// 判别力（`crates/mutations.tsv`）：带片的录入键照不带片的流算，这条红。
#[test]
fn a_part_range_judged_again_after_recording_again_reuses_every_block_under_its_own_recording_key()
{
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        let version = JudgingUnitsFingerprint([10; DIGEST_BYTES]);
        let (directory, mut store) = an_empty_library_for("part-range-again");
        let range = part_range_of(0, 1, 3);
        let (keys_of_the_first_run, judged_in_the_first_run) =
            judge_a_part_range_with_the_fake_judge(&mut store, flow, &domains, &range, &version);
        assert_eq!(judged_in_the_first_run, 2);
        let (keys_of_the_second_run, judged_in_the_second_run) =
            judge_a_part_range_with_the_fake_judge(&mut store, flow, &domains, &range, &version);
        assert_eq!(judged_in_the_second_run, 0, "全部复用、现判 0");
        assert_eq!(keys_of_the_first_run, keys_of_the_second_run);
        let recording_key_of = |part_range: &PartRangeOfASlicedFlow| {
            let selection = selection_of_the_part_range_of_the_flow(
                flow,
                &domains,
                part_range,
                BLOCKS_PER_PART_OF_THE_PART_CASES,
            )
            .expect("区间在份以内");
            recording_of_the_run_keyed_by_its_slices(
                "part-range-again",
                flow,
                &selection.segment_slices,
                registered,
                false,
                false,
            )
            .recording_key
        };
        assert_eq!(recording_key_of(&range), recording_key_of(&range));
        assert_ne!(
            recording_key_of(&range),
            recording_key_of(&part_range_of(1, 1, 3)),
            "别的区间录在别的键下"
        );
        let selection = selection_of_the_part_range_of_the_flow(
            flow,
            &domains,
            &range,
            BLOCKS_PER_PART_OF_THE_PART_CASES,
        )
        .expect("区间在份以内");
        assert_ne!(
            recording_key_of(&range),
            recording_of_the_run(
                "part-range-again",
                flow,
                &selection.segment_slices,
                registered,
                false,
                false
            )
            .recording_key,
            "不设区间那几趟的录入键不带片，与它不同"
        );
        drop(store);
        drop(directory);
    });
}

/// 没设块区间（照进度续跑）的那一趟：空库上续点从 0 起、一片 4 块（段内第 0 到第 3 块），录入键不随片挪，驱动行不多打区间字段，
/// 判完推进进度、分片进度字段逐字是照进度续跑的写法（登记的那一段进度 262144 个态、共 2^70 − 1 个态，片 1/2^52，余下的不在这一趟跑）；
/// 那几块的块键就是第 0 份整份照区间判到的块键（两种跑法互相复用）。
/// 判别力（`crates/mutations.tsv`）：续跑的片起点不取库里的续点，这条红。
#[test]
fn without_a_block_range_the_run_from_the_resume_point_keeps_its_slices_keys_and_line_fields() {
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        let version = JudgingUnitsFingerprint([11; DIGEST_BYTES]);
        let conclusions = ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version);
        let (directory, mut store) = an_empty_library_for("no-block-range-unchanged");
        let plan_to_find_the_resume_points = plan_crash_points_with_slices(
            flow,
            &placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains),
        );
        let slice_states = 4 * STATES_PER_BLOCK;
        let (segment_slices, _resume_points) = segment_slices_of_the_recording(
            &store,
            flow.segments.len(),
            &domains,
            &plan_to_find_the_resume_points,
            &conclusions,
            slice_states,
        )
        .expect("读库");
        assert_eq!(
            segment_slices[0],
            Some(CrashPointReplaySegmentSlice {
                start_within_the_domain: StateCount1280Bits::ZERO,
                state_count: 262_144,
            }),
            "空库：续点 0，一片 4 块 262144 个态"
        );
        let plan = plan_crash_points_with_slices(flow, &segment_slices);
        let (point_index, piece) = plan.sliced_segments()[0];
        let keys_from_the_resume_point: BTreeSet<VerdictBlockKey> = plan.points[point_index]
            .blocks()
            .into_iter()
            .filter(|local| piece.local_states.contains(&local.start))
            .map(|local| plan.points[point_index].block_key(&local))
            .collect();
        assert_eq!(
            keys_from_the_resume_point,
            keys_of_the_sliced_blocks(flow, &domains, 0..4)
        );
        let (part_zero_directory, mut part_zero_store) =
            an_empty_library_for("no-block-range-part-zero");
        let (keys_of_part_zero, _) = judge_a_part_range_with_the_fake_judge(
            &mut part_zero_store,
            flow,
            &domains,
            &part_range_of(0, 0, 4),
            &version,
        );
        drop(part_zero_store);
        drop(part_zero_directory);
        assert_eq!(
            keys_from_the_resume_point, keys_of_part_zero,
            "照进度续跑与照区间判同一批状态，块键相同"
        );
        let mut moved_slices = segment_slices.clone();
        moved_slices[0] = Some(CrashPointReplaySegmentSlice {
            start_within_the_domain: StateCount1280Bits::from_u64(slice_states),
            state_count: slice_states,
        });
        assert_eq!(
            recording_of_the_run(
                "no-block-range-unchanged",
                flow,
                &segment_slices,
                registered,
                false,
                false
            )
            .recording_key,
            recording_of_the_run(
                "no-block-range-unchanged",
                flow,
                &moved_slices,
                registered,
                false,
                false
            )
            .recording_key,
            "不设区间：录入键不随片挪"
        );
        assert_eq!(
            BlockRangeOfTheFlow::EveryBlockOfTheFlow.fields_of_the_line(4),
            "",
            "不设区间：驱动行不多打区间字段"
        );
        write_the_recorded_blocks(&mut store, &plan).expect("录入");
        store_a_clean_verdict_for_every_unjudged_block(&mut store, &plan, &version);
        advance_the_progress_of_the_sliced_segments(&mut store, &plan, &conclusions)
            .expect("推进进度");
        let domains_and_progress = domains_and_progress_of_the_registered_segments_of_the_run(
            &store,
            flow,
            &plan,
            registered,
            &conclusions,
            slice_states,
            None,
        )
        .expect("读库");
        assert_eq!(
                segment_fields_of_the_line(registered, &[], &domains_and_progress),
                " registered_segments=0:70 unregistered_segments_beyond_expansion=- registered_segment_domains=0:slice registered_segment_progress=0:262144/1180591620717411303423 registered_segment_slices=0:1/4503599627370496:rest_not_run_this_time",
                "分片进度字段逐字是照进度续跑的写法：登记的那一段进度 262144/(2^70 − 1)、片 1/2^52、余下的不在这一趟跑，不带区间字段"
            );
        drop(store);
        drop(directory);
    });
}

/// 份的拒绝：份号越界拒、拒绝句写这条流有几份、每份与末一份几块（2^54 块分片的段加 2 块不分片的块，一份 4 块：2^52 + 1 份、末一份 2 块）；
/// 止块过了这一份的块数拒、写这一份几块；末一份的 2 块是不分片的块，照收、分片的段片长为 0；一段点到分片的段的块超过一趟的片长上界拒；
/// 份数装不进双 128 位的域（2^280 − 1 个态、一份 1 块）拒成「第一版不支持」那一种，不排片。
/// 判别力（`crates/mutations.tsv`）：份号等于份数也收，这条红。
#[test]
fn a_part_range_past_the_parts_or_past_its_part_is_refused_naming_the_parts_and_the_blocks() {
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        let selection_of =
            |part_range: &PartRangeOfASlicedFlow, blocks_per_part: BlocksPerPartAsAPowerOfTwo| {
                selection_of_the_part_range_of_the_flow(flow, &domains, part_range, blocks_per_part)
            };
        let parts_of_the_flow: u64 = (1 << 52) + 1;
        let past_the_parts = selection_of(
            &part_range_of(parts_of_the_flow, 0, 1),
            BLOCKS_PER_PART_OF_THE_PART_CASES,
        )
        .expect_err("份号越界");
        assert!(matches!(
            past_the_parts,
            PartRangeRefused::PartPastTheParts { .. }
        ));
        let sentence_past_the_parts = past_the_parts.sentence("流");
        for named in [
            BLOCK_RANGE_ENVIRONMENT_VARIABLE,
            "有 4503599627370497 份",
            "每份 4 块",
            "末一份 2 块",
        ] {
            assert!(
                sentence_past_the_parts.contains(named),
                "拒绝句要写 {named}：{sentence_past_the_parts}"
            );
        }
        let past_the_part =
            selection_of(&part_range_of(0, 3, 5), BLOCKS_PER_PART_OF_THE_PART_CASES)
                .expect_err("止块过了这一份");
        let sentence_past_the_part = past_the_part.sentence("流");
        for named in ["有 4503599627370497 份", "第 0 份有 4 块"] {
            assert!(
                sentence_past_the_part.contains(named),
                "拒绝句要写 {named}：{sentence_past_the_part}"
            );
        }
        let last_part = parts_of_the_flow - 1;
        let sentence_past_the_last_part = selection_of(
            &part_range_of(last_part, 0, 3),
            BLOCKS_PER_PART_OF_THE_PART_CASES,
        )
        .expect_err("末一份只有 2 块")
        .sentence("流");
        assert!(
            sentence_past_the_last_part.contains("第 4503599627370496 份有 2 块"),
            "{sentence_past_the_last_part}"
        );
        let selection_of_the_last_part = selection_of(
            &part_range_of(last_part, 0, 2),
            BLOCKS_PER_PART_OF_THE_PART_CASES,
        )
        .expect("末一份 0-2 照收");
        assert_eq!(selection_of_the_last_part.blocks_of_the_plan, 0..2);
        assert_eq!(selection_of_the_last_part.blocks_in_the_plan, 2);
        assert_eq!(
            selection_of_the_last_part.segment_slices[0].map(|slice| slice.state_count),
            Some(0),
            "末一份是不分片的块，分片的段这一趟片长为 0"
        );
        let largest_blocks = LARGEST_REGISTERED_SEGMENT_SLICE_STATES / STATES_PER_BLOCK;
        assert!(matches!(
            selection_of(
                &part_range_of(0, 0, largest_blocks + 1),
                BlocksPerPartAsAPowerOfTwo(17)
            ),
            Err(PartRangeRefused::SliceLongerThanTheLargestSlice {
                segment_index: 0,
                ..
            })
        ));
        assert!(selection_of(
            &part_range_of(0, 0, largest_blocks),
            BlocksPerPartAsAPowerOfTwo(17)
        )
        .is_ok());
    });
    with_a_flow_registering_its_first_segment(&[(0, 280)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        assert!(matches!(
            selection_of_the_part_range_of_the_flow(
                flow,
                &domains,
                &part_range_of(0, 0, 1),
                BlocksPerPartAsAPowerOfTwo(0)
            ),
            Err(PartRangeRefused::PartNumbersPastTwoHundredFiftySixBitsUnsupported { .. })
        ));
    });
}

/// 份的写法：`<流名>:<份号>/<起块>-<止块>`，份号与块序号是不带前导 0 的十进制、装得进双 128 位（2^256 − 1 照收，2^256 拒）；
/// 少连字符、份号或块号不是数、带前导 0、起块大于止块、斜杠前空着，都拒。登了分片段的流不写份号、没登的流写了份号，各拒、点名该怎么写。
/// 判别力（`crates/mutations.tsv`）：份号照斜杠后那一截读，这条红。
#[test]
fn a_part_range_written_in_another_form_is_refused() {
    let parsed = BlockRangesOfTheFlows::parse("sliced-flow:3/1-2").expect("写法对");
    assert_eq!(
        parsed.part_range_of_the_sliced_flow("sliced-flow"),
        Ok(part_range_of(3, 1, 2))
    );
    let two_to_the_256 =
        "115792089237316195423570985008687907853269984665640564039457584007913129639936";
    let two_to_the_256_less_one =
        "115792089237316195423570985008687907853269984665640564039457584007913129639935";
    assert!(
        BlockRangesOfTheFlows::parse(&format!("sliced-flow:{two_to_the_256_less_one}/0-1")).is_ok()
    );
    for written in [
        "sliced-flow:0/1".to_string(),
        "sliced-flow:x/0-1".to_string(),
        "sliced-flow:0/01-2".to_string(),
        "sliced-flow:/0-1".to_string(),
        "sliced-flow:0/2-1".to_string(),
        format!("sliced-flow:{two_to_the_256}/0-1"),
        format!("sliced-flow:0/0-{two_to_the_256}"),
    ] {
        assert!(
            BlockRangesOfTheFlows::parse(&written).is_err(),
            "「{written}」要判错"
        );
    }
    let plain_form_for_a_sliced_flow = BlockRangesOfTheFlows::parse("sliced-flow:0-1")
        .expect("写法对")
        .part_range_of_the_sliced_flow("sliced-flow")
        .expect_err("登了分片段的流不写份号要拒");
    assert!(
        plain_form_for_a_sliced_flow.contains("<份号>/<起块>-<止块>"),
        "{plain_form_for_a_sliced_flow}"
    );
    let part_form_for_a_plain_flow = BlockRangesOfTheFlows::parse("plain-flow:0/0-1")
        .expect("写法对")
        .range_of_the_flow("plain-flow", 11)
        .expect_err("没登分片段的流写了份号要拒");
    assert!(
        part_form_for_a_plain_flow.contains("写了份号"),
        "{part_form_for_a_plain_flow}"
    );
}

/// 一项几条流：同一个区间变量里登了分片段的那条写份、没登的那条写块序号，同一趟各判各的——判到的块数就是各自的区间长度，
/// 驱动行各报各的区间；只点登了分片段的那条、另一条写空区间，没点的那条一块不判。
/// 判别力（`crates/mutations.tsv`）：选中的块在计划里照截内块号的头取尾，这条红。
#[test]
fn flows_of_one_item_each_judge_their_own_range_in_the_same_run() {
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        let blocks_of_the_plain_flow = 11u64;
        for (written, sliced_blocks, plain_blocks, plain_fields) in [
            (
                "sliced-flow:1/1-4,plain-flow:3-5",
                3u64,
                2usize,
                " block_range=3-5 blocks_of_the_flow=11",
            ),
            (
                "sliced-flow:1/0-2,plain-flow:4-4",
                2,
                0,
                " block_range=4-4 blocks_of_the_flow=11",
            ),
        ] {
            let ranges = BlockRangesOfTheFlows::parse(written).expect("写法对");
            let part_range = ranges
                .part_range_of_the_sliced_flow("sliced-flow")
                .expect("登了分片段的流写了份");
            let selection = selection_of_the_part_range_of_the_flow(
                flow,
                &domains,
                &part_range,
                BLOCKS_PER_PART_OF_THE_PART_CASES,
            )
            .expect("区间在份以内");
            let block_range_of_the_sliced_flow =
                BlockRangeOfTheFlow::BlocksInTheRange(selection.blocks_of_the_plan.clone());
            let selected_of_the_sliced_flow = blocks_of_this_share_in_the_block_range(
                selection.blocks_in_the_plan,
                &WeightedShares::new(vec![1]),
                0,
                &block_range_of_the_sliced_flow,
            );
            assert_eq!(
                u64::try_from(selected_of_the_sliced_flow.len()).expect("小"),
                sliced_blocks,
                "{written}：登了分片段的那条判的块数是它的区间长度"
            );
            assert_eq!(
                    block_range_line_of_a_part(
                        "sliced-flow",
                        HostShare { index: 0, count: 1 },
                        &selection.part,
                        selected_of_the_sliced_flow.len()
                    ),
                    format!(
                        "CRASH_AMPLIFICATION_BLOCK_RANGE flow=sliced-flow share=0/1 block_range=1/{}-{} parts_of_the_flow=4503599627370497 blocks_of_the_part=4 blocks_selected_here={sliced_blocks}",
                        part_range.blocks.start.decimal_text(),
                        part_range.blocks.end.decimal_text()
                    )
                );
            let block_range_of_the_plain_flow = ranges
                .range_of_the_flow("plain-flow", blocks_of_the_plain_flow)
                .expect("没登分片段的流写块序号");
            assert_eq!(
                blocks_of_this_share_in_the_block_range(
                    blocks_of_the_plain_flow,
                    &WeightedShares::new(vec![1]),
                    0,
                    &block_range_of_the_plain_flow,
                )
                .len(),
                plain_blocks,
                "{written}：没登分片段的那条判的块数是它的区间长度"
            );
            assert_eq!(
                block_range_of_the_plain_flow.fields_of_the_line(blocks_of_the_plain_flow),
                plain_fields
            );
        }
    });
}
