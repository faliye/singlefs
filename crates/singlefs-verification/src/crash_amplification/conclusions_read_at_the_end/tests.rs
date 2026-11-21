use super::*;

/// 场景：那条两个扇区的小流录进库、读回，每一块存一份红的判器结论（头一个状态违例），核对表里每一块各一行（头一块全核红、别的块全核绿）。
/// 三样各选各的块：比对比全部的块，计数向量只数头一块，读违例只读头一块之后的块。
/// 预期：判段收尾那一遍读（[`read_the_judged_conclusions_once_at_the_end`]）交回的三样与分开读的三个（[`compare_the_conclusions_and_the_verifier_of_the_blocks`]、
/// [`count_vector_disagreements_of_the_flow`]、[`violations_counted_by_the_block_range`]）逐项相同；违例正是头一块之后每块一条，
/// 计数向量对不上的几句都点名头一块，比对比了全部块的状态。
/// 判别力（`crates/mutations.tsv`）：那一遍读照计数向量的选法读违例（读到头一块、读不到后面的块），这条红。
#[test]
fn the_judged_conclusions_read_once_at_the_end_are_the_three_reads_apart() {
    let (_directory, mut store) = an_empty_library_for("judged-conclusions-read-once");
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let read_back = recorded.with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut store, flow).expect("流写得进库、读得回")
    });
    let flow = read_back.flow();
    let plan = plan_crash_points(&flow);
    let total_blocks: u64 = plan
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数装得进 u64"))
        .sum();
    assert!(total_blocks >= 2, "前提：至少两块，三样的选法才分得开");
    let versions = vec![
        ConclusionVersion(sha256_digest(
            b"version of the read once at the end unit test\n"
        ));
        plan.points.len()
    ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let verifier_digest = [7; DIGEST_BYTES];
    let mut block_sequence = 0u64;
    let mut states_of_every_block = 0u64;
    let mut place_of_the_first_block = String::new();
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
            let states = usize::try_from(local.end - local.start).expect("一块装得进 usize");
            let bit = u32::from(block_sequence == 0);
            store
                .store_verifier_block(
                    &point.block_key(&local),
                    &verifier_digest,
                    &[8; DIGEST_BYTES],
                    &vec![bit; states],
                )
                .expect("写得进库");
            if block_sequence == 0 {
                place_of_the_first_block = format!("点 {} 块 {}", point.name, local.start);
            }
            states_of_every_block += local.end - local.start;
            block_sequence += 1;
        }
    }
    let is_compared = |_block: u64| true;
    let is_mine = |block: u64| block == 0;
    let block_range = BlockRangeOfTheFlow::BlocksInTheRange(1..total_blocks);
    let read_once = read_the_judged_conclusions_once_at_the_end(
        &store,
        &flow,
        &plan,
        &conclusions,
        Some(ComparisonWithTheVerifier {
            verifier_digest: &verifier_digest,
            is_compared: &is_compared,
        }),
        &is_mine,
        &block_range,
    )
    .expect("读得了库");
    let read_apart = JudgedConclusionsReadOnceAtTheEnd {
        comparison: Some(
            compare_the_conclusions_and_the_verifier_of_the_blocks(
                &store,
                &plan,
                &verifier_digest,
                &conclusions,
                &is_compared,
            )
            .expect("读得了库"),
        ),
        count_vector_disagreements: count_vector_disagreements_of_the_flow(
            &store,
            &flow,
            &plan,
            &conclusions,
            &is_mine,
        )
        .expect("读得了库"),
        violations: violations_counted_by_the_block_range(
            &store,
            &plan,
            &conclusions,
            &block_range,
        )
        .expect("读得了库"),
    };
    assert_eq!(read_once, read_apart, "一遍读的三样与分开读的逐项相同");
    assert_eq!(
        u64::try_from(read_once.violations.len()).expect("条数装得进 u64"),
        total_blocks - 1,
        "违例正是头一块之后每块一条"
    );
    assert!(
        !read_once.count_vector_disagreements.is_empty()
            && read_once
                .count_vector_disagreements
                .iter()
                .all(|sentence| sentence.starts_with(&place_of_the_first_block)),
        "计数向量只数头一块：{:?}",
        read_once.count_vector_disagreements
    );
    assert_eq!(
        read_once
            .comparison
            .as_ref()
            .map(|comparison| comparison.states_compared),
        Some(states_of_every_block),
        "比对比了全部块的状态"
    );
}
