//! m3-close-i718-oldslot-r3 辩方腿原型（给定状态）：C560 那几个场景交给池级 checker 的镜像，在 harness 档里自己收下来、用池级 checker 原样判一遍，
//! 打出每张镜像上判红的不变量与 I-7.18 / I-3.1 的判定，和场景里写死的期望（源码里的 `exactly_these_violated`）并排比。
//! 池级 checker 那一遍本来在 checker 档（`crash_given_states_of_the_images_harness_scenarios_hand_out.rs`），这里不碰 checker 档。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs"]
mod scenarios_of_rollback_by_a_forward_publish;

use common_images_for_the_checker_tier::PoolCheckerImages;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;

fn judge_and_print(scenario: &str, images: PoolCheckerImages) {
    let PoolCheckerImages::CollectedForTheCheckerTier(list) = images else {
        panic!("收的是 CollectedForTheCheckerTier");
    };
    for handed_out in list {
        let verdicts = check_pool_image(&handed_out.image);
        let red: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(invariant, _)| *invariant)
            .collect();
        let letter = |name: &str| match verdicts.iter().find(|(invariant, _)| *invariant == name).map(|(_, verdict)| verdict) {
            Some(InvariantVerdict::Holds) => 'G',
            Some(InvariantVerdict::Violated(_)) => 'R',
            Some(InvariantVerdict::NotApplicable(_)) => 'N',
            Some(InvariantVerdict::PartiallyJudged(_)) => 'P',
            None => '-',
        };
        println!(
            "GIVEN scenario={scenario} label={:?} red={red:?} I-7.18={} I-3.1={} I-7.19={}",
            handed_out.label,
            letter("I-7.18"),
            letter("I-3.1"),
            letter("I-7.19"),
        );
        bad_writer_variants(scenario, &handed_out.label, &handed_out.image);
    }
}

const ROOT_CHECKSUM_OFFSET: usize = 138;
const FLOOR_OFFSET: usize = 130;
const THRESHOLD_OFFSET: usize = 428;
const PIN_OFFSET: usize = 436;
const ROOT_SLOT_BYTES: usize = 512;

/// 同一张镜像上把最新根的 t 或 p 改成两份坏写者的样子（t 写成 F、p 写成 255），池级 checker 原样再判一遍：
/// 看场景已经交出去的镜像里，哪些张坏写者会被今天的严格求等判红。
fn bad_writer_variants(scenario: &str, label: &str, image: &singlefs_harness::memory_pool::MemoryPool) {
    use singlefs_core::address::DeviceOffsetInBytes;
    use singlefs_core::checksum::wide_checksum_with_field_zeroed;
    use singlefs_core::recovery::{choose_system_configuration, readable_roots, PoolReader};
    let Ok(configuration) = choose_system_configuration(image) else { return };
    let roots = readable_roots(
        image,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    );
    let Some(newest) = roots.iter().max_by_key(|root| (root.checkpoint_txg, root.instance)) else { return };
    let (device, offset) = scenarios_of_rollback_by_a_forward_publish::root_slot_position_of(newest.checkpoint_txg.0);
    let Some(original) = PoolReader::read(image, device, offset, ROOT_SLOT_BYTES) else { return };
    for (variant, threshold, pinned) in [
        ("t_as_floor", newest.rollback_floor.0, newest.pinned_ring_slot.0),
        ("p_255", newest.reclaim_threshold.0, 255u8),
    ] {
        if threshold == newest.reclaim_threshold.0 && pinned == newest.pinned_ring_slot.0 {
            println!("GIVEN_BAD scenario={scenario} label={label:?} variant={variant} same_as_honest=1");
            continue;
        }
        let mut bytes = original.clone();
        bytes[THRESHOLD_OFFSET..THRESHOLD_OFFSET + 8].copy_from_slice(&threshold.to_le_bytes());
        bytes[FLOOR_OFFSET..FLOOR_OFFSET + 8].copy_from_slice(&newest.rollback_floor.0.to_le_bytes());
        bytes[PIN_OFFSET] = pinned;
        let digest = wide_checksum_with_field_zeroed(&bytes, ROOT_SLOT_BYTES, ROOT_CHECKSUM_OFFSET);
        bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        let mut edited = image.clone();
        edited.devices.get_mut(&device).expect("那块盘").write(DeviceOffsetInBytes(offset.0), &bytes);
        let verdicts = check_pool_image(&edited);
        let seven_eighteen = match verdicts.iter().find(|(invariant, _)| *invariant == "I-7.18").map(|(_, verdict)| verdict) {
            Some(InvariantVerdict::Holds) => 'G',
            Some(InvariantVerdict::Violated(_)) => 'R',
            Some(InvariantVerdict::NotApplicable(_)) => 'N',
            _ => '-',
        };
        println!("GIVEN_BAD scenario={scenario} label={label:?} variant={variant} I-7.18={seven_eighteen} newest_txg={} t={} F={} p={}", newest.checkpoint_txg.0, newest.reclaim_threshold.0, newest.rollback_floor.0, newest.pinned_ring_slot.0);
    }
}

#[test]
fn given_states_of_the_rollback_to_the_oldest_ring_root_torn() {
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash(&mut images);
    judge_and_print("rollback_torn", images);
}

#[test]
fn given_states_of_the_rollback_to_the_oldest_ring_root_zeroed() {
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash(&mut images);
    judge_and_print("rollback_zeroed", images);
}

#[test]
fn given_states_of_the_rollback_to_the_oldest_ring_root_refused_before_writing() {
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash(&mut images);
    judge_and_print("rollback_refused", images);
}

#[test]
fn given_states_of_the_plain_overwrite_zeroing_the_oldest_ring_root() {
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    scenarios_of_rollback_by_a_forward_publish::plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced(&mut images);
    judge_and_print("plain_overwrite", images);
}
