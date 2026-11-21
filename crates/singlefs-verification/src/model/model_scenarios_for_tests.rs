//! 单元测试共用的模型场景：建模型、走几步、按模型写出的样子造实现交回的结局。

use super::*;

pub(super) fn two_device_model() -> IdealModel {
    IdealModel::after_make_filesystem(ModelPoolGeometry {
        devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
        device_size_in_bytes: 4 << 30,
    })
}

pub(super) fn succeed(model: &mut IdealModel, answer: &ModelAnswer) {
    assert!(
        answer.required_refusals.is_empty(),
        "这一步模型要求拒：{:?}",
        answer.required_refusals
    );
    model.advance_by_success(answer);
}

pub(super) fn key(checkpoint_txg: u64, instance: u64) -> ModelRootKey {
    ModelRootKey {
        checkpoint_txg: ModelCheckpointTxg(checkpoint_txg),
        instance: ModelInstanceGeneration(instance),
    }
}

/// mkfs、同一进程取号暖机、第一个文件（txg 3）、可写挂载（实例 2：写行 txg 4、暖机 txg 5）、覆盖写 txg 6–9，内容是 txg 号那一个字节。
pub(super) fn model_after_four_overwrites_in_a_second_instance() -> IdealModel {
    let mut model = two_device_model();
    model.acquire_and_warm_up_in_the_make_filesystem_process();
    let first = model
        .answer_publish_first_file(&[3])
        .expect("会话开着、现行 txg 2");
    succeed(&mut model, &first);
    model.close_session();
    let mount = model.answer_mount_writable();
    succeed(&mut model, &mount);
    for content_byte in 6_u8..=9 {
        let overwrite = model
            .answer_publish_overwrite(&[content_byte])
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &overwrite);
    }
    model
}

/// 实现对这一版的交回原样照模型那一版写：文件内容、整张实例表、每个角色每块盘一条仍分配、代是写它的那次发布。
pub(super) fn observed_exactly_as_the_model_wrote(root: &ModelRoot) -> ObservedRoot {
    let records = |generation: ModelCheckpointTxg| {
        [ModelDeviceIdentity(0), ModelDeviceIdentity(1)]
            .into_iter()
            .map(|device| ObservedAllocationRecord {
                device,
                generation,
                is_released: false,
            })
            .collect::<Vec<_>>()
    };
    ObservedRoot {
        key: root.key,
        journal_counter: root.journal_counter,
        rollback_floor: root.rollback_floor,
        file: match root.contents.live_file() {
            Some(file) => ObservedFile::Decoded(file.content.to_vec()),
            None => ObservedFile::NoFile,
        },
        instance_table: ObservedInstanceTable::Rows(root.instance_table_rows.as_ref().clone()),
        unit_allocation_records: ObservedUnitAllocationRecords::EveryRoleOfTheVersion(
            ObservedAllocationRecordsOfEveryUnit {
                roles: root
                    .role_written_at
                    .iter()
                    .map(|(role, written_at)| (*role, records(*written_at)))
                    .collect(),
                data_units: (0_u64..)
                    .zip(root.data_unit_written_at.iter())
                    .map(|(index, written_at)| (index, records(*written_at)))
                    .collect(),
                data_units_of_the_files_other_than_the_first: root
                    .inodes_other_than_the_first
                    .iter()
                    .flat_map(|(number, inode)| {
                        (0_u64..).zip(inode.data_unit_written_at.iter()).map(
                            move |(index, written_at)| ((*number, index), records(*written_at)),
                        )
                    })
                    .collect(),
            },
        ),
        snapshot_state: snapshot_state_as_the_model_wrote(root),
        logged_compaction_intent: root.logged_compaction_intent,
        first_file_record_truncation_field: truncation_field_as_the_model_wrote(root),
        inodes_other_than_the_first: root
            .inodes_other_than_the_first
            .iter()
            .map(|(number, inode)| (*number, ObservedFile::Decoded(inode.content.to_vec())))
            .collect(),
        truncation_fields_of_the_inodes_other_than_the_first: root
            .inodes_other_than_the_first
            .iter()
            .map(|(number, inode)| (*number, inode.truncation_field.recorded_values()))
            .collect(),
    }
}

/// 第一个文件那条记录里的截断字段照模型记的原样交回（没有那条记录的一版交 None）。
pub(super) fn truncation_field_as_the_model_wrote(
    root: &ModelRoot,
) -> Option<ObservedTruncationField> {
    root.contents
        .first_file_record_truncation_field()
        .map(ModelTruncationField::recorded_values)
}

/// 这一版的快照那几样照模型记的原样交回（树表 0 条的一版没有活头那条条目）。
pub(super) fn snapshot_state_as_the_model_wrote(root: &ModelRoot) -> ObservedSnapshotState {
    ObservedSnapshotState {
        live_head_previous_snapshot_txg: root
            .contents
            .has_trees()
            .then_some(root.previous_snapshot_txg_of_the_live_head),
        snapshots: root
            .snapshots
            .iter()
            .map(|snapshot| ObservedSnapshot {
                birth_txg: snapshot.birth_txg,
                previous_snapshot_txg: snapshot.previous_snapshot_txg,
                is_being_destroyed: snapshot.is_being_destroyed,
                inode_tree_written_by: snapshot.inode_tree_written_by,
                extent_root_written_at: snapshot.extent_root_written_at,
            })
            .collect(),
        deadlist_entries: root.deadlist.as_ref().clone(),
        pinned_mapping_entries: root.pinned_mapping_entries,
    }
}

/// 模型里现行那一版的快照（会话开着）。
pub(super) fn current_snapshots(model: &IdealModel) -> Vec<ModelSnapshot> {
    model
        .snapshots_of_the_current_version()
        .expect("会话开着")
        .to_vec()
}

/// 整理的一次发布看得见的几样（[`ModelCompactionPublishSeen`]）：R 取 0 号盘第一个文件那次发布的那一段。
pub(super) fn compaction_publish_seen(
    units_relocated: u64,
    is_an_intent_logged_after_the_publish: bool,
    is_the_region_empty: bool,
    user_visible_root_units_relocated: Vec<ModelUserVisibleRootUnit>,
) -> ModelCompactionPublishSeen {
    ModelCompactionPublishSeen {
        units_at_most: 2,
        units_relocated,
        user_visible_root_units_relocated,
        is_an_intent_logged_after_the_publish,
        is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish:
            is_the_region_empty,
        region_forced_for_the_issue: ModelCompactionRegion {
            device: ModelDeviceIdentity(0),
            start_slot: 50_240,
            slot_count: CLUSTER_SEGMENT_SLOTS,
        },
    }
}

pub(super) fn aspect_of_the_compaction_publish(
    model: &IdealModel,
    seen: &ModelCompactionPublishSeen,
) -> Result<ModelOperationKind, ModelDisagreementAspect> {
    model
        .answer_compaction_publish(seen)
        .map(|answer| answer.operation)
        .map_err(|disagreement| disagreement.aspect)
}

pub(super) fn succeed_with_the_compaction_publish(
    model: &mut IdealModel,
    seen: &ModelCompactionPublishSeen,
) -> ModelRoot {
    let answer = model
        .answer_compaction_publish(seen)
        .expect("这一次整理发布合条款");
    succeed(model, &answer);
    model
        .current_version_in_the_session()
        .expect("会话开着")
        .clone()
}

/// 一个数据单元装多少字节的用户内容（净荷）。
pub(super) fn payload() -> usize {
    usize::try_from(data_unit_payload_capacity_in_bytes()).expect("三万多")
}

/// `length` 字节、没有一个字节是 0 的内容（补的 0 与洞读出的 0 看得出来）。
pub(super) fn content_without_zeros(length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from(index % 251).expect("小于 251") + 1)
        .collect()
}

/// 实例 2 覆盖写到 txg 9 之后，顺序写一个 `length` 字节的文件（txg 10）。
pub(super) fn model_with_a_file_of(length: usize) -> IdealModel {
    let mut model = model_after_four_overwrites_in_a_second_instance();
    let write = model
        .answer_publish_sequential_write(&content_without_zeros(length))
        .expect("会话开着、现行版本带文件");
    succeed(&mut model, &write);
    model
}

pub(super) fn current_version(model: &IdealModel) -> ModelRoot {
    model
        .current_version_in_the_session()
        .expect("会话开着")
        .clone()
}

/// 截到 `new_size` 字节（缩短是意图那次，不缩短是那一次只改记录的发布），做成，交回写出的那一版。
pub(super) fn truncate_to(model: &mut IdealModel, new_size: usize) -> ModelRoot {
    let answer = model
        .answer_truncation(u64::try_from(new_size).expect("长度"))
        .expect("会话开着、现行版本带活文件");
    assert_eq!(answer.operation, ModelOperationKind::PublishTruncation);
    assert_eq!(answer.expected_roots.len(), 1, "截断这次用户改动一次发布");
    succeed(model, &answer);
    current_version(model)
}

/// 走空的下一步，做成，交回写出的那一版。
pub(super) fn walk_one_step(model: &mut IdealModel) -> ModelRoot {
    let answer = model
        .answer_next_step_of_the_walk()
        .expect("现行版本有要走空的");
    succeed(model, &answer);
    current_version(model)
}

pub(super) fn file_of(root: &ModelRoot) -> &ModelFileVersion {
    root.contents.live_file().expect("这一版读得出文件")
}
