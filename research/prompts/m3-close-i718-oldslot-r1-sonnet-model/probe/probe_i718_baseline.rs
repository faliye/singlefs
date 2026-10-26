//! 调查用探针（只在草稿副本里）：跑 C558 那两个场景，收它们交给 checker 档的镜像，在 CPU 上逐张跑池级 checker，打印红的不变量与 I-3.1、I-7.18、I-7.19 的判定。
#![allow(dead_code, unused_imports)]
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs"]
mod scenarios_of_rollback_by_a_forward_publish;

use common_images_for_the_checker_tier::PoolCheckerImages;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;

fn judge(name: &str, scenario: impl FnOnce(&mut PoolCheckerImages)) {
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    scenario(&mut images);
    let collected = images.into_collected();
    println!("PROBE scenario={name} images={}", collected.len());
    for image in collected {
        assert!(image.writes.is_empty(), "只交单张镜像");
        let verdicts = check_pool_image(&image.image);
        let violated: Vec<(&str, String)> = verdicts
            .iter()
            .filter_map(|(invariant, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
                _ => None,
            })
            .collect();
        let named: Vec<(&str, &InvariantVerdict)> = verdicts
            .iter()
            .filter(|(invariant, _)| ["I-3.1", "I-7.18", "I-7.19"].contains(invariant))
            .map(|(invariant, verdict)| (*invariant, verdict))
            .collect();
        let configuration = singlefs_core::recovery::choose_system_configuration(&image.image).expect("系统配置");
        let mut ring = singlefs_core::recovery::readable_roots(
            &image.image,
            &configuration.immutable.region_devices,
            &configuration.immutable.sizes,
            &configuration.immutable.filesystem_identifier,
        );
        ring.sort_by_key(|root| (root.checkpoint_txg, root.instance));
        let newest = ring.last().expect("环里有根");
        println!(
            "PROBE_ROOTS image={} ring_txgs={:?} newest=(实例 {}, txg {}) F={} t={} p={:?}",
            image.label,
            ring.iter().map(|root| root.checkpoint_txg.0).collect::<Vec<_>>(),
            newest.instance.0, newest.checkpoint_txg.0, newest.rollback_floor.0, newest.reclaim_threshold.0, newest.pinned_ring_slot
        );
        println!(
            "PROBE_IMAGE scenario={name} image={} expected_exactly={:?} violated={violated:?} named={named:?}",
            image.label, image.expected.exactly_violated
        );
    }
}

#[test]
fn probe_plain() {
    judge(
        "plain",
        scenarios_of_rollback_by_a_forward_publish::plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced,
    );
}

#[test]
fn probe_torn() {
    judge(
        "torn",
        scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash,
    );
}

#[test]
fn probe_zeroed() {
    judge(
        "zeroed",
        scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash,
    );
}
