//! 整理搬根改变 D16（发布语义）「非空」状态（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J6；用户 2026-10-13 定
//! 「照字节比，搬根算非空」，调度记录 `records/2026-10-13-里程碑四预开发.md` 丙 4 交回那一条）：「非空」比的是树表条目里用户可见树根指针的
//! 盘上字节（`mount::user_visible_trees_changed`，抬 F 的上限与「4 个不同状态」按它数）。整理的一批搬了 inode 根或 extent 根时，树表照映射位置
//! 重写那一条的根指针（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「父节点重写时写映射位置」），字节变了，这一批算非空；
//! 只搬别的单元的批、签发与删意图那几次根指针照抄，不算非空。
//! 池按 `common::build_pool` 起（第一个文件那一次发布的提交内生块在段 50240，inode 根与 extent 根都在里面，重开之后的写行不重写它们），
//! 可写挂载，强制选段 50240、一批一个，显式入口推到停。
mod common;

use common::{build_pool, parameters};
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::compaction_plan::{
    CompactionBatchSize, CompactionBatchUnits, CompactionSwitches,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mount::{mount_writable, user_visible_trees_changed};
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession};
use singlefs_core::recovery::user_visible_tree_root_pointers;
use singlefs_core::transaction::{RelocatedUnitHolder, TransactionOutput, TransactionUnit};
use singlefs_harness::memory_pool::MemoryPool;

/// 第一个文件那次发布的提交内生块落在这一段（默认几何，D3（空间分配） 已定项 10 ⑤ 的开放段 [50240, 50304)）。
const SEGMENT_OF_THE_FIRST_PUBLISH: u64 = 50240;

/// 这次发布搬的单元里有没有用户可见的两棵树的根（inode 根、extent 根）。
fn relocates_a_user_visible_root(version: &TransactionOutput) -> bool {
    version
        .units_relocated_by_this_publish
        .iter()
        .any(|relocated| match relocated.holder {
            RelocatedUnitHolder::RoleOfThisVersion(role) => {
                matches!(
                    role,
                    TransactionUnit::InodeRoot | TransactionUnit::ExtentRoot
                )
            }
            RelocatedUnitHolder::KeptForTheSnapshots => false,
        })
}

/// 一版的根按「非空」的比法与它前一版比：两条根树表里用户可见树的根指针盘上字节不同就算非空。
fn is_non_empty_against(
    image: &MemoryPool,
    version: &TransactionOutput,
    previous: &TransactionOutput,
) -> bool {
    let pointers = user_visible_tree_root_pointers(image, &version.root)
        .expect("这一版的树表读得出：整理一趟的发布数远小于根环一圈，树表单元还没被复用");
    let previous_pointers = user_visible_tree_root_pointers(image, &previous.root)
        .expect("前一版的树表读得出：整理一趟的发布数远小于根环一圈，树表单元还没被复用");
    user_visible_trees_changed(&pointers, &previous_pointers)
}

/// 强制选段 50240、一批一个推到停：搬了 inode 根或 extent 根的那两批各算非空，签发、只搬别的单元的批与删意图那几次都不算。
#[test]
fn a_compaction_batch_relocating_a_user_visible_root_is_non_empty_and_the_other_compaction_publishes_are_not(
) {
    let mut pool = build_pool("compaction-batch-relocating-a-root-is-non-empty");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH),
    )
    .expect("64 的倍数");
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(1).expect("至少 1"),
            ),
            forced_region: Some(region),
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
    let before_the_run = session
        .current
        .version_of_trees()
        .expect("有树的一版")
        .clone();
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(run.outcome, CompactionRunOutcome::Stopped(_)),
        "这一趟照常停，不是一次发布没发成：{:?}",
        run.outcome
    );
    let image = pool.memory_pool();
    let mut previous = &before_the_run;
    let mut publishes_relocating_a_root = 0;
    for version in &run.versions_published {
        let relocates_a_root = relocates_a_user_visible_root(version);
        assert_eq!(
            is_non_empty_against(&image, version, previous),
            relocates_a_root,
            "txg {}：搬了根就算非空、没搬根就不算（这次搬的 {:?}）",
            version.root.checkpoint_txg.0,
            version.units_relocated_by_this_publish
        );
        if relocates_a_root {
            publishes_relocating_a_root += 1;
        }
        previous = version;
    }
    assert_eq!(
        publishes_relocating_a_root,
        2,
        "inode 根与 extent 根都在段 50240 里，一批一个：两批各搬一个根（这一趟发了 {} 次）",
        run.versions_published.len()
    );
}
