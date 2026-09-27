//! 实审 A1b Q1（主 agent 2026-09-27 定）：为空间不够推的抬 F，自己那一串空发布也取不到落点（预演里被 `PlacementRefused` 拒），
//! 仍算空间不够——D16（发布语义） 已定项 1「准入」那一行：准入放行而落点取不到时先推空发布抬 F 再判，做满仍不够才报 ENOSPC；
//! 抬 F 自己推不出空间就是推满仍不够。实审 A1 把抬 F 的错一律原样往上交之后，这一格被报成「抬 F 报错」：会话交回
//! `UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`、可写挂载返回 `MountError`（写行与暖机已落盘）。
//! 改成：抬 F 被空间不够那两种拒（取不到落点、空间准入不够）走 `FloorRaiseStop` 的一个成员——会话报空间不够，
//! 可写挂载走 C565（挂载处推满仍不够怎么收尾没定） 那一格（挂载照样做成，D2（RAID 条带策略） 已定项 13 那一格的例外只管空间不够）；
//! 块设备错与别的错照旧原样交回（`a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is.rs`）。
//!
//! 场景：两块单元区 256 槽的小盘，mkfs 同一个进程那条会话把空间准入关掉（只供测试的开关），落点是唯一的墙；连着覆盖写，
//! 单元区被根环里各版的账占满之后，推的抬 F 那一串在预演里就取不到落点。

mod common_admission;

use common_admission::{start_plain, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PlacementRefusal;
use singlefs_core::mount::{FloorRaiseStop, MountError, MountSpaceAdmission};
use singlefs_core::mounted_session::UserChangeRefused;
use singlefs_core::transaction::PublishError;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf256Slots;

/// 第一个文件之后，mkfs 同一个进程那条会话关掉空间准入，覆盖写 `overwrites` 次（都做成）。
fn pool_after_overwrites_with_the_space_admission_switched_off(
    overwrites: usize,
) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool = start_plain(DEVICE_WIDTH);
    pool.session
        .as_mut()
        .expect("第一个文件之后这个进程的会话开着")
        .allocator
        .set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    for index in 1..=overwrites {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {index} 次覆盖写做成：{refusal:?}"));
    }
    pool
}

/// 覆盖写 18 次之后第 19 次：用户那次发布取不到落点，会话推抬 F，那一串两次空发布在预演里第 2 次也取不到落点。
/// 会话报空间不够（`NoSpaceAfterRaisingTheFloor`），最后一次被拒是用户那次的落点拒绝、一串都没推成；
/// 这一次一个字节都没写（用户那次在落盘之前被拒，抬 F 那一串在预演里被拒），会话的现行版本不动。
#[test]
fn a_floor_raise_pushed_by_the_session_whose_own_publishes_find_no_slot_reports_no_space() {
    let mut pool = pool_after_overwrites_with_the_space_admission_switched_off(18);
    let image_before = pool.image();
    let root_before = *pool.session().current.root();
    let refusal = pool
        .overwrite(OVERWRITE_BYTES)
        .expect_err("第 19 次覆盖写：单元区满了，推抬 F 也推不出空间");
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
                refused_publish_in_the_sequence: 2,
                publishes_in_the_sequence: 2,
                cause: PublishError::PlacementRefused {
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                    ..
                },
            }
        ),
        "抬 F 那一串两次空发布在预演里第 2 次取不到落点：{raise_refusal:?}"
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

/// 覆盖写 17 次之后崩了再挂（空间准入判着）：取号之前不够，写行与暖机之后推抬 F，那一串三次空发布在预演里第 1 次就取不到落点。
/// 挂载照样做成（C565 那一格：推满仍不够，`StillShortAfterTheFloorRaises`），实例已取、一串都没推成。
#[test]
fn a_mount_whose_floor_raise_after_the_row_publish_finds_no_slot_is_still_made_and_still_short() {
    let mut pool = pool_after_overwrites_with_the_space_admission_switched_off(17);
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
                publishes_in_the_sequence: 3,
                cause: PublishError::PlacementRefused {
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                    ..
                },
            }
        ),
        "抬 F 那一串三次空发布在预演里第 1 次就取不到落点：{raise_refusal:?}"
    );
}
