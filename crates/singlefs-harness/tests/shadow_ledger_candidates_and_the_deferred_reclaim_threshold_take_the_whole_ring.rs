//! 原来同取一圈窗口的六处里另两处，用户 2026-10-04 定 RGall 一起退回整个环（D16（发布语义） 已定项 1；
//! `research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）：影子账的候选过滤（`mount::isolate_slots_referenced_only_by_abandoned_roots`：
//! 按实例表有效 ∧ txg ≥ F 的根都是候选，窗口外的有效根照样豁免它引用的槽）与池级 checker I-3.1（已分配统计对得上） 丁-defer 的门槛
//! （`walk::deferred_slots_reclaim_threshold`：max(F_生效, 环里最旧有效根)）。改回一圈窗口时这两处没有会红的用例
//! （判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二节 K1「影子账候选、I-3.1 丁-defer 门槛改回窗口」那一行，正推 K1-2）。
//!
//! 盘面是 S = 4 的 12 槽小环（`common_root_ring_holes`）。两形都先造一条被抛弃的根（实例 1 最新那次发布的系统配置轮换没落、下一次可写挂载时它的
//! 根槽与数据单元一时读不出，择根落到前一条根 B、实例 2 写行，之后它读得出、按实例表被抛弃），再让一次可写挂载崩在写行那次根槽写之前
//! （K1 那一类洞：txg 跳一个，洞那一槽里留着上一圈的根），之后的发布把锚往前推，洞里那条根落到一圈窗口外、仍是有效根。
//! - 丁-defer（`raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs` 那一形搬到有洞的小环上）：B 那一槽在洞里留着，
//!   F 抬到 B 与被抛弃的那条根之间（B 在 F 之下）。B 引用的实例 1 那张实例表在实例 2 写行那次被整条重写，释放代高于 F（还在 defer 里，分配器照算已分配），
//!   F 之上的有效根都不引用它，被抛弃的那条根不走、它的记录也不算被施加过。门槛取整个环是 max(F, 环里最旧有效根 B) = F，并进遍历；
//!   照一圈窗口取时门槛是窗口里最旧的有效根（高于那个释放代），那几槽不并，I-3.1 红在记账多的一边。
//!   不用 K1 那一形的「实例 1 连崩之后 F 抬到 24」：那里 txg 24 那一版由记录施加出来、照候选走（`walk::versions_applied_only_by_records`），
//!   实例 1 那张实例表由它走到，丁-defer 门槛取什么都一样，那一形钉不住门槛。
//! - 影子账：先造一条被抛弃的根——实例 1 最新的根 txg 24 那次发布的系统配置轮换没落、下一次可写挂载时它的根槽与数据单元一时读不出，择根落到 23、
//!   实例 2 写行（行 (1, 23)），之后 24 读得出、按实例表被抛弃。再连崩到 txg 36 那一槽也被跳过（24 那一槽不被盖掉）、做完一次可写挂载，
//!   再挂时影子账按候选集算：实例 1 那张实例表被被抛弃的 24 引用，也被洞里实例 1 的有效根（在窗口外）引用，豁免、不隔离；
//!   照一圈窗口滤候选时窗口外那几条不豁免，而那几槽这时还在 defer 里、分配位置着，可写挂载那一遍只隔离分配位没置着的槽，豁免不豁免都不隔离；
//!   所以判别力放在抬 F 那一串：再挂那一刻 24 的根槽读回全零（根环表里没记着它），抬 F 时照读盘取它的落点、整段隔离，不豁免就隔离那几槽。
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
#[path = "common_images_for_the_checker_tier/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs"]
mod scenarios_of_shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring::{inside_the_one_revolution_window, pool_with_the_newest_root_of_instance_one_abandoned, roots_of_the_ring, write_bytes};
use common_root_ring_holes::{
    crash_a_mount_before_its_row_root, devices_of,
    parameters_of_the_ring_hole_pools, pool_of, RingHoleDevices,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::{mount_writable, raise_rollback_floor, Mounted, ShadowLedger};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    effective_rollback_floor,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::memory_pool::MemoryPool;
use std::collections::{BTreeMap, BTreeSet};

/// 影子账那一形实例 1 覆盖写到的 txg（被抛弃的就是这一条）。
const LAST_TXG_OF_INSTANCE_ONE: u64 = 24;

/// 影子账那一形造出被抛弃的根之后连崩的次数：跳过的 txg 要盖住 24 那一槽的下一圈 txg 36（24 + 12），不让它被盖掉。
const CRASHES_BEFORE_THE_ROW_ROOT_AFTER_THE_ABANDONMENT: usize = 11;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring::raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring::raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring,
    );
}

/// 场景：实例 1 覆盖写到 24，24 那次发布被崩溃恢复抛弃（实例 2 写行 (1, 23)）；再连崩 11 次在写行根槽之前（跳过的 txg 盖住 36，24 那一槽留着）、
/// 一次可写挂载做完；再可写挂载一次，看重建分配器影子账隔离了几槽。
/// 预期（前提）：盘上只有一条被抛弃的根 24；有被它引用、也只被锚在最新有效根上的一圈窗口外的有效根引用（窗口里的有效根与现行那一版的账都不引用）的槽。
/// 影子账按候选集（按实例表有效 ∧ txg ≥ F，不按窗口滤）豁免：每块盘隔离的槽数 = 被抛弃的根引用的槽里，候选根与现行那一版的账都不引用的那几槽的跨度之和。
/// 这一条对影子账的候选过滤改回一圈窗口（`crates/mutations.tsv`「第10g批 影子账候选过滤改回一圈窗口…」）不敏感：窗口外那几条根不豁免时，
/// 它们与被抛弃的根同引用的槽还在 defer 里、分配位置着，可写挂载那一遍不隔离它们，隔离数不变。那一条变异由抬 F 那一串的下一条用例抓。
#[test]
fn a_remount_isolates_no_slot_an_abandoned_root_shares_with_valid_roots_left_in_the_holes_outside_the_one_revolution_window(
) {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut image = pool_with_the_newest_root_of_instance_one_abandoned(LAST_TXG_OF_INSTANCE_ONE);
    // 迭代上界：连崩的次数，每次一个洞；跨轮携带的是崩溃之后的盘面。
    for _ in 0..CRASHES_BEFORE_THE_ROW_ROOT_AFTER_THE_ABANDONMENT {
        let (crashed, _row_txg) = crash_a_mount_before_its_row_root(&image);
        image = crashed;
    }
    let mut devices = devices_of(&image);
    mount_writable(&parameters, &mut devices).expect("连崩之后的可写挂载做完");
    let image_before_the_remount = pool_of(&devices);

    let roots = roots_of_the_ring(&image_before_the_remount);
    let effective_floor = {
        let system_configuration =
            choose_system_configuration(&image_before_the_remount).expect("系统配置");
        let immutable = system_configuration.immutable;
        effective_rollback_floor(
            &image_before_the_remount,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
        )
    };
    let start_slots_allocated_under = |root: &RootRecord| -> Vec<(DeviceIdentity, u64, u64)> {
        allocation_records_under_root(&image_before_the_remount, root)
            .expect("这一版的账读得出")
            .into_iter()
            .filter(|record| !record.is_released)
            .map(|record| (record.device, record.slot.0, u64::from(record.span_slots)))
            .collect()
    };
    let newest_valid_root = roots
        .valid
        .iter()
        .map(|root| root.checkpoint_txg)
        .max()
        .expect("有有效根");
    let candidates: Vec<&RootRecord> = roots
        .valid
        .iter()
        .filter(|root| root.checkpoint_txg >= effective_floor)
        .collect();
    let referenced_by = |roots_referencing: &[&RootRecord]| -> BTreeSet<(DeviceIdentity, u64)> {
        roots_referencing
            .iter()
            .flat_map(|root| start_slots_allocated_under(root))
            .map(|(device, slot, _)| (device, slot))
            .collect()
    };
    let current_version = choose_root(
        &image_before_the_remount,
        &choose_system_configuration(&image_before_the_remount).expect("系统配置"),
    )
    .expect("现行那一版的根");
    let allocated_in_the_current_version: BTreeSet<(DeviceIdentity, u64)> =
        referenced_by(&[&current_version]);
    let referenced_by_candidates = referenced_by(&candidates);
    let candidates_inside_the_window: Vec<&RootRecord> = candidates
        .iter()
        .copied()
        .filter(|root| inside_the_one_revolution_window(root, newest_valid_root))
        .collect();
    let referenced_by_candidates_inside_the_window = referenced_by(&candidates_inside_the_window);
    let mut expected_isolated_per_device: BTreeMap<DeviceIdentity, u64> = BTreeMap::new();
    let mut shared_only_with_candidates_outside_the_window: BTreeSet<(DeviceIdentity, u64)> =
        BTreeSet::new();
    let mut counted: BTreeSet<(DeviceIdentity, u64)> = BTreeSet::new();
    // 迭代上界是被抛弃的根的条数乘以它们账里的记录数；没有提前出口。
    for abandoned in &roots.abandoned {
        for (device, slot, span) in start_slots_allocated_under(abandoned) {
            let key = (device, slot);
            if allocated_in_the_current_version.contains(&key) {
                continue;
            }
            if referenced_by_candidates.contains(&key) {
                if !referenced_by_candidates_inside_the_window.contains(&key) {
                    shared_only_with_candidates_outside_the_window.insert(key);
                }
                continue;
            }
            if counted.insert(key) {
                *expected_isolated_per_device.entry(device).or_insert(0) += span;
            }
        }
    }
    assert!(
        roots.abandoned.iter().map(|root| (root.instance, root.checkpoint_txg)).collect::<Vec<_>>()
            == vec![(InstanceGeneration(1), CheckpointTxg(LAST_TXG_OF_INSTANCE_ONE))]
            && !shared_only_with_candidates_outside_the_window.is_empty(),
        "前提：只有被抛弃的根 txg 24；它与窗口外的有效根同引用、窗口里的有效根与现行那一版的账都不引用的槽 {shared_only_with_candidates_outside_the_window:?}（最新有效根 {newest_valid_root:?}，F {effective_floor:?}）"
    );

    let mut devices_of_the_remount = devices_of(&image_before_the_remount);
    let remounted = mount_writable(&parameters, &mut devices_of_the_remount).expect("再可写挂载");
    let isolated_per_device: BTreeMap<DeviceIdentity, u64> = remounted
        .allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .filter(|(_, isolated)| *isolated > 0)
        .collect();
    assert_eq!(
        isolated_per_device, expected_isolated_per_device,
        "影子账按整个环的候选集豁免：窗口外的有效根与被抛弃的根同引用的 {shared_only_with_candidates_outside_the_window:?} 不隔离"
    );
}

fn read_bytes(
    devices: &RingHoleDevices,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length: usize,
) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘")
        .1
        .read_at(offset, &mut bytes)
        .expect("读内存盘");
    bytes
}

/// `image` 上被抛弃的根与它们引用、只与锚在最新有效根上的一圈窗口外的候选根（按实例表有效 ∧ txg ≥ `floor`）同引用的槽
/// （窗口里的候选根与现行那一版的账都不引用），按 (盘, 起始槽) 认。
struct SlotsSharedOnlyWithCandidatesOutsideTheWindow {
    abandoned: Vec<(InstanceGeneration, CheckpointTxg)>,
    newest_valid_root: CheckpointTxg,
    slots: BTreeSet<(DeviceIdentity, u64)>,
}

fn slots_shared_only_with_candidates_outside_the_window(
    image: &MemoryPool,
    floor: CheckpointTxg,
) -> SlotsSharedOnlyWithCandidatesOutsideTheWindow {
    let roots = roots_of_the_ring(image);
    let start_slots_allocated_under = |root: &RootRecord| -> Vec<(DeviceIdentity, u64)> {
        allocation_records_under_root(image, root)
            .expect("这一版的账读得出")
            .into_iter()
            .filter(|record| !record.is_released)
            .map(|record| (record.device, record.slot.0))
            .collect()
    };
    let newest_valid_root = roots
        .valid
        .iter()
        .map(|root| root.checkpoint_txg)
        .max()
        .expect("有有效根");
    let candidates: Vec<&RootRecord> = roots
        .valid
        .iter()
        .filter(|root| root.checkpoint_txg >= floor)
        .collect();
    let referenced_by_candidates_inside_the_window: BTreeSet<(DeviceIdentity, u64)> = candidates
        .iter()
        .filter(|root| inside_the_one_revolution_window(root, newest_valid_root))
        .flat_map(|root| start_slots_allocated_under(root))
        .collect();
    let referenced_by_candidates_outside_the_window: BTreeSet<(DeviceIdentity, u64)> = candidates
        .iter()
        .filter(|root| !inside_the_one_revolution_window(root, newest_valid_root))
        .flat_map(|root| start_slots_allocated_under(root))
        .collect();
    let allocated_in_the_current_version: BTreeSet<(DeviceIdentity, u64)> =
        start_slots_allocated_under(&roots.newest)
            .into_iter()
            .collect();
    let slots = roots
        .abandoned
        .iter()
        .flat_map(start_slots_allocated_under)
        .filter(|key| {
            referenced_by_candidates_outside_the_window.contains(key)
                && !referenced_by_candidates_inside_the_window.contains(key)
                && !allocated_in_the_current_version.contains(key)
        })
        .collect();
    SlotsSharedOnlyWithCandidatesOutsideTheWindow {
        abandoned: roots
            .abandoned
            .iter()
            .map(|root| (root.instance, root.checkpoint_txg))
            .collect(),
        newest_valid_root,
        slots,
    }
}

/// 场景：同上一条造出被抛弃的根 24 与洞里一圈窗口外的有效根（连崩 11 次、一次可写挂载做完）；再可写挂载一次，那一刻 24 的根槽读回全零、
/// 挂载交回之后原样写回；同一个进程里把 F 抬到盘上现算的生效值（F 不动，窗口外的有效根仍是候选）。
/// 抬 F 那一串的影子账里 24 在根环表里没记着（挂载那一刻那一槽读回全零，C 那一圈在那一槽也没记），照读盘取它引用的落点、整段隔离
/// （`PoolAllocator::isolate_abandoned`：回收时没有一份补得上，不看分配位）。
/// 预期（前提）：盘上只有一条被抛弃的根 24；有被它引用、也只被窗口外的候选根引用的槽。抬完之后那几槽一个都不隔离（候选按整个环豁免）。
/// 判别力：影子账的候选过滤改回一圈窗口（`crates/mutations.tsv`「第10g批 影子账候选过滤改回一圈窗口…」）时窗口外那几条根不豁免，
/// 那几槽在这里整段隔离，最后一项红。上一条用例在可写挂载那一遍取样：那几槽还在 defer 里、分配位置着，只隔离分配位没置着的槽的那一路
/// 豁免不豁免都不隔离，对这条变异不敏感。
#[test]
fn raising_the_floor_isolates_no_slot_an_abandoned_root_unrecorded_in_the_root_ring_table_shares_with_valid_roots_left_in_the_holes_outside_the_one_revolution_window(
) {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut image = pool_with_the_newest_root_of_instance_one_abandoned(LAST_TXG_OF_INSTANCE_ONE);
    // 迭代上界：连崩的次数，每次一个洞；跨轮携带的是崩溃之后的盘面。
    for _ in 0..CRASHES_BEFORE_THE_ROW_ROOT_AFTER_THE_ABANDONMENT {
        let (crashed, _row_txg) = crash_a_mount_before_its_row_root(&image);
        image = crashed;
    }
    let mut devices = devices_of(&image);
    mount_writable(&parameters, &mut devices).expect("连崩之后的可写挂载做完");

    let target = target_for_publish(
        CheckpointTxg(LAST_TXG_OF_INSTANCE_ONE),
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let root_slot_of_the_abandoned_root = read_bytes(
        &devices,
        root_slot_device,
        root_slot_offset,
        root_slot_bytes,
    );
    write_bytes(
        &mut devices,
        root_slot_device,
        root_slot_offset,
        &vec![0u8; root_slot_bytes],
    );
    let Mounted {
        mut allocator,
        current,
        ..
    } = mount_writable(&parameters, &mut devices).expect("24 的根槽读回全零时照常可写挂载");
    let PoolVersion::WithFile(mut current) = current else {
        panic!("现行那一版带文件");
    };
    assert_eq!(
        read_bytes(
            &devices,
            root_slot_device,
            root_slot_offset,
            root_slot_bytes
        ),
        vec![0u8; root_slot_bytes],
        "前提：这次挂载没写到 24 那一槽"
    );
    write_bytes(
        &mut devices,
        root_slot_device,
        root_slot_offset,
        &root_slot_of_the_abandoned_root,
    );
    let image_before_the_raise = pool_of(&devices);
    let floor = {
        let system_configuration =
            choose_system_configuration(&image_before_the_raise).expect("系统配置");
        let immutable = system_configuration.immutable;
        effective_rollback_floor(
            &image_before_the_raise,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
        )
    };
    let shared =
        slots_shared_only_with_candidates_outside_the_window(&image_before_the_raise, floor);
    assert!(
        shared.abandoned == vec![(InstanceGeneration(1), CheckpointTxg(LAST_TXG_OF_INSTANCE_ONE))]
            && !shared.slots.is_empty(),
        "前提：只有被抛弃的根 txg 24；它与窗口外的候选根同引用、窗口里的候选根与现行那一版的账都不引用的槽 {:?}（最新有效根 {:?}，F {floor:?}）",
        shared.slots,
        shared.newest_valid_root
    );

    raise_rollback_floor(
        &parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        floor,
        ShadowLedger::On,
    )
    .expect("F 抬到生效值（不高于上限）");
    let isolated_among_the_shared: Vec<(DeviceIdentity, u64)> = shared
        .slots
        .iter()
        .copied()
        .filter(|(device, slot)| {
            allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == *device)
                .expect("池里有这块盘")
                .is_isolated(SlotNumber(*slot))
        })
        .collect();
    assert_eq!(
        isolated_among_the_shared,
        Vec::<(DeviceIdentity, u64)>::new(),
        "影子账按整个环的候选集豁免：窗口外的有效根与读盘得来的被抛弃根 24 同引用的 {:?} 不隔离",
        shared.slots
    );
}
