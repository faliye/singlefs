
// ───────── 草稿目录里的矩阵（正推腿 m4-c860-r1，不进仓）：每个世界打一行，变体由环境变量 SONNET_RECOGNITION 选 ─────────

mod sonnet_matrix {
    use super::*;
    use crate::common_deletion_in_progress::{
        append_a_copy_of_the_first_tombstone_of_the_newest_root, change_the_first_tombstone_of_the_newest_root,
        drop_the_record_of_the_first_file_and_keep_its_data_units, image_after_the_first_batch_of_a_three_unit_file,
        image_after_the_intent_of_a_three_unit_file, object_birth_of, pool_with_a_three_unit_file,
        publish_new_inodes_and_advance_the_pool, publish_the_intent, publish_the_intent_of,
        publish_the_next_step_in_batches_of_one, set_the_nlink_of_the_queued_first_file_back_to_one,
        set_tombstone_field, tombstone_field,
    };
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn variant() -> String {
        let v = std::env::var("SONNET_RECOGNITION").unwrap_or_default();
        if v.is_empty() { "jia".to_string() } else { v }
    }

    fn line(world: &str, image: &MemoryPool) {
        let all = check_pool_image(image);
        let mut i113 = String::new();
        let mut others: Vec<String> = Vec::new();
        for (invariant, verdict) in &all {
            if *invariant == SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX {
                i113 = match verdict {
                    InvariantVerdict::Holds => "成立".to_string(),
                    InvariantVerdict::Violated(detail) => {
                        format!("违例[{}]", detail.chars().take(160).collect::<String>())
                    }
                    InvariantVerdict::NotApplicable(reason) => {
                        format!("不适用[{}]", reason.chars().take(80).collect::<String>())
                    }
                    InvariantVerdict::PartiallyJudged(_) => "部分没判".to_string(),
                };
            } else if matches!(verdict, InvariantVerdict::Violated(_)) {
                others.push(format!("{invariant}"));
            }
        }
        println!("MATRIX variant={} world={} I-1.13={} other_violated={:?}", variant(), world, i113, others);
    }

    fn world(name: &str, build: impl FnOnce() -> MemoryPool) {
        match catch_unwind(AssertUnwindSafe(build)) {
            Ok(image) => line(name, &image),
            Err(_) => println!("MATRIX variant={} world={} 造不出（构造 panic）", variant(), name),
        }
    }

    fn mirror_image() -> MemoryPool {
        let mut pool = pool_with_new_inodes("sonnet-mirror", NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER);
        let intent = unlink_to_the_end(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
        let right_leaf = &intent.inode_leaf_containers[1];
        let mut image = pool.memory_pool();
        erase_every_record_of_the_container(&mut image, &pool.output.retirement_containers[0].pointer);
        insert_the_entry_into_the_newest_mapping_leaf(
            &mut image,
            &build_mapping_entry(&mapping_key_for_node(UNIT_CLASS_PACKED, right_leaf.pointer), right_leaf.pointer.locations),
        );
        image
    }

    #[test]
    #[ignore = "草稿目录里的观测矩阵"]
    fn matrix() {
        // ── 合法世界 ──
        world("L01 意图之后(三单元)", || image_after_the_intent_of_a_three_unit_file("sm-l01"));
        world("L02 第一批之后(三单元)", || image_after_the_first_batch_of_a_three_unit_file("sm-l02"));
        world("L03 删一半(三单元:意图+一批)旁边有活着的空 inode 2", || {
            let mut pool = pool_with_a_three_unit_file("sm-l03");
            publish_new_inodes_and_advance_the_pool(&mut pool, 1);
            publish_the_intent(&mut pool);
            publish_the_next_step_in_batches_of_one(&mut pool);
            pool.memory_pool()
        });
        world("L04 先把空 inode 2 删到底、再开始删三单元的 1 号(意图之后)", || {
            let mut pool = pool_with_a_three_unit_file("sm-l04");
            publish_new_inodes_and_advance_the_pool(&mut pool, 1);
            unlink_to_the_end(&mut pool, 2);
            publish_the_intent(&mut pool);
            pool.memory_pool()
        });
        world("L05 先把 1 号删到底、再开始删空 inode 2(意图之后)", || {
            let mut pool = pool_with_a_three_unit_file("sm-l05");
            publish_new_inodes_and_advance_the_pool(&mut pool, 1);
            publish_the_intent(&mut pool);
            for _ in 0..3 { publish_the_next_step_in_batches_of_one(&mut pool); }
            publish_the_intent_of(&mut pool, 2);
            pool.memory_pool()
        });
        world("L06 右片叶只装一条删除中记录(234 意图之后、叶未退役)", || {
            let mut pool = pool_with_new_inodes("sm-l06", NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER);
            publish_the_intent_of(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
            pool.memory_pool()
        });
        world("L07 右片叶删空退役(234 删到底)", || {
            pool_whose_right_leaf_retired_after_the_unlink_of("sm-l07", THE_ONLY_INODE_OF_THE_RIGHT_LEAF).memory_pool()
        });
        world("L08 左片吸收右片(2 号删到底)", || {
            pool_whose_right_leaf_retired_after_the_unlink_of("sm-l08", AN_INODE_WHOSE_UNLINK_ABSORBS_THE_RIGHT_LEAF).memory_pool()
        });
        world("L09 回退到删除中途(三单元:意图+一批后回退到意图那版)", || {
            let mut pool = pool_with_a_three_unit_file("sm-l09");
            let intent = publish_the_intent(&mut pool);
            publish_the_next_step_in_batches_of_one(&mut pool);
            roll_back_to(&mut pool, &intent);
            pool.memory_pool()
        });
        world("L10 回退越过整个删除(234 删到底后回退到删之前)", || {
            let mut pool = pool_with_new_inodes("sm-l10", NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER);
            let before = pool.output.clone();
            unlink_to_the_end(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
            roll_back_to(&mut pool, &before);
            pool.memory_pool()
        });
        world("L11 退役发生在回退抛弃的那段(234 删到底后回退到意图那版)", || {
            let mut pool = pool_with_new_inodes("sm-l11", NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER);
            let intent = unlink_to_the_end(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
            roll_back_to(&mut pool, &intent);
            pool.memory_pool()
        });
        world("L12 快照下删(234 删到底前建快照)", || {
            let mut pool = pool_with_new_inodes("sm-l12", NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER);
            let snapshot_version = crate::common_snapshot_prefix::create_snapshot(&mut pool);
            pool.output = snapshot_version;
            unlink_to_the_end(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
            pool.memory_pool()
        });
        world("L13 快照下删到一半(三单元:建快照后意图+一批)", || {
            let mut pool = pool_with_a_three_unit_file("sm-l13");
            let snapshot_version = crate::common_snapshot_prefix::create_snapshot(&mut pool);
            pool.output = snapshot_version;
            publish_the_intent(&mut pool);
            publish_the_next_step_in_batches_of_one(&mut pool);
            pool.memory_pool()
        });
        world("L14 回退抛弃段里诞生又被删的对象(回退到删之前后又新建)", || {
            let mut pool = pool_with_new_inodes("sm-l14", 2);
            let base = pool.output.clone();
            publish_new_inodes_and_advance_the_pool(&mut pool, 1);
            unlink_to_the_end(&mut pool, 4);
            roll_back_to(&mut pool, &base);
            pool.memory_pool()
        });
        // ── 坏世界 ──
        world("X01 C860 镜像(退役记录抹掉+退役叶最后一版映射条目补回)", mirror_image);
        world("X10 C860 镜像,但镜像做在根环转过一圈之后(退役死亡 txg 的父版已不在回退候选集)", || {
            let mut pool = pool_with_new_inodes("sm-x10", NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER);
            let intent = unlink_to_the_end(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
            let right_leaf = &intent.inode_leaf_containers[1];
            crate::common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
            let mut image = pool.memory_pool();
            let retirement = pool.output.retirement_containers[0].pointer;
            erase_every_record_of_the_container(&mut image, &retirement);
            insert_the_entry_into_the_newest_mapping_leaf(
                &mut image,
                &build_mapping_entry(&mapping_key_for_node(UNIT_CLASS_PACKED, right_leaf.pointer), right_leaf.pointer.locations),
            );
            image
        });
        world("X02 只抹退役记录", || {
            let mut pool = pool_with_new_inodes("sm-x02", NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER);
            unlink_to_the_end(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
            let mut image = pool.memory_pool();
            erase_every_record_of_the_container(&mut image, &pool.output.retirement_containers[0].pointer);
            image
        });
        world("X03 抹谱系记录(回退过的池)", || {
            let pool = pool_rolled_back_past_an_overwrite_of_the_same_length("sm-x03");
            let mut image = pool.memory_pool();
            erase_every_record_of_the_container(&mut image, &pool.output.lineage_containers[0].pointer);
            image
        });
        world("X04 墓碑改指活 inode(2 号删除中，墓碑改指 3 号的 (号,出生代))", || {
            let mut pool = pool_with_new_inodes("sm-x04", 2);
            let birth_of_three = object_birth_of(&pool.output, 3).expect("3 号有记录").0;
            publish_the_intent_of(&mut pool, 2);
            let mut image = pool.memory_pool();
            change_the_first_tombstone_of_the_newest_root(&mut image, |t| {
                set_tombstone_field(t, 0, 3);
                set_tombstone_field(t, 8, birth_of_three);
            });
            image
        });
        world("X05 墓碑抹成不存在的号(2 号 nlink 0 而墓碑指 999)", || {
            let mut pool = pool_with_new_inodes("sm-x05", 2);
            publish_the_intent_of(&mut pool, 2);
            let mut image = pool.memory_pool();
            change_the_first_tombstone_of_the_newest_root(&mut image, |t| set_tombstone_field(t, 0, 999));
            image
        });
        world("X06 墓碑出生代差一(2 号删除中)", || {
            let mut pool = pool_with_new_inodes("sm-x06", 2);
            publish_the_intent_of(&mut pool, 2);
            let mut image = pool.memory_pool();
            change_the_first_tombstone_of_the_newest_root(&mut image, |t| {
                let birth = tombstone_field(t, 8);
                set_tombstone_field(t, 8, birth + 1);
            });
            image
        });
        world("X07 活对象带墓碑(三单元意图之后把 nlink 改回 1)", || {
            let mut image = image_after_the_intent_of_a_three_unit_file("sm-x07");
            set_the_nlink_of_the_queued_first_file_back_to_one(&mut image);
            image
        });
        world("X08 遍历方向两侧同时丢删除中对象的记录(三单元意图之后抹掉记录留单元)", || {
            let mut pool = pool_with_a_three_unit_file("sm-x08");
            publish_new_inodes_and_advance_the_pool(&mut pool, 1);
            publish_the_intent(&mut pool);
            let mut image = pool.memory_pool();
            drop_the_record_of_the_first_file_and_keep_its_data_units(&mut image);
            image
        });
        world("X09 墓碑多一条重复", || {
            let mut image = image_after_the_intent_of_a_three_unit_file("sm-x09");
            append_a_copy_of_the_first_tombstone_of_the_newest_root(&mut image);
            image
        });
    }
}
