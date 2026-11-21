use super::*;

/// J10：按段登记表五段一行，照第二段流名取行、按段号排：同一合表项别的流的行不取，第一段与流名相同而第二段不同的行也不取；
/// `#` 起头的行、空行、行末第六段 `#` 说明都跳过。
#[test]
fn the_segments_registry_hands_back_the_rows_of_the_flow_sorted_by_segment() {
    let digest = "ab".repeat(DIGEST_BYTES);
    let text = format!(
        "# 注释\n\nposition-addressed-trees\tposition_addressed_Grows\t7\t26\t{digest}\t# 说明\n\
             position-addressed-trees\tposition_addressed_Inline\t3\t24\t{digest}\n\
             position_addressed_Grows\tposition_addressed_Other\t5\t9\t{digest}\n\
             position-addressed-trees\tposition_addressed_Grows\t0\t2\t{digest}\n"
    );
    let registered =
        registered_segments_of_the_flow(&text, "position_addressed_Grows").expect("写法都对");
    assert_eq!(
        registered,
        vec![
            RegisteredSegment {
                segment_index: 0,
                write_count: 2,
                write_table_digest_text: digest.clone(),
            },
            RegisteredSegment {
                segment_index: 7,
                write_count: 26,
                write_table_digest_text: digest.clone(),
            },
        ]
    );
    assert!(
        registered_segments_of_the_flow(&text, "position-addressed-trees")
            .expect("写法都对")
            .is_empty(),
        "拿合表项名来取，一行都取不到：引擎按第二段流名取"
    );
}

/// J10：去重按（流名，段号）：同一合表项的两条流各登自己的第 0 段放行，不当成重复。
#[test]
fn two_flows_of_one_item_may_each_register_their_own_segment_of_the_same_index() {
    let digest = "cd".repeat(DIGEST_BYTES);
    let text = format!(
        "position-addressed-trees\tposition_addressed_Grows\t0\t322\t{digest}\n\
             position-addressed-trees\tposition_addressed_Inline\t0\t12\t{digest}\n"
    );
    assert_eq!(
        registered_segments_of_the_flow(&text, "position_addressed_Inline"),
        Ok(vec![RegisteredSegment {
            segment_index: 0,
            write_count: 12,
            write_table_digest_text: digest.clone(),
        }]),
        "两条流同一段号各一行：不是同一条流同一段"
    );
}

/// J10、M6-10：登记了的段照这一趟的展开上限展开：上限 n 时 n 次写全展开（罩着甲二）、n + 1 次写取甲二；n 取 6 与 30 各一组。
/// 判别力（`crates/mutations.tsv`）：登记段照固定 6 判、不看这一趟的上限，这条红在 n = 30 那一组。
#[test]
fn a_registered_segment_is_expanded_up_to_the_limit_of_the_run_and_takes_the_in_place_subsets_past_it(
) {
    for limit_in_writes in [6, 30] {
        let limit = ExpandUpToWritesPerSegment(limit_in_writes);
        let writes_up_to_the_limit: Vec<usize> = (0..limit_in_writes).collect();
        let writes_past_the_limit: Vec<usize> = (0..=limit_in_writes).collect();
        assert_eq!(
            expansion_of_a_registered_segment(&writes_up_to_the_limit, limit),
            CrashPointReplaySegmentExpansion::EveryProperSubset,
            "上限 {limit_in_writes}、{limit_in_writes} 次写：全展开"
        );
        assert_eq!(
            expansion_of_a_registered_segment(&writes_past_the_limit, limit),
            CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll,
            "上限 {limit_in_writes}、多一次写：取甲二"
        );
    }
}

/// J10：写坏的一行整趟拒，不当成「没登记」：错误里写第几行、哪一处不对。
#[test]
fn a_malformed_line_of_the_segments_registry_is_refused_naming_the_line() {
    let digest = "0f".repeat(DIGEST_BYTES);
    let item_and_flow = "multi-record-publish\tmulti_record_publish";
    for (line, reason) in [
        (format!("multi_record_publish\t7\t26\t{digest}"), "一行五段"),
        (format!("{item_and_flow}\t7\t26"), "一行五段"),
        (
            format!("\tmulti_record_publish\t7\t26\t{digest}"),
            "合表项名不许空",
        ),
        (format!("{item_and_flow}\t07\t26\t{digest}"), "段号"),
        (format!("{item_and_flow}\t7\t0\t{digest}"), "段写数"),
        (
            format!("{item_and_flow}\t7\t26\t{}", digest.to_uppercase()),
            "摘要",
        ),
        (format!("{item_and_flow}\t7\t26\t{}", &digest[1..]), "摘要"),
    ] {
        let text = format!("# 注释\n{line}\n");
        let refused = registered_segments_of_the_flow(&text, "multi_record_publish");
        assert!(
            refused
                .as_ref()
                .is_err_and(|error| error.contains("第 2 行") && error.contains(reason)),
            "「{line}」：{refused:?}"
        );
    }
    let twice = format!("{item_and_flow}\t7\t26\t{digest}\n{item_and_flow}\t7\t26\t{digest}\n");
    assert!(
        registered_segments_of_the_flow(&twice, "unlink_after_the_multi_record_publish")
            .is_err_and(|error| error.contains("同一条流同一段")),
        "同一条流同一段两行，别的流来读也拒"
    );
}

/// 驱动行新字段（C790 阻断二）：段写数 [322, 290, 1, 2] 的流、展开上限 6，没登记又盖不住的段逐个报，登了的、盖得住的不报。
#[test]
fn the_line_names_the_unregistered_segments_the_expansion_up_to_does_not_cover() {
    let segments_of_lengths = |lengths: &[usize]| -> Vec<Vec<usize>> {
        let mut next_write = 0usize;
        lengths
            .iter()
            .map(|length| {
                let segment: Vec<usize> = (next_write..next_write + length).collect();
                next_write += length;
                segment
            })
            .collect()
    };
    let registered = |segment_index: usize, write_count: usize| RegisteredSegment {
        segment_index,
        write_count,
        write_table_digest_text: "ee".repeat(DIGEST_BYTES),
    };
    let fields_of = |lengths: &[usize], registered_segments: &[RegisteredSegment]| {
        let segments = segments_of_lengths(lengths);
        segment_fields_of_the_line(
            registered_segments,
            &unregistered_segments_beyond_the_expansion(
                &segments,
                registered_segments,
                ExpandUpToWritesPerSegment(6),
            ),
            &[],
        )
    };
    let shape = [322, 290, 1, 2];
    assert_eq!(
            fields_of(&shape, &[registered(0, 322)]),
            " registered_segments=0:322 unregistered_segments_beyond_expansion=1:290 registered_segment_domains=- registered_segment_progress=- registered_segment_slices=-",
            "只登第 0 段：第 1 段 290 写没登、展开 6 盖不住"
        );
    assert_eq!(
            fields_of(&shape, &[registered(0, 322), registered(1, 290)]),
            " registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=- registered_segment_progress=- registered_segment_slices=-",
            "两段长的都登了"
        );
    assert_eq!(
            fields_of(&shape, &[]),
            " registered_segments=- unregistered_segments_beyond_expansion=0:322,1:290 registered_segment_domains=- registered_segment_progress=- registered_segment_slices=-",
            "一段都没登：两段长的按段号升序都报"
        );
    assert_eq!(
            fields_of(&[6, 1, 2], &[]),
            " registered_segments=- unregistered_segments_beyond_expansion=- registered_segment_domains=- registered_segment_progress=- registered_segment_slices=-",
            "每段都不超过 6 次写：展开上限盖住全部段"
        );
}

/// 溢出（C790）：322 写的长段登记之后，录入段排计划之前算态数的那一步不 panic，登记段越过展开上限 6、态数只按甲二，全量组合数（2^322）不算；
/// 没登记的长段照展开上限不展开。
#[test]
fn a_registered_long_segment_past_the_limit_of_the_run_is_counted_by_the_in_place_subsets_only() {
    let (writes, segments) = flow_parts_of_unit_and_journal_writes(&[(320, 2), (290, 0)]);
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![322, 290, 1, 2]
    );
    let base = MemoryPool {
        devices: BTreeMap::new(),
        device_size_in_bytes: 1 << 30,
    };
    let limit = ExpandUpToWritesPerSegment(6);
    let expansion_of_the_caller =
        move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment);
    let caller_flow = CrashFlow {
        base: &base,
        writes: &writes,
        segments: &segments,
        judged_root_index: 612,
        versions: &[],
        versions_without_a_file: &[],
        expansion: &expansion_of_the_caller,
        crash_points: vec![CrashPointSpan {
            name: "probe".to_string(),
            code_file_in_repository: "crates/singlefs-verification/src/crash_amplification.rs"
                .to_string(),
            writes: 0..writes.len(),
            ignored: false,
        }],
    };
    let registered = vec![RegisteredSegment {
        segment_index: 0,
        write_count: 322,
        write_table_digest_text: "aa".repeat(DIGEST_BYTES),
    }];
    let expansion_in_the_recording_of_the_flow = |segment_index: usize, segment: &[usize]| {
        expansion_in_the_recording(&registered, &caller_flow, limit, segment_index, segment)
    };
    let flow_to_record = CrashFlow {
        base: caller_flow.base,
        writes: caller_flow.writes,
        segments: caller_flow.segments,
        judged_root_index: caller_flow.judged_root_index,
        versions: caller_flow.versions,
        versions_without_a_file: caller_flow.versions_without_a_file,
        expansion: &expansion_in_the_recording_of_the_flow,
        crash_points: caller_flow.crash_points.clone(),
    };
    assert_eq!(
            states_of_the_flow_to_record(&flow_to_record, &no_segment_slices(4)),
            Ok(12),
            "第 0 段登记、322 写里 2 次 journal 记录写（原地写、一个扇区两态）：甲二 = 2^2 × 2（单元写全不落或全落）− 1 = 7；\
             第 1 段 290 写没登、展开 6 盖不住，0 个；第 2 段 1 写全展开 2^1 − 1 = 1；第 3 段 2 写全展开 2^2 − 1 = 3；\
             再加全部持久那一个：7 + 0 + 1 + 3 + 1 = 12"
        );
}

/// 2^70 − 1：70 次两态原地写的一段按甲二（没有单元写）的整域。
const DOMAIN_OF_SEVENTY_IN_PLACE_WRITES: &str = "1180591620717411303423";

/// C790：登记的段整域装得进 u64（322 写里 2 次 journal 记录写，甲二 7 个态）是 `full`、不分片：计划里没有分片的段，
/// 驱动行报 `registered_segment_domains=0:full registered_segment_progress=-`。
#[test]
fn a_registered_segment_whose_domain_fits_in_u64_is_full_and_is_not_sliced() {
    with_a_flow_registering_its_first_segment(&[(320, 2)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        assert_eq!(
            domains,
            vec![DomainOfARegisteredSegment {
                segment_index: 0,
                domain_state_count: StateCount1280Bits::from_u64(7),
                kind: DomainKindOfARegisteredSegment::Full,
            }],
            "2^2 × 2 − 1 = 7 装得进 u64"
        );
        assert_eq!(
            placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains),
            no_segment_slices(flow.segments.len())
        );
        let plan = plan_crash_points_with_slices(flow, &no_segment_slices(flow.segments.len()));
        assert!(plan.sliced_segments().is_empty());
        let (directory, store) = an_empty_library_for("full");
        let version = JudgingUnitsFingerprint([5; DIGEST_BYTES]);
        let domains_and_progress = domains_and_progress_of_the_registered_segments(
            &store,
            flow,
            &plan,
            registered,
            &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version),
            DEFAULT_REGISTERED_SEGMENT_SLICE_STATES,
        )
        .expect("读库");
        assert!(
                segment_fields_of_the_line(registered, &[], &domains_and_progress)
                    .ends_with(" registered_segment_domains=0:full registered_segment_progress=- registered_segment_slices=-"),
                "{}",
                segment_fields_of_the_line(registered, &[], &domains_and_progress)
            );
        drop(store);
        drop(directory);
    });
}

/// 第 12 条（崩溃点重放 position-addressed-trees 双机那一趟：本机先录先判、续点挪到下一片，只判那几趟另定的片换了录入键、找不到清单）：
/// 分片的段这一趟判哪一片不进录入键——两片录下的流字节不同（判段照它判哪一片），录入键相同；判表开不开照旧进录入键。
/// 判别力（`crates/mutations.tsv`）：录入键照带片的那一份流算，这条红。
#[test]
fn the_recording_key_stays_the_same_when_the_slice_of_a_registered_segment_moves() {
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        assert_eq!(domains[0].kind, DomainKindOfARegisteredSegment::Slice);
        let recording_of_the_slice = |start: u64, gpu_judges: bool| {
            let mut slices = no_segment_slices(flow.segments.len());
            slices[0] = Some(CrashPointReplaySegmentSlice {
                start_within_the_domain: StateCount1280Bits::from_u64(start),
                state_count: 300,
            });
            recording_of_the_run(
                "slice-of-a-registered-segment",
                flow,
                &slices,
                registered,
                false,
                gpu_judges,
            )
        };
        let first_slice = recording_of_the_slice(0, false);
        let next_slice = recording_of_the_slice(300, false);
        assert_ne!(
            first_slice.recorded_flow_bytes, next_slice.recorded_flow_bytes,
            "两片录下的流字节不同：判段照读回的那一份判哪一片"
        );
        assert_eq!(
            first_slice.recording_key, next_slice.recording_key,
            "片挪了，录入键不变：只判那几趟照同一把录入键找得到清单"
        );
        assert_ne!(
            first_slice.recording_key,
            recording_of_the_slice(0, true).recording_key,
            "判表开不开照旧进录入键"
        );
    });
}

/// 第 11 条（D13 已定项 16 改写，D47）：分片的段的续点只由内容寻址（段的内容摘要连同判器那一把判法单元指纹），不带流名、点名、设点文件、
/// 段号。两条流只差崩溃点的名字与设点文件：点的身份不同，分片进度的键逐字节相同；判器那一把换了，键不同。
/// 判别力（`crates/mutations.tsv`）：段的内容摘要里拼进点名，这条红。
#[test]
fn the_progress_key_of_a_sliced_segment_is_addressed_by_its_content_and_not_by_the_names_of_the_flow(
) {
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        assert_eq!(domains[0].kind, DomainKindOfARegisteredSegment::Slice);
        let slices = placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains);
        let renamed_flow = CrashFlow {
            base: flow.base,
            writes: flow.writes,
            segments: flow.segments,
            judged_root_index: flow.judged_root_index,
            versions: flow.versions,
            versions_without_a_file: flow.versions_without_a_file,
            expansion: flow.expansion,
            crash_points: vec![CrashPointSpan {
                name: "renamed_probe".to_string(),
                code_file_in_repository:
                    "crates/singlefs-verification/tests/another_setting_file.rs".to_string(),
                writes: flow.crash_points[0].writes.clone(),
                ignored: false,
            }],
        };
        let plan = plan_crash_points_with_slices(flow, &slices);
        let renamed_plan = plan_crash_points_with_slices(&renamed_flow, &slices);
        assert_ne!(
            plan.points[0].identity, renamed_plan.points[0].identity,
            "前提：两条流的点名与设点文件不同，点的身份不同"
        );
        let version = JudgingUnitsFingerprint([5; DIGEST_BYTES]);
        let conclusions = ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version);
        assert_eq!(
            progress_key_of_the_sliced_segment(&plan, 0, &conclusions),
            progress_key_of_the_sliced_segment(&renamed_plan, 0, &conclusions),
            "续点只由段的内容与判器寻址：名字不同，分片进度的键相同"
        );
        let another_version = JudgingUnitsFingerprint([6; DIGEST_BYTES]);
        assert_ne!(
            progress_key_of_the_sliced_segment(&plan, 0, &conclusions),
            progress_key_of_the_sliced_segment(
                &plan,
                0,
                &ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&another_version)
            ),
            "判法单元指纹换了，续点的键不同"
        );
    });
}

/// 跑不完的登记段如实记账（D13（验证路线） 已定项 9「枚举不完靠精确剪枝，一时跑不了的节点标 ignore 慢慢开」）：整域 2^100 态、一片 2^32 个态，第一趟判完一片之后结果行报
/// 累计判完 1 片、共几片写下界（2^68 片装不进 u64）、余下本次未跑——这个节点不算判过；装得进 u64 的整域（3 片多 5 个态）照实数，
/// 末片短也算一片，没判完的片不算判完，判到整域末尾才是判过。
/// 判别力（`crates/mutations.tsv`）：判完一片就报判过（余下那一半照「判到了整域末尾」报），这条红。
#[test]
fn a_sliced_segment_reports_the_slices_judged_so_far_and_the_rest_as_not_run_this_time() {
    let states_per_slice = 1u64 << 32;
    let domain_of_two_to_the_hundred = StateCount1280Bits::from_u64(1 << 50)
        .checked_mul_u64(1 << 50)
        .expect("2^100 装得进 1280 位");
    let after_one_slice = slice_account_of_a_sliced_segment(
        &StateCount1280Bits::from_u64(states_per_slice),
        &domain_of_two_to_the_hundred,
        states_per_slice,
    );
    assert_eq!(
        after_one_slice,
        SliceAccountOfASlicedSegment {
            judged_slices: SliceCountOfASlicedSegment::Exactly(1),
            slices: SliceCountOfASlicedSegment::AtLeastTwoToTheSixtyFour,
            rest: RestOfASlicedSegment::NotRunThisTime,
        },
        "判完一片：累计 1 片、共几片写下界、余下本次未跑"
    );
    assert_eq!(
        after_one_slice.field_text(1),
        "1:1/at_least_18446744073709551616:rest_not_run_this_time"
    );
    let domain_of_three_slices_and_five_states =
        StateCount1280Bits::from_u64(3 * states_per_slice + 5);
    let field_after = |judged_through: u64| {
        slice_account_of_a_sliced_segment(
            &StateCount1280Bits::from_u64(judged_through),
            &domain_of_three_slices_and_five_states,
            states_per_slice,
        )
        .field_text(0)
    };
    assert_eq!(field_after(0), "0:0/4:rest_not_run_this_time", "一片都没判");
    assert_eq!(
        field_after(2 * states_per_slice + 7),
        "0:2/4:rest_not_run_this_time",
        "判到第三片中间：整片判完的只有两片"
    );
    assert_eq!(
        field_after(3 * states_per_slice + 5),
        "0:4/4:every_state_judged",
        "判到整域末尾：末片短也算一片，判过"
    );
    let line = segment_fields_of_the_line(
        &[RegisteredSegment {
            segment_index: 1,
            write_count: 290,
            write_table_digest_text: "ee".repeat(DIGEST_BYTES),
        }],
        &[],
        &[DomainAndProgressOfARegisteredSegment {
            segment_index: 1,
            kind: DomainKindOfARegisteredSegment::Slice,
            judged_through: StateCount1280Bits::from_u64(states_per_slice),
            domain_state_count: domain_of_two_to_the_hundred,
            states_per_slice,
        }],
    );
    assert!(
            line.ends_with(&format!(
                " registered_segment_domains=1:slice registered_segment_progress=1:{states_per_slice}/{} registered_segment_slices=1:1/at_least_18446744073709551616:rest_not_run_this_time",
                domain_of_two_to_the_hundred.decimal_text()
            )),
            "结果行如实报这一段：{line}"
        );
}

/// C790：登记的段是 70 次两态原地写（2^70 − 1 个态，装不进 u64）是 `slice`。片长 300：第一趟从库里找续点是 0（逐块问库，一块都没判），
/// 判 [0, 300)、库里有这一块、进度记录是 300、驱动行进度 `0:300/<2^70 − 1>`；同一个库第二趟续点照进度记录核过是 300，判 [300, 600)：
/// 第一趟那一块照旧判过、这一趟没判的只有新片那一块，进度记录与驱动行是 600。总态数逐字是 2^70 − 1 的十进制。
#[test]
fn a_registered_segment_past_u64_is_judged_one_slice_per_run_from_the_resume_point_in_the_library()
{
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        assert_eq!(domains[0].kind, DomainKindOfARegisteredSegment::Slice);
        assert_eq!(
            domains[0].domain_state_count.decimal_text(),
            DOMAIN_OF_SEVENTY_IN_PLACE_WRITES
        );
        let plan_to_find_the_resume_points = plan_crash_points_with_slices(
            flow,
            &placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains),
        );
        let (directory, mut store) = an_empty_library_for("two-runs");
        let version = JudgingUnitsFingerprint([5; DIGEST_BYTES]);
        let conclusions = ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version);
        let progress_key =
            progress_key_of_the_sliced_segment(&plan_to_find_the_resume_points, 0, &conclusions);
        let mut previous_plan: Option<CrashPlan> = None;
        for (run, expected_start) in [(1u64, 0u64), (2, 300)] {
            let (segment_slices, resume_points) = segment_slices_of_the_recording(
                &store,
                flow.segments.len(),
                &domains,
                &plan_to_find_the_resume_points,
                &conclusions,
                300,
            )
            .expect("读库");
            assert_eq!(
                segment_slices[0],
                Some(CrashPointReplaySegmentSlice {
                    start_within_the_domain: StateCount1280Bits::from_u64(expected_start),
                    state_count: 300,
                }),
                "第 {run} 趟从段内编号 {expected_start} 起判 300 个"
            );
            assert_eq!(
                resume_points[0].1.found_by,
                if run == 1 {
                    ResumePointFoundBy::AWalkFromZeroThroughTheLibrary
                } else {
                    ResumePointFoundBy::TheProgressRecordChecked
                }
            );
            let plan = plan_crash_points_with_slices(flow, &segment_slices);
            write_the_recorded_blocks(&mut store, &plan).expect("录入");
            let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &conclusions).expect("对账");
            if let Some(previous_plan) = &previous_plan {
                assert_eq!(
                    unjudged.len(),
                    1,
                    "第二趟只有新片那一块没判，别的点照旧复用：{unjudged:?}"
                );
                assert!(
                    unjudged_blocks_of_the_flow(&store, previous_plan, &conclusions)
                        .expect("对账")
                        .is_empty(),
                    "第一趟的块照旧判过，不重判"
                );
            }
            store_a_clean_verdict_for_every_unjudged_block(&mut store, &plan, &version);
            advance_the_progress_of_the_sliced_segments(&mut store, &plan, &conclusions)
                .expect("推进进度");
            let judged_through = expected_start + 300;
            assert_eq!(
                store
                    .read_sliced_segment_progress(&progress_key)
                    .expect("读进度"),
                Some(SlicedSegmentProgress {
                    judged_through: StateCount1280Bits::from_u64(judged_through),
                    last_block_start: StateCount1280Bits::from_u64(expected_start),
                })
            );
            let domains_and_progress = domains_and_progress_of_the_registered_segments(
                &store,
                flow,
                &plan,
                registered,
                &conclusions,
                300,
            )
            .expect("读库");
            assert!(
                    segment_fields_of_the_line(registered, &[], &domains_and_progress).contains(
                        &format!(
                            " registered_segment_domains=0:slice registered_segment_progress=0:{judged_through}/{DOMAIN_OF_SEVENTY_IN_PLACE_WRITES} "
                        )
                    ),
                    "第 {run} 趟：{}",
                    segment_fields_of_the_line(registered, &[], &domains_and_progress)
                );
            previous_plan = Some(plan);
        }
        drop(store);
        drop(directory);
    });
}

/// C790 被打断与进度记录坏了：一片四块（片长 4 × 2^16），第一趟只判了前两块就停、进度记录没写；重跑从库里逐块问到第三块的起点接着，
/// 不从头、不漏块。进度记录落后（指着已判的块）、指到没判过的编号，都核不过、退回逐块问库，同样从第三块接着；记得对的照它接着。
#[test]
fn an_interrupted_slice_resumes_at_its_first_unjudged_block_and_a_wrong_progress_record_falls_back_to_a_walk(
) {
    with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
        let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
        let plan_to_find_the_resume_points = plan_crash_points_with_slices(
            flow,
            &placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains),
        );
        let (directory, mut store) = an_empty_library_for("interrupted");
        let version = JudgingUnitsFingerprint([6; DIGEST_BYTES]);
        let conclusions = ConclusionsStoredUnder::TheJudgingUnitsFingerprint(&version);
        let slice_states = 4 * STATES_PER_BLOCK;
        let resume = |the_library: &VerdictStore| {
            let (slices_of_this_run, resume_points) = segment_slices_of_the_recording(
                the_library,
                flow.segments.len(),
                &domains,
                &plan_to_find_the_resume_points,
                &conclusions,
                slice_states,
            )
            .expect("读库");
            (slices_of_this_run, resume_points[0].1)
        };
        let (segment_slices, _resume_point) = resume(&store);
        let plan = plan_crash_points_with_slices(flow, &segment_slices);
        write_the_recorded_blocks(&mut store, &plan).expect("录入");
        // 被打断：分片的段四块只判了前两块，进度记录没写
        let clean = store
            .register_verdict_vector(&crate::verdict_store::EncodedVerdictVector(
                b"clean".to_vec(),
            ))
            .expect("登记判定向量");
        let (point_index, piece) = plan.sliced_segments()[0];
        let point = &plan.points[point_index];
        let slice_blocks: Vec<Range<u64>> = point
            .blocks()
            .into_iter()
            .filter(|local| piece.local_states.contains(&local.start))
            .collect();
        assert_eq!(slice_blocks.len(), 4, "一片四块");
        for local in &slice_blocks[..2] {
            let states = usize::try_from(local.end - local.start).expect("一块装得进 usize");
            store
                .store_verdict_block(
                    &point.block_key(local),
                    &version,
                    &VerdictBlock::new(vec![clean; states]).expect("一块至少一个状态"),
                    &[],
                )
                .expect("存块");
        }
        let first_unjudged = StateCount1280Bits::from_u64(2 * STATES_PER_BLOCK);
        let expect_a_walk_to_the_third_block = |the_library: &VerdictStore, case: &str| {
            let (slices_from_the_resume_point, resume_point_found) = resume(the_library);
            assert_eq!(
                resume_point_found,
                ResumePointOfASlicedSegment {
                    judged_through: first_unjudged,
                    found_by: ResumePointFoundBy::AWalkFromZeroThroughTheLibrary,
                },
                "{case}：逐块问库走到第三块的起点"
            );
            assert_eq!(
                slices_from_the_resume_point[0].map(|slice| slice.start_within_the_domain),
                Some(first_unjudged),
                "{case}：从第三块接着判"
            );
        };
        expect_a_walk_to_the_third_block(&store, "没有进度记录");
        let progress_key =
            progress_key_of_the_sliced_segment(&plan_to_find_the_resume_points, 0, &conclusions);
        for (case, judged_through, last_block_start) in [
            (
                "进度记录落后一块（指着的下一块已判过）",
                STATES_PER_BLOCK,
                0,
            ),
            (
                "进度记录指到没判过的编号",
                5 * STATES_PER_BLOCK,
                4 * STATES_PER_BLOCK,
            ),
            ("进度记录说的最后一块长度对不上", 2 * STATES_PER_BLOCK, 0),
        ] {
            store
                .store_sliced_segment_progress(
                    &progress_key,
                    SlicedSegmentProgress {
                        judged_through: StateCount1280Bits::from_u64(judged_through),
                        last_block_start: StateCount1280Bits::from_u64(last_block_start),
                    },
                )
                .expect("写进度");
            expect_a_walk_to_the_third_block(&store, case);
        }
        store
            .store_sliced_segment_progress(
                &progress_key,
                SlicedSegmentProgress {
                    judged_through: first_unjudged,
                    last_block_start: StateCount1280Bits::from_u64(STATES_PER_BLOCK),
                },
            )
            .expect("写进度");
        assert_eq!(
            resume(&store).1,
            ResumePointOfASlicedSegment {
                judged_through: first_unjudged,
                found_by: ResumePointFoundBy::TheProgressRecordChecked,
            },
            "记得对的进度记录核过就照它接着"
        );
        // 重跑同一条命令：从第三块起录、判，判完推进到片尾
        let (segment_slices_of_the_rerun, _resume_point_of_the_rerun) = resume(&store);
        let plan_of_the_rerun = plan_crash_points_with_slices(flow, &segment_slices_of_the_rerun);
        write_the_recorded_blocks(&mut store, &plan_of_the_rerun)
            .expect("录入（第三、四块上一趟录过、块长相同）");
        store_a_clean_verdict_for_every_unjudged_block(&mut store, &plan_of_the_rerun, &version);
        advance_the_progress_of_the_sliced_segments(&mut store, &plan_of_the_rerun, &conclusions)
            .expect("推进进度");
        assert_eq!(
            store
                .read_sliced_segment_progress(&progress_key)
                .expect("读进度")
                .map(|progress| progress.judged_through),
            Some(StateCount1280Bits::from_u64(6 * STATES_PER_BLOCK)),
            "第二趟从第三块起判一片四块，判到第六块末尾"
        );
        drop(store);
        drop(directory);
    });
}

/// C790：登记的段整域越过 1280 位（1281 次两态原地写，2^1281 − 1 个态）在写库之前拒跑，拒跑句点名 1280 位与该位宽的出处。
#[test]
fn a_registered_segment_past_1280_bits_is_refused_before_recording() {
    with_a_flow_registering_its_first_segment(&[(0, 1281)], |flow, registered| {
        let refused = domains_of_the_registered_segments(flow, registered);
        assert!(
            refused
                .as_ref()
                .is_err_and(|error| error.contains("第 0 段（1281 次写）")
                    && error.contains("越过 1280 位")
                    && error.contains("用户 2026-10-13 定的位宽")
                    && error.contains("没写库")),
            "{refused:?}"
        );
    });
}

/// C790：第 0 段登记、70 次两态原地写（2^70 − 1 个态）要分片；第 1 段没登记、65 次两态原地写，展开上限 65 盖得住、全展开
/// （2^65 − 1 个态，装不进 u64）。录入段定片那一步拒跑、拒跑句点名第 1 段，不在排找续点那份计划（分片的段片长为 0）时 panic；
/// 库里没写：没有录入的块、没有这条流的清单。
#[test]
fn an_unregistered_segment_past_u64_beside_a_sliced_one_is_refused_before_recording() {
    with_a_flow_registering_its_first_segment_and_expanding_the_others_up_to(
        &[(0, 70), (0, 65)],
        ExpandUpToWritesPerSegment(65),
        |flow, registered| {
            let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
            assert_eq!(
                domains[0].kind,
                DomainKindOfARegisteredSegment::Slice,
                "第 0 段整域装不进 u64，要分片"
            );
            let (directory, store) = an_empty_library_for("unregistered-past-u64-beside-a-slice");
            let refused = segment_slices_of_the_recording_from_the_environment_and_the_library(
                &store,
                flow,
                "流",
                registered,
                ComputedOn::Cpu,
            );
            assert!(
                refused
                    .as_ref()
                    .is_err_and(|error| error.contains("第 1 段（65 次写）")
                        && error.contains("装不进 u64")
                        && error.contains("没写库")),
                "{refused:?}"
            );
            assert!(
                store.recorded_blocks().expect("读库").is_empty(),
                "拒跑之后库里没有录入的块"
            );
            assert_eq!(
                store.recorded_run_manifest_count().expect("读库"),
                0,
                "拒跑之后库里没有这条流的清单"
            );
            drop(store);
            drop(directory);
        },
    );
}

/// 片长变量：没设是 2^32；一块（65536 个态）整数倍的十进制正整数照用；0、负数、带别的字符、超过 2^32、不是一块整数倍（300、100000、65537）
/// 都拒，错误带变量名与原值（片长不是一块整数倍时片末那一块短，下一趟同一块键录成另一个块长、录入表拒，每趟卡在同一处：核查报告 8.4）。
/// 判别力（`crates/mutations.tsv`）：「片长不核是不是一块的整数倍」那一行让这条红。
#[test]
fn the_slice_states_variable_is_a_positive_decimal_up_to_two_to_the_thirty_second() {
    assert_eq!(
        registered_segment_slice_states_of_the_variable_value(None),
        Ok(1 << 32)
    );
    assert_eq!(
        registered_segment_slice_states_of_the_variable_value(Some("65536")),
        Ok(STATES_PER_BLOCK)
    );
    assert_eq!(
        registered_segment_slice_states_of_the_variable_value(Some("196608")),
        Ok(3 * STATES_PER_BLOCK)
    );
    assert_eq!(
        registered_segment_slice_states_of_the_variable_value(Some("4294967296")),
        Ok(1 << 32)
    );
    for refused in [
        "0",
        "-3",
        "3e2",
        " 65536",
        "",
        "4294967297",
        "300",
        "100000",
        "65537",
    ] {
        assert!(
            registered_segment_slice_states_of_the_variable_value(Some(refused))
                .is_err_and(|error| error
                    .contains(REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE)
                    && error.contains(&format!("={refused}："))),
            "{refused:?} 该拒"
        );
    }
}

/// J10、M4 第 15 条：登记的段写数与段写表摘要都对得上才放行；写表里那一段改了一个字节、段写数不同、段号越过这条流，都拒跑。
#[test]
fn a_registered_segment_whose_write_table_changed_is_refused() {
    let unit_write = |offset: u64, byte: u8| RetainedWrite {
        device: singlefs_core::address::DeviceIdentity(1),
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: singlefs_core::address::DeviceOffsetInBytes(offset),
        contents: WrittenContents::Bytes(vec![byte; 512]),
    };
    let base = MemoryPool {
        devices: BTreeMap::new(),
        device_size_in_bytes: 1 << 20,
    };
    let segments = vec![vec![0], vec![1, 2]];
    let expansion =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let flow_over = |writes: &[RetainedWrite]| -> String {
        let flow = CrashFlow {
            base: &base,
            writes,
            segments: &segments,
            judged_root_index: 0,
            versions: &[],
            versions_without_a_file: &[],
            expansion: &expansion,
            crash_points: Vec::new(),
        };
        segment_write_table_digest_text(&flow, 1).expect("第 1 段连续")
    };
    let writes = vec![unit_write(0, 1), unit_write(4096, 2), unit_write(8192, 3)];
    let digest = flow_over(&writes);
    let check = |writes_of_the_flow: &[RetainedWrite], registered: RegisteredSegment| {
        let flow = CrashFlow {
            base: &base,
            writes: writes_of_the_flow,
            segments: &segments,
            judged_root_index: 0,
            versions: &[],
            versions_without_a_file: &[],
            expansion: &expansion,
            crash_points: Vec::new(),
        };
        check_the_registered_segments_against_the_flow(&flow, "probe", &[registered])
    };
    let registered = |segment_index: usize, write_count: usize| RegisteredSegment {
        segment_index,
        write_count,
        write_table_digest_text: digest.clone(),
    };
    assert_eq!(check(&writes, registered(1, 2)), Ok(()), "对得上就放行");
    let mut changed = writes.clone();
    changed[2] = unit_write(8192, 4);
    assert!(
        check(&changed, registered(1, 2)).is_err_and(|error| error.contains("写表摘要")),
        "那一段改了一个写的内容"
    );
    assert!(
        check(&writes, registered(1, 3)).is_err_and(|error| error.contains("3 次写")),
        "段写数不同"
    );
    assert!(
        check(&writes, registered(5, 2)).is_err_and(|error| error.contains("只有 2 段")),
        "段号越过这条流"
    );
}
