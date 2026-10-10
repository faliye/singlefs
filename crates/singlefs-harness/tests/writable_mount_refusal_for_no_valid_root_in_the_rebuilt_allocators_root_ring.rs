//! 重建分配器读根环有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有：拒可写的拒因另起成员
//! （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`），不复用读阶段择不到根的 `Recovery(NoValidRoot)`；
//! D23（journal 的角色与格式） 已定项 14 那张单子加一行（用户 2026-10-05 定 Q3，岔路单 `research/prompts/m3-batch10-code-r1-forks.md`，
//! 判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）。判法在崩溃镜像上复算「按最新那条根的实例表，环里一条不被抛弃的
//! 自证有效根都没有」，与改法 G4 同形：注入了读故障的崩溃态上不判那一条；没注入读故障的崩溃态上复算为真，拒得不合法
//! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。
//! 两遍按槽并（用户 2026-10-05 定 R1，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第六节）之后这一拒的判据在崩溃镜像上
//! 照旧只读一遍：崩溃镜像上没有瞬时读错，两遍读出的逐槽相同，并起来就是那一遍；所以复算判据不变。
//! 盘面是 24 槽根环的小池（`common::build_pool`，第一个文件之后）；手工盘面是把它根环三个区域的每一个槽都清成全 0。

mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::mount::MountError;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, instance_table_of_root,
    readable_roots, recover, root_is_abandoned_by_the_instance_table, JournalPolicy,
    RecoveryFailure,
};
use singlefs_core::root_ring::slot_offset;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state,
    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount,
    refusal_reason_on_the_d23_list, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
    ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet,
    RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
};

/// 第一个文件之后的小池盘面。
fn image_after_the_first_file(tag: &str) -> MemoryPool {
    build_pool(tag).memory_pool()
}

/// 手工盘面：根环三个区域的每一个槽都清成全 0（静默损坏，不是崩溃摆得出来的）。
fn image_with_every_root_ring_slot_zeroed_by_hand(image: &MemoryPool) -> MemoryPool {
    let system_configuration =
        choose_system_configuration(image).expect("清根环之前择得到系统配置");
    let immutable = system_configuration.immutable;
    let root_slot_bytes = usize::try_from(immutable.sizes.physical_block_size).expect("根槽宽");
    let mut zeroed = image.clone();
    // 迭代上界是根环的槽数；没有提前出口。
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        zeroed
            .devices
            .get_mut(&device)
            .expect("区域归属表里的盘在池里")
            .write(
                slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
                &vec![0u8; root_slot_bytes],
            );
    }
    zeroed
}

/// 场景：两个拒因各交一次给「在不在单子上」的判法。
/// 预期：重建分配器读根环重读仍一条有效根都没有那一拒在单子上（Q3 那一行）；读阶段择不到根的 `Recovery(NoValidRoot)` 照旧不在单子上。
/// 判别力：单子不收新成员时第一项红。
#[test]
fn the_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring_is_on_the_d23_list_and_the_read_stage_one_is_not(
) {
    assert_eq!(
        refusal_reason_on_the_d23_list(
            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread
        ),
        Some(RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator),
        "重建分配器读根环整环重读一次仍一条有效根都没有：在 D23 已定项 14 那张单子上"
    );
    assert_eq!(
        refusal_reason_on_the_d23_list(&MountError::Recovery(RecoveryFailure::NoValidRoot)),
        None,
        "读阶段择不到根（只读挂载同样挂不上）：不在单子上"
    );
}

/// 场景：手工把根环每一个槽清成全 0 的盘面，崩溃态没注入读故障；拿 Q3 那一拒交给判官（盘上逐字节不变、只读读出在允许集合里）。
/// 预期：判据在镜像上复算为真（择不到根），U6 那四条都满足；没有读故障的历史留不下这份盘面，拒得不合法，没满足的恰是 Q3 那一条。
/// 判别力：判官不复算 Q3 那一条（`crates/mutations.tsv`「第10g批 Q3 判官不复算…」）时判成合法，这一项红。
#[test]
fn a_refusal_for_no_valid_root_on_a_crash_image_without_read_faults_whose_root_ring_was_zeroed_by_hand_is_illegal(
) {
    let image = image_with_every_root_ring_slot_zeroed_by_hand(&image_after_the_first_file(
        "no-valid-root-in-the-ring-zeroed-without-read-faults",
    ));
    assert_eq!(
        recover(&image, JournalPolicy::Consult).effective_root,
        None,
        "前提：根环全 0，只读恢复也择不到根"
    );
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &image,
            None,
            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
        ),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
            conditions_not_met: vec![
                RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults
            ],
        },
        "没注入读故障而判据在镜像上复算为真：拒是对的、造出这份盘面的那一段不对，判不合法"
    );
}

/// 场景：同一份手工盘面，崩溃态注入过读故障。
/// 预期：Q3 那一条不判（D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态），判据复算为真、U6 那四条都满足，拒得合法。
/// 判别力：注入过读故障的态上也判 Q3 那一条时这一项红。
#[test]
fn a_refusal_for_no_valid_root_on_a_read_faulted_crash_image_whose_root_ring_was_zeroed_by_hand_is_legal(
) {
    let image = image_with_every_root_ring_slot_zeroed_by_hand(&image_after_the_first_file(
        "no-valid-root-in-the-ring-zeroed-with-read-faults",
    ));
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &image,
            None,
            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::Injected,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator
        ),
        "注入过读故障的崩溃态上 Q3 那一条不判，判据为真：拒得合法"
    );
}

/// 场景：没动过的盘面（根环读得出），崩溃态没注入读故障，被拒的那次挂载读的就是这份镜像、不带故障层；拿 Q3 那一拒交给判官。
/// 预期：判据在镜像上复算为假（读阶段择得到的根按它自己的实例表不被抛弃），拒的理由是错的，没满足的恰是「判据复算为假」那一条
/// （Q3 那一条只在判据为真时才有）。
/// 判别力：判官把 Q3 那一拒的判据恒按真复算时这一项红。
#[test]
fn a_refusal_for_no_valid_root_on_a_crash_image_whose_root_ring_reads_back_is_illegal_because_its_criterion_is_false(
) {
    let image = image_after_the_first_file("no-valid-root-in-the-ring-readable");
    let selected_version = recover(&image, JournalPolicy::Consult).effective_root;
    assert!(
        selected_version.is_some(),
        "前提：根环读得出，只读恢复择得到根"
    );
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &image,
            selected_version,
            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
        ),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        },
        "判据在镜像上复算为假：拒得不合法，理由是错的"
    );
}

/// 场景：同一份读得出的盘面，崩溃态注入过读故障、被拒的那次挂载自己的读经故障层（重建那一遍与它的重读读坏是这次挂载里的瞬时读错）。
/// 预期：判据在镜像上复算为假不算不合法（与「读阶段判完之后系统配置又读不出」那一拒同一个办法，C664），盘上逐字节不变、只读读出在允许集合里，
/// 拒得合法。
/// 判别力：被拒的挂载在读故障下跑时也照 U6 判「判据复算为假」时这一项红。
#[test]
fn a_refusal_for_no_valid_root_by_a_mount_reading_through_injected_faults_is_legal_though_the_crash_image_reads_back(
) {
    let image = image_after_the_first_file("no-valid-root-in-the-ring-readable-transient");
    let selected_version = recover(&image, JournalPolicy::Consult).effective_root;
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
            &image,
            selected_version,
            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::Injected,
            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadThroughInjectedFaults,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator
        ),
        "被拒的那次挂载在读故障下跑：两遍读之间的瞬时读错复算不出，判据复算为假不算不合法"
    );
}

/// 崩溃恢复抛弃了一条根的盘面（与 `a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` 的
/// `pool_after_a_recovery_abandoned_the_third_version` 同一段脚本）：A、B（实例 1）→ 重开取号 2、写行（txg 5）、暖机（6、7）→ C（txg 8）→
/// 崩溃恢复抛弃 C（系统配置没见证 C，实例 3 写行 9、暖机 10），C 的根槽写回、又读得出。
fn image_with_an_abandoned_root(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &first,
        &[0x5A; 4100],
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    let mut devices = pool.reopen_recorded();
    let mounted = singlefs_core::mount::mount_writable(&common::parameters(), &mut devices)
        .expect("B 之后重开，可写挂载（实例 2）");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let before_the_abandoned_publish = pool.output.clone();
    let abandoned = publish_overwrite_in_process(
        &mut pool,
        &before_the_abandoned_publish,
        &[0xC3; 2500],
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(2),
    )
    .expect("C");
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned);
    pool.memory_pool()
}

/// 场景：崩溃恢复抛弃了一条根 C 的盘面，根环里有实例 0（mkfs）到实例 3 的根，最新那条根（实例 3）指着的实例表有实例 1、2 的行；
/// 崩溃态没注入读故障，被拒的那次挂载读的就是这份镜像；拿 Q3 那一拒交给判官。
/// 预期：判据在镜像上复算为假（C 按表被抛弃，别的根有效），拒的理由是错的。判官「按最新那条根的实例表判被抛弃」那半句在合法盘面上不改判据的值：
/// 写行只给比要取的号小的实例写行（`mount.rs` 的 `instance_rows_to_write`），最新那条根所在的实例在它自己的表里没有行，它按表恒不被抛弃，
/// 判据恒假——前两项断言把这一点钉住。
/// 留档（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第二行）：正推 `research/prompts/m3-batch10-code-r2-sonnet-output.md` K1-3
/// 把那半句换成「表里有该实例的行就算被抛弃」、5 条仍全绿，在合法盘面上是等价变异，这一条在它下面照样绿（推的：只在本仓写出来的实例表上成立，
/// 手工改出一张给最新那条根自己的实例写了行的表才分得开，那不是合法盘面）。换成会改行为的那一条：判官把「有一条不被抛弃的根」写成「每条都不被抛弃」
/// （`crates/mutations.tsv`「第10h批 Q3 判官把有一条有效根写成每条根都有效」），这一盘面上有被抛弃的 C，判据复算为真，最后一项红。
#[test]
fn a_refusal_for_no_valid_root_on_a_crash_image_holding_an_abandoned_root_is_illegal_because_the_newest_root_is_never_abandoned_by_its_own_table(
) {
    let image = image_with_an_abandoned_root("no-valid-root-in-the-ring-with-an-abandoned-root");
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let newest = choose_root(&image, &system_configuration).expect("择得到最新那条根");
    let table_of_the_newest_root =
        instance_table_of_root(&image, &newest).expect("最新那条根的实例表读得出");
    let immutable = &system_configuration.immutable;
    let roots_in_the_ring = readable_roots(
        &image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    assert!(
        roots_in_the_ring
            .iter()
            .any(|root| root_is_abandoned_by_the_instance_table(root, &table_of_the_newest_root)),
        "前提：环里有一条按最新那条根的实例表被抛弃的根（C）"
    );
    assert!(
        table_of_the_newest_root
            .rows
            .iter()
            .all(|row| row.instance < newest.instance)
            && !root_is_abandoned_by_the_instance_table(&newest, &table_of_the_newest_root),
        "最新那条根（实例 {}）的实例表只有更早的实例的行：{:?}；它按表不被抛弃",
        newest.instance.0,
        table_of_the_newest_root.rows
    );
    let root_of_an_instance_without_a_row = |root: &singlefs_core::root_record::RootRecord| {
        !table_of_the_newest_root
            .rows
            .iter()
            .any(|row| row.instance == root.instance)
    };
    assert!(
        roots_in_the_ring.iter().any(root_of_an_instance_without_a_row),
        "按「表里有该实例的行就算被抛弃」判，也还有一条有效根（最新那条根自己）：两种判法下判据都为假"
    );
    let selected_version = recover(&image, JournalPolicy::Consult).effective_root;
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &image,
            selected_version,
            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
        ),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        },
        "环里有被抛弃的 C、也有有效根：判据在镜像上复算为假，拒得不合法"
    );
}
