//! 重建分配器读根环那一遍一条不被抛弃的自证有效根都没读出（择根那几遍读得出）时，把根环整环再读一遍，根环表、回收门槛与影子账用两遍逐槽
//! 并起来的读数（用户 2026-10-05 定 R1「按槽并两遍」；第一遍整环读坏时并起来就是重读那一遍）；并起来还是一条都没有，就在任何写之前拒这次可写挂载（`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，用户 2026-10-05 定
//! Q3 另起拒因，之前报 `Recovery(NoValidRoot)`）。只有一块盘没读出时也整环重读（Q4 rr2）的用例在
//! `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs`。用户 2026-10-05 定「重读」：
//! 岔路单 `research/prompts/m3-batch17-code-r1-forks.md`「重读 先把根环再读一遍」那一行，判决 `research/prompts/m3-batch17-code-r1-main-verification.md`
//! 第二节「N1 甲二」那一行与第四节。不重读、给空表往下走时，写行那次记根算回收门槛只看得到它自己，释放代高于 F_生效 的一截当场放开、被暖机写掉，
//! 盘上环里更旧的有效根还引用着（攻方 `research/prompts/m3-batch17-code-r1-opus-output.md` 第 1.3 节）。
//!
//! 历史照攻方那一段（`264:6:4:0` n = 15，同 `the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` 第三条用例的起点与卸载刀）：
//! 不补环窗口、准入关着造的窄盘（单元区 264 槽），第一个文件之后覆盖写 6 次、顺序写长到 4 个单元，在这个进程里发 unlink 的意图就崩；
//! 重开可写（准入判着、影子账开、批大小 1）续做走完；卸载落下 15 个写就崩；两块盘都在、再可写挂载。
//! 读故障由这里的 [`DeviceFailingPassesOfTheRootRing`] 造：根环每个槽按它自己被读的次数从 1 数，点名的那几次读报块设备错（盘上内容不动）。
//! 一次不注错的再挂里根环整遍读若干遍、每遍每槽读一次，第 6 遍（从 1 数）是重建分配器那一遍（攻方用回溯栈认过：第 0 遍择根……第 5 遍
//! 重建分配器，从 0 数）；这一认法由 [`RereadsOfThisMount::root_ring_of_the_rebuilt_allocator`] 现核：只坏那一遍时它报重读过。
mod common_admission;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs"]
mod scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::{
    narrow_pool_after_a_crashed_unmount, plan_failing_every_write_from,
    RootRingPassesFailingDuringTheRemount, SharedRootRingReads,
    READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    MountError, ShadowLedger,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadFaultsUnderTheRefusedMount,
    ReadOnlyReadBackOfTheCrashImage, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
};
use std::collections::BTreeMap;

/// 只读挂载、把挂载态里每个活着的文件从头读到尾：走到的根 txg 与每个活文件（inode 号、全部字节）。删除中的 inode（nlink = 0）不读。
#[derive(Debug, PartialEq, Eq)]
struct WholeReadOfTheLiveFiles {
    root_txg: CheckpointTxg,
    live_files: Vec<(u64, Vec<u8>)>,
}

/// 只读挂载与整读每个活文件；哪一步读不出就交回那一步的错（只读挂载、打开、读）。
fn whole_read_of_the_live_files(image: &MemoryPool) -> Result<WholeReadOfTheLiveFiles, String> {
    let read_only = mount_read_only(image).map_err(|failure| format!("只读挂载：{failure:?}"))?;
    let mut live_files = Vec::new();
    // 迭代上界是这一版 inode 树的记录数；提前出口只有读不出的那三种错。
    for record in read_only.mounted.inode_records() {
        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = read_only
            .mounted
            .open_file(image, InodeNumber(record.inode))
            .map_err(|failure| format!("打开 inode {}：{failure:?}", record.inode))?;
        let read = file
            .read_at(image, FileOffsetInBytes(0), record.size)
            .map_err(|failure| format!("读 inode {}：{failure:?}", record.inode))?;
        live_files.push((record.inode, read.bytes));
    }
    Ok(WholeReadOfTheLiveFiles {
        root_txg: read_only.effective_root.checkpoint_txg,
        live_files,
    })
}

/// 一份镜像去掉一块盘。
fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
    MemoryPool {
        devices: image
            .devices
            .iter()
            .filter(|(identity, _)| **identity != lost)
            .map(|(identity, device)| (*identity, device.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over,
    );
}

/// 重建分配器那一遍与它的重读都坏（两块盘全部根环槽报读错）：在任何写之前拒（Q3 另起的拒因
/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`），盘面与挂载之前逐字节相同；拒因在 D23 已定项 14 那张单子上，
/// 被拒的那次挂载在读故障下跑，崩溃镜像上判据复算为假不算不合法，判官判它合法。
/// 判别力：重读还读不出时不拒、照旧给空表往下写，挂载写了盘（写行、暖机）之后报 `FloorRaiseFailedAfterTheMountsPublishes`（变异表里这一刀量过），
/// 第一项红；拒因照旧报 `Recovery(NoValidRoot)` 时第一项红，判官把它排在单子外、最后一项也红。
#[test]
fn the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mount_before_any_write(
) {
    let history = narrow_pool_after_a_crashed_unmount();
    let (remounted, image_after_the_refusal, root_ring_reads) =
        history.remount(RootRingPassesFailingDuringTheRemount::TheAllocatorRebuildAndItsReread);
    assert!(
        matches!(
            remounted,
            Err(MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)
        ),
        "重建分配器那一遍与重读都一条有效根读不出：拒这次可写挂载（Q3 那一拒）：{:?}",
        remounted.as_ref().map(|_| "做成了")
    );
    assert!(
        image_after_the_refusal == history.image_after_the_crashed_unmount,
        "拒在任何写之前：两块盘与挂载之前逐字节相同"
    );
    assert_eq!(
        root_ring_reads.failed_reads(),
        2 * history.root_ring_slot_count(),
        "读坏的恰是两整遍（重建那一遍与它的重读）：每个根环槽两次"
    );
    let reads_of_each_slot = root_ring_reads.reads_of_each_slot();
    assert!(
        reads_of_each_slot.len() == usize::try_from(history.root_ring_slot_count()).expect("槽数")
            && reads_of_each_slot
                .values()
                .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD + 1),
        "重读那一遍之后根环再没被读过（拒在重读之后、取号之前）：{reads_of_each_slot:?}"
    );
    let Err(refusal) = remounted else {
        panic!("上面判过这次挂载拒了");
    };
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
            &history.image_after_the_crashed_unmount,
            recover(&history.image_after_the_crashed_unmount, JournalPolicy::Consult).effective_root,
            &refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::Injected,
            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadThroughInjectedFaults,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator
        ),
        "拒因在 D23 已定项 14 那张单子上；被拒的那次挂载在读故障下跑，崩溃镜像上判据复算为假不算不合法：判合法"
    );
}

/// 只坏重建分配器那一遍（两块盘全部根环槽报读错），重读之前的钩子（`BeforeTheOneReread::CallTheTestOnlyHookFirst`）让根环此后每次读都坏：
/// 重读那一遍也一条有效根读不出，在任何写之前拒（Q3 那一拒），两块盘与挂载之前逐字节相同。钩子恰调一次，调的那一刻每个根环槽
/// 恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次（重建那一遍之后、重读之前）。
/// 这个挂载入口不收删除开关、取产品值：拒在建分配器那一步，删除开关在它之后才装上，挂载读根环的次序与别的用例相同（读次数那两条断言现核）。
/// 判别力：重建分配器整环重读之前不调钩子时，重读那一遍照常读得出、挂载做成，第一项红。
#[test]
fn a_hook_before_the_root_ring_reread_failing_that_reread_too_refuses_the_writable_mount_before_any_write(
) {
    let history = narrow_pool_after_a_crashed_unmount();
    let root_ring_reads = SharedRootRingReads::new(
        history.every_root_ring_slot(),
        RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild.failing_reads_of_each_slot(),
    );
    let mut devices = history.devices_failing(&root_ring_reads, |_, device| device);
    let failing_from_the_hook = root_ring_reads.clone();
    let mut reads_of_each_slot_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>> = Vec::new();
    let mut fail_the_reread_too = || {
        reads_of_each_slot_at_each_hook_call.push(failing_from_the_hook.reads_of_each_slot());
        failing_from_the_hook.fail_every_later_read_of_each_slot();
    };
    let remounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &history.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut fail_the_reread_too),
    );
    let image_after_the_refusal = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect(),
        device_size_in_bytes: history.image_after_the_crashed_unmount.device_size_in_bytes,
    };
    assert!(
        matches!(
            remounted,
            Err(MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)
        ),
        "钩子让重读那一遍也读坏：重建分配器那一遍与重读都一条有效根读不出，拒这次可写挂载（Q3 那一拒）：{:?}",
        remounted.as_ref().map(|_| "做成了")
    );
    assert!(
        image_after_the_refusal == history.image_after_the_crashed_unmount,
        "拒在任何写之前：两块盘与挂载之前逐字节相同"
    );
    let slot_count = usize::try_from(history.root_ring_slot_count()).expect("槽数");
    assert!(
        reads_of_each_slot_at_each_hook_call.len() == 1
            && reads_of_each_slot_at_each_hook_call[0].len() == slot_count
            && reads_of_each_slot_at_each_hook_call[0]
                .values()
                .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD),
        "钩子恰调一次，在重建那一遍之后、重读之前（每个根环槽恰读过 {READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD} 次）：{reads_of_each_slot_at_each_hook_call:?}"
    );
    assert_eq!(
        root_ring_reads.failed_reads(),
        2 * history.root_ring_slot_count(),
        "读坏的恰是两整遍（重建那一遍与它的重读）：每个根环槽两次"
    );
    let reads_of_each_slot = root_ring_reads.reads_of_each_slot();
    assert!(
        reads_of_each_slot.len() == slot_count
            && reads_of_each_slot
                .values()
                .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD + 1),
        "重读那一遍之后根环再没被读过（拒在重读之后、取号之前）：{reads_of_each_slot:?}"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before,
    );
}

/// 攻方第 1.3 节那一段具体的历史：只坏重建分配器那一遍的再挂，写行的根落在盘 1、暖机写了单元，暖机落到盘 0 的那条根写下去之前掉电；丢盘 1。
/// 剩下的盘 0 只读挂载、整读每个活文件，与卸载之前盘 0 上读出的相同（走到的根 txg 与内容）。
/// 判别力：不重读、照旧给空表往下走时，暖机的单元写进盘 0 上最新的根还引用的槽（攻方量过 `UnitUnreadable { slot: SlotNumber(1441) }`），最后一项红。
#[test]
fn only_the_allocator_rebuild_pass_failing_and_a_crash_before_the_warm_up_root_lands_on_device_zero_keep_what_device_zero_reads(
) {
    let history = narrow_pool_after_a_crashed_unmount();
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let slots_per_region = history.pool.parameters.geometry.root_ring_slots_per_region;
    let device_of_the_root_of = |txg: CheckpointTxg| {
        history.pool.parameters.region_devices
            [usize::try_from(target_for_publish(txg, slots_per_region).region).expect("区域号")]
    };
    // 先在一份拷贝上不注错地整做一次再挂，认出写行与暖机各条根的 txg、暖机里第一条落在盘 0 上的根（这几次发布的 txg 只由新实例第一次发布的 txg
    // 与发布的先后定，读不读坏根环都一样；不拿读坏的那一次认：不重读时那一次在暖机之后报错，交不回暖机的根）。
    let (whole_remount, _, _) =
        history.remount(RootRingPassesFailingDuringTheRemount::NoneOfThePasses);
    let whole_remount =
        whole_remount.unwrap_or_else(|error| panic!("不注错的再挂照常可写：{error:?}"));
    let row_txg = whole_remount.output.row_publish.root().checkpoint_txg;
    assert_eq!(
        device_of_the_root_of(row_txg),
        DeviceIdentity(1),
        "写行那次（txg {row_txg:?}）的根落在盘 1（攻方那一段的样子）"
    );
    let warm_up_txg_on_device_zero = whole_remount
        .output
        .warm_up_publishes
        .iter()
        .map(|version| version.root().checkpoint_txg)
        .find(|txg| device_of_the_root_of(*txg) == DeviceIdentity(0))
        .expect("暖机在每块盘上都落一条根：有一条落在盘 0");
    let root_slot_offset_on_device_zero = slot_offset(
        target_for_publish(warm_up_txg_on_device_zero, slots_per_region),
        history
            .pool
            .parameters
            .geometry
            .fixed_structure_slot_spacing,
    );
    // 再在一份拷贝上同样读坏地再挂，只让那条根的写报错，认出它是整池第几次写。
    let write_ordinal_of_the_warm_up_root_on_device_zero = {
        let root_write_plan = SharedFaultPlan::armed(
            device_width.fixed_geometry(),
            FaultSchedule {
                fault: InjectedFault::WriteFails,
                device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(0)),
                placement: FaultPlacement::OffsetExactly(root_slot_offset_on_device_zero),
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::TheNthMatchingCall(1),
            },
        );
        let root_ring_reads = SharedRootRingReads::new(
            history.every_root_ring_slot(),
            RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild
                .failing_reads_of_each_slot(),
        );
        let mut devices = history.devices_failing(&root_ring_reads, |identity, device| {
            FaultInjectingBlockDevice::new(identity, device, root_write_plan.clone())
        });
        let remounted = history.mount_writable(&mut devices);
        assert!(
            remounted.is_err(),
            "暖机落到盘 0 的那条根（txg {warm_up_txg_on_device_zero:?}）写报错：挂载报错"
        );
        let fired = root_write_plan.fired();
        assert_eq!(fired.len(), 1, "那条根的写恰被拦下一次：{fired:?}");
        fired[0].call_ordinal_across_the_pool
    };
    // 真那一次：从那条根的写起每次写都报错（盘停在它之前的写之后），丢盘 1。
    let crash_plan = plan_failing_every_write_from(
        device_width,
        write_ordinal_of_the_warm_up_root_on_device_zero,
    );
    let root_ring_reads = SharedRootRingReads::new(
        history.every_root_ring_slot(),
        RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild.failing_reads_of_each_slot(),
    );
    let mut devices = history.devices_failing(&root_ring_reads, |identity, device| {
        FaultInjectingBlockDevice::new(identity, device, crash_plan.clone())
    });
    let remounted = history.mount_writable(&mut devices);
    assert!(
        remounted.is_err(),
        "暖机落到盘 0 的那条根写下去之前掉电：挂载报错"
    );
    drop(remounted);
    let image_after_the_crash = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: history.image_after_the_crashed_unmount.device_size_in_bytes,
    };
    let before_the_unmount = whole_read_of_the_live_files(&image_without(
        &history.image_before_the_unmount,
        DeviceIdentity(1),
    ));
    assert!(
        before_the_unmount.is_ok(),
        "卸载之前盘 0 只读挂载、整读每个活文件做得成：{before_the_unmount:?}"
    );
    assert_eq!(
        whole_read_of_the_live_files(&image_without(&image_after_the_crash, DeviceIdentity(1))),
        before_the_unmount,
        "丢盘 1 之后剩下的盘 0 走到的根 txg 与每个活文件的内容与卸载之前盘 0 上读出的相同：再挂那一次当场放开的槽没有一个被盘 0 上最新的根引用"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::a_reread_still_failing_on_device_one_hands_over_what_both_passes_failing_only_on_device_one_hand_over`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_reread_still_failing_on_device_one_hands_over_what_both_passes_failing_only_on_device_one_hand_over(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read::a_reread_still_failing_on_device_one_hands_over_what_both_passes_failing_only_on_device_one_hand_over,
    );
}
