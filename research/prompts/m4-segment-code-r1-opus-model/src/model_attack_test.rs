
    /// 攻方腿（m4-segment-code-r1 T11）：快照个数相同、活头相同、快照抄的根不同的两版，「非空」要分得出。
    /// 建快照 S1（抄活头 A）→ 覆盖写（活头 B）——记为甲；删 S1（意图 + 最后一批）→ 建快照 S2（抄活头 B）——记为乙。
    /// 甲乙的活头都是 B、都只有一个快照、都没搬过根，只差快照抄的是 A 还是 B。
    #[test]
    fn attack_two_versions_with_one_snapshot_each_differ_when_the_snapshots_copy_different_roots() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照 S1");
        succeed(&mut model, &creation);
        let overwrite = model.answer_publish_overwrite(&[42]).expect("覆盖写");
        succeed(&mut model, &overwrite);
        let before_the_destruction = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        let intent = model.answer_destroy_snapshot(0).expect("意图");
        succeed(&mut model, &intent);
        let final_batch = model.answer_next_step_of_the_walk().expect("最后一批");
        succeed(&mut model, &final_batch);
        let recreation = model.answer_create_snapshot().expect("建快照 S2");
        succeed(&mut model, &recreation);
        let after_the_recreation = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        eprintln!("ATTACK_T11 before_the_destruction={before_the_destruction:?}");
        eprintln!("ATTACK_T11 after_the_recreation={after_the_recreation:?}");
        assert_ne!(before_the_destruction, after_the_recreation, "快照抄的根不同：非空");
    }
}
