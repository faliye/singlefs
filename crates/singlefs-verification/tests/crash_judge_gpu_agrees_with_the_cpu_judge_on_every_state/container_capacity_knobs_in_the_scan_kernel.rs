//! 三棵容器树的旋钮值编进 GPU 扫描内核（I-9.21、I-9.22、I-9.36、I-9.37）：容器号的次序、非最右那片装满、旋钮逐条换值、根环转过一圈。

use super::*;

// ───────────── G2、G3、G6：容器号从 1 起、逐片加一、每片至少一条；旋钮值编进 GPU 扫描内核（I-9.21、I-9.22、I-9.36、I-9.37）─────────────

/// 打包记录单元头里的容器号（类身份段：出生树 43、打包记录类型 51、容器号 53、容器出生代 61）。
const PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER: usize = 53;

/// 只这一棵容器树装 `records_per_container` 条一片的旋钮，另两棵当没装（交给 GPU 扫描内核与 CPU 新入口的同一组旋钮值）。
fn knobs_of_the_lineage_tree(records_per_container: u64) -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: None,
        lineage: Some(LineageContainerCapacityKnob::new(records_per_container).expect("1..=1359")),
        retirement: None,
    }
}

fn knobs_of_the_retirement_tree(records_per_container: u64) -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: None,
        lineage: None,
        retirement: Some(
            RetirementContainerCapacityKnob::new(records_per_container).expect("1..=959"),
        ),
    }
}

fn knobs_of_the_tombstone_tree(records_per_container: u64) -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: Some(
            TombstoneContainerCapacityKnob::new(records_per_container).expect("1..=815"),
        ),
        lineage: None,
        retirement: None,
    }
}

/// 带旋钮值的那一条（墓碑树 I-9.22，谱系树与退役树 I-9.37）。
const KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE: &str =
    singlefs_checker::image::TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE;

const KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES: &str =
    singlefs_checker::image::LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE;

/// 旋钮下合法的流：GPU 扫描内核编进 `knobs`，每个状态的池级 checker 违例位与 CPU 新入口带同一组旋钮值逐位相同；
/// CPU 一个状态都不红 `knob_capacity_invariant`、至少一个状态上判过而成立（旋钮那一支真走到了）。
fn assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
    test_name: &str,
    flow: &RecordedFlow,
    knobs: ContainerCapacityKnobs,
    knob_capacity_invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) =
        compare_on_every_state_given_the_container_capacity_knobs(cards, flow, knobs)
    else {
        return;
    };
    assert!(
        !comparison
            .cpu_checker_red_invariants
            .contains(knob_capacity_invariant)
            && comparison
                .cpu_checker_held_invariants
                .contains(knob_capacity_invariant),
        "流 {} 上 CPU 新入口带旋钮值：{knob_capacity_invariant} 判过而成立、一个状态都不红（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}（GPU 扫描内核编进旋钮值）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 旋钮下坏的流：同上，CPU 新入口要红 `knob_capacity_invariant`，GPU 红在同样的状态、同样的位上。
fn assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
    test_name: &str,
    flow: &RecordedFlow,
    knobs: ContainerCapacityKnobs,
    knob_capacity_invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) =
        compare_on_every_state_given_the_container_capacity_knobs(cards, flow, knobs)
    else {
        return;
    };
    assert!(
        comparison
            .cpu_checker_red_invariants
            .contains(knob_capacity_invariant),
        "流 {} 上 CPU 新入口带旋钮值要红 {knob_capacity_invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}（GPU 扫描内核编进旋钮值）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// G6：退役树两片各 1 条（[`record_two_leaves_retired_with_one_retirement_record_per_container`]），GPU 扫描内核编进退役旋钮 1：
/// 旋钮下的合法流，I-9.37 两边都判过而成立、一个状态都不红（CPU 产品入口在这条流上红 I-9.36，另一条用例钉）。
#[test]
fn the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_on_two_retirement_containers_of_one_record(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
        "the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_on_two_retirement_containers_of_one_record",
        &record_two_leaves_retired_with_one_retirement_record_per_container(
            "crash-judge-gpu-retirement-knob-two-containers",
        ),
        knobs_of_the_retirement_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G6：谱系树两片各 1 条（[`record_two_rollbacks_with_one_lineage_record_per_container`]），GPU 扫描内核编进谱系旋钮 1：合法流。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_on_two_lineage_containers_of_one_record(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_on_two_lineage_containers_of_one_record",
        &record_two_rollbacks_with_one_lineage_record_per_container(
            "crash-judge-gpu-lineage-knob-two-containers",
        ),
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G6：墓碑树两片各 1 条（[`record_two_empty_inodes_unlinked_with_one_tombstone_per_container`]），GPU 扫描内核编进墓碑旋钮 1：合法流，
/// I-9.22 两边都判过而成立（CPU 产品入口在这条流上红 I-9.21）。
#[test]
fn the_gpu_judge_given_the_tombstone_knob_agrees_with_the_cpu_on_two_tombstone_containers_of_one_record(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
        "the_gpu_judge_given_the_tombstone_knob_agrees_with_the_cpu_on_two_tombstone_containers_of_one_record",
        &record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
            "crash-judge-gpu-tombstone-knob-two-containers",
        ),
        knobs_of_the_tombstone_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
    );
}

/// 新池新建文件、建 1 个空 inode（txg 4），分配器装上谱系容器一片 `lineage_records_per_container` 条的旋钮，挂着回退到第一个文件那一版
/// （txg 3）`rollbacks` 次（同 [`record_two_rollbacks_with_one_lineage_record_per_container`]）。
fn record_rollbacks_under_the_lineage_knob(
    tag: &str,
    name: &'static str,
    lineage_records_per_container: u64,
    rollbacks: usize,
) -> RecordedFlow {
    let pool = pool_rolled_back_under_these_lineage_knobs(
        tag,
        &vec![lineage_records_per_container; rollbacks],
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 新池新建文件、建 1 个空 inode（txg 4），挂着回退到第一个文件那一版（txg 3），每次回退之前分配器换上谱系容器一片
/// `lineage_records_per_container_of_each_rollback` 里那一项条的旋钮（回退的发布路径照分配器上的开关取容量）：旋钮可以在中途换值。
fn pool_rolled_back_under_these_lineage_knobs(
    tag: &str,
    lineage_records_per_container_of_each_rollback: &[u64],
) -> BuiltPool {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    // 迭代上界：切片的长度。
    for lineage_records_per_container in lineage_records_per_container_of_each_rollback {
        pool.allocator
            .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
            batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
            tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
            lineage_container_capacity: LineageContainerCapacity::CappedForTests(
                singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
                    *lineage_records_per_container,
                )
                .expect("1..=1359"),
            ),
            retirement_container_capacity:
                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
            container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
        });
        roll_back_to(&mut pool, &first_file_version);
    }
    pool
}

/// G3 ③ 在 GPU 上（I-9.37 多于旋钮值那一判）：谱系容器一片 2 条写出两条谱系记录（一片 2 条），GPU 与 CPU 新入口都给谱系旋钮 1：那片装了 2 条，
/// CPU 红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_a_lineage_container_holds_more_records_than_the_knob(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_a_lineage_container_holds_more_records_than_the_knob",
        &record_rollbacks_under_the_lineage_knob(
            "crash-judge-gpu-lineage-container-past-the-knob",
            "two-rollbacks-two-lineage-records-in-one-container",
            2,
            2,
        ),
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

fn is_a_container_of_record_type(bytes: &[u8], record_type: u16) -> bool {
    unit_class(bytes) == 3 && packed_record_type(bytes) == record_type
}

/// 容器树根（层级 1）第 `entry_index` 条条目身份引用里的容器号改成 `container_number`（分隔 key 与子指针不动）。
fn set_the_container_number_of_an_entry(
    bytes: &mut [u8],
    entry_index: usize,
    container_number: u64,
) {
    assert!(
        entry_index < usize::from(index_node_entry_count(bytes)),
        "根里有第 {entry_index} 条条目"
    );
    let field = first_entry_of_a_container_number_node()
        + entry_index * usize::from(CONTAINER_TREE_ENTRY_BYTES)
        + CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET;
    bytes[field..field + 8].copy_from_slice(&container_number.to_le_bytes());
}

/// 流里最后写的那片 `record_type` 容器（两份副本）头里的容器号与最后写的那棵树根（两份副本）里指它的那一条（第 `entry_index` 条）条目的身份容器号
/// 一起改成 `container_number`，各自重封到根（身份照旧与子头相符）。最后写的那片容器只被最后那几版的根指着、那几版的根就是最后写的那个根。
fn renumber_the_last_container_of_the_flow(
    flow: &mut RecordedFlow,
    record_type: u16,
    entry_index: usize,
    container_number: u64,
) {
    let containers = last_unit_copies(
        flow,
        "最后写的那片容器",
        &|bytes| is_a_container_of_record_type(bytes, record_type),
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(flow, &containers, &|bytes| {
        bytes[PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER
            ..PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER + 8]
            .copy_from_slice(&container_number.to_le_bytes());
    });
    let roots = last_unit_copies(
        flow,
        "最后写的那棵容器树根",
        &|bytes| is_a_container_tree_root_over_containers_of_record_type(bytes, record_type),
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(flow, &roots, &|bytes| {
        set_the_container_number_of_an_entry(bytes, entry_index, container_number);
    });
}

/// G3 ①（GPU 那一份）：谱系树两片各 1 条，第二片容器与根里指它的条目的容器号 2 → 1（两片同号，攻方 `mut-w-same`）。GPU 与 CPU 新入口都给谱系旋钮 1：
/// 记录数都在旋钮之内，CPU 只因「相邻两片加一」红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_two_lineage_containers_share_a_number(
) {
    let mut flow = record_rollbacks_under_the_lineage_knob(
        "crash-judge-gpu-lineage-containers-share-a-number",
        "two-rollbacks-lineage-containers-numbered-one-and-one",
        1,
        2,
    );
    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_LINEAGE, 1, 1);
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_two_lineage_containers_share_a_number",
        &flow,
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G3 ①（GPU 那一份）：退役树两片各 1 条，第二片的容器号 2 → 1（两片同号）。GPU 与 CPU 新入口都给退役旋钮 1，CPU 红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_when_two_retirement_containers_share_a_number(
) {
    let mut flow = record_two_leaves_retired_with_one_retirement_record_per_container(
        "crash-judge-gpu-retirement-containers-share-a-number",
    );
    flow.name = "two-leaves-retired-retirement-containers-numbered-one-and-one";
    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_RETIREMENT, 1, 1);
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_when_two_retirement_containers_share_a_number",
        &flow,
        knobs_of_the_retirement_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G2（GPU 那一份，攻方 `mut-w-gap2`）：谱系树两片各 1 条，第二片的容器号 2 → 3（1、3）。CPU 新入口红 I-9.37（不是逐片加一），GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_skip_a_number()
{
    let mut flow = record_rollbacks_under_the_lineage_knob(
        "crash-judge-gpu-lineage-containers-skip-a-number",
        "two-rollbacks-lineage-containers-numbered-one-and-three",
        1,
        2,
    );
    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_LINEAGE, 1, 3);
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_skip_a_number",
        &flow,
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G2（GPU 那一份，攻方 `mut-w-start2`）：谱系树两片各 1 条，流里每一片谱系容器头里的容器号与每个谱系树根里每条条目的身份容器号都加 1（2、3）：
/// 回退一次那一版的谱系树只一片、号 2，回退两次那一版两片 2、3。CPU 新入口红 I-9.37（第一片不是 1），GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_start_from_two()
{
    let mut flow = record_rollbacks_under_the_lineage_knob(
        "crash-judge-gpu-lineage-containers-start-from-two",
        "two-rollbacks-lineage-containers-numbered-two-and-three",
        1,
        2,
    );
    let every_lineage_container: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| {
                    is_a_container_of_record_type(bytes, PACKED_RECORD_TYPE_LINEAGE)
                })
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        every_lineage_container.len(),
        4,
        "两片谱系容器各写一次、每次两份副本"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &every_lineage_container, &|bytes| {
        let container_number =
            read_u64_at(bytes, PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER);
        bytes[PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER
            ..PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER + 8]
            .copy_from_slice(&(container_number + 1).to_le_bytes());
    });
    let every_lineage_root: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite && write.bytes().is_some_and(is_the_lineage_tree_root)
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        every_lineage_root.len(),
        4,
        "两次回退各写一个谱系树根、每次两份副本"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &every_lineage_root, &|bytes| {
        // 迭代上界：根里的条目数（这条流里至多 2）。
        for entry_index in 0..usize::from(index_node_entry_count(bytes)) {
            let field = first_entry_of_a_container_number_node()
                + entry_index * usize::from(CONTAINER_TREE_ENTRY_BYTES)
                + CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET;
            let container_number = read_u64_at(bytes, field);
            set_the_container_number_of_an_entry(bytes, entry_index, container_number + 1);
        }
    });
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_start_from_two",
        &flow,
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G2（GPU 那一份，攻方 `mut-w-eager`）：谱系容器一片 1 条回退三次（三片各 1 条），最后写的那片清空（记录数与声明长度写 0、记录区写 0）：
/// 最右多一片 0 条。CPU 新入口红 I-9.37（每片至少一条），GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_the_rightmost_lineage_container_is_empty(
) {
    let mut flow = record_rollbacks_under_the_lineage_knob(
        "crash-judge-gpu-lineage-rightmost-container-empty",
        "three-rollbacks-rightmost-lineage-container-empty",
        1,
        3,
    );
    let last_container = last_unit_copies(
        &flow,
        "最后写的那片谱系容器",
        &|bytes| is_a_container_of_record_type(bytes, PACKED_RECORD_TYPE_LINEAGE),
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &last_container, &|bytes| {
        assert_eq!(
            read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET),
            1,
            "最后写的那片谱系容器一条记录"
        );
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
            .copy_from_slice(&0u16.to_le_bytes());
        bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2]
            .copy_from_slice(&0u16.to_le_bytes());
        bytes[PACKED_UNIT_RECORDS_START..].fill(0);
    });
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_the_rightmost_lineage_container_is_empty",
        &flow,
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// 甲二（每段原地写任意组合 × 单元写全不落或全落，判决 m3-crash-enumeration-merge-r2 第二节 L6-2）展开的新池新建文件那条流：
/// GPU 探针内核拆出的持久掩码与 CPU 判表（`crash_facts` 那一套拆法）、crash.rs 枚举（`judge_crash_state_range` 交给回调的镜像）逐序号相同，
/// 判器三个内核在每个状态上判出的对拍字与 CPU 判器逐状态相同。
#[test]
fn the_gpu_judge_decodes_and_judges_in_place_subsets_with_copy_on_write_none_or_all_like_the_cpu() {
    const TEST_NAME: &str =
        "the_gpu_judge_decodes_and_judges_in_place_subsets_with_copy_on_write_none_or_all_like_the_cpu";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let flow = record_new_pool_file_creation(
        "crash-judge-gpu-in-place-subsets",
        "new-pool-in-place-subsets",
    );
    let expansion =
        singlefs_verification::crash::in_place_subsets_with_copy_on_write_none_or_all_expansion;
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    assert!(
        tables.segment_expansions.iter().any(|segment_expansion| matches!(
            segment_expansion,
            singlefs_verification::crash_facts::SegmentExpansionFact::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
                copy_on_write_writes,
                ..
            } if copy_on_write_writes.len() >= 2
        )),
        "这条流里有带两次以上单元写的甲二段：GPU 按全量拆就拆出单元写只落一部分的状态"
    );
    // 判表先入库（对拍那一类的库）、崩溃点重放的流也先入库（崩溃点重放枚举那一类的库），两样都录完才看只录那一趟要不要当场返回
    let packed = packed_tables_through_the_library(TEST_NAME, &tables);
    let mut persisted_sets_of_the_crash_enumeration: Vec<Vec<bool>> = Vec::new();
    let judged = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: TEST_NAME,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..tables.state_count,
        &mut |_ordinal, image, _judgement| {
            persisted_sets_of_the_crash_enumeration.push(image.persisted.clone())
        },
    );
    let (Some(packed), Some(judged)) = (packed, judged.judged()) else {
        return;
    };
    assert_eq!(judged, tables.state_count);
    for (ordinal, persisted) in persisted_sets_of_the_crash_enumeration.iter().enumerate() {
        assert_eq!(
            &tables.persisted_writes_of_state(u64::try_from(ordinal).expect("序号装得进 u64")),
            persisted,
            "序号 {ordinal}：判表那一套拆法与 crash.rs 枚举拆出的持久集合相同"
        );
    }
    let Some(lease) =
        cards.lease_card_or_report_not_run(TEST_NAME, mebibytes_a_judge_needs(&packed, 1))
    else {
        return;
    };
    let probe_words = singlefs_verification::crash_judge_dispatch::persisted_masks_on_the_card(
        lease.card(),
        &packed,
        0..tables.state_count,
    );
    drop(lease);
    let enumerated_writes = usize::try_from(tables.enumerated_write_count).expect("装得进 usize");
    for (ordinal, words) in probe_words.iter().enumerate() {
        let (persisted, undecidable) =
            singlefs_verification::crash_judge_gpu::persisted_set_of_the_probe_words(
                words,
                enumerated_writes,
            );
        assert_eq!(undecidable, 0, "序号 {ordinal}：探针拆得了");
        assert_eq!(
            persisted, persisted_sets_of_the_crash_enumeration[ordinal],
            "序号 {ordinal}：GPU 拆出的持久掩码与 CPU 相同"
        );
    }
    let Some(comparison) =
        compare_on_every_state_of_these_segments(cards, &flow, &flow.segments, &expansion)
    else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "甲二展开的 {} 个状态里 {} 个 GPU 与 CPU 的对拍字不同：\n{}",
        comparison.states,
        comparison.mismatches.state_count(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 33 次两态写的一段（判决 m3-crash-enumeration-merge-r2 第二节 L1-7，攻方模型 decode_alias.py 那一形）：从第 2³² 个状态起拆 8 个，
/// GPU 探针按 lo / hi 两个字算段内余数，拆出的掩码与 CPU 判表逐序号相同（第 2³² + 5 个是写 0、写 2、写 32 持久，不被截成低 32 位的 5）。
#[test]
fn the_gpu_judge_decodes_an_ordinal_of_a_thirty_three_write_segment_past_thirty_two_bits_like_the_cpu(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_decodes_an_ordinal_of_a_thirty_three_write_segment_past_thirty_two_bits_like_the_cpu";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let writes = 33u32;
    let states = (1u64 << writes) - 1;
    let tables = singlefs_verification::crash_judge_tables::JudgeTables {
        recorded_write_count: writes,
        enumerated_write_count: writes,
        is_tearable: vec![false; 33],
        torn_images: vec![],
        segments: vec![singlefs_verification::crash_facts::SegmentFact {
            writes: (0..writes).collect(),
            states: 0..states,
            slice_start: None,
        }],
        segment_expansions: vec![
            singlefs_verification::crash_facts::SegmentExpansionFact::EveryProperSubset,
        ],
        state_source:
            singlefs_verification::crash_judge_tables::JudgeStateSource::SegmentExpansions,
        state_count: states + 1,
        devices: vec![0],
        device_bytes: 1 << 26,
        unit_area_start_in_bytes: 1 << 24,
        judged_root_index: 0,
        writes: vec![],
        locations: vec![],
        publishes: vec![],
        expected_versions: vec![],
    };
    // 探针也只拿读回的那一份表算；只录那一趟用例当场返回
    let Some(packed) = packed_tables_through_the_library(TEST_NAME, &tables) else {
        return;
    };
    let Some(lease) =
        cards.lease_card_or_report_not_run(TEST_NAME, mebibytes_a_judge_needs(&packed, 1))
    else {
        return;
    };
    let first = 1u64 << 32;
    let probe_words = singlefs_verification::crash_judge_dispatch::persisted_masks_on_the_card(
        lease.card(),
        &packed,
        first..first + 8,
    );
    drop(lease);
    for (offset, words) in probe_words.iter().enumerate() {
        let ordinal = first + u64::try_from(offset).expect("装得进 u64");
        let (persisted, undecidable) =
            singlefs_verification::crash_judge_gpu::persisted_set_of_the_probe_words(words, 33);
        assert_eq!(undecidable, 0);
        assert_eq!(
            persisted,
            tables.persisted_writes_of_state(ordinal),
            "序号 {ordinal}"
        );
    }
    let (fifth, _) = singlefs_verification::crash_judge_gpu::persisted_set_of_the_probe_words(
        &probe_words[5],
        33,
    );
    let persisted_writes: Vec<usize> = fifth
        .iter()
        .enumerate()
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|(write_index, _)| write_index)
        .collect();
    assert_eq!(persisted_writes, vec![0, 2, 32]);
}

/// 给定状态表（判决 m3-crash-enumeration-merge-r2 第二节 L6-3，随机历史两项上 GPU 要的那一种展开）：新池新建文件那条流全量展开的掩码，
/// 打乱次序（倒过来、隔一个取一个）再接上全部持久那一个（它在倒过来那一串的头一个里已经有了，清单里同一个状态出现两次也照判）做成清单，GPU 照清单判出的对拍字与 CPU 判器在同一张镜像上的逐状态相同。
#[test]
fn the_gpu_judge_on_given_persisted_sets_agrees_with_the_cpu_judge_on_every_state() {
    const TEST_NAME: &str =
        "the_gpu_judge_on_given_persisted_sets_agrees_with_the_cpu_judge_on_every_state";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let flow =
        record_new_pool_file_creation("crash-judge-gpu-given-states", "new-pool-given-states");
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let enumerated = build_judge_tables(&judge_flow).expect("建得出表");
    let mut given_ordinals: Vec<u64> = (0..enumerated.state_count).rev().step_by(2).collect();
    given_ordinals.push(enumerated.state_count - 1);
    let given: Vec<Vec<bool>> = given_ordinals
        .iter()
        .map(|ordinal| enumerated.persisted_writes_of_state(*ordinal))
        .collect();
    let tables =
        singlefs_verification::crash_judge_tables::build_judge_tables_for_the_given_persisted_sets(
            &judge_flow,
            &[],
            given.clone(),
        )
        .expect("给定状态表建得出");
    assert_eq!(
        tables.state_count,
        u64::try_from(given.len()).expect("装得进 u64")
    );
    let Some(gpu) =
        singlefs_verification::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
    else {
        return;
    };
    // CPU 参照一侧：流先入崩溃点重放枚举那一类的库、再从库里读回，按序号判（CPU 参照也从库读），
    // 清单第 i 个取它在全量展开里的序号那一份
    let Some(cpu_by_ordinal) = cpu_cross_check_words_read_back_from_the_library(
        &format!("{TEST_NAME}-cpu-reference"),
        &flow,
        &expansion,
        0..enumerated.state_count,
    ) else {
        return;
    };
    let mismatches: Vec<String> = given_ordinals
        .iter()
        .zip(&given)
        .zip(&gpu)
        .enumerate()
        .filter_map(|(position, ((ordinal, persisted), gpu_words))| {
            let (cpu_persisted, cpu_cross_check_words) = &cpu_by_ordinal[ordinal];
            assert_eq!(
                cpu_persisted, persisted,
                "清单第 {position} 个：CPU 从库里读回判的序号 {ordinal} 与清单那一个是同一个状态"
            );
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (!cross_check_words_agree_in_the_cross_check(
                &gpu_cross_check_words,
                cpu_cross_check_words,
            ))
            .then(|| {
                format!(
                    "清单第 {position} 个（序号 {ordinal}）：CPU {cpu_cross_check_words:?}；GPU {gpu_cross_check_words:?}"
                )
            })
        })
        .collect();
    assert_eq!(gpu.len(), given.len(), "GPU 判了清单里的每一个");
    assert!(
        mismatches.is_empty(),
        "{} 个点名的状态 GPU 与 CPU 不同：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// CPU 判器经崩溃点重放枚举那一类的库判这条流在 `ordinals` 上的每个状态（流先入库、再从库里读回判，
/// `crash_state_enumeration::sample_crash_states_on_the_cpu_with_threads_against_the_version_table`），交回按序号的持久集合与对拍字；
/// 只录那一趟交 `None`。线程数照 [`CPU_THREADS_ENVIRONMENT_VARIABLE`]。
pub(super) fn cpu_cross_check_words_read_back_from_the_library(
    item_name: &str,
    flow: &RecordedFlow,
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    ordinals: std::ops::Range<u64>,
) -> Option<BTreeMap<u64, (Vec<bool>, CrossCheckWords)>> {
    let judged_by_ordinal: std::sync::Mutex<BTreeMap<u64, (Vec<bool>, CrossCheckWords)>> =
        std::sync::Mutex::new(BTreeMap::new());
    let judged = sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
        CrashPointReplayItem {
            name: item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        CrashPointReplayVersionTable::new(&flow.versions, &[]),
        expansion,
        ordinals.clone(),
        cpu_threads_from_the_value_of_the_variable(
            process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
            process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        ),
        &|ordinal, image, judgement| {
            judged_by_ordinal.lock().expect("判定线程不 panic").insert(
                ordinal,
                (
                    image.persisted.clone(),
                    cross_check_words_of_cpu_judgement(judgement),
                ),
            );
        },
    )
    .judged()?;
    let judged_by_ordinal = judged_by_ordinal.into_inner().expect("判定线程不 panic");
    assert_eq!(
        u64::try_from(judged_by_ordinal.len()).expect("装得进 u64"),
        ordinals.end - ordinals.start,
        "CPU 从库里读回之后判了点名序号段的每个状态（交回 {judged} 个）"
    );
    Some(judged_by_ordinal)
}

/// 判表核段长（判决 m3-crash-enumeration-merge-r2 第二节 L1-7「判表打包核段长」）：建表时每段按它的展开方式现算的状态数要与枚举计划的序号区间相同。
/// 第二次问展开方式时改了口（枚举计划时每段都不展开、序号区间都是空的，建判表那一份拆法时换成甲二、每段至少一个状态）的流，
/// 建表交「段的状态数对不上」，不 panic、不建出一张拆错的表。
// crash-case-check:not-a-crash-case 只建一张判表，不判任何崩溃状态
#[test]
fn a_judge_flow_whose_expansion_changes_between_two_asks_is_refused_for_the_segment_state_count() {
    let flow = record_new_pool_file_creation(
        "crash-judge-gpu-expansion-changes",
        "new-pool-expansion-changes",
    );
    let asks = std::cell::Cell::new(0usize);
    let segment_count = flow.segments.len();
    let expansion = |_segment_index: usize, _segment: &[usize]| {
        asks.set(asks.get() + 1);
        if asks.get() <= segment_count {
            CrashPointReplaySegmentExpansion::NotExpanded
        } else {
            CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
        }
    };
    let refusal = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect_err("段的状态数对不上，不建表");
    assert!(
        matches!(
            refusal,
            singlefs_verification::crash_judge_tables::JudgeTablesRefusal::SegmentStateCountNotRepresentable { .. }
        ),
        "{refusal:?}"
    );
}

// ───────────── H1、H2：旋钮下非最右那片装满旋钮值；产品支墓碑那一个调用点 ─────────────

/// H1（GPU 那一份，攻方 `k2_knob_bad_images` 那一形）：旋钮 1 写出的两片各 1 条，GPU 扫描内核与 CPU 新入口都给这一棵旋钮 2：每片都不超旋钮值、
/// 容器号从 1 逐片加一，只「不是最右的那片装满旋钮值」不成立，CPU 红这一棵的那一条（I-9.22 / I-9.37），GPU 红在同样的状态、同样的位上。
#[test]
fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_two_tombstone_containers_written_under_the_knob_one(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_two_tombstone_containers_written_under_the_knob_one",
        &record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
            "crash-judge-gpu-tombstone-knob-two-over-containers-of-one-record",
        ),
        knobs_of_the_tombstone_tree(2),
        KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
    );
}

#[test]
fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_two_lineage_containers_written_under_the_knob_one(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_two_lineage_containers_written_under_the_knob_one",
        &record_two_rollbacks_with_one_lineage_record_per_container(
            "crash-judge-gpu-lineage-knob-two-over-containers-of-one-record",
        ),
        knobs_of_the_lineage_tree(2),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

#[test]
fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_two_retirement_containers_written_under_the_knob_one(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_two_retirement_containers_written_under_the_knob_one",
        &record_two_leaves_retired_with_one_retirement_record_per_container(
            "crash-judge-gpu-retirement-knob-two-over-containers-of-one-record",
        ),
        knobs_of_the_retirement_tree(2),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// H2 产品支墓碑那一个调用点要的流：新池新建文件、再建 816 个空 inode（一次发布），照产品容量逐个 unlink 前 815 个到底（墓碑树一片装满 815 条），
/// 再 unlink 第 816 个（意图那次开第二片、1 条）：墓碑树 [815, 1]，产品镜像合法。起点镜像是 mkfs 到第 816 个 unlink 之前的整段录制流施加出来的那一份，
/// 写表只录第 816 个 unlink 那几次发布（前 815 次 unlink 约 1630 次发布，整条流从 mkfs 录起时它们的写也进枚举写表、
/// 超过判器持久掩码的宽，建表交 `EnumeratedWritesPastTheMaskWidth`）。
fn record_a_second_tombstone_container_opened_after_a_full_one_at_product_capacity(
    tag: &str,
) -> RecordedFlow {
    let tombstones_filling_one_container = singlefs_format::TOMBSTONE_CONTAINER_RECORDS;
    let mut pool = pool_with_new_inodes(tag, tombstones_filling_one_container + 1);
    // 迭代上界：815 个号。
    for inode in FIRST_FILE.0 + 1..=FIRST_FILE.0 + tombstones_filling_one_container {
        unlink_an_empty_inode_to_the_end_under_the_capacities(
            &mut pool,
            inode,
            ContainerTreeCapacities::FROM_THE_FORMAT,
        );
    }
    let operations_before_the_last_unlink = pool.stream.operation_count();
    unlink_an_empty_inode_to_the_end_under_the_capacities(
        &mut pool,
        FIRST_FILE.0 + tombstones_filling_one_container + 1,
        ContainerTreeCapacities::FROM_THE_FORMAT,
    );
    let tombstone_record_counts: Vec<usize> = pool
        .output
        .tombstone_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect();
    assert_eq!(
        tombstone_record_counts,
        [
            usize::try_from(tombstones_filling_one_container).expect("装得进 usize"),
            1
        ],
        "墓碑树第一片装满 815、第二片 1 条"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    let operations = pool.retained_operations();
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&operations[pool.mkfs_operation_count..operations_before_the_last_unlink]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_last_unlink..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("第 816 个 unlink 的发布有根槽写");
    RecordedFlow {
        name: "product-tombstone-containers-after-a-full-one",
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

/// H2（GPU 产品支墓碑那一个调用点，攻方 `mut-w-gap2` 那一形）：产品容量下墓碑树 [815, 1]，第二片容器与根里指它的条目的容器号 2 → 3（1、3）：
/// 第一片装满、容量那句成立，CPU 产品入口只因「相邻两片加一」红 I-9.21，GPU 红在同样的状态、同样的位上。写表只录第 816 个 unlink 那几次发布
/// （之前的录制流施加成起点镜像），每段照展开上限展开；GPU 扫描内核产品支不调三判（`d_judge_tombstone_container_capacity` 里那一个调用点）
/// 就在这里与 CPU 不一致。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number";
    let mut flow = record_a_second_tombstone_container_opened_after_a_full_one_at_product_capacity(
        "crash-judge-gpu-i921-product-containers-skip-a-number",
    );
    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_TOMBSTONE, 1, 3);
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.21"),
        "流 {} 上 CPU 池级 checker 要红 I-9.21（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

// ───────────── J3：旋钮支三片、中间那片少一条 ─────────────

/// 写的时候逐条换的旋钮值：2、2、1、1 写出 [2, 1, 1]（攻方 `research/prompts/m3-batch57-code-r1-opus-model/probe-crate/tests/k4_middle_short_container.rs` 那一形）。
/// 写侧每一步都照当时的旋钮满了才关；两片容器时「只判叶序第一片」与正确写法等价，三片时分得开。
const KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER: [u64; 4] = [2, 2, 1, 1];

/// 按叶序每片的记录数。
const RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER: [usize; 3] = [2, 1, 1];

/// 判的时候编进 GPU 扫描内核、交给 CPU 新入口的旋钮值：容器 2 不是最右那片、只装 1 条，不是旋钮值。
const KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER: u64 = 2;

/// 只墓碑树压到 `records_per_container` 条一片的写入口（另两棵与节点照产品）。
fn capacities_of_the_tombstone_container_knob(
    records_per_container: u64,
) -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(records_per_container)
                .expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 只退役树压到 `records_per_container` 条一片的写入口（另两棵与节点照产品）。
fn capacities_of_the_retirement_container_knob(
    records_per_container: u64,
) -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
                    records_per_container,
                )
                .expect("1..=959"),
            ),
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 墓碑树那一形：新池新建文件、建 4 个空 inode（2–5 号），逐个 unlink 到底，写入口按
/// [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER`] 逐个换墓碑旋钮：墓碑树 [2, 1, 1]。
fn record_tombstone_containers_of_two_one_and_one_records_under_changing_knobs(
    tag: &str,
) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(
        tag,
        u64::try_from(KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER.len())
            .expect("装得进 u64"),
    );
    // 迭代上界：4 个号。
    for (inode, records_per_container) in
        (FIRST_FILE.0 + 1..).zip(KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER)
    {
        unlink_an_empty_inode_to_the_end_under_the_capacities(
            &mut pool,
            inode,
            capacities_of_the_tombstone_container_knob(records_per_container),
        );
    }
    let tombstone_record_counts: Vec<usize> = pool
        .output
        .tombstone_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect();
    assert_eq!(
        tombstone_record_counts, RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER,
        "墓碑树按叶序每片的记录数"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "tombstone-containers-of-two-one-and-one-records-under-changing-knobs",
        versions,
    )
}

/// 谱系树那一形：新池新建文件、建 1 个空 inode，挂着回退到第一个文件那一版 4 次，每次回退之前按
/// [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER`] 换谱系旋钮：谱系树 [2, 1, 1]。
fn record_lineage_containers_of_two_one_and_one_records_under_changing_knobs(
    tag: &str,
) -> RecordedFlow {
    let pool = pool_rolled_back_under_these_lineage_knobs(
        tag,
        &KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER,
    );
    let lineage_record_counts: Vec<usize> = pool
        .output
        .lineage_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect();
    assert_eq!(
        lineage_record_counts, RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER,
        "谱系树按叶序每片的记录数"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "lineage-containers-of-two-one-and-one-records-under-changing-knobs",
        versions,
    )
}

/// 退役树那一形：新池新建文件之外再建 233 个空 inode（叶 [1..=233]、[234]），unlink 234 到底（最右那片叶删空、退役一条），
/// 之后每次建 1 个（开新叶）、unlink 它到底（又退役一条），共退役 4 次，写入口按 [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER`]
/// 逐次换退役旋钮：退役树 [2, 1, 1]。
fn record_retirement_containers_of_two_one_and_one_records_under_changing_knobs(
    tag: &str,
) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
    // 迭代上界：4 次退役。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
    for (retirement_index, records_per_container) in
        KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER
            .into_iter()
            .enumerate()
    {
        if retirement_index > 0 {
            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
        }
        let inode = pool.output.inode_number_watermark() - 1;
        unlink_an_empty_inode_to_the_end_under_the_capacities(
            &mut pool,
            inode,
            capacities_of_the_retirement_container_knob(records_per_container),
        );
    }
    let retirement_record_counts: Vec<usize> = pool
        .output
        .retirement_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect();
    assert_eq!(
        retirement_record_counts, RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER,
        "退役树按叶序每片的记录数"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "retirement-containers-of-two-one-and-one-records-under-changing-knobs",
        versions,
    )
}

/// J3（GPU 那一份）：墓碑树 [2, 1, 1]，GPU 扫描内核与 CPU 新入口都给墓碑旋钮 2：每片都不超旋钮值、容器号从 1 逐片加一，
/// 只「不是最右的容器 2 装满旋钮值」不成立，CPU 红 I-9.22，GPU 红在同样的状态、同样的位上（GPU 只判叶序第一片就在这里与 CPU 不一致）。
#[test]
fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_and_one_records_written_under_changing_knobs(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_and_one_records_written_under_changing_knobs",
        &record_tombstone_containers_of_two_one_and_one_records_under_changing_knobs(
            "crash-judge-gpu-tombstone-containers-of-two-one-and-one-records",
        ),
        knobs_of_the_tombstone_tree(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
        KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
    );
}

/// J3（GPU 那一份）：谱系树 [2, 1, 1]，两边都给谱系旋钮 2，CPU 红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_and_one_records_written_under_changing_knobs(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_and_one_records_written_under_changing_knobs",
        &record_lineage_containers_of_two_one_and_one_records_under_changing_knobs(
            "crash-judge-gpu-lineage-containers-of-two-one-and-one-records",
        ),
        knobs_of_the_lineage_tree(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// J3（GPU 那一份）：退役树 [2, 1, 1]，两边都给退役旋钮 2，CPU 红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_and_one_records_written_under_changing_knobs(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_and_one_records_written_under_changing_knobs",
        &record_retirement_containers_of_two_one_and_one_records_under_changing_knobs(
            "crash-judge-gpu-retirement-containers-of-two-one-and-one-records",
        ),
        knobs_of_the_retirement_tree(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

// ───────────── L1：少一条的那片之后还有满的一片、根环转过一圈 ─────────────
// ───────────── N1：少一条的那片离最右三片 ─────────────

/// 只一棵容器树逐条换旋钮写、再把根环转过一圈的那几张盘怎么造：写第 k 条记录时这一棵一片几条，根环转过一圈之后这一棵按叶序每片的记录数。
struct ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver {
    records_per_container_of_each_record: &'static [u64],
    record_counts_in_leaf_order: &'static [usize],
}

/// 少一条的那片之后还有一片满的（L1）：旋钮 2、2、1、1、2、1 写出 [2, 1, 2, 1]（`crates/singlefs-verification/tests/checker_known_bad_images.rs` 的
/// `A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` 那一形）。J3 那三条 [2, 1, 1] 上少一条的那片只会落在倒数第二位，GPU 扫描内核
/// 「只判倒数第二片」在它们上面与正确判法逐位相同；这里少一条的容器 2 之后还有一片满的。
const A_SHORT_CONTAINER_BEFORE_A_FULL_ONE:
    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver =
    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver {
        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 1],
        record_counts_in_leaf_order: &[2, 1, 2, 1],
    };

/// 少一条的那片离最右三片（N1）：旋钮 2、2、1、1、2、2、2、1 写出 [2, 1, 2, 2, 1]（`crates/singlefs-verification/tests/checker_known_bad_images.rs` 的
/// `A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST` 那一形，L2）。上面 [2, 1, 2, 1] 与 J3 那几条上少一条的那片离最右
/// 都不超过两片，GPU 扫描内核「只判最右之前两片」（攻方 `research/prompts/m3-batch59-code-r1-opus-model/` 的 `w-last-two`）在它们上面与正确判法逐位相同；
/// 这里容器 2 在倒数第三片之前。写的中途 [2, 1, 1]、[2, 1, 2]、[2, 1, 2, 1] 那几版里容器 2 离最右不超过两片，留在候选集里时那一形也红，所以要转出去。
const A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST:
    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver =
    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver {
        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 2, 2, 1],
        record_counts_in_leaf_order: &[2, 1, 2, 2, 1],
    };

/// L1、N1 那几条流写的是哪一棵容器树（另两棵照产品）。
#[derive(Clone, Copy, Debug)]
enum ContainerTreeWrittenBeforeTheRootRingTurnsOver {
    Tombstone,
    Lineage,
    Retirement,
}

impl ContainerTreeWrittenBeforeTheRootRingTurnsOver {
    /// 新池新建文件之后，第 k 条记录写的时候只这一棵压到 `records_per_container_of_each_record[k]` 条一片：墓碑树逐个 unlink 空 inode 到底
    /// （同 [`record_tombstone_containers_of_two_one_and_one_records_under_changing_knobs`]），谱系树挂着回退到第一个文件那一版
    /// （[`pool_rolled_back_under_these_lineage_knobs`]），退役树每次退役独占最右那片叶的号（同
    /// [`record_retirement_containers_of_two_one_and_one_records_under_changing_knobs`]）。
    fn pool_written_under_these_knobs(
        self,
        tag: &str,
        records_per_container_of_each_record: &[u64],
    ) -> BuiltPool {
        match self {
            Self::Tombstone => {
                let mut pool = pool_with_new_inodes(
                    tag,
                    u64::try_from(records_per_container_of_each_record.len()).expect("装得进 u64"),
                );
                // 迭代上界：切片的长度。
                for (inode, records_per_container) in
                    (FIRST_FILE.0 + 1..).zip(records_per_container_of_each_record)
                {
                    unlink_an_empty_inode_to_the_end_under_the_capacities(
                        &mut pool,
                        inode,
                        capacities_of_the_tombstone_container_knob(*records_per_container),
                    );
                }
                pool
            }
            Self::Lineage => pool_rolled_back_under_these_lineage_knobs(
                tag,
                records_per_container_of_each_record,
            ),
            Self::Retirement => {
                let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
                // 迭代上界：切片的长度。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
                for (retirement_index, records_per_container) in
                    records_per_container_of_each_record.iter().enumerate()
                {
                    if retirement_index > 0 {
                        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(
                            &mut pool, 1,
                        );
                    }
                    let inode = pool.output.inode_number_watermark() - 1;
                    unlink_an_empty_inode_to_the_end_under_the_capacities(
                        &mut pool,
                        inode,
                        capacities_of_the_retirement_container_knob(*records_per_container),
                    );
                }
                pool
            }
        }
    }

    /// 这一版这一棵按叶序每片的记录数。
    fn record_counts_of_each_container(self, pool: &BuiltPool) -> Vec<usize> {
        match self {
            Self::Tombstone => pool
                .output
                .tombstone_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
            Self::Lineage => pool
                .output
                .lineage_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
            Self::Retirement => pool
                .output
                .retirement_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
        }
    }

    /// 判的时候编进 GPU 扫描内核、交给 CPU 新入口的旋钮值：只这一棵装 `records_per_container` 条一片。
    fn knobs_judging(self, records_per_container: u64) -> ContainerCapacityKnobs {
        match self {
            Self::Tombstone => knobs_of_the_tombstone_tree(records_per_container),
            Self::Lineage => knobs_of_the_lineage_tree(records_per_container),
            Self::Retirement => knobs_of_the_retirement_tree(records_per_container),
        }
    }

    /// 这一棵带旋钮值的那一条（墓碑树 I-9.22，谱系树与退役树 I-9.37）。
    fn knob_capacity_invariant(self) -> &'static str {
        match self {
            Self::Tombstone => KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
            Self::Lineage | Self::Retirement => {
                KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES
            }
        }
    }
}

/// 只 `tree` 这一棵按 `layout` 逐条换旋钮写出三片以上的容器，再建 30 次空 inode（[`turn_the_root_ring_over_by_creating_inodes`]）把根环转过一圈：
/// 写这棵树的那几版、连同写到一半时少一条的那片离最右更近的那几版，都转出写完之后各版的回退候选集，那几版的状态上照判写到一半的那一形，
/// 写完之后的状态上候选集里只剩 `layout.record_counts_in_leaf_order` 那一形（与 harness `checker_known_bad_images.rs` 里同名那一形的三条同形）。
/// 流从 mkfs 之后录起，被判的是最后一次根槽写。
fn record_containers_written_under_changing_knobs_with_the_root_ring_turned_over(
    tree: ContainerTreeWrittenBeforeTheRootRingTurnsOver,
    layout: &ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver,
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool =
        tree.pool_written_under_these_knobs(tag, layout.records_per_container_of_each_record);
    turn_the_root_ring_over_by_creating_inodes(&mut pool);
    assert_eq!(
        tree.record_counts_of_each_container(&pool),
        layout.record_counts_in_leaf_order,
        "{tree:?} 按叶序每片的记录数（根环转过一圈之后）"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// L1、N1：`tree` 这一棵按 `layout` 写、根环转过一圈，GPU 扫描内核与 CPU 新入口都给这一棵旋钮 2：每片都不超旋钮值、容器号从 1 逐片加一、
/// 每片至少一条，只「不是最右的容器 2 装了 1 条，不是旋钮值 2」不成立。CPU 在这条流的全部状态上红过的不变量恰好是这一棵的那一条
/// （I-9.22 / I-9.37；另两棵照产品、不红，别的不变量在合法写出来的盘上也不红），GPU 红在同样的状态、同样的位上。
/// [2, 1, 2, 1] 上 GPU 扫描内核只判倒数第二片（第 3 片，装满 2 条）、[2, 1, 2, 2, 1] 上只判最右之前两片（第 3、4 片，都装满 2 条），
/// 在根环转过之后的状态上照绿，就在那里与 CPU 不一致。
fn assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
    test_name: &str,
    tree: ContainerTreeWrittenBeforeTheRootRingTurnsOver,
    layout: &ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver,
    tag: &str,
    name: &'static str,
) {
    let flow = record_containers_written_under_changing_knobs_with_the_root_ring_turned_over(
        tree, layout, tag, name,
    );
    let knob_capacity_invariant = tree.knob_capacity_invariant();
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_given_the_container_capacity_knobs(
        cards,
        &flow,
        tree.knobs_judging(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
    ) else {
        return;
    };
    assert_eq!(
        comparison.cpu_checker_red_invariants,
        std::collections::BTreeSet::from([knob_capacity_invariant]),
        "流 {} 上 CPU 新入口带旋钮值红过的不变量恰好是 {knob_capacity_invariant}",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}（GPU 扫描内核编进旋钮值）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

#[test]
fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Tombstone,
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
        "crash-judge-gpu-tombstone-containers-of-two-one-two-and-one-records-ring-turned",
        "tombstone-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Lineage,
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
        "crash-judge-gpu-lineage-containers-of-two-one-two-and-one-records-ring-turned",
        "lineage-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Retirement,
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
        "crash-judge-gpu-retirement-containers-of-two-one-two-and-one-records-ring-turned",
        "retirement-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Tombstone,
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
        "crash-judge-gpu-tombstone-containers-of-two-one-two-two-and-one-records-ring-turned",
        "tombstone-containers-of-two-one-two-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Lineage,
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
        "crash-judge-gpu-lineage-containers-of-two-one-two-two-and-one-records-ring-turned",
        "lineage-containers-of-two-one-two-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Retirement,
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
        "crash-judge-gpu-retirement-containers-of-two-one-two-two-and-one-records-ring-turned",
        "retirement-containers-of-two-one-two-two-and-one-records-with-the-root-ring-turned-over",
    );
}
