//! 代码三方 m2-closeout-code-r2（判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节 Y2-a、Y2-b，第二节「四个入口再核两样」）：
//! 挂着之后收盘表的每个入口——会话发布、正常卸载、抬 F、管理员回退——在「可见」
//! 那一判之后再核两样（D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」）：
//! - 每块盘自证过的系统配置槽里记的本盘设备号等于盘表给它的身份（盘体对调的盘表，Y2-b）；
//! - 这块盘最新那份自证系统配置的 (实例代号, journal tail) 不落后于现行那一版末条记录的 jsn、落后就读现行那一版的单元，缺一份就拒
//!   （同池旧快照的盘，Y2-a；与可写挂载取号之前逐盘核的落后支同一判）。
//!
//! 任一不过都在任何写之前拒，两块盘逐字节不变；之后拿同一组盘可写挂载，照不经这些入口的对照一样被落后支拒。
//! 只剩一槽自证的盘（那一槽停在上一次轮换、单元都在）四个入口都不误拒（Y2-c）。
//!
//! 攻方用例（`research/prompts/m2-closeout-code-r2-opus-model/opus_r2_y2_session_and_entries_with_stale_or_swapped_devices.rs`）只打印、
//! 钉改之前的样子：旧快照经会话发布或卸载照常做成、之后可写挂载做成（实例 2）、池级 checker 红 I-2.1；盘体对调经会话照常发布。
//! 这里搬过来反过来钉，每形一条；攻方放开扫的 16 格（旧快照取自哪一刻 × 换哪块盘 × 之后会话发 1/2/3 次或卸载一次）单列一条。
//!
//! 起点同攻方：两块 4 GiB 内存稀疏盘上 mkfs、取号、暖机、第一个文件（txg 3），会话里覆盖写两次（txg 4、5）。
//!
//! C585（会话发布放进发布途中取下的旧快照；里程碑三二·2，用户定改法 C）：落后支改成「落后，并且缺现行那一版的单元，**或**现行那一版的根
//! 写在这块盘上而那个根环槽里不是它」才拒，四个入口与可写挂载取号之前的逐盘核同一判。攻方
//! （`research/prompts/m2-closeout-code-r3-opus-output.md`「X6-a」）把一次发布 P 途中（单元与记录已写、根槽 FUA 未写）取下的、P 的根
//! 住的那块盘换上来，会话覆盖写照常做成、池级 checker 红 I-3.1；抬 F、卸载、回退当时靠「已知根槽读坏」拒掉，不是落后支拒的。
//! 这里每个入口一条钉「在任何写之前拒成落后支缺根那一半、两块盘逐字节不变」；另钉两形不误拒：切在 P 的根槽 FUA 之后的那块盘
//! （根已落、轮换没写，等同崩溃状态）会话照常发布，切在根槽之前的那块盘上可写挂载照常施加 P 的记录挂上（P 的根不在根环里，不核缺根那一半）。
//!
//! 代码三方 m3-code-r1（判决 `research/prompts/m3-code-r1-main-verification.md` 第三节 H1–H3）补的几格：
//! - H1：切在 P 的根槽 FUA 之后的那块盘（落后、单元都在、根槽里是 P），缺根那一半第一次读那一槽碰上一次瞬时读错、重读读得出 P，
//!   会话发布与可写挂载都照常做成（攻方原型 `research/prompts/m3-code-r1-opus-model/proto/tests/z2_reread_one_transient_root_slot_read_error.rs`
//!   的甲那一形；去掉重读时两处都拒成落后支）。
//! - H2：P 的根住盘 1（txg 7，区域 1），与根环转过一圈、P 的根槽里是上一圈自证得过的另一条根（txg 27 那一槽里住着 txg 3）——
//!   「只查盘 0 的根槽」「根槽里自证得过就算是它」两种改错各让一格红。
//! - H3：同一块切在根槽之后的盘上可写挂载，所选那一版就是择到的根（写进了根环），那一槽在缺根那一半连着两次读错：
//!   拒成 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot`，拒在任何写之前、两块盘逐扇区不变。
//!
//! 代码三方 m3-code-r2（判决 `research/prompts/m3-code-r2-main-verification.md` 第三节 J1–J3）补的几格：
//! - J1：P 是 txg 8（区域 2；`region_devices` 是 [盘 0, 盘 1, 盘 0]，区域 2 归盘 0）。「把区域号当盘号」时区域 2 被当成盘表里没有的盘 2，
//!   盘 0 的缺根不核、会话照常发布。
//! - J2：H1 那一形（缺根那一半第一次读 P 的根槽读坏、重读读得出 P）在正常卸载、准入抬 F、管理员回退三个入口各一格；
//!   这三个入口不重读时拒成缺根。
//! - J3：根环转过一圈的那一形在这三个入口各两格（管理员回退取现行那一版的前一版做目标）：txg 27（区域 0、住盘 0，槽里是上一圈
//!   第一个文件那一版 txg 3）与 txg 25（区域 1、住盘 1，槽里是上一圈暖机那一版 txg 1）。这一形上缺根那一半是三个入口唯一的一道
//!   （没有「重读仍坏」兜底）：「根槽里自证得过就算是它」两种 txg 都红；「只查区域 0 那块盘」只在 txg 25 红——txg 27 的根就住在区域 0 那块盘上。
//!
//! 代码三方 m3-code-r3（判决 `research/prompts/m3-code-r3-main-verification.md` 第三节 L1、L2）补的几格：
//! - L1：H1 的两格与 J2 的三格各加一版「根槽第一次读回的字节坏一位」（读报成功、自证不过），重读读得出 P，五个入口都照常做成、
//!   注入真发生了 1 次（D16（发布语义） 已定项 1「读不出或自证不过就重读一次」的后一半）。「只在第一次读不出时才重读」（攻方 RU）五格都红。
//! - L2：根环转过一圈的那一形在正常卸载、准入抬 F、管理员回退各加 txg 26 一格（区域 2、槽 0，区域 2 归盘 0，槽里是上一圈暖机写的
//!   第二条根 txg 2）。这三个入口上「把区域号当盘号」（攻方 RGC）时区域 2 被当成盘表里没有的盘 2，缺根不核、入口照常做成。
mod common_admission;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/entries_after_mount_refuse_swapped_or_behind_devices.rs"]
mod scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices;

use common_admission::{start_plain, OVERWRITE_BYTES};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::{
    copies_of, copy_of_device, describe, devices_from, go_through, go_through_rolling_back_to,
    overwrite_after, pool_with_a_snapshot_after_the_first_file,
    root_device_of_the_publish_taken_mid_publish, with_a_fault_on_reads_of_the_publish_root_slot,
    writable_mount_outcome_on_a_copy_of, EntryAfterTheMount, EntryRefusal,
    RollbackTargetOfTheEntry, RootDeviceOfThePublishTakenMidPublish, SparseDevices,
    WhereThePublishIsCut, DEVICE_WIDTH, PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{
    mount_writable, CallerInputsDisagreeingWithTheDisk, DeviceBehindTheCurrentVersion,
    DeviceTableDisagreement, DeviceWithoutTheSelectedVersion, MountError,
    OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, RingSlotReadingOtherThanTheVersionRoot,
    RollbackError, RollbackTarget, SelectedVersionLackingOnDevice, VersionLackingOnADeviceBehindIt,
    VersionRootNotInItsRingSlot,
};
use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
use singlefs_core::recovery::BadRootRingSlotReading;
use singlefs_core::root_ring::RootRingSlot;
use singlefs_core::transaction::FirstFile;
use singlefs_harness::fault_injection::{FaultOccurrence, InjectedFault};
use singlefs_harness::heavy_threads::{
    default_heavy_threads_of_the_cap_value, registered_heavy_threads,
    THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};
use singlefs_harness::numeric_environment_variable::process_environment_variable;

/// 盘表里每块盘此刻的镜像，按盘表次序。
fn images_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> Vec<SparseDevice> {
    devices
        .iter()
        .map(|(_, device)| device.image.clone())
        .collect()
}

/// 落后支拒的那一块盘：盘 `device` 被换成旧快照，最新那份自证系统配置停在第一个文件（实例 1、计数器 3），现行那一版末条记录是 (1, 5)。
fn assert_refused_as_behind(
    refusal: Option<EntryRefusal>,
    entry: EntryAfterTheMount,
    device: DeviceIdentity,
) {
    let devices_behind: Vec<DeviceBehindTheCurrentVersion> = match refusal {
        Some(EntryRefusal::Session(
            UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Mount(
            MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Rollback(RollbackError::CallerInputsDisagreeWithTheDisk(
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                devices,
            },
        ))) => devices,
        other => panic!("{entry:?}：该被落后支拒，实际 {}", describe(&other)),
    };
    assert_eq!(
        devices_behind
            .iter()
            .map(|behind| behind.device)
            .collect::<Vec<_>>(),
        vec![device],
        "{entry:?}：点名换成旧快照的那一块"
    );
    let behind = &devices_behind[0];
    assert_eq!(
        (
            behind
                .newest_system_configuration_journal_tail
                .instance_generation,
            behind.newest_system_configuration_journal_tail.counter,
        ),
        (InstanceGeneration(1), 3),
        "{entry:?}：旧快照最新那份系统配置停在第一个文件那一次发布"
    );
    assert_eq!(
        (
            behind.current_version_journal_position.instance_generation,
            behind.current_version_journal_position.counter,
        ),
        (InstanceGeneration(1), 5),
        "{entry:?}：现行那一版末条记录"
    );
    assert!(
        matches!(
            &behind.lacking,
            VersionLackingOnADeviceBehindIt::UnitsMissing(units_missing) if !units_missing.is_empty()
        ),
        "{entry:?}：现行那一版有单元在旧快照上不在：{:?}",
        behind.lacking
    );
}

/// Y2-a 的一格：盘 1 换成第一个文件之后的旧快照，经 `entry` 交进去。在任何写之前拒成落后支、点名盘 1，两块盘逐字节不变、会话的现行版本不动；
/// 之后拿这组盘可写挂载，与不经入口的对照逐字相同（被落后支拒）。
fn stale_snapshot_of_device_one_through(entry: EntryAfterTheMount) {
    let (mut pool, snapshot, now) = pool_with_a_snapshot_after_the_first_file();
    let mut devices = devices_from([
        (DeviceIdentity(0), &now, DeviceIdentity(0)),
        (DeviceIdentity(1), &snapshot, DeviceIdentity(1)),
    ]);
    let control = writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices);
    assert!(
        control.as_ref().is_err_and(
            |error| error.starts_with("WritableMountRefusedByDevicesWithoutTheSelectedVersion")
        ),
        "对照：不经入口直接可写挂载，被落后支拒：{control:?}"
    );
    let images_before = images_of(&devices);
    let current_before = pool.current_file_version().root;
    let refusal = go_through(entry, &mut pool, &mut devices);
    assert_refused_as_behind(refusal, entry, DeviceIdentity(1));
    assert_eq!(
        images_of(&devices),
        images_before,
        "{entry:?}：两块盘逐字节不变"
    );
    assert_eq!(
        pool.current_file_version().root,
        current_before,
        "{entry:?}：会话的现行版本不动"
    );
    assert_eq!(
        writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices),
        control,
        "{entry:?}：之后可写挂载照对照一样被拒"
    );
}

#[test]
fn a_session_publish_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write() {
    stale_snapshot_of_device_one_through(EntryAfterTheMount::SessionPublish);
}

#[test]
fn a_normal_unmount_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write() {
    stale_snapshot_of_device_one_through(EntryAfterTheMount::NormalUnmount);
}

#[test]
fn a_floor_raise_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write() {
    stale_snapshot_of_device_one_through(EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling);
}

#[test]
fn an_administrator_rollback_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write() {
    stale_snapshot_of_device_one_through(EntryAfterTheMount::AdministratorRollback);
}

/// 本盘设备号那一判拒的：盘表 [0, 1] 交进来的是对调的盘体，每块盘两槽里记的都是另一个身份。
fn assert_refused_as_swapped(refusal: Option<EntryRefusal>, entry: EntryAfterTheMount) {
    let expected_pairs = vec![
        (DeviceIdentity(0), DeviceIdentity(1)),
        (DeviceIdentity(1), DeviceIdentity(0)),
    ];
    let pairs: Vec<(DeviceIdentity, DeviceIdentity)> = match refusal {
        Some(EntryRefusal::Session(UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable {
            disagreements,
        })) => disagreements
            .into_iter()
            .map(
                |OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn {
                     identity_handed_in,
                     own_device_number_on_disk,
                 }| (identity_handed_in, own_device_number_on_disk),
            )
            .collect(),
        Some(EntryRefusal::Mount(MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
            disagreeing_fields,
            disagreeing_device_table,
        }))
        | Some(EntryRefusal::Rollback(RollbackError::CallerInputsDisagreeWithTheDisk(
            CallerInputsDisagreeingWithTheDisk::ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields,
                disagreeing_device_table,
            },
        ))) => {
            assert!(disagreeing_fields.is_empty(), "{entry:?}：参数都对得上");
            disagreeing_device_table
                .into_iter()
                .map(|disagreement| match disagreement {
                    DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn {
                        identity_handed_in,
                        own_device_number_on_disk,
                    } => (identity_handed_in, own_device_number_on_disk),
                    DeviceTableDisagreement::DeviceCountDiffersFromTheSystemConfiguration { .. }
                    | DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn {
                        ..
                    } => panic!("{entry:?}：只该是本盘设备号不符，实际 {disagreement:?}"),
                })
                .collect()
        }
        other => panic!("{entry:?}：该被本盘设备号那一判拒，实际 {}", describe(&other)),
    };
    assert_eq!(
        pairs, expected_pairs,
        "{entry:?}：(盘表给的身份, 盘上记的本盘设备号)"
    );
}

/// Y2-b 的一格：盘表身份照旧是 [0, 1]，盘 0 的位置交盘 1 的盘体、盘 1 的位置交盘 0 的盘体。经 `entry` 交进去，在任何写之前拒成
/// 本盘设备号不符、两块都点名，两块盘逐字节不变、会话的现行版本不动；盘体放回各自的身份之后照常可写挂载。
/// 会话那一格改之前只比盘表的身份次序：照常发布、把错的本盘设备号写进两块盘，之后这个池可写挂不上。
fn swapped_device_bodies_through(entry: EntryAfterTheMount) {
    let (mut pool, _, now) = pool_with_a_snapshot_after_the_first_file();
    let mut devices = devices_from([
        (DeviceIdentity(0), &now, DeviceIdentity(1)),
        (DeviceIdentity(1), &now, DeviceIdentity(0)),
    ]);
    let images_before = images_of(&devices);
    let current_before = pool.current_file_version().root;
    let refusal = go_through(entry, &mut pool, &mut devices);
    assert_refused_as_swapped(refusal, entry);
    assert_eq!(
        images_of(&devices),
        images_before,
        "{entry:?}：两块盘逐字节不变"
    );
    assert_eq!(
        pool.current_file_version().root,
        current_before,
        "{entry:?}：会话的现行版本不动"
    );
    let bodies_put_back: SparseDevices = vec![
        (DeviceIdentity(0), devices.remove(1).1),
        (DeviceIdentity(1), devices.remove(0).1),
    ];
    assert!(
        writable_mount_outcome_on_a_copy_of(&pool.parameters, &bodies_put_back).is_ok(),
        "{entry:?}：盘体放回各自的身份之后照常可写挂载"
    );
}

#[test]
fn a_session_publish_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    swapped_device_bodies_through(EntryAfterTheMount::SessionPublish);
}

#[test]
fn a_normal_unmount_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    swapped_device_bodies_through(EntryAfterTheMount::NormalUnmount);
}

#[test]
fn a_floor_raise_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    swapped_device_bodies_through(EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling);
}

#[test]
fn an_administrator_rollback_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    swapped_device_bodies_through(EntryAfterTheMount::AdministratorRollback);
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry,
    );
}

/// 放开扫的线程数：每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP` 的值，`None` 是没设；
/// 设了照它，没设或空串取 `.claude/heavy-thread-default` 的登记值，判法在 `singlefs_harness::heavy_threads`）；不超过格数。
///
/// # Panics
/// 设了而认不出（含 0）：写错的值不悄悄退回默认（代码三方 test-script-align-code-r2 判决「数值解析收成一份」，C606 同族）。
fn sweep_thread_count(thread_cap_value: Option<&str>, cell_count: usize) -> usize {
    default_heavy_threads_of_the_cap_value(thread_cap_value)
        .get()
        .clamp(1, cell_count)
}

/// 放开扫的线程数变量没设：取登记的每台默认重活线程数，不取本机核数。
#[test]
fn the_swept_user_steps_take_the_registered_heavy_thread_default_when_the_thread_cap_is_unset() {
    assert_eq!(
        sweep_thread_count(None, usize::MAX),
        registered_heavy_threads().get(),
        "SINGLEFS_THREAD_CAP 没设取 .claude/heavy-thread-default 的登记值"
    );
    assert_eq!(sweep_thread_count(Some("5"), usize::MAX), 5, "设了照它");
    assert_eq!(sweep_thread_count(Some("5"), 3), 3, "不超过格数");
}

/// 放开扫的线程数变量写错：panic，带变量名与原值，不悄悄退回默认。
#[test]
#[should_panic(expected = "SINGLEFS_THREAD_CAP=\"twelve\" 认不出")]
fn an_unreadable_thread_cap_for_the_swept_user_steps_panics_instead_of_falling_back_to_the_default()
{
    let _thread_count = sweep_thread_count(Some("twelve"), 16);
}

/// 放开扫的线程数变量设成 0：照共用解析的口径（`NonZeroUsize`）panic，不当成「取默认」、不悄悄抬成 1。
#[test]
#[should_panic(expected = "SINGLEFS_THREAD_CAP=\"0\" 认不出")]
fn a_thread_cap_of_zero_for_the_swept_user_steps_panics_instead_of_taking_the_default() {
    let _thread_count = sweep_thread_count(Some("0"), 16);
}

/// 攻方放开扫的 16 格（`stale_snapshot_substitution_swept_over_the_user_steps`）：旧快照取自第一个文件之后或覆盖写 1 之后 × 换盘 0 或盘 1 ×
/// 之后经会话发 1 / 2 / 3 次或正常卸载一次。每一格：对照（不经入口直接可写挂载）被拒；入口的每一次都拒成落后支、点名换掉的那一块；
/// 两块盘逐字节不变；之后可写挂载与对照逐字相同。改之前 16 格入口都做成、之后可写挂载做成（实例 2）、池级 checker 红 1–6 条 I-2.1。
#[test]
fn stale_snapshot_substitution_is_refused_by_every_entry_over_the_swept_user_steps() {
    let mut cells = Vec::new();
    for snapshot_after_overwrites in [0_usize, 1] {
        for stale_device in [DeviceIdentity(0), DeviceIdentity(1)] {
            for session_publishes in [Some(1_u64), Some(2), Some(3), None] {
                cells.push((snapshot_after_overwrites, stale_device, session_publishes));
            }
        }
    }
    // 16 格彼此独立（各自建池）：按线程数切片并行（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」），
    // 线程数见 sweep_thread_count；每格自己断言，一格红就在 join 时带着它的名字红。
    let thread_count = sweep_thread_count(
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        cells.len(),
    );
    let cells_per_thread = cells.len().div_ceil(thread_count);
    std::thread::scope(|scope| {
        for slice in cells.chunks(cells_per_thread) {
            scope.spawn(move || {
                for (snapshot_after_overwrites, stale_device, session_publishes) in slice {
                    eprintln!(
                        "扫到：旧快照在第 {snapshot_after_overwrites} 次覆盖写之后、换盘 {}、{session_publishes:?}",
                        stale_device.0
                    );
                    stale_snapshot_sweep_cell(*snapshot_after_overwrites, *stale_device, *session_publishes);
                }
            });
        }
    });
}

/// 放开扫的一格：旧快照取自第 `snapshot_after_overwrites` 次覆盖写之后、换的是 `stale_device`、之后经会话发 `session_publishes` 次
/// （`None` 是正常卸载一次）。
fn stale_snapshot_sweep_cell(
    snapshot_after_overwrites: usize,
    stale_device: DeviceIdentity,
    session_publishes: Option<u64>,
) {
    let cell = format!(
        "旧快照在第 {snapshot_after_overwrites} 次覆盖写之后、换盘 {}、{}",
        stale_device.0,
        session_publishes.map_or("正常卸载一次".to_string(), |count| format!(
            "会话发 {count} 次"
        ))
    );
    let mut pool = start_plain(DEVICE_WIDTH);
    let mut snapshot = pool.image();
    for overwrite in 0..2 {
        if overwrite == snapshot_after_overwrites {
            snapshot = pool.image();
        }
        pool.overwrite(OVERWRITE_BYTES).expect("会话里覆盖写");
    }
    let now = pool.image();
    let mut devices: SparseDevices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            let image = if identity == stale_device {
                &snapshot
            } else {
                &now
            };
            (identity, copy_of_device(image, identity))
        })
        .collect();
    let control = writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices);
    // 换盘 1 时对照拒在落后支；换盘 0 时最新那条根（txg 5）只在盘 0 上、所选那一版退到 txg 4，盘 1 的系统配置见证过 txg 5，
    // 先被 C554 乙拒（`NewerStateStillUnreadableAfterOneReread`）。两种都是可写挂载在取号之前拒，这里只钉「拒」与之后逐字相同。
    assert!(control.is_err(), "{cell}：对照可写挂载被拒：{control:?}");
    let images_before = images_of(&devices);
    let refusals: Vec<Option<EntryRefusal>> = match session_publishes {
        Some(count) => (0..count)
            .map(|_| go_through(EntryAfterTheMount::SessionPublish, &mut pool, &mut devices))
            .collect(),
        None => vec![go_through(
            EntryAfterTheMount::NormalUnmount,
            &mut pool,
            &mut devices,
        )],
    };
    for refusal in refusals {
        let devices_behind: Vec<DeviceIdentity> = match refusal {
            Some(EntryRefusal::Session(
                UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    devices: behind_devices,
                },
            ))
            | Some(EntryRefusal::Mount(
                MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    devices: behind_devices,
                },
            )) => behind_devices.iter().map(|behind| behind.device).collect(),
            other => panic!("{cell}：该被落后支拒，实际 {}", describe(&other)),
        };
        assert_eq!(
            devices_behind,
            vec![stale_device],
            "{cell}：点名换掉的那一块"
        );
    }
    assert_eq!(
        images_of(&devices),
        images_before,
        "{cell}：两块盘逐字节不变"
    );
    assert_eq!(
        writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices),
        control,
        "{cell}：之后可写挂载与对照逐字相同"
    );
}

/// P 的 txg：根环槽在区域 1（盘 1，`region_devices` 是 [盘 0, 盘 1, 盘 0]）、槽 2，那一槽 P 之前没写过。
const PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ONE: CheckpointTxg = CheckpointTxg(7);

/// P 的 txg：根环 3 × 8 = 24 槽转过一圈，P 的根环槽（区域 0、槽 1）里住着上一圈 txg 3（第一个文件那一版）自证得过的根。
const PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE: CheckpointTxg = CheckpointTxg(27);

/// txg 27 那一版的根槽上一圈住着的那条根：第一个文件那一版（实例 1、txg 3）。
const ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE: RingSlotReadingOtherThanTheVersionRoot =
    RingSlotReadingOtherThanTheVersionRoot::AnotherRoot {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    };

/// P 之前没写过那一槽（mkfs 清零之后）：两次读都自证不过。
const NEVER_WRITTEN_RING_SLOT: RingSlotReadingOtherThanTheVersionRoot =
    RingSlotReadingOtherThanTheVersionRoot::Bad(BadRootRingSlotReading::NotSelfVerified);

/// P 的 txg：根环槽在区域 2（`region_devices` 是 [盘 0, 盘 1, 盘 0]，区域 2 归盘 0）、槽 2，那一槽 P 之前没写过。
const PUBLISH_TXG_WITH_ITS_ROOT_IN_REGION_TWO: CheckpointTxg = CheckpointTxg(8);

/// P 的 txg：根环转过一圈，P 的根环槽（区域 1、槽 0，住盘 1）里住着上一圈 txg 1（取号之后暖机写的第一条根）自证得过的根。
const PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_ON_DEVICE_ONE: CheckpointTxg =
    CheckpointTxg(25);

/// txg 25 那一版的根槽上一圈住着的那条根：暖机写的第一条根（实例 1、txg 1）。
const ROOT_OF_THE_FIRST_WARM_UP_PUBLISH_A_LAP_BEFORE: RingSlotReadingOtherThanTheVersionRoot =
    RingSlotReadingOtherThanTheVersionRoot::AnotherRoot {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(1),
    };

/// P 的 txg：根环转过一圈，P 的根环槽（区域 2、槽 0；`region_devices` 是 [盘 0, 盘 1, 盘 0]，区域 2 归盘 0）里住着上一圈 txg 2
/// （取号之后暖机写的第二条根）自证得过的根。
const PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_IN_REGION_TWO: CheckpointTxg =
    CheckpointTxg(26);

/// txg 26 那一版的根槽上一圈住着的那条根：暖机写的第二条根（实例 1、txg 2）。
const ROOT_OF_THE_SECOND_WARM_UP_PUBLISH_A_LAP_BEFORE: RingSlotReadingOtherThanTheVersionRoot =
    RingSlotReadingOtherThanTheVersionRoot::AnotherRoot {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(2),
    };

/// 落后支缺根那一半拒的：点名 P 的根住的那块盘，缺的是根——那个根环槽两次读到的都是 `reading_of_the_ring_slot`
/// （这一槽 P 之前没写过：mkfs 清零之后自证不过；根环转过一圈：上一圈那条自证得过的根）。
fn assert_refused_as_behind_without_the_root(
    refusal: Option<EntryRefusal>,
    entry: EntryAfterTheMount,
    root_device_of_the_publish: DeviceIdentity,
    ring_slot_of_the_publish_root: RootRingSlot,
    reading_of_the_ring_slot: RingSlotReadingOtherThanTheVersionRoot,
) {
    let devices_behind: Vec<DeviceBehindTheCurrentVersion> = match refusal {
        Some(EntryRefusal::Session(
            UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Mount(
            MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Rollback(RollbackError::CallerInputsDisagreeWithTheDisk(
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                devices,
            },
        ))) => devices,
        other => panic!(
            "{entry:?}：该被落后支缺根那一半拒，实际 {}",
            describe(&other)
        ),
    };
    let lacking: Vec<(DeviceIdentity, VersionLackingOnADeviceBehindIt)> = devices_behind
        .into_iter()
        .map(|behind| (behind.device, behind.lacking))
        .collect();
    assert_eq!(
        lacking,
        vec![(
            root_device_of_the_publish,
            VersionLackingOnADeviceBehindIt::RootNotInItsRingSlot(VersionRootNotInItsRingSlot {
                ring_slot: ring_slot_of_the_publish_root,
                first_reading: reading_of_the_ring_slot,
                reread: reading_of_the_ring_slot,
            }),
        )],
        "{entry:?}：点名 P 的根住的那块盘，单元都在、缺的是 P 的根"
    );
}

/// C585 的一格：P（txg `publish_txg`）的根住的那块盘换成切在 P 的根槽 FUA 之前的样子，经 `entry` 交进去。在任何写之前拒成
/// 落后支缺根那一半（那一槽两次读到的都是 `reading_of_the_ring_slot`），两块盘逐字节不变、会话的现行版本不动。交回 P 的根住的那块盘。
/// 改回「落后且缺单元才拒」时会话发布照常做成（之后池级 checker 红 I-3.1），另三个入口拒在读根环那一步（`RootRingSlotKnownToHoldARoot`），
/// 报的不是落后支——根环转过一圈的那一形除外：P 的根槽里是上一圈自证得过的根，读根环那一步不拒，另三个入口也照常做成。
/// `entry` 是管理员回退时回退到 `rollback_target`。
fn root_device_taken_before_the_publish_root_slot_write_through(
    entry: EntryAfterTheMount,
    rollback_target: RollbackTargetOfTheEntry,
    publish_txg: CheckpointTxg,
    reading_of_the_ring_slot: RingSlotReadingOtherThanTheVersionRoot,
) -> DeviceIdentity {
    let RootDeviceOfThePublishTakenMidPublish {
        mut pool,
        mut devices,
        root_device_of_the_publish,
        ring_slot_of_the_publish_root,
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::BeforeItsRootSlotWrite,
        publish_txg,
    );
    let images_before = images_of(&devices);
    let current_before = pool.current_file_version().root;
    let refusal = go_through_rolling_back_to(entry, rollback_target, &mut pool, &mut devices);
    assert_refused_as_behind_without_the_root(
        refusal,
        entry,
        root_device_of_the_publish,
        ring_slot_of_the_publish_root,
        reading_of_the_ring_slot,
    );
    assert_eq!(
        images_of(&devices),
        images_before,
        "{entry:?}：两块盘逐字节不变"
    );
    assert_eq!(
        pool.current_file_version().root,
        current_before,
        "{entry:?}：会话的现行版本不动"
    );
    root_device_of_the_publish
}

#[test]
fn a_session_publish_onto_the_root_device_taken_before_the_publish_root_slot_write_is_refused_before_any_write(
) {
    root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::SessionPublish,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        NEVER_WRITTEN_RING_SLOT,
    );
}

#[test]
fn a_normal_unmount_onto_the_root_device_taken_before_the_publish_root_slot_write_is_refused_before_any_write(
) {
    root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::NormalUnmount,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        NEVER_WRITTEN_RING_SLOT,
    );
}

#[test]
fn a_floor_raise_onto_the_root_device_taken_before_the_publish_root_slot_write_is_refused_before_any_write(
) {
    root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        NEVER_WRITTEN_RING_SLOT,
    );
}

#[test]
fn an_administrator_rollback_onto_the_root_device_taken_before_the_publish_root_slot_write_is_refused_before_any_write(
) {
    root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::AdministratorRollback,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        NEVER_WRITTEN_RING_SLOT,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused,
    );
}

/// 不误拒：同一组盘（P 的根住的那块切在根槽 FUA 之前）上可写挂载。所选那一版是在 txg 5 的根上施加 P 的记录得来的，P 的根不在根环里，
/// 落后支不核缺根那一半；P 的单元都在，照常挂上、所选那一版就是 P（攻方 X6-a「崩了再可写挂载」那一格：择 P、之后 checker 0）。
#[test]
fn a_writable_mount_onto_the_root_device_taken_before_the_publish_root_slot_write_applies_the_publish_record_and_mounts(
) {
    let RootDeviceOfThePublishTakenMidPublish { pool, devices, .. } =
        root_device_of_the_publish_taken_mid_publish(
            WhereThePublishIsCut::BeforeItsRootSlotWrite,
            PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        );
    let publish_root = pool.current_file_version().root;
    let mut copies = copies_of(&devices);
    let mounted = mount_writable(&pool.parameters, &mut copies)
        .unwrap_or_else(|error| panic!("P 的根不在根环里、单元都在：照常可写挂载，实际 {error:?}"));
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (publish_root.instance, publish_root.checkpoint_txg),
        "所选那一版是施加 P 的记录得来的 P"
    );
}

/// H2 的一格（攻方改错 A「缺根那一半只读 region 0 那块盘的根槽」）：P 是 txg 7，根环槽在区域 1、住盘 1。盘 1 换成切在 P 的根槽 FUA
/// 之前的样子，会话发布在任何写之前拒成落后支缺根那一半、点名盘 1。只查盘 0 的根槽时盘 1 的缺根不核、会话照常发布。
#[test]
fn a_session_publish_onto_device_one_holding_the_publish_root_taken_before_its_root_slot_write_is_refused_before_any_write(
) {
    let root_device_of_the_publish = root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::SessionPublish,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ONE,
        NEVER_WRITTEN_RING_SLOT,
    );
    assert_eq!(
        root_device_of_the_publish,
        DeviceIdentity(1),
        "txg 7 的根环槽在区域 1、住盘 1"
    );
}

/// H2 的一格（攻方改错 E「根槽里自证得过的根都算是它」）：根环转过一圈，P 是 txg 27，根环槽（区域 0、槽 1）里住着上一圈
/// 第一个文件那一版（txg 3）自证得过的根。P 的根住的那块盘换成切在 P 的根槽 FUA 之前的样子，会话发布在任何写之前拒成
/// 落后支缺根那一半，两次读到的都是 `AnotherRoot`（实例 1、txg 3）。自证得过就算是它时上一圈的旧根被当成 P 的根、会话照常发布。
#[test]
fn a_session_publish_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    let root_device_of_the_publish = root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::SessionPublish,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE,
        ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE,
    );
    assert_eq!(
        root_device_of_the_publish,
        DeviceIdentity(0),
        "txg 27 的根环槽在区域 0、住盘 0"
    );
}

/// J1 的一格（代码三方 m3-code-r2，攻方改错 RG「把区域号当盘号」）：P 是 txg 8，根环槽在区域 2、槽 2，区域 2 归盘 0。
/// 盘 0 换成切在 P 的根槽 FUA 之前的样子，会话发布在任何写之前拒成落后支缺根那一半、点名盘 0。
/// 拿区域号当盘号时区域 2 被当成盘表里没有的盘 2，盘 0 的缺根不核、会话照常发布（之后池级 checker 红 I-3.1）。
#[test]
fn a_session_publish_onto_device_zero_holding_the_publish_root_in_region_two_taken_before_its_root_slot_write_is_refused_before_any_write(
) {
    let root_device_of_the_publish = root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::SessionPublish,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_IN_REGION_TWO,
        NEVER_WRITTEN_RING_SLOT,
    );
    assert_eq!(
        root_device_of_the_publish,
        DeviceIdentity(0),
        "txg 8 的根环槽在区域 2、区域 2 归盘 0"
    );
}

/// 缺根那一半第一次读 P 的根槽，在会话发布里是那一槽被读的第几次（从 1 数）：会话发布读根环之前只核这一判。
const ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_SESSION_PUBLISH: u64 = 1;

/// 缺根那一半第一次读 P 的根槽，在可写挂载里是那一槽被读的第几次（从 1 数）：读阶段（择根、见证读、重建）先读过它 7 次，
/// 不出错时第 8 次就是取号之前逐盘核那一判的读，也是这一槽在取号之前的最后一次读（攻方原型扫出来的；
/// [`a_writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_whose_root_slot_fails_two_reads_in_a_row_is_refused_before_any_write`]
/// 从第 8 次起读坏、恰好坏两次就拒成缺根，钉住它是那一判的第一次读）。
const ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_WRITABLE_MOUNT: u64 = 8;

/// H1 的一格：P 的根住的那块盘切在 P 的根槽 FUA 之后（落后、单元都在、根槽里是 P），会话发布时缺根那一半第一次读那一槽读坏
/// （按 `fault`：读不出，或读回的字节坏一位、自证不过）、重读读得出 P：照常发布，那一次注入真发生了。
/// 去掉重读（第一次读不是它就拒）时两种读坏都拒成落后支缺根那一半；只在第一次读不出时才重读（攻方 RU）时字节坏一位的那一种拒。
fn session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
    fault: InjectedFault,
) {
    let RootDeviceOfThePublishTakenMidPublish {
        mut pool,
        devices,
        ring_slot_of_the_publish_root,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let (mut wrapped, plan) = with_a_fault_on_reads_of_the_publish_root_slot(
        devices,
        &pool,
        ring_slot_of_the_publish_root,
        fault,
        FaultOccurrence::TheNthMatchingCall(
            ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_SESSION_PUBLISH,
        ),
    );
    let (content, write_time_seconds) = overwrite_after(&pool, 0);
    let outcome = pool
        .session
        .as_mut()
        .expect("这一刻有可写会话")
        .publish_user_change(
            &mut wrapped,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds,
            }),
        );
    assert_eq!(
        plan.fired_count(),
        1,
        "{fault:?}：P 的根槽上那一次读坏真发生了：{:?}",
        plan.fired()
    );
    assert!(
        outcome.is_ok(),
        "{fault:?}：一次读坏、重读读得出 P：会话照常发布，实际 {:?}",
        outcome.err()
    );
}

#[test]
fn a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
        InjectedFault::ReadFails,
    );
}

/// L1 的一格：H1 会话发布那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
#[test]
fn a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}

/// H1 的一格：同一组盘（P 的根住的那块切在根槽 FUA 之后）上可写挂载，所选那一版就是择到的 P（写进了根环），
/// 取号之前逐盘核时缺根那一半第一次读那一槽读坏（按 `fault`）、重读读得出 P：照常挂上、所选那一版是 P，那一次注入真发生了。
/// 去掉重读时两种读坏都拒成 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`；只在第一次读不出时才重读（攻方 RU）时
/// 字节坏一位的那一种拒。
fn writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
    fault: InjectedFault,
) {
    let RootDeviceOfThePublishTakenMidPublish {
        pool,
        devices,
        ring_slot_of_the_publish_root,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let publish_root = pool.current_file_version().root;
    let (mut wrapped, plan) = with_a_fault_on_reads_of_the_publish_root_slot(
        copies_of(&devices),
        &pool,
        ring_slot_of_the_publish_root,
        fault,
        FaultOccurrence::TheNthMatchingCall(ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_WRITABLE_MOUNT),
    );
    let mounted = mount_writable(&pool.parameters, &mut wrapped);
    assert_eq!(
        plan.fired_count(),
        1,
        "{fault:?}：P 的根槽上那一次读坏真发生了：{:?}",
        plan.fired()
    );
    let mounted = mounted.unwrap_or_else(|error| {
        panic!("{fault:?}：一次读坏、重读读得出 P：照常可写挂载，实际 {error:?}")
    });
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (publish_root.instance, publish_root.checkpoint_txg),
        "{fault:?}：所选那一版是 P"
    );
}

#[test]
fn a_writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
        InjectedFault::ReadFails,
    );
}

/// L1 的一格：H1 可写挂载那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
#[test]
fn a_writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}

/// H3：同一组盘（P 的根住的那块切在根槽 FUA 之后）上可写挂载，所选那一版就是择到的 P，取号之前逐盘核时缺根那一半读那一槽
/// 连着两次读坏（从第 8 次起每一次都坏，真发生的恰好是两次：读一次、重读一次）：拒成 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，
/// 点名 P 的根住的那块盘、缺的是根（两次都读不出），拒在任何写之前（整池一次写都没发）、两块盘逐扇区不变。
/// 可写挂载一律不核缺根那一半（所选那一版一律当成没写进根环）时，这一形挂得上或拒成别的。
#[test]
fn a_writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_whose_root_slot_fails_two_reads_in_a_row_is_refused_before_any_write(
) {
    let RootDeviceOfThePublishTakenMidPublish {
        pool,
        devices,
        root_device_of_the_publish,
        ring_slot_of_the_publish_root,
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let publish_root = pool.current_file_version().root;
    let images_before = images_of(&devices);
    let (mut wrapped, plan) = with_a_fault_on_reads_of_the_publish_root_slot(
        copies_of(&devices),
        &pool,
        ring_slot_of_the_publish_root,
        InjectedFault::ReadFails,
        FaultOccurrence::EveryMatchingCallFromTheNthOnward(
            ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_WRITABLE_MOUNT,
        ),
    );
    let refusal = mount_writable(&pool.parameters, &mut wrapped).err();
    let devices_without = match refusal {
        Some(MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
            selected_version,
            devices: devices_without,
            ..
        }) => {
            assert_eq!(
                selected_version,
                RollbackTarget {
                    instance: publish_root.instance,
                    checkpoint_txg: publish_root.checkpoint_txg,
                },
                "所选那一版是择到的 P"
            );
            devices_without
        }
        other => {
            panic!("该拒成 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 {other:?}")
        }
    };
    let lacking: Vec<(DeviceIdentity, Option<VersionRootNotInItsRingSlot>)> = devices_without
        .into_iter()
        .map(|DeviceWithoutTheSelectedVersion { device, lacking }| {
            let root_not_in_its_ring_slot = match lacking {
                SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndItsRootNotInItsRingSlot {
                    root_not_in_its_ring_slot,
                    ..
                } => Some(root_not_in_its_ring_slot),
                SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                    ..
                } => None,
            };
            (device, root_not_in_its_ring_slot)
        })
        .collect();
    let unreadable =
        RingSlotReadingOtherThanTheVersionRoot::Bad(BadRootRingSlotReading::Unreadable);
    assert_eq!(
        lacking,
        vec![(
            root_device_of_the_publish,
            Some(VersionRootNotInItsRingSlot {
                ring_slot: ring_slot_of_the_publish_root,
                first_reading: unreadable,
                reread: unreadable,
            })
        )],
        "点名 P 的根住的那块盘，缺的是根（那一槽读一次、重读一次都读不出），不是单元"
    );
    let images_after: Vec<SparseDevice> = wrapped
        .iter()
        .map(|(_, device)| device.wrapped_device().image.clone())
        .collect();
    assert_eq!(plan.calls().writes, 0, "拒在任何写之前：整池一次写都没发");
    assert_eq!(images_after, images_before, "两块盘逐扇区不变");
    assert_eq!(
        plan.fired_count(),
        2,
        "那一槽真读坏的恰好两次（缺根那一半读一次、重读一次）：{:?}",
        plan.fired()
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_entries_after_mount_refuse_swapped_or_behind_devices::an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot,
    );
}

/// J3 的一格（攻方改错 EC「另三个入口上根槽里自证得过就算是它」、AC「另三个入口只查区域 0 那块盘」）：根环转过一圈，P 是
/// `publish_txg`，P 的根住的那块盘（`root_device_of_the_publish`）换成切在 P 的根槽 FUA 之前的样子，那一槽里是上一圈自证得过的根
/// （`root_of_the_previous_lap`）。经 `entry`（管理员回退取现行那一版的前一版做目标）交进去，在任何写之前拒成落后支缺根那一半，
/// 两次读到的都是上一圈那条根，两块盘逐字节不变。这一形上读根环那一步不拒（槽里自证得过），缺根那一半是这三个入口唯一的一道：
/// 去掉它、或上一圈的根被当成 P 的根、或根住的盘不是区域 0 那块而只查区域 0 那块时，入口照常做成、之后池级 checker 红 I-3.1。
fn root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
    entry: EntryAfterTheMount,
    publish_txg: CheckpointTxg,
    root_of_the_previous_lap: RingSlotReadingOtherThanTheVersionRoot,
    root_device_of_the_publish: DeviceIdentity,
) {
    assert_eq!(
        root_device_taken_before_the_publish_root_slot_write_through(
            entry,
            RollbackTargetOfTheEntry::VersionBeforeTheCurrent,
            publish_txg,
            root_of_the_previous_lap,
        ),
        root_device_of_the_publish,
        "{entry:?}：txg {} 的根住的那块盘",
        publish_txg.0
    );
}

#[test]
fn a_normal_unmount_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::NormalUnmount,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE,
        ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

#[test]
fn a_floor_raise_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE,
        ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

#[test]
fn an_administrator_rollback_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::AdministratorRollback,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE,
        ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

#[test]
fn a_normal_unmount_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::NormalUnmount,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_ON_DEVICE_ONE,
        ROOT_OF_THE_FIRST_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(1),
    );
}

#[test]
fn a_floor_raise_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_ON_DEVICE_ONE,
        ROOT_OF_THE_FIRST_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(1),
    );
}

#[test]
fn an_administrator_rollback_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::AdministratorRollback,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_ON_DEVICE_ONE,
        ROOT_OF_THE_FIRST_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(1),
    );
}

/// L2 的一格（攻方改错 RGC「正常卸载、准入抬 F、管理员回退那条路上把区域号当盘号」）：根环转过一圈，P 是 txg 26，根环槽在区域 2、槽 0，
/// 区域 2 归盘 0，那一槽里是上一圈暖机写的第二条根（实例 1、txg 2）。盘 0 换成切在 P 的根槽 FUA 之前的样子，经正常卸载交进去，
/// 在任何写之前拒成落后支缺根那一半、点名盘 0，两次读到的都是上一圈那条根。把区域号当盘号时区域 2 被当成盘表里没有的盘 2，
/// 盘 0 的缺根不核，读根环那一步也不拒（槽里自证得过），入口照常做成。
#[test]
fn a_normal_unmount_onto_device_zero_whose_ring_slot_in_region_two_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::NormalUnmount,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_IN_REGION_TWO,
        ROOT_OF_THE_SECOND_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

/// L2 的一格：同上，经准入抬 F 交进去。
#[test]
fn a_floor_raise_onto_device_zero_whose_ring_slot_in_region_two_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_IN_REGION_TWO,
        ROOT_OF_THE_SECOND_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

/// L2 的一格：同上，经管理员回退交进去（目标取现行那一版的前一版）。
#[test]
fn an_administrator_rollback_onto_device_zero_whose_ring_slot_in_region_two_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::AdministratorRollback,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_IN_REGION_TWO,
        ROOT_OF_THE_SECOND_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}
