//! 为空间不够推的抬 F，自己那一串空发布也取不到落点（预演里被 `PlacementRefused` 拒），
//! 仍算空间不够——D16（发布语义） 已定项 1「准入」那一行：准入放行而落点取不到时先推空发布抬 F 再判，做满仍不够才报 ENOSPC；
//! 抬 F 自己推不出空间就是推满仍不够。实审 A1 把抬 F 的错一律原样往上交之后，这一格被报成「抬 F 报错」：会话交回
//! `UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`、可写挂载返回 `MountError`（写行与暖机已落盘）。
//! 改成：抬 F 被空间不够那两种拒（取不到落点、空间准入不够）走 `FloorRaiseStop` 的一个成员——会话报空间不够，
//! 可写挂载走 C565（挂载处推满仍不够怎么收尾没定） 那一格（挂载照样做成，D2（RAID 条带策略） 已定项 13 那一格的例外只管空间不够）；
//! 块设备错与别的错照旧原样交回（`a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is.rs`）。
//!
//! 场景：两块单元区 256 槽、不为环窗口补足的窄盘（用户 2026-10-02 定 Q-T「甲」：关准入的那几条单开不补足的窄盘，第一个文件之前就关准入；
//! 补足之后稳态占用到不了单元区墙），mkfs 同一个进程那条会话在第一个文件之前就把空间准入关掉（只供测试的开关），落点是唯一的墙；
//! 连着覆盖写，单元区被根环里各版的账占满之后，推的抬 F 那一串在预演里就取不到落点。

mod common_admission;

use common_admission::{
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::allocator::PlacementRefusal;
use singlefs_core::mount::{FloorRaiseStop, MountError, MountSpaceAdmission};
use singlefs_core::mounted_session::UserChangeRefused;
use singlefs_core::transaction::PublishError;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

/// 参数取 256 槽那一档（6 MiB 环）。
const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf256Slots;

/// 不为环窗口补足的单元区槽数：四个 64 槽的聚簇段。
const UNIT_AREA_SLOTS_WITHOUT_THE_RING_WINDOW_ALLOWANCE: u64 = 256;

/// 第一个文件之前就关掉空间准入，第一个文件之后在 mkfs 同一个进程那条会话里覆盖写 `overwrites` 次（都做成）。
fn pool_after_overwrites_with_the_space_admission_switched_off(
    overwrites: usize,
) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            DEVICE_WIDTH,
            UNIT_AREA_SLOTS_WITHOUT_THE_RING_WINDOW_ALLOWANCE,
        );
    for index in 1..=overwrites {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {index} 次覆盖写做成：{refusal:?}"));
    }
    pool
}

/// 覆盖写 19 次之后第 20 次：用户那次发布取不到落点，会话推抬 F，那一串三次空发布在预演里第 1 次就取不到落点
/// （A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是「18 次之后第 19 次、两次空发布里第 2 次」：分配记录树按绝对槽号按位置寻址，
/// 单元区挪了位置，每次发布重写的节点跟着变，撞墙的步数与那一串的次数都变；草稿探针在新几何上逐次覆盖写现跑的数）。
/// 会话报空间不够（`NoSpaceAfterRaisingTheFloor`），最后一次被拒是用户那次的落点拒绝、一串都没推成；
/// 这一次一个字节都没写（用户那次在落盘之前被拒，抬 F 那一串在预演里被拒），会话的现行版本不动。
#[test]
fn a_floor_raise_pushed_by_the_session_whose_own_publishes_find_no_slot_reports_no_space() {
    let mut pool = pool_after_overwrites_with_the_space_admission_switched_off(19);
    let image_before = pool.image();
    let root_before = *pool.session().current.root();
    let refusal = pool
        .overwrite(OVERWRITE_BYTES)
        .expect_err("第 20 次覆盖写：单元区满了，推抬 F 也推不出空间");
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("抬 F 自己也取不到落点，该报空间不够 NoSpaceAfterRaisingTheFloor，实际 {refusal:?}")
    };
    assert!(
        matches!(
            no_space.last_refusal,
            PublishError::PlacementRefused {
                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                ..
            }
        ),
        "最后一次被拒是用户那次发布的落点拒绝：{:?}",
        no_space.last_refusal
    );
    assert!(
        no_space.floor_raises.is_empty()
            && no_space.refusals_that_pushed_the_floor_raises.is_empty(),
        "一串都没推成：{} 串",
        no_space.floor_raises.len()
    );
    let FloorRaiseStop::FloorRaiseRefusedForSpace {
        refusal: raise_refusal,
    } = &no_space.stop
    else {
        panic!(
            "不再推的原因是抬 F 那一串自己也取不到落点，实际 {:?}",
            no_space.stop
        )
    };
    assert!(
        matches!(
            **raise_refusal,
            MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                refused_publish_in_the_sequence: 1,
                publishes_in_the_sequence: 3,
                cause: PublishError::PlacementRefused {
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                    ..
                },
            }
        ),
        "抬 F 那一串三次空发布在预演里第 1 次就取不到落点：{raise_refusal:?}"
    );
    assert_eq!(
        pool.image(),
        image_before,
        "用户那次在落盘之前被拒、抬 F 那一串在预演里被拒：两块盘逐字节不变"
    );
    assert_eq!(
        *pool.session().current.root(),
        root_before,
        "会话的现行版本不动"
    );
}

/// 覆盖写 18 次之后崩了再挂（空间准入判着）：取号之前不够，写行与暖机之后推抬 F，那一串两次空发布在预演里第 1 次就取不到落点。
/// （A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是「17 次、三次空发布里第 1 次」；新几何下 17 次那一串是三次里第 3 次才取不到，
/// 草稿探针在 16–18 次上现跑，取 18 次，留住「第 1 次就取不到」那一形。）
/// 挂载照样做成（C565 那一格：推满仍不够，`StillShortAfterTheFloorRaises`），实例已取、一串都没推成。
#[test]
fn a_mount_whose_floor_raise_after_the_row_publish_finds_no_slot_is_still_made_and_still_short() {
    let mut pool = pool_after_overwrites_with_the_space_admission_switched_off(18);
    let output = pool.crash_and_mount_writable().unwrap_or_else(|error| {
        panic!("抬 F 自己也取不到落点是推满仍不够：挂载照样做成（C565 那一格），实际 {error:?}")
    });
    assert_eq!(output.instance, InstanceGeneration(2), "这次挂载取号 2");
    let MountSpaceAdmission::StillShortAfterTheFloorRaises {
        floor_raises, stop, ..
    } = &output.space_admission
    else {
        panic!(
            "取号之前不够、推的抬 F 推不出空间：推满仍不够，实际 {:?}",
            output.space_admission
        )
    };
    assert!(
        floor_raises.is_empty(),
        "第一串就在预演里被拒：一串都没推成（{} 串）",
        floor_raises.len()
    );
    let FloorRaiseStop::FloorRaiseRefusedForSpace {
        refusal: raise_refusal,
    } = stop
    else {
        panic!("不再推的原因是抬 F 那一串自己也取不到落点，实际 {stop:?}")
    };
    assert!(
        matches!(
            **raise_refusal,
            MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                refused_publish_in_the_sequence: 1,
                publishes_in_the_sequence: 2,
                cause: PublishError::PlacementRefused {
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                    ..
                },
            }
        ),
        "抬 F 那一串两次空发布在预演里第 1 次就取不到落点：{raise_refusal:?}"
    );
}

/// `deleted_inodes` 根装不下（队列多于一个码 2 根装得下的 2031 个号，D8（核心索引结构） 已定项 16 第 6 条「多于一个根装得下那一支拒成类型化错误、
/// 不是空间不够」）那一种拒绝归「不是空间不够」那一组：会话与挂载续做都不为它推抬 F，原样交回（代码轮 U3）。
/// 拒绝取自构造根条目的那个函数对 2032 个号真交回的错，不手拼。
/// 判别力：把它归进空间不够那两种（推抬 F 再试）时这一判红。
#[test]
fn a_deleted_inodes_queue_longer_than_one_root_holds_is_not_short_of_space_and_pushes_no_floor_raise(
) {
    use singlefs_core::address::InodeNumber;
    use singlefs_core::deleted_inodes::{
        deleted_inodes_one_root_node_holds, deleted_inodes_root_entries,
    };
    let one_too_many: Vec<InodeNumber> =
        (1..=u64::try_from(deleted_inodes_one_root_node_holds()).expect("一个根装得下的号数") + 1)
            .map(InodeNumber)
            .collect();
    let refusal = deleted_inodes_root_entries(&one_too_many)
        .expect_err("比一个根装得下的多一个号：拒成类型化错误");
    assert!(
        !singlefs_core::mounted_session::refusal_is_short_of_space(
            &PublishError::DeletedInodesQueueRefused(refusal)
        ),
        "deleted_inodes 装不下不是空间不够：不推抬 F（{refusal:?}）"
    );
}
