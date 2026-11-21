//! 退役记录（I-9.32 到 I-9.35）与一片一条的退役、谱系容器，删 → 回退 → 再删那一条合法流。

use super::*;

/// 删 → 回退 → 再删的合法历史：新池新建文件、建空 inode 2（txg 4），把它 unlink 到底，挂着回退到 txg 4
/// （谱系 (回退那次的 txg, 4)；墓碑树随现行那一版，头一条墓碑的死亡 txg 落在被回退抛弃的那一段里、不在谱系上），再把它 unlink 到底
/// （又一条 (2, 同一个对象出生代) 的墓碑，在谱系上）。不按谱系滤的写法在这里判 I-9.20 不重号红。
fn record_new_inode_unlinked_rolled_back_and_unlinked_again(tag: &str) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, 1);
    let before_the_intent = pool.output.clone();
    unlink_to_the_end(&mut pool, FIRST_FILE.0 + 1);
    roll_back_to(&mut pool, &before_the_intent);
    unlink_to_the_end(&mut pool, FIRST_FILE.0 + 1);
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "new-inode-unlinked-rolled-back-and-unlinked-again",
        versions,
    )
}

/// 退役记录 34 字节（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条）：被退役身份的出生树 0、打包记录类型 8、容器号 10、
/// 容器出生代 18、死亡 txg 26；退役容器是码 3、打包记录类型 3。记录宽、容器号与死亡 txg 的偏移取 `common_deletion_in_progress` 那一份。
const RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: usize = 18;

pub(super) const PACKED_RECORD_TYPE_RETIREMENT: u16 = 3;

/// 一片 inode 叶容器装 233 条（`singlefs_format::INODE_LEAF_RECORDS`）：第一个文件之外再建这么多个空 inode，叶就是满的一片加一条的一片。
pub(super) const INODE_RECORDS_PER_LEAF: u64 = 233;

fn is_a_retirement_container(bytes: &[u8]) -> bool {
    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_RETIREMENT
}

/// 退役容器第一条退役记录偏移 `field` 起的 8 字节改成 `change` 给的值。
fn change_the_first_retirement_record_field(
    bytes: &mut [u8],
    field: usize,
    change: impl Fn(u64) -> u64,
) {
    assert!(
        read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET) >= 1,
        "退役容器里至少一条退役记录"
    );
    let offset = PACKED_UNIT_RECORDS_START + field;
    let value = change(read_u64_at(bytes, offset));
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// 删完的流（[`record_new_pool_through_the_last_deletion_batch`]：最后一批删掉唯一那片 inode 叶，同一次发布开出退役树、写一条退役记录，
/// 死亡 txg = 最后一批那次的 txg 8）上改坏那片退役容器（两份副本）的一处、重封到根。
fn record_retirement_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_last_deletion_batch(tag, name);
    let targets = last_unit_copies(&flow, name, &is_a_retirement_container, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// I-9.32 与 I-9.35：删完的流上那条退役记录的死亡 txg 8 → 9、重封到根。死亡 txg 大于这一版的 txg 8：CPU 判 I-9.32 红；
/// 死于这一版的退役记录一条都没有，父版（txg 7）那片叶的身份没记：I-9.35 漏记红。
fn record_retirement_dying_after_its_version(tag: &str) -> RecordedFlow {
    record_retirement_with_a_resealed_field(tag, "retirement-dies-after-its-version", &|bytes| {
        change_the_first_retirement_record_field(
            bytes,
            RETIREMENT_RECORD_DEATH_TXG_OFFSET,
            |death_txg| death_txg + 1,
        );
    })
}

/// I-9.33：删完的流上那条退役记录原样再追加一条（记录数加一、声明长度加 34）、重封到根。两条都在谱系上、身份相同：CPU 判 I-9.33 红；
/// 死于这一版的身份集照旧是那一片，I-9.35 照旧成立。
fn record_retirement_written_twice(tag: &str) -> RecordedFlow {
    record_retirement_with_a_resealed_field(tag, "retirement-written-twice", &|bytes| {
        let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
        assert_eq!(record_count, 1, "删到底那一批只写一条退役记录");
        let first = PACKED_UNIT_RECORDS_START;
        bytes.copy_within(
            first..first + RETIREMENT_RECORD_BYTES,
            first + RETIREMENT_RECORD_BYTES,
        );
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
            .copy_from_slice(&2u16.to_le_bytes());
        let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
        bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
            &(declared_length + u16::try_from(RETIREMENT_RECORD_BYTES).expect("34")).to_le_bytes(),
        );
    })
}

/// I-9.35 漏记那一向：删完的流上那条退役记录的死亡 txg 8 → 7、重封到根。容器出生代 3 < 7 ≤ 8、死于 7 也在谱系上，I-9.32、I-9.33 照旧成立；
/// 死于这一版（8）的退役记录一条都没有，父版（txg 7）那片叶的身份没记：CPU 只红 I-9.35。
fn record_retirement_dying_at_the_parent_version(tag: &str) -> RecordedFlow {
    record_retirement_with_a_resealed_field(
        tag,
        "retirement-dies-at-the-parent-version",
        &|bytes| {
            change_the_first_retirement_record_field(
                bytes,
                RETIREMENT_RECORD_DEATH_TXG_OFFSET,
                |death_txg| death_txg - 1,
            );
        },
    )
}

/// I-9.34 与 I-9.35：新池新建文件之外再建 233 个空 inode（叶装成满的一片 [1..=233] 与一条的一片 [234]），unlink 2 号到底：最后一批左片吸收右片、
/// 右片身份消失，写一条 (右片身份, 这次的 txg) 的退役记录。把这条退役记录的容器号、容器出生代改成幸存左片的、重封到根：它在谱系上、指着这一版的活叶，
/// CPU 判 I-9.34 红；死于这一版的身份集成了左片、右片没记，I-9.35 漏记多记一起红。
fn record_absorption_with_the_retirement_record_naming_the_surviving_left_leaf(
    tag: &str,
) -> RecordedFlow {
    const NAME: &str = "absorption-retirement-record-names-the-surviving-left-leaf";
    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
    let identities_before: Vec<_> = pool
        .output
        .inode_leaf_containers
        .iter()
        .map(|container| container.contents.identity)
        .collect();
    let [left, right] = identities_before[..] else {
        panic!("234 条记录装成两片叶：{identities_before:?}");
    };
    unlink_to_the_end(&mut pool, FIRST_FILE.0 + 1);
    let retired: Vec<_> = pool
        .output
        .retirement_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| record.retired)
        .collect();
    assert_eq!(retired, [right], "左吸收右：退役树里一条右片的退役记录");
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    let mut flow = recorded_flow_of_the_pool(&pool, NAME, versions);
    let targets = last_unit_copies(&flow, NAME, &is_a_retirement_container, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &move |bytes| {
        change_the_first_retirement_record_field(
            bytes,
            RETIREMENT_RECORD_CONTAINER_OFFSET,
            |_container| left.container,
        );
        change_the_first_retirement_record_field(
            bytes,
            RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET,
            |_container_birth| left.container_birth.0,
        );
    });
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version",
        &record_retirement_dying_after_its_version("crash-judge-gpu-i932-retirement-death-txg"),
        &["I-9.32", "I-9.35"],
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_is_written_twice() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_is_written_twice",
        &record_retirement_written_twice("crash-judge-gpu-i933-retirement-written-twice"),
        "I-9.33",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_names_the_surviving_left_leaf() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_names_the_surviving_left_leaf",
        &record_absorption_with_the_retirement_record_naming_the_surviving_left_leaf(
            "crash-judge-gpu-i934-retirement-names-a-live-leaf",
        ),
        &["I-9.34", "I-9.35"],
    );
}

/// I-9.36 退役树那一形（`opus_probe_container_closed_early`）：
/// 新池新建文件之外再建 233 个空 inode（叶 [1..=233]、[234]），写入口装退役容器一片 1 条的旋钮，unlink 234 到底（最右那片删空、退役一条），
/// 再建 1 个（235 开新叶）、unlink 235 到底（又退役一条）：退役树两片容器各 1 条。旋钮下的合法镜像；CPU 判器走池级 checker 的旧入口
/// （当没装旋钮），不是最右的那片只装 1 条、不是 959，判 I-9.36 红；GPU 扫描内核只有没装旋钮那一支，同判。
pub(super) fn record_two_leaves_retired_with_one_retirement_record_per_container(
    tag: &str,
) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
    let capacities = ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(1)
                    .expect("1..=959"),
            ),
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    let rightmost_leaf_inode = FIRST_FILE.0 + INODE_RECORDS_PER_LEAF;
    unlink_an_empty_inode_to_the_end_under_the_capacities(
        &mut pool,
        rightmost_leaf_inode,
        capacities,
    );
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    unlink_an_empty_inode_to_the_end_under_the_capacities(
        &mut pool,
        rightmost_leaf_inode + 1,
        capacities,
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "two-leaves-retired-one-retirement-record-per-container",
        versions,
    )
}

/// I-9.36 谱系树那一形：新池新建文件、建 1 个空 inode（txg 4），分配器装上谱系容器一片 1 条的旋钮（回退的发布路径照分配器上的开关取容量），
/// 挂着回退到第一个文件那一版（txg 3）两次：谱系树两片容器各 1 条。旋钮下的合法镜像；CPU 判器走旧入口判 I-9.36 红，GPU 同判。
pub(super) fn record_two_rollbacks_with_one_lineage_record_per_container(
    tag: &str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    pool.allocator
        .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
            batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
            tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
            lineage_container_capacity: LineageContainerCapacity::CappedForTests(
                singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(1)
                    .expect("1..=1359"),
            ),
            retirement_container_capacity:
                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
            container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
        });
    roll_back_to(&mut pool, &first_file_version);
    roll_back_to(&mut pool, &first_file_version);
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "two-rollbacks-one-lineage-record-per-container",
        versions,
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_container_closes_before_it_is_full() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_container_closes_before_it_is_full",
        &record_two_leaves_retired_with_one_retirement_record_per_container(
            "crash-judge-gpu-i936-retirement-container-closed-early",
        ),
        "I-9.36",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full",
        &record_two_rollbacks_with_one_lineage_record_per_container(
            "crash-judge-gpu-i936-lineage-container-closed-early",
        ),
        "I-9.36",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version",
        &record_retirement_dying_at_the_parent_version(
            "crash-judge-gpu-i935-retirement-death-txg-at-the-parent",
        ),
        "I-9.35",
    );
}

/// 删 → 回退 → 再删的每个状态上 GPU 与 CPU 整份相同；CPU 池级 checker 在这条流上判过 I-9.18、I-9.19、I-9.20、I-9.24、I-9.29 而成立，
/// 墓碑与谱系那九条一个状态都不红（谱系外那条旧墓碑不进 I-9.20 不重号、不进 I-9.29 的表）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again() {
    let flow = record_new_inode_unlinked_rolled_back_and_unlinked_again(
        "crash-judge-gpu-unlink-rolled-back-and-unlinked-again",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-9.18", "I-9.19", "I-9.20", "I-9.24", "I-9.29"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "删 → 回退 → 再删的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
    }
    for invariant in TOMBSTONE_AND_LINEAGE_INVARIANTS {
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "删 → 回退 → 再删的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            comparison.cpu_checker_red_invariants
        );
    }
}

/// 墓碑树、谱系树条目的类型段与孩子的真实类不符时，原来的读法跟着连带红的那几条（攻方 N4 那几形，
/// `research/prompts/m3-batch22-code-r1-opus-output.md` 第一节）：按类型段定的长度读孩子读不出（I-2.1），记走读失败让最新根走不完
/// （I-7.2、I-4.8、I-7.4）。
pub(super) const INVARIANTS_A_WALK_FAILURE_REDDENS: [&str; 4] =
    ["I-2.1", "I-4.8", "I-7.2", "I-7.4"];
