//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ② 行：被抛弃根带的 F 算进 F_生效（D16（发布语义） 已定项 1，
//! 被攻过一轮；`research/prompts/abandoned-floor-r1-main-verification.md` M2）。实现一侧 `recovery::effective_rollback_floor` 与池级 checker
//! 各算一份（两份不共用代码），两份都要把被抛弃根带的 F 算进去。
//!
//! 被抛弃根带着比各盘有效根与系统配置都高的 F，只在「系统配置里那一槽一时读不出」时的可写挂载上造得出（攻方腿 E4）；C331 取甲之后那样的
//! 可写挂载被拒，所以这里按坏镜像的造法直接造盘面：崩溃恢复抛弃 C（收口表第 43 行调查员那一形，
//! `common::abandon_the_third_version_by_a_crash_before_its_rotation`）之后，把 C 的根记录改成带 F = 4 重新编码写回——各盘有效根与系统配置里的 F 都是 0。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs"]
mod scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor;

use common::parameters;
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor::{
    pool_whose_abandoned_root_carries_a_higher_floor, FLOOR_CARRIED_BY_THE_ABANDONED_ROOT,
};
use singlefs_core::recovery::{choose_system_configuration, effective_rollback_floor};

/// 实现一侧：F_生效 取到被抛弃的 C 带的 F。改之前只看各盘最新的有效根（带 0）与系统配置（0），F_生效 = 0。
#[test]
fn the_implementations_effective_floor_counts_the_floor_an_abandoned_root_carries() {
    let pool = pool_whose_abandoned_root_carries_a_higher_floor("abandoned-floor-counts-core");
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("择系统配置");
    assert_eq!(
        effective_rollback_floor(
            &image,
            &parameters().region_devices,
            &system_configuration.immutable.sizes,
            &parameters().filesystem_identifier,
        ),
        FLOOR_CARRIED_BY_THE_ABANDONED_ROOT,
        "被抛弃的 C 带的 F 算进 F_生效"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor::the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries() {
    scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor::the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries(&mut PoolCheckerImages::left_to_the_checker_tier());
}
