use super::*;

/// 场景：那条两个扇区的小流，每一块的镜像判定存在三把版本下（存的次序与字节序不同）、观察者结论存在另一把下；另有一块不在计划里、
/// 块键头两个摘要与计划里头一块相同（块起点不同），它的镜像判定存在第五把下。
/// 预期：整份计划一起扫回的每块的版本（[`VerdictStore::conclusion_versions_stored_for_blocks`]）与逐块扫 [`VerdictStore::conclusion_versions_stored_for`]
/// 的逐块相同；镜像判定每块正是那三把、按字节序，观察者那一种每块正是那一把；不在计划里的那一块的版本不出现。
/// 判别力（`crates/mutations.tsv`）：一起扫时不看种类（观察者那一把也算进镜像判定），这条红。
#[test]
fn the_conclusion_versions_of_the_plan_read_together_are_those_each_block_reads_alone() {
    let (plan, _block_count) = plan_of_the_two_sector_flow_with_its_block_count();
    let (_directory, mut store) = an_empty_library_for("conclusion-versions-read-together");
    let image_judgement_versions_in_the_order_stored = [
        ConclusionVersion([3; DIGEST_BYTES]),
        ConclusionVersion([1; DIGEST_BYTES]),
        ConclusionVersion([2; DIGEST_BYTES]),
    ];
    for version in image_judgement_versions_in_the_order_stored {
        store_a_judge_conclusion_for_every_block(
            &mut store,
            &plan,
            &ConclusionsStoredUnder::TheConclusionKind {
                kind: ConclusionKind::ImageJudgement,
                version_of_each_point: &vec![version; plan.points.len()],
            },
            &[],
            None,
        );
    }
    let observer = ObserverForTheReuseSentinels {
        holds_each_point: vec![true; plan.points.len()],
        conclusion: ConclusionVersion([4; DIGEST_BYTES]),
        input_row: (
            ConclusionKind::Observer,
            ConclusionVersion([6; DIGEST_BYTES]),
        ),
    };
    let mut block_keys: Vec<VerdictBlockKey> = Vec::new();
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            store_an_observer_conclusion_of_the_block(
                &mut store,
                &plan,
                &observer,
                point_index,
                &local,
                None,
            );
            block_keys.push(point.block_key(&local));
        }
    }
    let first_key = block_keys.first().expect("前提：计划里至少一块").clone();
    let outside_the_plan = VerdictBlockKey {
        node_input_digest: first_key.node_input_digest,
        enumeration_plan_hash: first_key.enumeration_plan_hash,
        block_start: BlockStartStateIndex(1 << 40),
    };
    assert!(
        !block_keys.contains(&outside_the_plan),
        "前提：那一块不在计划里"
    );
    store_a_judge_conclusion_without_the_slots(
        &mut store,
        &outside_the_plan,
        &ConclusionVersion([5; DIGEST_BYTES]),
        &(0..1),
    );
    for (kind, expected_versions_of_each_block) in [
        (
            ConclusionKind::ImageJudgement,
            vec![
                ConclusionVersion([1; DIGEST_BYTES]),
                ConclusionVersion([2; DIGEST_BYTES]),
                ConclusionVersion([3; DIGEST_BYTES]),
            ],
        ),
        (
            ConclusionKind::Observer,
            vec![ConclusionVersion([4; DIGEST_BYTES])],
        ),
    ] {
        let read_together = store
            .conclusion_versions_stored_for_blocks(&block_keys, &kind)
            .expect("读得了库");
        let read_alone: Vec<Vec<ConclusionVersion>> = block_keys
            .iter()
            .map(|key| {
                store
                    .conclusion_versions_stored_for(key, &kind)
                    .expect("读得了库")
            })
            .collect();
        assert_eq!(
            read_together, read_alone,
            "{kind:?}：一起扫回的与逐块扫的逐块相同"
        );
        assert_eq!(
            read_together,
            vec![expected_versions_of_each_block; block_keys.len()],
            "{kind:?}：每块正是存进去的那几把、按字节序，不在计划里的那一块的不出现"
        );
    }
}

/// 场景：那条两个扇区的小流排出的计划，两条路线（GPU 判器、CPU 判器）各取一遍每个点的结论版本。
/// 预期：每个点、两条路线都是判崩溃镜像那一种结论的那一把（[`ConclusionVersion::of_the_crash_image_judgement`]）：由结论种类与判器引到的被检测产品代码定，
/// 不取任何一组判法单元（判镜像的定义、GPU 判器都不取），两条路线同一把；点数与计划的点数相同。
/// 判别力（`crates/mutations.tsv`）：版本换成给定状态那一把（结论种类写错），这条红。
#[test]
fn both_routes_store_under_the_version_of_the_crash_image_judgement() {
    let (plan, _block_count) = plan_of_the_two_sector_flow_with_its_block_count();
    let versions = ConclusionVersionsOfThePlan::of_the_plan(&plan);
    for judged_by in [JudgedBy::GpuJudge, JudgedBy::CpuJudge] {
        assert_eq!(
            versions.versions_of_each_point_when_judged_by(judged_by),
            vec![ConclusionVersion::of_the_crash_image_judgement(); plan.points.len()].as_slice(),
            "{judged_by:?}：每个点都是判崩溃镜像那一把"
        );
    }
}

/// 场景：挂一个观察者（两槽全废盘面那一个），判它交回的盘的池级 checker 分别走 CPU 与走 GPU，各取一遍观察者的几把版本。
/// 预期：两条路线池级 checker 那一把相同，都是观察者盘上的池级 checker 那一种结论的那一把
/// （[`ConclusionVersion::of_the_pool_checker_on_the_images_after_the_observers`]：由结论种类与判器引到的被检测产品代码定，池级 checker、GPU 判器与观察者结论拼块的代码不进）；
/// 两条路线的结论那一把也相同。
/// 判别力（`crates/mutations.tsv`）：池级 checker 那一把换成判崩溃镜像那一把，这条红。
#[test]
fn the_pool_checker_version_of_an_observer_is_fixed_by_its_conclusion_kind_on_both_routes() {
    let observer = the_two_slot_observer_on_the_nodes_of_the_unit_test_flow();
    let versions_of = |pool_checker: JudgedBy| {
        observer_versions_keeping_the_refused_out(&observer, pool_checker).0
    };
    for pool_checker in [JudgedBy::CpuJudge, JudgedBy::GpuJudge] {
        assert_eq!(
            versions_of(pool_checker).pool_checker_version,
            ConclusionVersion::of_the_pool_checker_on_the_images_after_the_observers(),
            "{pool_checker:?}：池级 checker 那一把由结论种类与判器引到的被检测产品代码定"
        );
    }
    assert_eq!(
        versions_of(JudgedBy::CpuJudge).conclusion,
        versions_of(JudgedBy::GpuJudge).conclusion,
        "结论那一把两条路线同一把"
    );
}

/// 场景：流水线这一趟要认登没登记的几组，不挂观察者、挂一个观察者各排一遍。
/// 预期：不挂观察者时一组都不要（判镜像的定义、GPU 判器、两种核对器、观察者结论拼块都不进键）；挂一个观察者时只要它那一组。
/// 判别力（`crates/mutations.tsv`）：流水线这一趟另要判镜像的定义那一组，这条红。
#[test]
fn the_pipeline_needs_only_the_groups_of_the_observers() {
    assert_eq!(
        judging_units_needed_by_the_run(&[]),
        Vec::<JudgingUnits>::new(),
        "不挂观察者：一组都不要"
    );
    let observer = the_two_slot_observer_on_the_nodes_of_the_unit_test_flow();
    assert_eq!(
        judging_units_needed_by_the_run(std::slice::from_ref(&observer)),
        vec![JudgingUnits::ObserverCode {
            observer_code_file: observer.observer_code_file.clone(),
            observer_name: observer.observer_name.as_str().to_string(),
        }],
        "挂一个观察者：只要它那一组"
    );
}

/// 场景：那条两个扇区的小流，每一块的镜像判定都存在另一把版本下（库里留着的一把别的版本），这一趟的版本是镜像判定那一把。
/// 预期：照另一把对账一块都不缺；照这一趟两种判器存结论的那一处对账每一块都缺（续跑一块都不复用，整条流现判）。
/// 判别力（`crates/mutations.tsv`）：判过没判过不看版本、只看这一种结论有没有，这条红。
#[test]
fn blocks_judged_under_another_version_are_all_judged_again() {
    let (plan, block_count) = plan_of_the_two_sector_flow_with_its_block_count();
    let now = ConclusionVersionsOfThePlan::of_the_plan(&plan);
    let earlier_definition_key =
        vec![
            ConclusionVersion(sha256_digest(b"another version kept in the library\n"));
            plan.points.len()
        ];
    assert_ne!(
        earlier_definition_key[0],
        now.versions_of_each_point_when_judged_by(JudgedBy::CpuJudge)[0],
        "前提：早先那一把与这一趟的不同"
    );
    let under_the_earlier_definition_key = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &earlier_definition_key,
    };
    let (_directory, mut store) = an_empty_library_for("blocks-under-another-definition-key");
    store_a_judge_conclusion_for_every_block(
        &mut store,
        &plan,
        &under_the_earlier_definition_key,
        &[],
        None,
    );
    assert!(
        unjudged_blocks_of_the_flow(&store, &plan, &under_the_earlier_definition_key)
            .expect("读得了库")
            .is_empty(),
        "前提：早先那一把下每一块都判过"
    );
    for judged_by in [JudgedBy::GpuJudge, JudgedBy::CpuJudge] {
        assert_eq!(
            unjudged_blocks_of_the_flow(&store, &plan, &now.stored_under(judged_by))
                .expect("读得了库")
                .len(),
            block_count,
            "{judged_by:?}：版本不是这一把，那一把下的块一块都不算判过"
        );
    }
}
