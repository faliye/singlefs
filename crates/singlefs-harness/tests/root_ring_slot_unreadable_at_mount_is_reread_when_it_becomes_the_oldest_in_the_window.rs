//! 挂载那一刻读不出的根环槽，进程内成了一圈窗口里最小的那个时先重读（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定；
//! `research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表二之后那一段，攻方 K2-e `unreadable_slot_i31.rs` 的
//! `u31-s4-9-mount.out` 那一形搬到 mkfs 默认的 24 槽环上）。
//!
//! 盘面：实例 1 发到 txg 23，txg 24 那次发布崩在「记录已持久、根槽与之后的系统配置轮换没持久」：txg 24 落到的区域 0 槽 0 里留着 mkfs 写的
//! txg 0 的根。可写挂载（实例 2）那一刻区域 0 槽 0 读不出（设备报读错），挂载之后读得出。挂载按记录施加出 txg 24 那一版，根环表把那一槽记成
//! 「挂载时读不出、能住的最大 txg 是 24」。
//!
//! - 照最大 txg 算（改之前）：写 txg 47 那一版时窗口是 txg > 23，txg 25–47 是这个进程写的有效根，那一槽按 24 算、成了最小的，门槛停在 24；
//!   池级 checker 读盘，那一槽里是窗口外的 txg 0，窗口里最旧的有效根是 25 ⇒ 记账比遍历多算释放代 25 的那几个槽，I-3.1 红。
//! - E2：记 txg 47 那条根之前那一槽成了窗口里最小的那个，先重读：读出 txg 0 的根（按实例表有效、在窗口外）⇒ 那一槽当没有，门槛取 25，与 checker 对得上。
//!   txg 46 时窗口里还有 txg 23 的根，比 24 旧，不重读，那一槽仍按 24 记着。
//!
//! 回收门槛退回整个环之后（用户 2026-10-04 定 RGall，`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1），读坏的槽在
//! 回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或确认为空之前门槛不越过它（用户 2026-10-04 定，重读时机照 E2；条文还没写回，
//! 定案在 `records/2026-09-30-里程碑三开工第一轮.md`）：那一槽可能住过的最旧 txg 是 0，钉在 0，挂载那一刻起就是根环表里最小的那个，
//! 挂载之后第一次记根就重读、读出 txg 0 的根（按实例表有效），照 0 算，直到 txg 48 这个进程写掉那一槽。txg 0 引用的单元一个都不回收，
//! 挂载之后每一版那几条不变量都不红。文件名留着「成了窗口里最小的那个才重读」那一轮的叫法。
//!
//! 照能住的最大 txg 24 记（第 10 批 core 簇那一版）：txg 25–47 那几版上进程内按 24 回收，门槛越过那一槽里真住着的 txg 0，txg 0 引用的单元
//! 被回收复用，而 txg 0 是环里读得出的有效根，池级 checker 照当候选走、读到复用的槽（txg 27 起 I-2.1、I-4.8、I-3.1 红）。
//!
//! 同一份盘面上那一槽从挂载起一直读回全零（报成功、字节全 0，盘上那一槽里其实是 txg 0 的根）：读回全零的槽在回收门槛里照「自证不过」那一类
//! 钉在它可能住过的最旧 txg，只是不并进准入抬 F 的上限；进程内重读读回全零不算确认为空，照旧钉着（用户 2026-10-05 定乙，条文还没写回，
//! 依据 `records/2026-09-30-里程碑三开工第一轮.md`「第 10b 批 core 修订交回」那一条的②）。照「从没写过」不钉时（第 10b 批那一版），
//! 挂载那一刻门槛就越过 txg 0，txg 0 引用的单元当场回收。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs"]
mod scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window::a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window::a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window::a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window::a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it,
    );
}
