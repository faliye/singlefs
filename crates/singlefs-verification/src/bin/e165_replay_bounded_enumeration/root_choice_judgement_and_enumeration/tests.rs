use super::*;

#[test]
fn root_slot_reading_with_the_same_txg_and_the_other_instance_disagrees_with_the_truth() {
    let material_of = |checkpoint_txg: u64, instance: u32| {
        root_material_without_records(
            instance,
            checkpoint_txg,
            NewRootSegment::off_timeline(u64::from(instance), checkpoint_txg),
        )
    };
    let root_of = |material: &RootChoicePublishMaterial| {
        RootRecord::parse_slot(&material.root_bytes, &ROOT_FILESYSTEM_IDENTIFIER)
            .expect("生成的根自证过")
    };
    let instance_one = material_of(7, 1);
    let instance_two = material_of(7, 2);
    assert!(root_slot_reading_agrees_with_the_truth(
        RootRingSlotReading::SelfVerified(root_of(&instance_one)),
        Some(&instance_one)
    ));
    assert!(
        !root_slot_reading_agrees_with_the_truth(
            RootRingSlotReading::SelfVerified(root_of(&instance_one)),
            Some(&instance_two)
        ),
        "同 txg、实例代号互换：对拍必须报不一致"
    );
    assert!(!root_slot_reading_agrees_with_the_truth(
        RootRingSlotReading::SelfVerified(root_of(&instance_one)),
        None
    ));
    assert!(root_slot_reading_agrees_with_the_truth(
        RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
        None
    ));
    assert!(!root_slot_reading_agrees_with_the_truth(
        RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
        Some(&instance_one)
    ));
    assert!(!root_slot_reading_agrees_with_the_truth(
        RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable),
        None
    ));
}

#[test]
fn the_local_root_choice_rules_pick_as_the_registered_comparisons_on_hand_made_rings() {
    let version = |checkpoint_txg: u64, instance: u32| {
        Some(RootVersion {
            checkpoint_txg,
            instance,
        })
    };
    let ring = [version(100, 2), version(100, 1), version(101, 1)];
    let picked = |rule: LocalRootChoiceRule| rule.chooses_among(&ring).expect("环里有根");
    assert_eq!(
        picked(LocalRootChoiceRule::NewestWins).describe(),
        "txg101/instance1"
    );
    assert_eq!(
        picked(LocalRootChoiceRule::InstanceFirstThenTxg).describe(),
        "txg100/instance2"
    );
    assert_eq!(
        picked(LocalRootChoiceRule::FirstSelfVerifiedWins).describe(),
        "txg100/instance2"
    );
    assert_eq!(
        picked(LocalRootChoiceRule::IgnoresTheInstance).describe(),
        "txg101/instance1"
    );
    let tie = [version(100, 2), version(100, 1)];
    assert_eq!(
        LocalRootChoiceRule::TieTakesTheLowerInstance
            .chooses_among(&tie)
            .expect("环里有根")
            .describe(),
        "txg100/instance1"
    );
    assert_eq!(
        LocalRootChoiceRule::NewestWins
            .chooses_among(&tie)
            .expect("环里有根")
            .describe(),
        "txg100/instance2"
    );
    assert_eq!(
        LocalRootChoiceRule::NewestWins.chooses_among(&[None, None]),
        None
    );
}

#[test]
fn the_landing_slot_adversarial_root_rings_have_27_states_one_of_them_without_a_self_verified_root_and_zero_kj2_violations_today(
) {
    let today = root_choice_tally_of_a_cell(
        RootChoiceDomain::AdversarialRootRing,
        RootChoiceCell {
            mounts: 0,
            user_publishes: 0,
        },
        RootChoiceImplementation::TodayThreeSteps,
        1,
    );
    assert_eq!(today.count_of(RootChoiceCounter::States), 27);
    assert_eq!(today.count_of(RootChoiceCounter::ChoseNothingStates), 1);
    assert_eq!(today.count_of(RootChoiceCounter::Kj0Violations), 1);
    assert_eq!(today.count_of(RootChoiceCounter::Kj2Judged), 26);
    assert_eq!(today.count_of(RootChoiceCounter::Kj2Violations), 0);
    assert_eq!(
        today.count_of(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
        0
    );
    assert_eq!(
        today.count_of(RootChoiceCounter::SameTxgDifferentInstanceRoots),
        0
    );
    assert_eq!(
        today.count_of(RootChoiceCounter::HigherInstanceLowerTxgRoots),
        7
    );
}

/// KL 的 27 态照登记 1.3 与 5.6 H1 摆：落点槽按读根环次序是根槽 2、9、17（txg 102、100、101），状态号三位三进制、最前的落点槽是最高位，
/// 六条根的新根段各不相同，见证串写 `empty` 或 `(txg,实例)`。
#[test]
fn the_landing_slot_adversarial_root_rings_put_each_root_in_its_landing_slot_with_six_distinct_segments(
) {
    let rings = landing_slot_adversarial_root_rings();
    assert_eq!(rings.len(), 27);
    let occupied_slots_of = |ring: &LandingSlotAdversarialRootRing| -> Vec<(usize, RootVersion)> {
        ring.root_slots
            .iter()
            .enumerate()
            .filter_map(|(slot_index, slot)| {
                slot.as_ref().map(|material| (slot_index, material.version))
            })
            .collect()
    };
    assert_eq!(rings[0].witness, "kl_landing_slots=[empty,empty,empty]");
    assert!(occupied_slots_of(&rings[0]).is_empty());
    assert_eq!(rings[5].witness, "kl_landing_slots=[empty,(100,1),(101,2)]");
    assert_eq!(
        rings[26].witness,
        "kl_landing_slots=[(102,2),(100,2),(101,2)]"
    );
    let version = |checkpoint_txg, instance| RootVersion {
        checkpoint_txg,
        instance,
    };
    assert_eq!(
        occupied_slots_of(&rings[26]),
        vec![
            (2, version(102, 2)),
            (9, version(100, 2)),
            (17, version(101, 2))
        ]
    );
    let mut segments: Vec<NewRootSegment> = Vec::new();
    for ring in &rings {
        for material in ring.root_slots.iter().flatten() {
            if !segments.contains(&material.fields.segment) {
                segments.push(material.fields.segment);
            }
        }
    }
    assert_eq!(segments.len(), 6, "六条根的新根段各不相同");
}
