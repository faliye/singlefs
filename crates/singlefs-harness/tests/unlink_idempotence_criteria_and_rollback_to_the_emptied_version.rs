//! unlink 的幂等判据（D8（核心索引结构） 已定项 16 第 7 条）在发布层的三格：① 计划相等（从两批之间的镜像重挂载续做，
//! 续做的每一次发布与一路做下来的那几次逻辑内容逐次相同）、③ 一个写点截断之后重开续做到底（终态逻辑内容与不崩溃的终态相同、
//! 崩溃态与终态池级 checker 全绿；全量逐写点枚举归 checker 档）、④ 每一版的记账标签 4、6 等于从这一版镜像现算的值；
//! 另加回退到删空的一版（D23（journal 的角色与格式） 已定项 14 候选集「带文件（那一版写过文件，不是还没写过文件的那一版）」：
//! 删空的那一版写过文件，照样是候选）。三个单元的第一个文件、只供测试的批大小 1：意图 → 留 2 → 留 1 → 最后一批。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs"]
mod scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version;

use common::{disk_snapshot, memory_pool_of_sparse_devices, parameters};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::{
    accounting_value, logical_content_of, mount_a_copy_of_the_image_with_batches_of_one,
    pool_with_a_three_unit_file, publish_the_intent_in_process, publish_the_next_step_in_process,
    LogicalContent,
};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::records::{
    STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_PENDING_DELETE_BYTES,
};
use singlefs_core::recovery::{rebuild_version, RebuiltVersion};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

fn rebuilt_version_of_trees(
    image: &MemoryPool,
    root: &singlefs_core::root_record::RootRecord,
    record_standing: &singlefs_core::journal::JournalRecord,
) -> TransactionOutput {
    match rebuild_version(image, root, Some(record_standing.clone()))
        .unwrap_or_else(|failure| panic!("txg {:?} 从盘上重建：{failure:?}", root.checkpoint_txg))
    {
        RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version) => version,
        RebuiltVersion::WithoutFile => panic!("txg {:?} 这一版有树", root.checkpoint_txg),
    }
}

/// ①：两批之间（意图、留 2 之后）的镜像重挂载，挂载续做的两次发布（留 1、最后一批）与一路做下来的那两次逻辑内容逐次相同——
/// 续做的计划只由现行那一版定，从盘上重建的那一版与进程里那一版算出同一个计划。
#[test]
fn the_publishes_continued_by_a_remount_between_two_batches_equal_the_rest_of_the_uninterrupted_run(
) {
    let mut pool = pool_with_a_three_unit_file("idempotence-plan-equal-on-remount");
    publish_the_intent_in_process(&mut pool);
    publish_the_next_step_in_process(&mut pool);
    let image_between_two_batches = pool.memory_pool();
    let uninterrupted_rest = [
        publish_the_next_step_in_process(&mut pool),
        publish_the_next_step_in_process(&mut pool),
    ];
    let stream = SharedStream::new();
    let (mounted, devices) =
        mount_a_copy_of_the_image_with_batches_of_one(&image_between_two_batches, &stream);
    let last_row_or_warm_up_txg = mounted
        .output
        .warm_up_publishes
        .last()
        .unwrap_or(&mounted.output.row_publish)
        .root()
        .checkpoint_txg;
    let record_standing = mounted
        .current
        .version_of_trees()
        .expect("续做之后有树")
        .record
        .clone();
    let continued_image = memory_pool_of_sparse_devices(&devices);
    let mut continuation_roots: Vec<singlefs_core::root_record::RootRecord> =
        disk_snapshot(&continued_image, &stream)
            .readable_roots
            .into_iter()
            .filter(|root| root.checkpoint_txg > last_row_or_warm_up_txg)
            .collect();
    continuation_roots.sort_by_key(|root| root.checkpoint_txg);
    let continued: Vec<LogicalContent> = continuation_roots
        .iter()
        .map(|root| {
            logical_content_of(&rebuilt_version_of_trees(
                &continued_image,
                root,
                &record_standing,
            ))
        })
        .collect();
    assert_eq!(
        continued,
        uninterrupted_rest
            .iter()
            .map(logical_content_of)
            .collect::<Vec<LogicalContent>>(),
        "挂载续做两次（留 1、最后一批），逐次与一路做下来的那两次逻辑内容相同"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to_the_uninterrupted_end_state,
    );
}

/// ④：删除前、意图、两批、最后一批每一版记账行里的标签 4 与 6，等于从这一版镜像现算的值：
/// 标签 4 恒 0；标签 6 = 2 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)，n_live 数重建出来的 inode 树里
/// nlink ≥ 1 的记录、n_记录 数它的全部记录（含 nlink = 0 的，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条），
/// H_墓碑、H_退役 从墓碑树、退役树根单元的码 2 头现读（零根时 0）。删除前 n_live = n_记录 = 1（196760），意图起到最后一批之前 n_live = 0、
/// n_记录 = 1（196678），最后一批之后 n_记录 = 0（196608）。
#[test]
fn tags_four_and_six_of_every_deletion_version_equal_the_values_recomputed_from_its_image() {
    let mut pool = pool_with_a_three_unit_file("idempotence-tags-four-and-six");
    let mut published = vec![
        pool.output.clone(),
        publish_the_intent_in_process(&mut pool),
    ];
    for _ in 0..3 {
        published.push(publish_the_next_step_in_process(&mut pool));
    }
    let image = pool.memory_pool();
    let recorded_and_recomputed: Vec<((u64, u64), (u64, u64))> = published
        .iter()
        .map(|version| {
            let rebuilt = rebuilt_version_of_trees(&image, &version.root, &version.record);
            let live_objects = u64::try_from(
                rebuilt
                    .inode_leaf_containers
                    .iter()
                    .flat_map(|container| container.contents.records.iter())
                    .filter(|record| record.nlink >= 1)
                    .count(),
            )
            .expect("记录数");
            // n_记录 数 inode 树里全部记录、含 nlink = 0 的（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）。
            let records = u64::try_from(
                rebuilt
                    .inode_leaf_containers
                    .iter()
                    .map(|container| container.contents.records.len())
                    .sum::<usize>(),
            )
            .expect("记录数");
            let internal_levels_below_the_root = |root_role: TransactionUnit, what: &str| {
                rebuilt
                    .units
                    .iter()
                    .find(|unit| unit.identity == root_role)
                    .map_or(0, |root| {
                        u64::from(
                            parse_index_node(&root.bytes)
                                .unwrap_or_else(|_| panic!("{what}根解得开"))
                                .level,
                        ) - 1
                    })
            };
            let tombstone_tree_internal_levels =
                internal_levels_below_the_root(TransactionUnit::TombstoneRoot, "墓碑树");
            let retirement_tree_internal_levels =
                internal_levels_below_the_root(TransactionUnit::RetirementRoot, "退役树");
            (
                (
                    accounting_value(version, STATISTIC_PENDING_DELETE_BYTES),
                    accounting_value(version, STATISTIC_COMMITTED_RESERVATION_BYTES),
                ),
                (
                    0,
                    2 * ((2
                        + tombstone_tree_internal_levels
                        + 1
                        + 2
                        + retirement_tree_internal_levels
                        + 1)
                        * 16384
                        + 41 * live_objects
                        + 35 * records),
                ),
            )
        })
        .collect();
    assert_eq!(
        recorded_and_recomputed
            .iter()
            .map(|(recorded, _)| *recorded)
            .collect::<Vec<(u64, u64)>>(),
        recorded_and_recomputed
            .iter()
            .map(|(_, recomputed)| *recomputed)
            .collect::<Vec<(u64, u64)>>(),
        "每一版记账行里的标签 4、6 等于从镜像现算的"
    );
    assert_eq!(
        recorded_and_recomputed
            .iter()
            .map(|((_, recorded_tag_six), _)| *recorded_tag_six)
            .collect::<Vec<u64>>(),
        vec![196_760, 196_678, 196_678, 196_678, 196_608],
        "删除前一个活对象、一条记录；从意图那次起第一个文件 nlink = 0、不再算活对象，记录还在（n_记录 含 nlink = 0 的）；最后一批记录删掉。\
         墓碑 3 槽与退役 3 槽常驻：2 × 6 × 16384 = 196608，活对象每个 2 × 41、记录每条 2 × 35"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file()
{
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_unlink_idempotence_criteria_and_rollback_to_the_emptied_version::rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_trees_and_no_file,
    );
}

/// 回退那次发布的标签 6 按新根现算（D23（journal 的角色与格式） 已定项 14「准入与记账」那一行）：新根的 inode 树与 `deleted_inodes`
/// 取 R_old 的，n_live = R_old 那一版 inode 树的记录数减 `deleted_inodes` 的 key 数，n_记录 = R_old 那一版 inode 树的全部记录数。
/// 删完唯一的文件（标签 6 = 2 × (6 × 16384) = 196608，n_live = n_记录 = 0；墓碑 3 槽与退役 3 槽常驻，判决
/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）之后回退到意图之前那一版（第一个文件活着）：
/// 新根的标签 6 = 2 × (98304 + 41 × 1 + 35 × 1) = 196760，被带回来的对象的墓碑与退役份额照抬。
#[test]
fn rolling_back_to_the_version_before_the_intent_recomputes_tag_six_from_the_rollback_targets_inode_tree(
) {
    let mut pool = pool_with_a_three_unit_file("rollback-recomputes-tag-six");
    let before_the_intent = pool.output.clone();
    publish_the_intent_in_process(&mut pool);
    for _ in 0..3 {
        publish_the_next_step_in_process(&mut pool);
    }
    let tag_six_before_the_rollback =
        accounting_value(&pool.output, STATISTIC_COMMITTED_RESERVATION_BYTES);
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: before_the_intent.root.instance,
            checkpoint_txg: before_the_intent.root.checkpoint_txg,
        },
    )
    .expect("回退到意图之前那一版");
    assert_eq!(
        (
            tag_six_before_the_rollback,
            accounting_value(&pool.output, STATISTIC_COMMITTED_RESERVATION_BYTES),
        ),
        (196_608, 196_760),
        "删完时没有活对象也没有记录；回退新根按 R_old 的 inode 树现算，第一个文件又是一个活对象、一条记录"
    );
}
