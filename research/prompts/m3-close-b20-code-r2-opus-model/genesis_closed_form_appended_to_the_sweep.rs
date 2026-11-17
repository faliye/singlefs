
/// 攻方腿的算术核（m3-close-b20-code-r2）：只进副本。第 0 代根判据在 S = 4..=16 每一档、最大 txg 走到三圈多，与闭式逐格比：
/// 区域 r 槽 j 第一次被写的 txg 是 3j + r（区域 0 槽 0 是 3S，txg 0 不是发布）；判据为真 ⟺ 最大 txg 不小于它。
#[cfg(test)]
mod genesis_criterion_closed_form_tests {
    use super::*;
    use crate::root_ring::RootRingSlotsPerRegion;

    #[test]
    fn genesis_criterion_matches_the_closed_form_for_every_slots_per_region() {
        let mut cells_compared = 0u64;
        for declared in 4..=16u64 {
            let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(declared)
                .expect("区间里的值");
            let ring_length_in_slots = usize::try_from(3 * declared).expect("环长");
            for highest in 0..=(3 * declared * 3 + 5) {
                for region in 0..3u64 {
                    for slot in 0..declared {
                        let first_publish_into_the_slot = if region == 0 && slot == 0 {
                            3 * declared
                        } else {
                            3 * slot + region
                        };
                        assert_eq!(
                            genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached(
                                CheckpointTxg(0),
                                RootRingSlot { region, slot },
                                CheckpointTxg(highest),
                                slots_per_region,
                                ring_length_in_slots,
                            ),
                            highest >= first_publish_into_the_slot,
                            "S={declared} H={highest} 槽=({region},{slot})"
                        );
                        cells_compared += 1;
                    }
                }
            }
        }
        println!("name=genesis_closed_form cells_compared={cells_compared}");
        assert!(cells_compared > 0);
    }
}
