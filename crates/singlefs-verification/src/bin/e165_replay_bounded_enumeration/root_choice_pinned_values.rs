//! 择根合判一族的钉值（登记第七节乙类，出自 `anchors_r3_k.py` 独立生成器；不符走 TV4）与本地常量回比。

use super::*;

// ── 钉绝对值（登记第七节乙类：出自 `anchors_r3_k.py` 独立生成器，常量写在断言右边；不符走 TV4）

#[derive(Clone, Copy, Debug)]
pub(super) struct PinnedRootChoiceCell {
    pub(super) domain: RootChoiceDomain,
    pub(super) mounts: usize,
    pub(super) user_publishes: usize,
    pub(super) states: u64,
    pub(super) confirmed_states: u64,
    first_self_verified_root_is_not_the_newest: u64,
    same_txg_different_instance_roots: u64,
    higher_instance_lower_txg_roots: u64,
    two_or_more_instances: u64,
    ideal_recovery_applied: u64,
    pub(super) above_chosen_root_maximum: usize,
    pub(super) largest_counter: u64,
    /// KF1（所选根取版本最小的根）下 KJ1、KJ2 必须读到的数。
    pub(super) oldest_root_kj1: u64,
    pub(super) oldest_root_kj2: u64,
}

pub(super) fn pinned_root_choice_cell(
    domain: RootChoiceDomain,
    cell: RootChoiceCell,
) -> Option<&'static PinnedRootChoiceCell> {
    PINNED_ROOT_CHOICE_CELLS.iter().find(|pinned| {
        pinned.domain == domain
            && pinned.mounts == cell.mounts
            && pinned.user_publishes == cell.user_publishes
    })
}

/// KL 上各种本地择根序的 KJ2 违例数（第 7 次重跑登记 `research/prompts/e165-r7-prereg.md` 第七节乙类 B2：今天的序 0、平局取低 0、
/// 实例代号为主 6、比较不看实例 0、取第一个有效者 4），出自那份登记第十三节的 `anchors_e165_r7_ka.py`（`new_ka_three_txgs` 那两行）。
pub(super) const fn pinned_root_choice_adversarial_kj2_violations(
    rule: LocalRootChoiceRule,
) -> u64 {
    match rule {
        LocalRootChoiceRule::NewestWins => 0,
        LocalRootChoiceRule::TieTakesTheLowerInstance => 0,
        LocalRootChoiceRule::InstanceFirstThenTxg => 6,
        LocalRootChoiceRule::IgnoresTheInstance => 0,
        LocalRootChoiceRule::FirstSelfVerifiedWins => 4,
    }
}

/// KL 的形状（第 7 次重跑登记第七节乙类 B1，出自那份登记第十三节的 `anchors_e165_r7_ka.py` 与 `anchors_e165_r7_kl_counts.py`）：
/// 27 态，其中根环全空 1（`choose_root` 交回空，KJ0 在这一态记一条）、同 txg 不同实例同在 0、实例代号更高而 txg 更低同在 7、
/// 两个以上实例 12、读根环次序里第一个自证过的根不是版本最大那条 4。
const PINNED_ROOT_CHOICE_ADVERSARIAL_STATES: u64 = 27;
const PINNED_ROOT_CHOICE_ADVERSARIAL_RING_EMPTY_STATES: u64 = 1;
const PINNED_ROOT_CHOICE_ADVERSARIAL_SAME_TXG_DIFFERENT_INSTANCE_STATES: u64 = 0;
const PINNED_ROOT_CHOICE_ADVERSARIAL_HIGHER_INSTANCE_LOWER_TXG_STATES: u64 = 7;
const PINNED_ROOT_CHOICE_ADVERSARIAL_TWO_OR_MORE_INSTANCES_STATES: u64 = 12;
const PINNED_ROOT_CHOICE_ADVERSARIAL_FIRST_ROOT_NOT_THE_NEWEST_STATES: u64 = 4;

/// KL 的形状逐格等于登记第七节乙类 B1（这些量只看生成器与真相，与被测实现无关）；r3、r4 两档跑 KL 都过这里，不符走作废条款 TV4（第 7 次重跑登记 LV1）。
pub(super) fn assert_the_pinned_landing_slot_adversarial_shape(tally: &RootChoiceTally) {
    for (counter, pinned, what) in [
        (
            RootChoiceCounter::States,
            PINNED_ROOT_CHOICE_ADVERSARIAL_STATES,
            "状态数",
        ),
        (
            RootChoiceCounter::StatesWithoutASelfVerifiedRoot,
            PINNED_ROOT_CHOICE_ADVERSARIAL_RING_EMPTY_STATES,
            "根环全空的态数",
        ),
        (
            RootChoiceCounter::SameTxgDifferentInstanceRoots,
            PINNED_ROOT_CHOICE_ADVERSARIAL_SAME_TXG_DIFFERENT_INSTANCE_STATES,
            "同 txg 不同实例同在的态数",
        ),
        (
            RootChoiceCounter::HigherInstanceLowerTxgRoots,
            PINNED_ROOT_CHOICE_ADVERSARIAL_HIGHER_INSTANCE_LOWER_TXG_STATES,
            "实例代号更高而 txg 更低同在的态数",
        ),
        (
            RootChoiceCounter::TwoOrMoreInstancesInTheRing,
            PINNED_ROOT_CHOICE_ADVERSARIAL_TWO_OR_MORE_INSTANCES_STATES,
            "根环里有两个以上实例的态数",
        ),
        (
            RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest,
            PINNED_ROOT_CHOICE_ADVERSARIAL_FIRST_ROOT_NOT_THE_NEWEST_STATES,
            "读根环次序里第一个自证过的根不是版本最大那条的态数",
        ),
    ] {
        assert_eq!(
            tally.count_of(counter),
            pinned,
            "TV4：KL {what}（登记 research/prompts/e165-r7-prereg.md 第七节乙类 B1）"
        );
    }
}

/// 每个跑到的合法历史格，装置数出的状态数与几样计数等于独立生成器的数；不符走作废条款 TV4（这些量只看生成器与真相，与被测实现无关）。
pub(super) fn assert_the_pinned_root_choice_cell(
    domain: RootChoiceDomain,
    cell: RootChoiceCell,
    tally: &RootChoiceTally,
) {
    let pinned = pinned_root_choice_cell(domain, cell)
        .expect("每个跑到的合法历史格都在登记第七节乙类的表里");
    let place = format!(
        "域 {} M = {} U = {}",
        domain.label(),
        cell.mounts,
        cell.user_publishes
    );
    assert_eq!(
        tally.count_of(RootChoiceCounter::States),
        pinned.states,
        "TV4：{place} 状态数"
    );
    assert_eq!(
        tally.count_of(RootChoiceCounter::ConfirmedStates),
        pinned.confirmed_states,
        "TV4：{place} 有已确认发布的态数"
    );
    assert_eq!(
        tally.count_of(RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest),
        pinned.first_self_verified_root_is_not_the_newest,
        "TV4：{place} 读根环次序里第一个自证过的根不是版本最大那条的态数"
    );
    assert_eq!(
        tally.count_of(RootChoiceCounter::SameTxgDifferentInstanceRoots),
        pinned.same_txg_different_instance_roots,
        "TV4：{place} 根环里同 txg 不同实例同在的态数"
    );
    assert_eq!(
        tally.count_of(RootChoiceCounter::HigherInstanceLowerTxgRoots),
        pinned.higher_instance_lower_txg_roots,
        "TV4：{place} 实例代号更高而 txg 更低的一对根同在的态数"
    );
    assert_eq!(
        tally.count_of(RootChoiceCounter::TwoOrMoreInstancesInTheRing),
        pinned.two_or_more_instances,
        "TV4：{place} 根环里有两个以上实例的根的态数"
    );
    assert_eq!(
        tally.count_of(RootChoiceCounter::IdealRecoveryApplied),
        pinned.ideal_recovery_applied,
        "TV4：{place} 理想恢复施加过至少一次发布的态数"
    );
    assert_eq!(
        tally.count_of(RootChoiceCounter::StatesWithoutASelfVerifiedRoot),
        0,
        "TV4：{place} 根环里没有一条自证过的根的态数（合法历史里第 0 代根至少还有一份）"
    );
    assert_eq!(
        tally.above_newest_root_maximum, pinned.above_chosen_root_maximum,
        "TV4：{place} 真相里版本最大的根之上同实例的记录条数的最大值"
    );
    assert_eq!(
        tally.largest_counter, pinned.largest_counter,
        "TV4：{place} 盘上读得出的最大计数器"
    );
}

/// 本地常量逐个对着实现侧的同名常量与函数回比（登记第三节回比表；不符走停机条款 TH2）。
pub(super) fn cross_check_root_choice_local_constants_against_the_implementation() {
    assert_eq!(
        ROOT_CHOICE_REGION_COUNT,
        singlefs_format::ROOT_RING_REGIONS,
        "TH2：根环区域数"
    );
    assert_eq!(
        ROOT_CHOICE_BASE_SLOT,
        singlefs_format::ROOT_RING_BASE_SLOT,
        "TH2：根环基槽"
    );
    assert_eq!(
        ROOT_CHOICE_PRIME_STEP,
        singlefs_format::ROOT_RING_PRIME_STEP,
        "TH2：区域步长"
    );
    assert_eq!(
        ROOT_CHOICE_CHUNK_BYTES,
        singlefs_format::ROOT_RING_CHUNK_BYTES,
        "TH2：区域 chunk 字节数"
    );
    for region in 0..ROOT_CHOICE_REGION_COUNT {
        assert_eq!(
            root_choice_region_start_in_bytes(region),
            singlefs_core::root_ring::region_start(region).0,
            "TH2：本地算的区域起点与 root_ring::region_start 不等（区域 {region}）"
        );
    }
    assert_eq!(
        ROOT_CHOICE_REGION_DEVICES,
        singlefs_format::ROOT_RING_REGION_DEVICES,
        "TH2：根环区域的设备归属（格式常量）"
    );
    assert_eq!(
        ROOT_CHOICE_REGION_DEVICES.map(DeviceIdentity),
        singlefs_core::make_filesystem::FIRST_VERSION_REGION_DEVICES,
        "TH2：根环区域的设备归属（mkfs 第一版）"
    );
    assert_eq!(
        ROOT_CHOICE_SLOTS_PER_REGION_WIDE,
        RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
        "TH2：mkfs 取的每区槽数"
    );
    assert_eq!(
        ROOT_CHOICE_SLOTS_PER_REGION_NARROW,
        singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        "TH2：每区槽数下限"
    );
    assert_eq!(
        ROOT_CHOICE_MKFS_INSTANCE,
        singlefs_core::make_filesystem::MKFS_INSTANCE_GENERATION.0,
        "TH2：mkfs 第 0 代根的实例代号"
    );
    assert!(
        u64::try_from(ROOT_CHOICE_ROOT_SLOT_BYTES).expect("512 装得进 u64")
            >= singlefs_format::ROOT_RECORD_BYTES,
        "TH2：根槽宽装得下根记录"
    );
    assert_eq!(
        u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES),
        singlefs_format::FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
        "TH2：固定结构槽距"
    );
    for domain in RootChoiceDomain::ALL {
        for checkpoint_txg in 0..=40 {
            assert_the_local_root_slot_arithmetic_against_the_implementation(
                *domain,
                checkpoint_txg,
            );
        }
    }
    // 新池第一次挂载：写行 txg 1、暖机 txg 2（开销发布两次），第一个用户发布 txg 3（登记第七节甲类、第三节回比表第 5 行）。
    let first_mount_plan = root_choice_warm_up_plan(1);
    assert_eq!(
        u64::try_from(first_mount_plan.len()).expect("开销发布不超过 4 次"),
        singlefs_format::WARM_UP_EMPTY_PUBLISHES,
        "TH2 / TF9：新池第一次挂载的开销发布次数与 WARM_UP_EMPTY_PUBLISHES 不等"
    );
    assert_eq!(
        first_mount_plan,
        [1, 2],
        "TF9：新池第一次挂载写行 txg 1、暖机 txg 2"
    );
    assert_eq!(
        first_mount_plan.last().expect("至少一次开销发布") + 1,
        singlefs_format::NEW_POOL_FILE_CREATION_TXG,
        "TH2 / TF9：新池第一个用户发布的 txg 与 NEW_POOL_FILE_CREATION_TXG 不等"
    );
}
const PINNED_ROOT_CHOICE_CELLS: [PinnedRootChoiceCell; 36] = [
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 0,
        states: 10,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 6,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 6,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 2,
        oldest_root_kj1: 0,
        oldest_root_kj2: 6,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 1,
        states: 39,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 39,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 39,
        ideal_recovery_applied: 21,
        above_chosen_root_maximum: 2,
        largest_counter: 4,
        oldest_root_kj1: 0,
        oldest_root_kj2: 39,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 2,
        states: 117,
        confirmed_states: 117,
        first_self_verified_root_is_not_the_newest: 117,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 117,
        ideal_recovery_applied: 63,
        above_chosen_root_maximum: 2,
        largest_counter: 6,
        oldest_root_kj1: 117,
        oldest_root_kj2: 117,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 3,
        states: 351,
        confirmed_states: 351,
        first_self_verified_root_is_not_the_newest: 351,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 351,
        ideal_recovery_applied: 189,
        above_chosen_root_maximum: 2,
        largest_counter: 8,
        oldest_root_kj1: 351,
        oldest_root_kj2: 351,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 1,
        user_publishes: 4,
        states: 1053,
        confirmed_states: 1053,
        first_self_verified_root_is_not_the_newest: 1053,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 1053,
        ideal_recovery_applied: 567,
        above_chosen_root_maximum: 2,
        largest_counter: 10,
        oldest_root_kj1: 1053,
        oldest_root_kj2: 1053,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 0,
        states: 125,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 109,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 109,
        ideal_recovery_applied: 57,
        above_chosen_root_maximum: 1,
        largest_counter: 4,
        oldest_root_kj1: 0,
        oldest_root_kj2: 109,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 1,
        states: 780,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 780,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 780,
        ideal_recovery_applied: 411,
        above_chosen_root_maximum: 2,
        largest_counter: 6,
        oldest_root_kj1: 0,
        oldest_root_kj2: 780,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 2,
        states: 4401,
        confirmed_states: 2880,
        first_self_verified_root_is_not_the_newest: 4401,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 4401,
        ideal_recovery_applied: 2376,
        above_chosen_root_maximum: 2,
        largest_counter: 9,
        oldest_root_kj1: 2880,
        oldest_root_kj2: 4401,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 3,
        states: 16281,
        confirmed_states: 16281,
        first_self_verified_root_is_not_the_newest: 16281,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 16281,
        ideal_recovery_applied: 8694,
        above_chosen_root_maximum: 2,
        largest_counter: 11,
        oldest_root_kj1: 16281,
        oldest_root_kj2: 16281,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 2,
        user_publishes: 4,
        states: 62127,
        confirmed_states: 62127,
        first_self_verified_root_is_not_the_newest: 62127,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 62127,
        ideal_recovery_applied: 33210,
        above_chosen_root_maximum: 2,
        largest_counter: 13,
        oldest_root_kj1: 62127,
        oldest_root_kj2: 62127,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 0,
        states: 1480,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 1416,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 1416,
        ideal_recovery_applied: 741,
        above_chosen_root_maximum: 1,
        largest_counter: 7,
        oldest_root_kj1: 0,
        oldest_root_kj2: 1416,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 1,
        states: 13770,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 13770,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 13770,
        ideal_recovery_applied: 7266,
        above_chosen_root_maximum: 2,
        largest_counter: 9,
        oldest_root_kj1: 0,
        oldest_root_kj2: 13770,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 2,
        states: 92565,
        confirmed_states: 46890,
        first_self_verified_root_is_not_the_newest: 92565,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 92565,
        ideal_recovery_applied: 49068,
        above_chosen_root_maximum: 2,
        largest_counter: 12,
        oldest_root_kj1: 46890,
        oldest_root_kj2: 92565,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 3,
        states: 512514,
        confirmed_states: 453195,
        first_self_verified_root_is_not_the_newest: 512514,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 512514,
        ideal_recovery_applied: 274563,
        above_chosen_root_maximum: 2,
        largest_counter: 14,
        oldest_root_kj1: 453195,
        oldest_root_kj2: 512514,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::BigRing,
        mounts: 3,
        user_publishes: 4,
        states: 2225961,
        confirmed_states: 2225961,
        first_self_verified_root_is_not_the_newest: 2225961,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 2225961,
        ideal_recovery_applied: 1186326,
        above_chosen_root_maximum: 2,
        largest_counter: 16,
        oldest_root_kj1: 2225961,
        oldest_root_kj2: 2225961,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 1,
        user_publishes: 0,
        states: 10,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 6,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 6,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 2,
        oldest_root_kj1: 0,
        oldest_root_kj2: 6,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 1,
        user_publishes: 1,
        states: 39,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 39,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 39,
        ideal_recovery_applied: 21,
        above_chosen_root_maximum: 2,
        largest_counter: 4,
        oldest_root_kj1: 0,
        oldest_root_kj2: 39,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 1,
        user_publishes: 2,
        states: 117,
        confirmed_states: 117,
        first_self_verified_root_is_not_the_newest: 117,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 117,
        ideal_recovery_applied: 63,
        above_chosen_root_maximum: 2,
        largest_counter: 6,
        oldest_root_kj1: 117,
        oldest_root_kj2: 117,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 2,
        user_publishes: 0,
        states: 125,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 109,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 109,
        ideal_recovery_applied: 57,
        above_chosen_root_maximum: 1,
        largest_counter: 4,
        oldest_root_kj1: 0,
        oldest_root_kj2: 109,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 2,
        user_publishes: 1,
        states: 780,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 780,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 780,
        ideal_recovery_applied: 411,
        above_chosen_root_maximum: 2,
        largest_counter: 6,
        oldest_root_kj1: 0,
        oldest_root_kj2: 780,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::SmallRingNine,
        mounts: 2,
        user_publishes: 2,
        states: 4401,
        confirmed_states: 2880,
        first_self_verified_root_is_not_the_newest: 4401,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 4401,
        ideal_recovery_applied: 2376,
        above_chosen_root_maximum: 2,
        largest_counter: 9,
        oldest_root_kj1: 2880,
        oldest_root_kj2: 4401,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 0,
        states: 10,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 6,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 6,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 2,
        oldest_root_kj1: 0,
        oldest_root_kj2: 6,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 1,
        states: 5,
        confirmed_states: 0,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 3,
        oldest_root_kj1: 0,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 2,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 4,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 3,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 5,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 4,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 6,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 5,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 7,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 6,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 8,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 7,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 9,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 8,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 10,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 9,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 5,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 11,
        oldest_root_kj1: 5,
        oldest_root_kj2: 5,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 10,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 4,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 4,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 12,
        oldest_root_kj1: 4,
        oldest_root_kj2: 4,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 11,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 1,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 0,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 13,
        oldest_root_kj1: 0,
        oldest_root_kj2: 0,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 12,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 0,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 14,
        oldest_root_kj1: 0,
        oldest_root_kj2: 0,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 13,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 0,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 15,
        oldest_root_kj1: 0,
        oldest_root_kj2: 0,
    },
    PinnedRootChoiceCell {
        domain: RootChoiceDomain::WrappingRootRing,
        mounts: 1,
        user_publishes: 14,
        states: 5,
        confirmed_states: 5,
        first_self_verified_root_is_not_the_newest: 5,
        same_txg_different_instance_roots: 0,
        higher_instance_lower_txg_roots: 0,
        two_or_more_instances: 0,
        ideal_recovery_applied: 3,
        above_chosen_root_maximum: 1,
        largest_counter: 16,
        oldest_root_kj1: 0,
        oldest_root_kj2: 0,
    },
];

#[cfg(test)]
mod tests;
