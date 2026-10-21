
    /// 正推腿探针（T11）：两条根的状态只差「快照抄的是哪个根」（都是活头 txg 11、各带一个快照，一个抄 txg 9 的根、一个抄 txg 11 的根），
    /// 真实现按根指针字节比是两个状态（`mount::newest_first_distinct_states` 比 `snapshot_tree_roots`）。
    /// 序列：四次覆盖写之后 建快照（txg 10）→ 覆盖写（11）→ 删快照（意图 12、最后一批 13）→ 再建快照（14）。
    /// 非空根从新到旧 14、13、11、10、9……状态各不相同，第 4 新的是 txg 10；`snapshot_roots` 被去掉时 txg 14 与 txg 11 的状态相等（都是「活头 11、一个快照」），
    /// 去重少算一个，第 4 新的变成 txg 9。
    #[test]
    fn probe_forward_two_roots_differing_only_in_the_roots_their_snapshots_copied_are_two_states_for_the_ceiling() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let overwrite = model
            .answer_publish_overwrite(&[11])
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &overwrite);
        let intent = model.answer_destroy_snapshot(0).expect("意图");
        succeed(&mut model, &intent);
        let final_batch = model.answer_next_step_of_the_walk().expect("最后一批");
        succeed(&mut model, &final_batch);
        let second_creation = model.answer_create_snapshot().expect("再建快照");
        succeed(&mut model, &second_creation);
        println!(
            "name=probe_forward_model_ceiling ceiling={:?}",
            model.rollback_floor_ceiling()
        );
        assert_eq!(
            model.rollback_floor_ceiling(),
            Some(ModelCheckpointTxg(10)),
            "非空 14、13、11、10、9：第 4 新的是 10"
        );
    }
