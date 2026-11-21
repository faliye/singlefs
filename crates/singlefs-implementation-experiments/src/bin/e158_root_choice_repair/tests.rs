use super::*;

/// 每块盘世代号最大的那一槽系统配置在盘上的偏移（槽字节按装置自己的 `newest_system_configuration_slot_bytes` 挑，再认它在两槽里的哪一槽）。
pub(crate) fn newest_system_configuration_slot_offset(
    pool: &MemoryPool,
    device: DeviceIdentity,
) -> u64 {
    let slot_spacing = u64::from(
        parameters_for(&GEOMETRY_PRIMARY)
            .geometry
            .fixed_structure_slot_spacing,
    );
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    let newest = newest_system_configuration_slot_bytes(pool, device, slot_spacing)
        .expect("每块盘都有自证的系统配置槽");
    [0u64, slot_spacing]
        .into_iter()
        .find(|offset| pool.read(device.0, *offset, slot_bytes).as_ref() == Some(&newest))
        .expect("最新那一槽在两槽之一")
}

/// `device` 上世代号最大的那一槽系统配置（世代号 g）的偏移，与崩在最新那次发布的根槽 FUA 之后、它那次系统配置轮换写之前
/// 那一槽该有的字节：另一槽（世代号 g − 1）的内容、世代号改成 g − 2、重新编码。写回之后两槽都自证得过，见证值停在最新那条根之前。
/// 做法同 `tests/common/mod.rs` `abandon_the_newest_root_by_a_recovery_hiding_its_root_slot_and`；不清零：清零之后那块盘只剩一槽自证过，
/// 每块盘都这样时可写挂载照 C971（撕裂轮换判不出）乙入判不出拒可写（`research/prompts/e205-preregistration.md`「乙入判不出」）。
pub(crate) fn system_configuration_slot_before_the_rotation_of_the_newest_publish(
    pool: &MemoryPool,
    device: DeviceIdentity,
) -> (u64, Vec<u8>) {
    let slot_spacing = u64::from(
        parameters_for(&GEOMETRY_PRIMARY)
            .geometry
            .fixed_structure_slot_spacing,
    );
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    let newest_offset = newest_system_configuration_slot_offset(pool, device);
    let older_offset = if newest_offset == 0 { slot_spacing } else { 0 };
    let parse_the_slot_at = |offset: u64| {
        singlefs_core::system_configuration::SystemConfiguration::parse_slot(
            &pool
                .read(device.0, offset, slot_bytes)
                .expect("两槽系统配置都在盘上"),
        )
        .expect("改写之前每块盘两槽都自证得过")
    };
    let newest_generation = parse_the_slot_at(newest_offset).quantities.slot_generation;
    let mut rotation_not_written = parse_the_slot_at(older_offset);
    rotation_not_written.quantities.slot_generation = newest_generation
        .checked_sub(2)
        .expect("最新那次发布之前至少还有两次轮换");
    (newest_offset, rotation_not_written.to_slot())
}

/// 崩溃恢复抛弃最新那条根、而系统配置没见证到它（C554 乙合进 `crates` 之后还造得出被抛弃根的只剩这一形：见证在时可写挂载
/// 重读一次仍读不出就拒可写，`MountError::NewerStateStillUnreadableAfterOneReread`）：`pool` 的拷贝，每块盘世代号最大的那一槽
/// 写成 [`system_configuration_slot_before_the_rotation_of_the_newest_publish`] 给的字节。落点表不动，下标照旧。
pub(crate) fn with_the_rotation_after_the_newest_publish_not_written(
    pool: &MemoryPool,
) -> MemoryPool {
    let mut rewound = pool.clone();
    for (device, rewound_device) in &mut rewound.devices {
        let (offset, rotation_not_written) =
            system_configuration_slot_before_the_rotation_of_the_newest_publish(pool, *device);
        rewound_device.write(DeviceOffsetInBytes(offset), &rotation_not_written);
    }
    rewound
}

/// Q3-1 用的 P332 辅助函数：只测纯逻辑，`pool` 填空壳（这几条断言不碰盘上字节）。
pub(super) fn bare_node(
    timeline: Vec<TimelineRoot>,
    content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>>,
    rollback_events: Vec<RollbackEvent>,
) -> SimNode {
    SimNode {
        pool: MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 0,
        },
        session: None,
        timeline,
        rollback_events,
        path: Vec::new(),
        content_by_root,
    }
}
