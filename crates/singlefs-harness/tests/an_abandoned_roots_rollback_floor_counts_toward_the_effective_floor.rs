//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ② 行：被抛弃根带的 F 算进 F_生效（D16（发布语义） 已定项 1，用户 2026-09-28 定「算进」，
//! 被攻过一轮；`research/prompts/abandoned-floor-r1-main-verification.md` M2）。实现一侧 `recovery::effective_rollback_floor` 与池级 checker
//! 各算一份（两份不共用代码），两份都要把被抛弃根带的 F 算进去。
//!
//! 被抛弃根带着比各盘有效根与系统配置都高的 F，只在「系统配置里那一槽一时读不出」时的可写挂载上造得出（攻方腿 E4）；C331 取甲之后那样的
//! 可写挂载被拒，所以这里按坏镜像的造法直接造盘面：崩溃恢复抛弃 C（收口表第 43 行调查员那一形，
//! `common::abandon_the_third_version_by_a_crash_before_its_rotation`）之后，把 C 的根记录改成带 F = 4 重新编码写回——各盘有效根与系统配置里的 F 都是 0。
mod common;

use common::{
    abandon_the_third_version_by_a_crash_before_its_rotation, build_pool, parameters,
    publish_overwrite_in_process, system_configuration_slots_of_every_device, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::recovery::{choose_system_configuration, effective_rollback_floor};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::TransactionOutput;

/// 改写之后 C 带的 F：高于 0（各盘有效根与系统配置里的 F），不高于 C 自己的 txg 5。
const FLOOR_CARRIED_BY_THE_ABANDONED_ROOT: CheckpointTxg = CheckpointTxg(4);

fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
    let previous = pool.output.clone();
    let output = publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// A、B、C（实例 1）→ 崩溃恢复抛弃 C（落到 B、实例 2 写行与暖机）→ C 的根记录改带 F = 4。
fn pool_whose_abandoned_root_carries_a_higher_floor(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite(&mut pool, &content_seeded_by(4100, 3));
    let system_configuration_before_the_third = system_configuration_slots_of_every_device(&pool);
    let third = overwrite(&mut pool, &content_seeded_by(2500, 11));
    abandon_the_third_version_by_a_crash_before_its_rotation(
        &mut pool,
        &third,
        &system_configuration_before_the_third,
    );
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        third.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        target,
        publish_parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    let devices = pool.devices.as_mut().expect("镜像开着");
    let (_, device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == root_slot_device)
        .expect("池里有这块盘");
    let mut slot = vec![0u8; root_slot_bytes];
    device.read_at(root_slot_offset, &mut slot).expect("读 C 的根槽");
    let mut abandoned_root = RootRecord::parse_slot(&slot, &publish_parameters.filesystem_identifier)
        .expect("C 的根自证得过");
    assert_eq!(abandoned_root.checkpoint_txg, CheckpointTxg(5), "根槽里住的是 C");
    assert_eq!(abandoned_root.rollback_floor, CheckpointTxg(0));
    abandoned_root.rollback_floor = FLOOR_CARRIED_BY_THE_ABANDONED_ROOT;
    device
        .write_at(
            root_slot_offset,
            &abandoned_root.to_slot(root_slot_bytes),
            WriteDurability::Plain,
        )
        .expect("C 的根改带 F = 4 写回");
    pool
}

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

/// 池级 checker 一侧：F_生效 取到 4，高于最新根带的 F（0）——I-3.1（已分配统计对得上） 那一格按「最新根带的 F 低于 F 生效值」报不适用。
/// 改之前 checker 的 F_生效 = 0 = 最新根带的 F，I-3.1 照常判。
#[test]
fn the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries() {
    let pool = pool_whose_abandoned_root_carries_a_higher_floor("abandoned-floor-counts-checker");
    let verdict = check_pool_image(&pool.memory_pool())
        .into_iter()
        .find_map(|(invariant, verdict)| (invariant == "I-3.1").then_some(verdict))
        .expect("checker 报 I-3.1");
    assert!(
        matches!(&verdict, InvariantVerdict::NotApplicable(reason) if reason.contains("最新根带的 F 低于 F 生效值")),
        "checker 的 F_生效 取到被抛弃根带的 4、高于最新根带的 0：{verdict:?}"
    );
}
