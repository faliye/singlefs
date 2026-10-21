
// ===== 正推腿探针（T6）：带活整理意图的新镜像在只读挂载、可写挂载（产品开关）、回退三条路上各被当成什么 =====
use singlefs_core::address::InstanceGeneration;
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::mounted_read::mount_read_only;

#[test]
fn probe_forward_an_image_with_a_live_compaction_intent_on_the_three_paths() {
    let (mut pool, (issued_txg, intent), _records_left) =
        pool_crashed_after_the_compaction_publish("probe-forward-intent-three-paths", 1);
    // 只读挂载：
    let image = pool.memory_pool();
    let read_only = mount_read_only(&image);
    println!("name=probe_forward_read_only_mount_with_a_live_intent result={}", match &read_only {
        Ok(_) => "Ok".to_string(),
        Err(failure) => format!("Err({failure:?})"),
    });
    assert!(read_only.is_ok());
    // 可写挂载（产品开关）：
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let continuation_outcome = mounted
        .output
        .compaction_continuation
        .run
        .as_ref()
        .map(|run| format!("{:?}", run.outcome));
    println!("name=probe_forward_product_mount_with_a_live_intent continuation={continuation_outcome:?}");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let excluded_after_mount = session.allocator.region_excluded_from_placement();
    println!("name=probe_forward_product_mount_with_a_live_intent region_excluded={excluded_after_mount:?} intent_region={:?}", intent.region);
    // 回退：随现行那一版，意图跨回退留着。
    let mut current = session.current.version_of_trees().expect("有树").clone();
    let mut outcomes: Vec<String> = Vec::new();
    let mut done = false;
    'search: for instance in 1..=current.root.instance.0 {
        for txg in 1..current.root.checkpoint_txg.0 {
            let target = RollbackTarget {
                instance: InstanceGeneration(instance),
                checkpoint_txg: CheckpointTxg(txg),
            };
            let mut attempt = current.clone();
            let rolled = roll_back_by_a_forward_publish(
                &parameters(),
                &mut devices,
                &mut session.allocator,
                &mut attempt,
                target,
            );
            match rolled {
                Err(singlefs_core::mount::RollbackError::TargetNotACandidate { .. }) => {}
                Ok(_) => {
                    println!(
                        "name=probe_forward_rollback_with_a_live_intent target={target:?} result=Ok intent_still_in_the_new_version={} region_still_excluded={:?} new_version_txg={}",
                        live_compaction_intent_of_the_entries(&attempt.logged_operations).is_some(),
                        session.allocator.region_excluded_from_placement(),
                        attempt.root.checkpoint_txg.0
                    );
                    done = true;
                    break 'search;
                }
                Err(refusal) => {
                    outcomes.push(format!("{target:?} => {refusal:?}"));
                }
            }
        }
    }
    if !done {
        println!("name=probe_forward_rollback_with_a_live_intent no_target_accepted other_outcomes={outcomes:?}");
    }
    pool.devices = Some(devices);
}

/// 盘上那一条意图改坏（R 起始槽 50241 不对齐，节点重封、校验和沿引用链补到根槽）：可写挂载整个拒（原有用例钉了），只读挂载呢。
#[test]
fn probe_forward_a_malformed_logged_intent_is_refused_by_the_writable_mount_as_a_whole_and_what_the_read_only_mount_does() {
    let (pool, _live, _records_left) =
        pool_crashed_after_the_compaction_publish("probe-forward-intent-malformed", 1);
    let mut image = pool.memory_pool();
    let root_slot = the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_LOGGED_OPERATIONS).root);
    let mut node = unit_bytes_at(&image, root_slot, node_bytes());
    let start_slot_offset = index_node_header_bytes(LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES)
        + COMPACTION_INTENT_REGION_START_SLOT_OFFSET_IN_THE_ENTRY;
    node[start_slot_offset..start_slot_offset + 6]
        .copy_from_slice(&(SEGMENT_OF_THE_FIRST_PUBLISH + 1).to_le_bytes()[..6]);
    reseal_index_node(&mut node);
    replace_the_unit_and_propagate_its_checksum(&mut image, root_slot, &node);
    let writable = writable_mount_refused_before_any_write(&image);
    println!("name=probe_forward_malformed_intent writable_mount={writable:?}");
    let read_only = mount_read_only(&image);
    println!(
        "name=probe_forward_malformed_intent read_only_mount={}",
        match &read_only {
            Ok(_) => "Ok".to_string(),
            Err(failure) => format!("Err({failure:?})"),
        }
    );
    let mut fallbacks = 0;
    let mounted = read_only.expect("只读挂载收下");
    let walked = walk_to_file(&image, &mounted.chosen_root, &mut fallbacks);
    println!("name=probe_forward_malformed_intent cold_walk_of_the_chosen_root={:?}", walked.as_ref().map(|content| content.as_ref().map(Vec::len)));
}
