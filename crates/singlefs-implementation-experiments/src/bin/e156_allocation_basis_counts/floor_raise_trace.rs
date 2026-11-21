//! PQ2（前提 2）：抬 F 那一串的设备写，切在系统配置写完之后、每条推空根落盘之后，读 F_生效。

use super::*;

// ============================================================================================
// PQ2（前提 2）：抬 F 那一串的设备写，切在系统配置写完之后、每条推空根落盘之后，读 F_生效。
// ============================================================================================

pub(super) struct FloorRaiseTrace {
    pub(super) raised: bool,
    pub(super) floor_before: u64,
    pub(super) new_floor: u64,
    pub(super) effective_floor_before_any_write: u64,
    pub(super) system_configuration_writes: usize,
    pub(super) effective_floor_after_the_last_system_configuration_write: Option<u64>,
    pub(super) system_configuration_writes_after_the_first_root: usize,
    pub(super) pushed_roots: Vec<(u64, u64, u64)>,
    pub(super) publishes: usize,
    pub(super) reclaimed_placements: usize,
}

impl FloorRaiseTrace {
    /// 前提 2 的判定（登记第六节 6.1 PQ2 那一行）：① = 新 F 且 ② 每一条两数相等（= 新 F）⇒「两个口径合成一个」。
    pub(super) fn the_two_readings_merge(&self) -> bool {
        self.raised
            && self.effective_floor_after_the_last_system_configuration_write
                == Some(self.new_floor)
            && !self.pushed_roots.is_empty()
            && self.pushed_roots.iter().all(|(_, root_floor, effective)| {
                *root_floor == self.new_floor && *effective == self.new_floor
            })
    }
}

/// PQ2：S = 8、ρ = 1，β0 → 覆盖写 8 次 → 走产品入口 `raise_rollback_floor_to_the_admission_ceiling` 抬 F，录下这一串的设备写。
pub(super) fn run_floor_raise_trace() -> FloorRaiseTrace {
    let parameters = parameters_with_slots_per_region(E156_SLOTS_PER_REGION);
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("PQ2 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("PQ2 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("PQ2 暖机")
    };
    let mut current = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("PQ2 新池新建文件")
    };
    for step in 1..=8u64 {
        current = publish_one_workload(
            &parameters,
            devices.as_mut_slice(),
            &mut allocator,
            &current,
            instance,
            WorkloadPublishKind::Overwrite,
            step,
        )
        .expect("PQ2 覆盖写");
    }
    let floor_before = current.root.rollback_floor.0;
    let before_image = memory_pool_snapshot(&devices);
    let effective_floor_before_any_write = effective_rollback_floor(
        &before_image,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .0;
    let stream = SharedStream::retaining_contents();
    let mut recording_devices = recording_copies_of(&before_image, &stream);
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &parameters,
        &mut recording_devices,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("PQ2 抬 F");
    let (was_raised, new_floor, publishes, reclaimed_placements) = match &raised {
        RaiseToTheAdmissionCeiling::Raised(raised) => (
            true,
            raised.ceiling.0,
            raised.publishes.len(),
            raised.reclaimed.len(),
        ),
        RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { ceiling, .. } => {
            (false, ceiling.0, 0, 0)
        }
    };
    let operations = stream.retained_operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let kinds: Vec<StepKind> = operations
        .iter()
        .map(|retained| geometry.classify(&retained.operation))
        .collect();
    let first_root_write = kinds
        .iter()
        .position(|kind| matches!(kind, StepKind::RootRecordFua));
    let system_configuration_indexes: Vec<usize> = kinds
        .iter()
        .enumerate()
        .filter(|(_, kind)| matches!(kind, StepKind::SystemConfigurationSlot))
        .map(|(index, _)| index)
        .collect();
    let last_system_configuration_before_the_first_root = system_configuration_indexes
        .iter()
        .copied()
        .filter(|index| first_root_write.is_none_or(|first| *index < first))
        .max();
    let system_configuration_writes_after_the_first_root = system_configuration_indexes
        .iter()
        .filter(|index| first_root_write.is_some_and(|first| **index > first))
        .count();
    let image_up_to = |last_index: usize| {
        let mut image = before_image.clone();
        image.apply(&operations[..=last_index]);
        image
    };
    let effective_after = |image: &MemoryPool| {
        effective_rollback_floor(
            image,
            &parameters.region_devices,
            &parameters.geometry,
            &parameters.filesystem_identifier,
        )
        .0
    };
    let effective_floor_after_the_last_system_configuration_write =
        last_system_configuration_before_the_first_root
            .map(|index| effective_after(&image_up_to(index)));
    let pushed_roots: Vec<(u64, u64, u64)> = kinds
        .iter()
        .enumerate()
        .filter(|(_, kind)| matches!(kind, StepKind::RootRecordFua))
        .map(|(index, _)| {
            let bytes = operations[index]
                .contents
                .as_ref()
                .expect("开了内容保留的流");
            let root = RootRecord::parse_slot(bytes, &parameters.filesystem_identifier)
                .expect("推空那条根自证过");
            (
                root.checkpoint_txg.0,
                root.rollback_floor.0,
                effective_after(&image_up_to(index)),
            )
        })
        .collect();
    FloorRaiseTrace {
        raised: was_raised,
        floor_before,
        new_floor,
        effective_floor_before_any_write,
        system_configuration_writes: system_configuration_indexes.len(),
        effective_floor_after_the_last_system_configuration_write,
        system_configuration_writes_after_the_first_root,
        pushed_roots,
        publishes,
        reclaimed_placements,
    }
}
