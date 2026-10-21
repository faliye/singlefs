//! D8（核心索引结构） 已定项 16 第 6 条「会话走空时删除类发布被落点那一判以空间拒，推一串抬 F、回收后再试」对第一个文件之外的号
//! （代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 2 条；正推腿 `m3-close-w5-code-r1-sonnet-output.md` W1、W5 两节的缺口 E：
//! 多文件用例里一次空间拒都没有）。盘面照 `admission_raises_the_floor_before_refusing.rs` 里删第一个文件、最后一批被拒那一条的造法：两块单元区 384 槽的小盘，
//! 第一个文件之后崩了再挂，这次挂载里建 inode 2、给它写一个数据单元，覆盖写第一个文件 12 次（换下的单元进 defer），再把每块盘没挡的槽标成已分配、
//! 只留 30 槽（取法见 [`OVERWRITES_BEFORE_THE_UNLINK_WHOSE_DELETION_BATCH_IS_REFUSED`]）。经会话 unlink inode 2：意图那次放行、走空里的删除批被落得下那一判拒，会话推一串抬 F 的空发布、回收之后接着走空，unlink 做成。
//! 池级 checker 不判（同那一份：标成已分配的槽只在分配器的位图上，记账与遍历按造法就对不上）。
mod common_admission;

use common_admission::{content_of, start_plain, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{InodeNumber, SlotNumber};
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::{PoolVersion, PublishError};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

/// 覆盖写第一个文件几次、每块盘留几个没挡的槽：草稿探针（这一批实现员副本上，覆盖写 8–13 次 × 留 14–64 槽每隔 2 槽现跑）量到覆盖写 9、10、12、13 次时
/// 28–34 槽是「意图放行、走空里的删除批被落得下那一判拒、推一串抬 F 之后做成」那一形（14–22 槽多半是意图那次被拒、推过之后做成，24–26 槽推的那一串
/// 在预演里也落不下、交回空间不够，36 槽起一次都不拒；覆盖写 8、11 次时这一形只在 34 槽）。取 12 次、30 槽。
const OVERWRITES_BEFORE_THE_UNLINK_WHOSE_DELETION_BATCH_IS_REFUSED: usize = 12;
const UNBLOCKED_SLOTS_LEFT_FOR_THE_DELETION_BATCH_TO_BE_REFUSED: usize = 30;
/// 被删的那个第一个文件之外的号：第一个文件之后建的第一个。
const THE_OTHER_INODE: u64 = 2;

/// 第一个文件之后崩了再挂、建 inode 2 并给它写一个数据单元、覆盖写第一个文件 `overwrites` 次，再把每块盘没挡的槽标成已分配、只留 `unblocked_slots_left` 槽。
fn pool_with_another_file_ready_to_unlink_with_few_unblocked_slots(
    overwrites: usize,
    unblocked_slots_left: usize,
) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    pool.create_one_inode().expect("建 inode 2");
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let content_of_the_other_inode = content_of(OVERWRITE_BYTES, write_time_seconds);
    pool.publish(
        UserChange::OverwriteOfAnInode {
            inode: InodeNumber(THE_OTHER_INODE),
            content: &content_of_the_other_inode,
            write_time_seconds,
        },
        None,
    )
    .unwrap_or_else(|refusal| panic!("给 inode 2 写一个数据单元：{refusal:?}"));
    for overwrite_index in 1..=overwrites {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("第 {overwrite_index} 次覆盖写第一个文件做成：{refusal:?}")
        });
    }
    let session = pool.session.as_mut().expect("这次挂载的会话");
    let slots_left: Vec<u64> = {
        let device_map = &session.allocator.devices[0];
        let first_slot = device_map.unit_area_start().slot().0;
        (first_slot..first_slot + device_map.unit_area_slots())
            .filter(|slot| device_map.is_free(SlotNumber(*slot)))
            .take(unblocked_slots_left)
            .collect()
    };
    for device_map in &mut session.allocator.devices {
        let first_slot = device_map.unit_area_start().slot().0;
        // 迭代上界是单元区槽数，跨轮只改这一槽的分配位。
        for slot in first_slot..first_slot + device_map.unit_area_slots() {
            if !slots_left.contains(&slot) && device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    pool
}

/// 会话里 unlink 第一个文件之外的 inode 2，走空时它的删除批被落得下那一判拒、推抬 F、回收之后接着走（用户 2026-10-02 定案 Q-V「推抬 F 再试」，
/// 对任一号同一条）：unlink 做成；推过抬 F，推它的都是空间不够那两种；推的那一串接在意图之后（它的第一次空发布的 txg 比 unlink 之前那一版至少大 2：
/// 意图那次占了一个）；F 抬上去了；现行那一版仍带文件（第一个文件活着），inode 2 的记录与单元都没了、队列走空。
/// 判别力：会话推过一串抬 F 之后不再接着走空、当成走空了交回（`crates/mutations.tsv` 以「会话走空推抬 F：」起头那一行）时 inode 2 留在队列里，这一条红。
#[test]
fn an_unlink_of_another_inode_whose_deletion_batch_finds_no_unblocked_slot_is_published_after_raising_the_floor_in_the_session(
) {
    let mut pool = pool_with_another_file_ready_to_unlink_with_few_unblocked_slots(
        OVERWRITES_BEFORE_THE_UNLINK_WHOSE_DELETION_BATCH_IS_REFUSED,
        UNBLOCKED_SLOTS_LEFT_FOR_THE_DELETION_BATCH_TO_BE_REFUSED,
    );
    let floor_before = pool.rollback_floor();
    let checkpoint_txg_before_the_unlink = pool.session().current.root().checkpoint_txg;
    let published = pool
        .publish(
            UserChange::Unlink {
                inode: InodeNumber(THE_OTHER_INODE),
                write_time_seconds: pool.write_time_seconds,
            },
            None,
        )
        .unwrap_or_else(|refusal| panic!("unlink inode 2 走空时推过抬 F 之后做成：{refusal:?}"));
    assert!(
        !published.refusals_that_pushed_the_floor_raises.is_empty()
            && published
                .refusals_that_pushed_the_floor_raises
                .iter()
                .all(|refusal| matches!(
                    refusal,
                    PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. }
                )),
        "推过抬 F，推它的是空间不够那两种：{:?}",
        published.refusals_that_pushed_the_floor_raises
    );
    let first_floor_raise_publish_txg = published.floor_raises[0]
        .publishes
        .first()
        .expect("推的那一串至少一次空发布")
        .root
        .checkpoint_txg;
    assert!(
        first_floor_raise_publish_txg.0 >= checkpoint_txg_before_the_unlink.0 + 2,
        "推的那一串接在意图那次之后（被拒的是走空里的删除批）：第一次空发布 {first_floor_raise_publish_txg:?}，unlink 之前 {checkpoint_txg_before_the_unlink:?}"
    );
    assert!(
        pool.rollback_floor() > floor_before,
        "F 从 {floor_before:?} 抬上去了"
    );
    let PoolVersion::WithFile(current) = &pool.session().current else {
        panic!("删完 inode 2 而第一个文件活着：现行那一版带文件");
    };
    assert!(
        current.inode_record.is_some()
            && current
                .inode_record_of(InodeNumber(THE_OTHER_INODE))
                .is_none()
            && current.files_other_than_the_first.is_empty()
            && current.deleted_inodes.is_empty(),
        "第一个文件活着、inode 2 的记录与单元都没了、队列走空"
    );
}
