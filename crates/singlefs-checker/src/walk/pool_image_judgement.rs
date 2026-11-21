//! 池级 checker 对外的入口 `check_pool_image` 一族，与把各条不变量的判定按次序串起来的那一趟。

use super::*;

/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。当三棵容器树都没装只供测试的容量旋钮（产品镜像）：
/// I-9.21（产品镜像里非最右墓碑容器装满） 与 I-9.36（产品镜像里非最右谱系与退役容器装满） 照产品容量判，
/// I-9.22、I-9.37（测试镜像里容器不超旋钮值） 报不适用。
#[must_use]
pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
    check_pool_image_given_the_container_capacity_knobs(reader, ContainerCapacityKnobs::PRODUCT)
}

/// 池级 checker 带旋钮值的入口：旋钮不落盘
/// （D18（块里携带什么信息） 已定项 24 第 3 条），写这张镜像时压没压墓碑容器容量由调用方从镜像外给。`None` 与 [`check_pool_image`] 相同；
/// `Some` 时 I-9.22 按旋钮值判、I-9.21 报不适用。
#[must_use]
pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
    reader: &dyn ImageReader,
    tombstone_container_capacity_knob: Option<TombstoneContainerCapacityKnob>,
) -> Vec<(&'static str, InvariantVerdict)> {
    check_pool_image_given_the_container_capacity_knobs(
        reader,
        ContainerCapacityKnobs {
            tombstone: tombstone_container_capacity_knob,
            lineage: None,
            retirement: None,
        },
    )
}

/// 池级 checker 带三棵容器树旋钮值的入口（同墓碑那一个入口的
/// 取 K1）：旋钮不落盘，写这张镜像时三棵容器树各压没压容量由调用方
/// 从镜像外给。[`ContainerCapacityKnobs::PRODUCT`] 与 [`check_pool_image`] 相同；墓碑旋钮 `Some` 时 I-9.22 按旋钮值判、I-9.21 报不适用；
/// 谱系或退役旋钮 `Some` 时那一棵按 I-9.37 判，`None` 的那一棵照 I-9.36 判。
#[must_use]
pub fn check_pool_image_given_the_container_capacity_knobs(
    reader: &dyn ImageReader,
    container_capacity_knobs: ContainerCapacityKnobs,
) -> Vec<(&'static str, InvariantVerdict)> {
    judge_pool_image_given_the_container_capacity_knobs(reader, container_capacity_knobs)
        .into_report()
}

/// 同 [`check_pool_image`]，另交回判红时点名的（不变量, 盘, 起点槽）（
/// 故障归类器按它比「被吞的写插回去之后仍红」）。点名槽的是哪几条见 [`VerdictsNamingTheViolationSites`]。
#[must_use]
pub fn check_pool_image_naming_the_violation_sites(
    reader: &dyn ImageReader,
) -> VerdictsNamingTheViolationSites {
    judge_pool_image_given_the_container_capacity_knobs(reader, ContainerCapacityKnobs::PRODUCT)
        .into_report_naming_the_violation_sites()
}

/// 根环里一条自证过的根：住在哪个区域、区域里第几槽，它的实例代号与 `checkpoint_txg`（[`root_ring_the_ring_judgements_read`] 交回的一行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RootRingSlotTheJudgementsRead {
    pub region: u64,
    pub slot_in_region: u64,
    pub instance: u32,
    pub checkpoint_txg: u64,
}

/// I-3.5（引用区间的精确性）、I-3.1（已分配统计对得上）、I-7.9（回退下界 F 不高于抬 F 的上限） 三条判法与择根、回退候选集读的那一份根环：
/// [`judge_pool_image_given_the_container_capacity_knobs`] 只读这一次（`roots`），I-3.1 与 I-3.5 经回退候选集的走读用它，I-7.9 直接拿它算抬 F 的上限，
/// 三条各带各的锚与参数，读槽这一层是同一份（C700（三条判法各读各的根环没有跨装置检查））。读槽层读松（该有的根读成没有）、读严，
/// 三条一起偏、单看判定看不出，跨装置的那一道是验证包拿历史生成器的洞表逐位置比（[`root_ring_the_ring_judgements_read`]）。
fn roots_the_ring_judgements_read(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
) -> Vec<(u64, u64, crate::RootView)> {
    valid_roots(reader, geometry)
}

/// 同 [`roots_the_ring_judgements_read`]，按择到的第一块盘的系统配置几何（与 [`check_pool_image`] 同一个取法）读一遍，交回每条自证过的根
/// 住的（区域、区域里第几槽）与它的（实例、txg），按区域、槽排。一块盘都择不到系统配置时 `None`（[`check_pool_image`] 那时整池报不适用）。
/// 给跨装置检查用（C700（三条判法各读各的根环没有跨装置检查））：harness 的历史生成器知道每个位置写过哪一条根，逐位置与它比。
#[must_use]
pub fn root_ring_the_ring_judgements_read(
    reader: &dyn ImageReader,
) -> Option<Vec<RootRingSlotTheJudgementsRead>> {
    let geometry = chosen_system_configurations(reader)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))?;
    let mut slots: Vec<RootRingSlotTheJudgementsRead> =
        roots_the_ring_judgements_read(reader, &geometry)
            .into_iter()
            .map(
                |(region, slot_in_region, root)| RootRingSlotTheJudgementsRead {
                    region,
                    slot_in_region,
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                },
            )
            .collect();
    slots.sort_unstable();
    Some(slots)
}

/// [`check_pool_image_given_the_container_capacity_knobs`] 的判定累加器那一份（交回之前不折成报告）。
fn judge_pool_image_given_the_container_capacity_knobs(
    reader: &dyn ImageReader,
    container_capacity_knobs: ContainerCapacityKnobs,
) -> Judgements {
    let system_configurations = chosen_system_configurations(reader);
    let mut root_ring_judgements = Judgements::default();
    let chosen: Vec<_> = system_configurations
        .iter()
        .filter_map(|(device, chosen)| {
            chosen
                .clone()
                .map(|(view, geometry)| (*device, view, geometry))
        })
        .collect();
    // 一块盘的两个系统配置槽都无效时**不早退**：恢复会改用别的盘上那一份、照常挂上
    // （`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`，D22（单元原子性怎么合成） 已定项 8
    // 「系统配置每盘放一份」买的就是这份冗余）。早退会让这种镜像上每一条不变量都报「不适用」，
    // 而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过
    // （.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。
    // 任一盘任一槽自证过而带这一版读者不收的池级值（格式版本、加密类型、槽距、physical_block_size、环长、journal 环起点、
    // 根环起点、单元区起始槽号）：实现整池拒绝挂载
    // （`RecoveryFailure::SystemConfigurationValueRefused`），checker 报违例、别的不变量同「挂不上的镜像」一样不作保
    // 只有一部分槽带越界值时不拿别的槽照判下去。
    let system_configuration_slot_readings_of_each_device =
        system_configuration_slot_readings(reader);
    let any_system_configuration_slot_carries_a_refused_value =
        judge_system_configuration_values_the_reader_accepts(
            &system_configuration_slot_readings_of_each_device,
            &mut root_ring_judgements,
        );
    if any_system_configuration_slot_carries_a_refused_value {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "池里有一槽自证过的系统配置带这一版读者不收的池级值：实现整池拒绝挂载，这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements;
    }
    // I-8.1 的三个字段等式（在飞上限、最坏占用、F = 格式常量）任一盘任一可择槽有一格不等：实现择系统配置时同判、整池拒绝挂载
    // （`RecoveryFailure::SystemConfigurationValueRefused`），checker 与上面 I-7.13 那一支同口径——报 I-8.1 违例、别的不变量一律不作保
    // （`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 4 行）。三格先判进一份只装它们的判定里：整池不作保时
    // 别的不变量（连同已判过成立的 I-7.13）都报不适用；三格都成立时再判进这一份，照常往下判。
    let mut judgements_of_the_journal_ring_geometry_fields_alone = Judgements::default();
    judge_journal_ring_geometry_fields_agree(
        &system_configuration_slot_readings_of_each_device,
        &mut judgements_of_the_journal_ring_geometry_fields_alone,
    );
    if judgements_of_the_journal_ring_geometry_fields_alone
        .violation_count(JOURNAL_RING_GEOMETRY_FIELDS_AGREE)
        > 0
    {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            judgements_of_the_journal_ring_geometry_fields_alone.not_applicable(
                invariant,
                "池里有一槽可择的系统配置 journal 环几何三个字段（在飞上限、最坏占用、F）有一格不等：实现整池拒绝挂载，这不是一个挂得上的镜像",
            );
        }
        return judgements_of_the_journal_ring_geometry_fields_alone;
    }
    judge_journal_ring_geometry_fields_agree(
        &system_configuration_slot_readings_of_each_device,
        &mut root_ring_judgements,
    );
    // I-8.5（V-池）：全池皆废判红；一块盘交得出可择的系统配置就照常往下判（单盘全废走旁路报告，不进违例通道）。
    // 带越界值的槽、环几何三格不等的槽整池拒挂，上面那两支整池报不适用、不判这一条。
    // 一块盘都交不出可择的系统配置、而有槽自证过（incompat 位不认得、R / S 越界这类自证过而不可择的槽）：这不是一个挂得上的镜像，
    // I-8.5 同别的不变量一起报不适用，不判成成立（`InvariantVerdict` 的「不适用必须带理由，不许静默算成立」）；一个槽都没自证过的照判红。
    if chosen.is_empty() {
        let mut judgement_of_some_device_carrying_a_self_verified_slot = Judgements::default();
        judge_some_device_carries_a_self_verified_system_configuration(
            &system_configuration_slot_readings_of_each_device,
            &mut judgement_of_some_device_carrying_a_self_verified_slot,
        );
        match judgement_of_some_device_carrying_a_self_verified_slot
            .first_violation_of(SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION)
        {
            Some(detail) => {
                let detail = detail.to_owned();
                root_ring_judgements.judge(
                    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
                    false,
                    || detail,
                );
            }
            None => root_ring_judgements.not_applicable(
                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
                "池里有系统配置槽自证过，而没有一块盘交得出可择的系统配置：这不是一个挂得上的镜像",
            ),
        }
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "池里没有一块盘交得出有效的系统配置：这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements;
    }
    judge_some_device_carries_a_self_verified_system_configuration(
        &system_configuration_slot_readings_of_each_device,
        &mut root_ring_judgements,
    );
    let geometry = chosen[0].2;
    // 各盘择到的系统配置要属于同一个池：fsid 逐盘相同（一块别的池的旧盘插进来，实例代号可以恰好也是 1，I-7.7 看不出来）。
    for (device, view, _) in &chosen {
        root_ring_judgements.judge(
            "I-1.4",
            view.filesystem_identifier == geometry.filesystem_identifier,
            || {
                format!(
                    "盘 {device} 择到的系统配置 fsid 与盘 {} 的不同：这块盘不属于这个池",
                    chosen[0].0
                )
            },
        );
        // I-7.15：各盘所选的系统配置槽设备数相同（不要求等于盘表：降级镜像里 checker 手里的盘可以少于设备数）。
        // 实现那一侧 `choose_system_configuration` 在几块盘的设备数对不上时报 `SystemConfigurationsDisagree`、整池挂不上。
        root_ring_judgements.judge(
            DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
            view.device_count == chosen[0].1.device_count,
            || {
                format!(
                    "盘 {device} 择到的系统配置设备数 {} 与盘 {} 的 {} 不同",
                    view.device_count, chosen[0].0, chosen[0].1.device_count
                )
            },
        );
        // I-1.5（系统配置记管道身份，C868（系统配置管道身份没有 checker 判定））：择到的那一槽自举头里实现标识 16 字节不全零。
        let records_a_writer_identity = match view.writer_implementation_identifier {
            WriterImplementationIdentifier::AllZero => false,
            WriterImplementationIdentifier::Written => true,
        };
        root_ring_judgements.judge(
            SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY,
            records_a_writer_identity,
            || {
                format!(
                    "盘 {device} 择到的系统配置（世代号 {}）写入者身份里的实现标识 16 字节全零：没记下是哪条管道写的这个镜像",
                    view.slot_generation
                )
            },
        );
    }
    judge_system_configuration_slot_generations_follow_the_rotation(
        &system_configuration_slot_readings_of_each_device,
        &mut root_ring_judgements,
    );
    let chosen_geometries: Vec<PoolGeometry> = chosen
        .iter()
        .map(|(_, _, geometry_of_this_device)| *geometry_of_this_device)
        .collect();
    let decision_width =
        decision_width_declared_by_the_chosen_system_configurations(&chosen_geometries)
            .expect("上面判过 chosen 非空：至少一块盘择到了系统配置");
    judge_root_slots_aligned_to_the_decision_width(
        &geometry,
        decision_width,
        &mut root_ring_judgements,
    );
    let devices = reader.devices();
    let regions = usize::try_from(geometry.regions).expect("R");
    let region_devices = &geometry.region_devices[..regions.min(3)];
    let distinct: BTreeSet<u32> = region_devices.iter().copied().collect();
    let heaviest = devices
        .iter()
        .map(|device| {
            region_devices
                .iter()
                .filter(|candidate| *candidate == device)
                .count()
        })
        .max()
        .unwrap_or(0);
    let pigeonhole = regions.div_ceil(devices.len().max(1));
    root_ring_judgements.judge(
        "I-7.6",
        distinct.len() == regions.min(devices.len()) && heaviest <= pigeonhole && region_devices.iter().all(|device| devices.contains(device)),
        || format!("根环区域的设备 {region_devices:?}：不同值 {}、最重的盘背 {heaviest} 个（上界 {pigeonhole}）", distinct.len()),
    );
    let filesystem_identifier_low = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("8 字节"),
    );
    let roots = roots_the_ring_judgements_read(reader, &geometry);
    judge_root_record_algorithm_nonce_and_mac_are_zero(&mut root_ring_judgements, &roots);
    judge_root_records_live_in_the_slot_their_txg_names(
        &roots,
        &geometry,
        &mut root_ring_judgements,
    );
    judge_instance_table_rows_against_the_roots_referencing_them(
        reader,
        &roots,
        &mut root_ring_judgements,
    );
    // 每块盘两槽里自证过、fsid 与本池相同的系统配置槽（与实现取号同一读法，D18（块里携带什么信息） 已定项 11）：
    // I-7.12 逐盘比，回退候选集的 F 生效值取它们带的 F 的最大值（C556（checker 与崩溃点重放不读系统配置里的 F））。
    let verified_system_configurations_of_this_pool: Vec<(
        u32,
        Vec<crate::SystemConfigurationView>,
    )> = verified_system_configuration_slots(reader)
        .into_iter()
        .map(|(device, slots)| {
            (
                device,
                slots
                    .into_iter()
                    .map(|(view, _)| view)
                    .filter(|view| view.filesystem_identifier == geometry.filesystem_identifier)
                    .collect(),
            )
        })
        .collect();
    judge_own_device_number_is_the_device_identity(
        &verified_system_configurations_of_this_pool,
        &mut root_ring_judgements,
    );
    judge_system_configuration_floor_against_the_roots_on_each_device(
        &verified_system_configurations_of_this_pool,
        &roots,
        &geometry,
        &mut root_ring_judgements,
    );
    judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
    // I-8.6、I-8.7、I-8.8、I-8.9 与 I-8.13 只读 journal 环，不读根：根环全灭的镜像上它们照样判得了（那一格归 I-7.1）。
    let ScannedJournalRings {
        records_by_device: journal_records_by_device,
        records_the_recovery_reader_reads_by_device:
            journal_records_the_recovery_reader_reads_by_device,
        self_verified_records_by_device: self_verified_journal_records_by_device,
        slots_that_do_not_read_back_by_device: journal_slots_that_do_not_read_back_by_device,
    } = scanned_journal_rings_by_device(reader, &geometry, filesystem_identifier_low);
    judge_journal_records_live_in_the_ring_slot_their_counter_names(
        &self_verified_journal_records_by_device,
        geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES,
        &mut root_ring_judgements,
    );
    judge_journal_back_chain(&journal_records_by_device, &mut root_ring_judgements);
    judge_location_order_of_journal_named_entries(
        &journal_records_by_device,
        &mut root_ring_judgements,
    );
    let comparable_pairs_of_transaction_numbers = judge_transaction_numbers_per_instance(
        &journal_records_by_device,
        &mut root_ring_judgements,
    );
    let transaction_numbers_from_one = judge_transaction_numbers_start_from_one_per_instance(
        &journal_records_by_device,
        geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES,
        &mut root_ring_judgements,
    );
    note_transaction_numbers_without_a_comparable_pair(
        comparable_pairs_of_transaction_numbers,
        transaction_numbers_from_one,
        &mut root_ring_judgements,
    );
    judge_commit_markers_per_transaction(&journal_records_by_device, &mut root_ring_judgements);
    judge_publish_ordinals_and_last_record_flags(
        &journal_records_by_device,
        &mut root_ring_judgements,
    );
    judge_journal_record_txg_per_instance(&journal_records_by_device, &mut root_ring_judgements);
    judge_records_of_one_publish_within_the_in_flight_record_limit(
        &journal_records_by_device,
        geometry.declared_in_flight_record_limit,
        &mut root_ring_judgements,
    );
    judge_named_entries_against_the_unit_headers(
        reader,
        &journal_records_by_device,
        &mut root_ring_judgements,
    );
    judge_new_root_segment_of_the_last_record_against_the_roots(
        &journal_records_by_device,
        &roots,
        &mut root_ring_judgements,
    );
    // I-8.14（C764）与 I-3.19（C740）看根环每一槽读没读得出（全零的槽不算读不出）。
    let root_ring_slot_readings = root_ring_slot_readings(reader, &geometry);
    judge_the_first_txg_of_each_instance_above_the_older_instances(
        &roots,
        &records_as_the_recovery_reads_them(&journal_records_the_recovery_reader_reads_by_device),
        &root_ring_slot_readings,
        &journal_slots_that_do_not_read_back_by_device,
        &mut root_ring_judgements,
    );
    root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
        "根环里一条自证过的根都没有".to_string()
    });
    if roots.is_empty() {
        root_ring_judgements.not_applicable(
            "I-7.3",
            "根环里一条自证过的根都没有：S 空，没有「代号最大者」可谈",
        );
        root_ring_judgements.not_applicable(
            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
            "根环里一条自证过的根都没有：没有最新根，遍历方向那三张表无从读、已发布谓词也没有输入",
        );
        return root_ring_judgements;
    }
    judge_root_ring_health(&roots, &mut root_ring_judgements);
    // 择根：按 (txg, 实例) 取最大（D22（单元原子性怎么合成） 已定项 7，与实现 `recovery::choose_root` 同一个读法）；
    // 最大的不止一条时取读根环次序里最前的那条（`roots` 按 `root_slot_positions` 的区域、槽次序排，与 `recovery::every_root_ring_slot` 同序）：
    // 只在严格更大时才换。`max_by_key` 在键相同时交回最后一个，与 `choose_root` 相反。
    let newest_index = roots
        .iter()
        .enumerate()
        .fold(
            None,
            |newest_so_far: Option<(usize, (u64, u32))>, (index, (_, _, root))| {
                let choice_key = (root.checkpoint_txg, root.instance);
                match newest_so_far {
                    Some((_, newest_key)) if choice_key <= newest_key => newest_so_far,
                    Some(_) | None => Some((index, choice_key)),
                }
            },
        )
        .map(|(index, _)| index)
        .expect("上面判过根环里至少一条自证过的根");
    let mut walk = Walk::starting_with(
        reader,
        root_ring_judgements,
        filesystem_identifier_low,
        roots[newest_index].2.instance,
        roots[newest_index].2.checkpoint_txg,
    );
    // I-3.5 (b) 的谱系那一半每走完一版读一次那一版的谱系树：只读不判，另用一份缓存，不与后面几遍共用的那一份互相改。
    let mut index_node_cache_of_the_lineage_reads = IndexNodeCache::new();
    // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
    walk.walk_root(&roots[newest_index].2.record_bytes, true);
    walk.note_the_version_just_walked_for_the_appearance_rule(
        roots[newest_index].2.instance,
        roots[newest_index].2.checkpoint_txg,
        0,
        instance_table_pointer_birth_txg_of(
            &roots[newest_index].2.record_bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER],
        ),
        lineage_records_of_a_version_for_the_appearance_rule(
            reader,
            &roots[newest_index].2.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
            &roots[newest_index].2.record_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER],
            &mut index_node_cache_of_the_lineage_reads,
        ),
    );
    let newest_failures = walk.walk_failures.clone();
    // I-3.11（已分配减 defer 等于最新根走读）要的「从最新有效根走读到的、这块盘上被引用的槽数」：走法与 I-3.1 同一个
    // （`note_reference` 记下的 (设备, 起点槽, 跨度)），只取最新根——这一刻 `references` 里还只有最新根这一遍记下的。
    let slots_referenced_by_the_newest_root = slots_referenced_per_device(&walk.references);
    let start_slots_referenced_by_the_newest_root: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .map(|(device, start_slot, _)| (*device, *start_slot))
        .collect();
    // I-3.17 ② 要的「最新根引用的每一个 (盘, 槽)」：同一刻的 `references` 按跨度展开。
    let slots_covered_by_the_newest_root: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .flat_map(|(device, start_slot, span_slots)| {
            (*start_slot..start_slot.saturating_add(*span_slots)).map(move |slot| (*device, slot))
        })
        .collect();
    // I-4.8（近 K 代根校验和自洽）与 I-7.4（近 K 代块未被复用）：候选集里任一根（最新根也在候选集里）出发遍历，所有块的校验和
    // 都与父指针一致、走读不断——最新根那一次各算一格；候选集只剩最新根时两条都还判得到（全称量词在单元素集合上照样成立）。
    let newest_txg = roots[newest_index].2.checkpoint_txg;
    let newest_walked_into_reused_or_erased_unit =
        !newest_failures.is_empty() || walk.judgements.violation_count("I-2.1") > 0;
    walk.judgements
        .judge("I-4.8", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）出发的遍历有单元对不上或读不出")
        });
    walk.judgements
        .judge("I-7.4", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）")
        });
    let accounting_seen = walk.accounting_seen;
    let account_of_the_newest_root_version =
        account_of_the_newest_root(walk.accounting_tree_root_birth_txg, newest_txg);
    let accounting = walk.accounting.clone();
    let inode_number_watermark_rows_of_the_newest_root = walk.inode_number_watermark_rows.clone();
    // 别的根只取按最新根指着的实例表仍然有效的：(i, T) 有效 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti
    // （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）；被抛弃时间线的根引用的单元由影子账隔离、不在当前账里。
    // 再加一条：txg ≥ 回退下界 F 的生效值（D16（发布语义） 已定项 1 的回退候选集，`effective_rollback_floor`）；F 之下的根引用的单元可以已被回收复用，
    // 它们不在当前账里、也不再是「近 K 代」——I-2.1 只在候选集里的根上判。
    let instance_table_rows = walk.instance_table_rows.clone();
    let abandoned_by_the_newest_roots_table = |root: &crate::RootView| {
        instance_table_rows.iter().any(|row| {
            row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
        })
    };
    // 上面那张表是主走读沿链判下来的行，链断在哪一片就只有那一片之前的行：读不全时滤不掉的根认不出来。
    // 滤没滤全就看这同一遍走读走没走到末片（`Walk::walk_instance_table_chain` 记下的），不另读一遍。
    let abandoned_roots_filtering = walk.abandoned_roots_filtering;
    // 回退候选集的下界取 F 生效值（D16（发布语义） 已定项 1「生效」，SysPre）：max(各幸存盘最新持久有效根所带 F 的最大值,
    // 池里自证过的系统配置槽带的 F)。有效 = 按最新根指着的实例表判不是被抛弃的；每块盘取落在它上面的根环区域里 (txg, 实例) 最大的那一条
    // （最新根的实例表读不出时一行都没有、不滤）；被抛弃时间线上的根带的 F 也算进来（D16（发布语义） 已定项 1）。
    // 只读根上的 F 时，带新 F 的根全坏、系统配置里的新 F 还在的合法镜像上，F 之下、单元已被合法复用的根会被当成候选，
    // I-7.4（近 K 代块未被复用） 等按候选集判的几条误红（C556（checker 与崩溃点重放不读系统配置里的 F））。
    let mut newest_valid_root_on_each_device: BTreeMap<u32, &crate::RootView> = BTreeMap::new();
    for (region, _, root) in &roots {
        if abandoned_by_the_newest_roots_table(root) {
            continue;
        }
        let device = geometry.region_devices[usize::try_from(*region).expect("区域号")];
        let newest_on_this_device = newest_valid_root_on_each_device
            .entry(device)
            .or_insert(root);
        if (root.checkpoint_txg, root.instance)
            > (
                newest_on_this_device.checkpoint_txg,
                newest_on_this_device.instance,
            )
        {
            *newest_on_this_device = root;
        }
    }
    let highest_floor_on_the_abandoned_roots = roots
        .iter()
        .filter(|(_, _, root)| abandoned_by_the_newest_roots_table(root))
        .map(|(_, _, root)| root.rollback_floor)
        .max()
        .unwrap_or(0);
    let highest_floor_on_the_newest_valid_and_the_abandoned_roots =
        newest_valid_root_on_each_device
            .values()
            .map(|root| root.rollback_floor)
            .max()
            .unwrap_or(0)
            .max(highest_floor_on_the_abandoned_roots);
    let effective_rollback_floor = verified_system_configurations_of_this_pool
        .iter()
        .flat_map(|(_, slots)| slots.iter().map(|view| view.rollback_floor))
        .fold(
            highest_floor_on_the_newest_valid_and_the_abandoned_roots,
            u64::max,
        );
    // 一圈窗口：N 取最新根（它按自己的实例表恒有效），3S 是根环槽数。只给 I-7.9 挡洞里留下的旧根用（[`OneRevolutionWindow`]）；
    // 候选过滤与丁-defer 门槛不再看它（RGall）。
    let window = OneRevolutionWindow {
        anchor_txg: newest_txg,
        root_ring_slot_count: geometry.regions * geometry.slots_per_region,
    };
    // 掉出遍历的根槽按理由各记一次（几样都占的各边都记）：I-3.1 判红时这几个数就是「遍历为什么少算」的机理标识。
    let mut root_slots_dropped_as_abandoned = 0u64;
    let mut root_slots_dropped_below_floor = 0u64;
    let candidate_indexes: Vec<usize> = roots
        .iter()
        .enumerate()
        .filter(|(index, (_, _, root))| {
            // 被抛弃：按最新根指着的实例表判（D23（journal 的角色与格式） 已定项 14 候选集）。
            let abandoned = abandoned_by_the_newest_roots_table(root);
            let below_floor = root.checkpoint_txg < effective_rollback_floor;
            // 原来还有「一圈窗口之外」一格（环上有洞时洞那一槽里留着的上一圈的根）：候选过滤是原来同取一圈窗口的六处之一，
            // RGall 一起退回整个环，那条根照样是候选、照样走。
            let walked = *index == newest_index || (!abandoned && !below_floor);
            if !walked {
                if abandoned {
                    root_slots_dropped_as_abandoned += 1;
                }
                if below_floor {
                    root_slots_dropped_below_floor += 1;
                }
            }
            walked
        })
        .map(|(index, _)| index)
        .collect();
    for index in candidate_indexes.iter().copied() {
        let (_, _, root) = &roots[index];
        if index != newest_index {
            let mismatches_before = walk.judgements.violation_count("I-2.1");
            let failures_before = walk.walk_failures.len();
            walk.walk_root(&root.record_bytes, false);
            walk.note_the_version_just_walked_for_the_appearance_rule(
                root.instance,
                root.checkpoint_txg,
                failures_before,
                instance_table_pointer_birth_txg_of(
                    &root.record_bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER],
                ),
                lineage_records_of_a_version_for_the_appearance_rule(
                    reader,
                    &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
                    &root.record_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER],
                    &mut index_node_cache_of_the_lineage_reads,
                ),
            );
            // 这条候选根引用的单元有一个校验和对不上或头用不了，就是它指着的块被复用或抹头了（I-7.4（近 K 代块未被复用）），
            // 从它出发的遍历也就不自洽（I-4.8（近 K 代根校验和自洽））；两条按每条候选根各判一格。
            let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
                > mismatches_before
                || walk.walk_failures.len() > failures_before;
            let root_txg = root.checkpoint_txg;
            walk.judgements
                .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                    format!(
                        "候选根 txg {root_txg} 引用的单元已被复用或抹头（校验和对不上或头用不了）"
                    )
                });
            walk.judgements
                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                    format!("候选根 txg {root_txg} 出发的遍历有单元对不上或读不出")
                });
        }
    }
    // 回退候选集补上「由记录施加出来、根槽从没落盘的那一版」（候选 b）：它们没有根槽，
    // 上面按根环走的那一遍走不到，而它们换下的单元还在 defer 里、仍算在已分配里（增补 2 收口表第 54 行那 12 个状态差的 65 536 字节）。
    // 走法与判定照候选根：引用进同一个并集（I-3.1 / I-5.1），单元照判 I-2.1 等，I-7.4 / I-4.8 每一版各判一格。
    let versions_applied_only_by_records = versions_applied_only_by_records(
        &journal_records_the_recovery_reader_reads_by_device,
        &roots,
        &instance_table_rows,
        effective_rollback_floor,
    );
    for version in &versions_applied_only_by_records {
        let mismatches_before = walk.judgements.violation_count("I-2.1");
        let failures_before = walk.walk_failures.len();
        walk.walk_version_applied_only_by_records(version);
        let (tree_table_pointer, mapping_root_pointer) =
            version.tree_table_and_mapping_root_pointers();
        walk.note_the_version_just_walked_for_the_appearance_rule(
            version.instance,
            version.checkpoint_txg,
            failures_before,
            instance_table_pointer_birth_txg_of(
                &version.new_root_segment[NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER],
            ),
            lineage_records_of_a_version_for_the_appearance_rule(
                reader,
                tree_table_pointer,
                mapping_root_pointer,
                &mut index_node_cache_of_the_lineage_reads,
            ),
        );
        let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
            > mismatches_before
            || walk.walk_failures.len() > failures_before;
        let (instance, version_txg) = (version.instance, version.checkpoint_txg);
        walk.judgements
            .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）"
                )
            });
        walk.judgements
            .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）出发的遍历有单元对不上或读不出"
                )
            });
    }
    judge_the_appearance_rule_of_adjacent_candidate_versions(
        &walk.versions_folded_into_units_by_the_final_fold_table(),
        &walk.birth_txg_of_walked_units,
        &walk.birth_txg_of_walked_instance_table_pages,
        &walk.instance_table_rows,
        &mut walk.judgements,
    );
    // I-5.5：回退候选集里每条根与由记录施加出来的那几版，各自重读树表与 extent 树（只读），认出从 mkfs 水位第一次建树的那一版再判落点；
    // U 取所选系统配置偏移 417（`geometry` 是第一块择到系统配置的盘的那一份，I-7.13 判过它是环长现算的那个槽）。
    let versions_to_judge_the_first_data_unit_of: Vec<VersionToJudgeTheFirstDataUnitOf<'_>> =
        candidate_indexes
            .iter()
            .map(|index| {
                let root = &roots[*index].2;
                VersionToJudgeTheFirstDataUnitOf {
                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
                    checkpoint_txg: root.checkpoint_txg,
                    tree_table_pointer: &root.record_bytes[36..122],
                    mapping_root_pointer: &root.record_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER],
                }
            })
            .chain(versions_applied_only_by_records.iter().map(|version| {
                VersionToJudgeTheFirstDataUnitOf {
                    name: format!(
                        "由记录施加出来的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    checkpoint_txg: version.checkpoint_txg,
                    tree_table_pointer: version.tree_table_and_mapping_root_pointers().0,
                    mapping_root_pointer: version.tree_table_and_mapping_root_pointers().1,
                }
            }))
            .collect();
    judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
        reader,
        &versions_to_judge_the_first_data_unit_of,
        geometry.unit_area_start_slot,
        &mut walk.judgements,
    );
    if walk.units_judged_against_an_older_candidate_root == 0 {
        walk.judgements.not_applicable(
            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
            "回退候选集里只有最新那条根、或不是最新的候选根一个被引用单元都没走到：没有「不是最新的候选根」可判（最新那条归 I-4.2）",
        );
    }
    judge_blocks_referenced_by_abandoned_roots(
        reader,
        &geometry,
        &roots,
        newest_index,
        &instance_table_rows,
        filesystem_identifier_low,
        &mut walk.judgements,
    );
    judge_trees_of_the_replayed_version_are_traversable(
        reader,
        &journal_records_the_recovery_reader_reads_by_device,
        &roots[newest_index].2,
        geometry.journal_ring_bytes,
        filesystem_identifier_low,
        &mut walk.judgements,
    );
    let referenced_units_judged_against_their_pointer =
        walk.referenced_units_judged_against_their_pointer;
    let mut judgements = walk.judgements;
    judgements.judge("I-7.2", newest_failures.is_empty(), || {
        format!("最新的根走不完：{}", newest_failures.join("；"))
    });
    // I-1.2 与 I-4.2 在遍历方向上判的是同一批单元：走读一个被引用单元都没走到时两条都没有对象
    // （根槽读得出而树表读不出、或每个单元的头都用不了），报「不适用」并带理由，不报成立。
    if referenced_units_judged_against_their_pointer == 0 {
        for invariant in ["I-1.2", "I-4.2"] {
            judgements.not_applicable(
                invariant,
                "走读一个被引用的单元都没读到（实例表单元按 I-1.2 那一行的例外不进这两条）：出生身份与已发布谓词都没有对象",
            );
        }
    }
    // I-1.8：扫描方向按已发布谓词过滤之后归并成组。谓词要挂载根的 (实例代号, txg) 与它指着的那一版实例表，
    // 所以这一遍排在走读之后（`instance_table_rows` 是走最新根时取的）。
    // 扫描式方向用它自己那一份谓词（[`PublishedPredicateOfTheScanDirection`]，C410）；遍历式这边 I-9.6 墓碑那一半用 [`PublishedPredicate`]，
    // 两份的输入是同一组值。
    let published = PublishedPredicate {
        mount_root_instance: roots[newest_index].2.instance,
        mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
        instance_table_rows: &instance_table_rows,
    };
    let published_of_the_scan_direction = PublishedPredicateOfTheScanDirection {
        mount_root_instance: roots[newest_index].2.instance,
        mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
        instance_table_rows: &instance_table_rows,
    };
    let content_units = scanned_content_units(
        reader,
        &geometry,
        filesystem_identifier_low,
        &published_of_the_scan_direction,
    );
    if judge_merged_version_total_order(reader, &content_units, &mut judgements) == 0 {
        judgements.not_applicable(
            "I-1.8",
            "扫描方向没有两份归并到一起、或两组归并到同一个 key 的码 1 / 码 3 已发布单元：归并与定序都比不出来",
        );
    }
    // 这两遍各自按候选集里每条根读树表与树节点：共用一份按位置条目记的缓存，候选根常指着同一个单元，崩溃点重放每个崩溃状态都跑。
    let mut index_node_cache = IndexNodeCache::new();
    judge_release_generation_and_tree_table_birth(
        reader,
        &roots,
        &candidate_indexes,
        newest_index,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_tree_identifier_birth_txg_across_every_readable_root(
        reader,
        &geometry,
        &roots,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-5.4 按版本那一层：回退候选集里每条根，加由记录施加出来的那几版与下一次挂载先施加的那一版（改法 RV，见
    // [`judge_allocation_records_disjoint`]）。坏盘上同一版两类都认得出时判两遍：同一棵分配记录树按根节点只读一次，违例不重报。
    let version_the_next_mount_applies_first = version_the_next_mount_applies_first(
        reader,
        &journal_records_the_recovery_reader_reads_by_device,
        &roots[newest_index].2,
        geometry.journal_ring_bytes,
    );
    let versions_to_judge_the_allocation_records_of: Vec<VersionToJudgeTheAllocationRecordsOf<'_>> =
        candidate_indexes
            .iter()
            .map(|index| {
                let root = &roots[*index].2;
                VersionToJudgeTheAllocationRecordsOf {
                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
                    tree_table_pointer: &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
                    allocation_record_tree_root_held_outside_the_tree_table: &root.record_bytes
                        [ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT],
                    mapping_root_pointer: &root.record_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER],
                }
            })
            .chain(versions_applied_only_by_records.iter().map(|version| {
                VersionToJudgeTheAllocationRecordsOf {
                    name: format!(
                        "由记录施加出来的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    tree_table_pointer: &version.new_root_segment
                        [NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
                    allocation_record_tree_root_held_outside_the_tree_table: &version
                        .new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER],
                    mapping_root_pointer: &version.new_root_segment
                        [NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER],
                }
            }))
            .chain(version_the_next_mount_applies_first.iter().map(|version| {
                VersionToJudgeTheAllocationRecordsOf {
                    name: format!(
                        "下一次挂载先施加的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    tree_table_pointer: &version.new_root_segment
                        [NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
                    allocation_record_tree_root_held_outside_the_tree_table: &version
                        .new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER],
                    mapping_root_pointer: &version.new_root_segment
                        [NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER],
                }
            }))
            .collect();
    judge_allocation_records_disjoint(
        reader,
        &versions_to_judge_the_allocation_records_of,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-5.7（C706）：同一份「由记录施加出来的那几版与下一次挂载先施加的那一版」，前件取自那次发布的记录自述的点名项。
    let records_the_replay_reads =
        records_as_the_recovery_reads_them(&journal_records_the_recovery_reader_reads_by_device);
    let versions_to_judge_the_allocation_record_tree_root_birth_of: Vec<
        VersionToJudgeTheAllocationRecordTreeRootBirthOf<'_>,
    > = versions_applied_only_by_records
        .iter()
        .map(|version| VersionToJudgeTheAllocationRecordTreeRootBirthOf {
            name: format!(
                "由记录施加出来的那一版（实例 {}、txg {}）",
                version.instance, version.checkpoint_txg
            ),
            checkpoint_txg: version.checkpoint_txg,
            allocation_record_tree_root_held_outside_the_tree_table: &version.new_root_segment
                [NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER],
            publish_names_some_unit: publish_names_some_unit(
                &records_the_replay_reads,
                version.instance,
                version.checkpoint_txg,
            ),
        })
        .chain(version_the_next_mount_applies_first.iter().map(|version| {
            VersionToJudgeTheAllocationRecordTreeRootBirthOf {
                name: format!(
                    "下一次挂载先施加的那一版（实例 {}、txg {}）",
                    version.instance, version.checkpoint_txg
                ),
                checkpoint_txg: version.checkpoint_txg,
                allocation_record_tree_root_held_outside_the_tree_table: &version.new_root_segment
                    [NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER],
                publish_names_some_unit: publish_names_some_unit(
                    &records_the_replay_reads,
                    version.instance,
                    version.checkpoint_txg,
                ),
            }
        }))
        .collect();
    judge_allocation_record_tree_roots_born_with_the_versions_applied_only_by_records(
        &versions_to_judge_the_allocation_record_tree_root_birth_of,
        &mut judgements,
    );
    // I-3.19（C740）：最新根实例表里每一行的 W 落在这一行容许的集合里。
    judge_the_applied_transaction_high_water_of_the_instance_table_rows(
        &instance_table_rows,
        abandoned_roots_filtering,
        &roots,
        &root_ring_slot_readings,
        &geometry,
        &records_the_replay_reads,
        &mut judgements,
    );
    let allocation_record_trees = allocation_record_trees_of_the_candidate_versions(
        reader,
        &roots,
        &candidate_indexes,
        &versions_applied_only_by_records,
        tree_table_and_mapping_root_pointers_of_the_version_the_next_mount_applies_first(
            version_the_next_mount_applies_first.as_ref(),
        ),
        &mut index_node_cache,
    );
    judge_allocation_generations_against_unit_births(
        reader,
        &allocation_record_trees,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_rollback_floor_raises_against_their_ceilings(
        reader,
        &geometry,
        &roots,
        window,
        &root_ring_slot_readings,
        &records_the_replay_reads,
        &mut index_node_cache,
        &mut judgements,
    );
    // 删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6；判哪几版、配对与读不全的报法照 I-9.5 那一行的共用判法，判决
    // `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F4）：回退候选集里每条根与由记录施加出来的那几版，各自重读一遍（只读）。
    let versions_to_scan_the_deletion_state_of: Vec<VersionToScanTheDeletionStateOf<'_>> =
        candidate_indexes
            .iter()
            .map(|index| {
                let root = &roots[*index].2;
                VersionToScanTheDeletionStateOf {
                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
                    checkpoint_txg: root.checkpoint_txg,
                    tree_table_pointer: &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
                    mapping_root_pointer: &root.record_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER],
                }
            })
            .chain(versions_applied_only_by_records.iter().map(|version| {
                VersionToScanTheDeletionStateOf {
                    name: format!(
                        "由记录施加出来的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    checkpoint_txg: version.checkpoint_txg,
                    tree_table_pointer: version.tree_table_and_mapping_root_pointers().0,
                    mapping_root_pointer: version.tree_table_and_mapping_root_pointers().1,
                }
            }))
            .collect();
    judge_deletion_state_of_the_versions(
        reader,
        &versions_to_scan_the_deletion_state_of,
        &mut index_node_cache,
        &mut judgements,
    );
    // 墓碑与谱系那几条（I-9.18–I-9.22、I-9.29、I-9.10 墓碑那一个等号）：同一份版本清单，每一版从自己的 txg 与自己的谱系树起算。
    judge_tombstones_and_the_lineage_of_the_versions(
        reader,
        &versions_to_scan_the_deletion_state_of,
        abandoned_roots_filtering,
        container_capacity_knobs,
        &mut index_node_cache,
        &mut judgements,
    );
    // 只读快照与 deadlist（I-3.5 ③ ④ 与逆向、I-3.6、I-3.7）：
    // 同一份版本清单，每一版重读自己的树表。
    let versions_to_scan_the_snapshots_and_the_deadlist_of: Vec<
        VersionToScanTheSnapshotsAndTheDeadlistOf<'_>,
    > = versions_to_scan_the_deletion_state_of
        .iter()
        .map(|version| VersionToScanTheSnapshotsAndTheDeadlistOf {
            name: version.name.clone(),
            checkpoint_txg: version.checkpoint_txg,
            tree_table_pointer: version.tree_table_pointer,
            mapping_root_pointer: version.mapping_root_pointer,
        })
        .collect();
    judge_snapshots_and_deadlists_of_the_versions(
        reader,
        &versions_to_scan_the_snapshots_and_the_deadlist_of,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-9.17、I-9.23 在主走读里每条内部条目判；一条条目都没走到时的理由。
    judgements.not_applicable(
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    );
    judgements.not_applicable(
        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    );
    // I-3.15、I-5.6、I-4.4、I-1.14：同一份版本清单
    // （回退候选集里每条根与由记录施加出来的那几版），每一版重读自己的树表、分配记录、中央映射与 logged_ops。I-3.15 要隔离判据的位置项，
    // 在 I-3.1 那一段之后判。
    let versions_with_their_root_pointers: Vec<VersionWithItsRootPointers<'_>> =
        candidate_indexes
            .iter()
            .map(|index| {
                let root = &roots[*index].2;
                VersionWithItsRootPointers {
                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
                    checkpoint_txg: root.checkpoint_txg,
                    tree_table_pointer: &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
                    instance_table_pointer: &root.record_bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER],
                    mapping_root_pointer: &root.record_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER],
                    allocation_record_tree_root_held_outside_the_tree_table: &root.record_bytes
                        [ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT],
                }
            })
            .chain(versions_applied_only_by_records.iter().map(|version| {
                VersionWithItsRootPointers {
                    name: format!(
                        "由记录施加出来的那一版（实例 {}、txg {}）",
                        version.instance, version.checkpoint_txg
                    ),
                    checkpoint_txg: version.checkpoint_txg,
                    tree_table_pointer: &version.new_root_segment
                        [NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
                    instance_table_pointer: &version.new_root_segment
                        [NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER],
                    mapping_root_pointer: &version.new_root_segment
                        [NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER],
                    allocation_record_tree_root_held_outside_the_tree_table: &version
                        .new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER],
                }
            }))
            .collect();
    judge_compaction_intents_of_the_versions(
        reader,
        &versions_with_their_root_pointers,
        geometry.unit_area_start_slot,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_unit_bytes_of_each_mapping_key_across_the_versions(
        reader,
        &versions_with_their_root_pointers,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-9.6（水位大于两处最大号）：记账里那条「inode 号水位」要大于遍历侧算出的 inode 树内最大 key。
    // 两条独立路径——水位是发布路径在记账树里写下的一个数，最大 key 是 checker 逐片叶容器逐条记录数出来的。
    // 另一半（> 全部已发布的墓碑记录的对象 ID）接在这一判后面：墓碑是打包记录类型 1，意图那次发布往墓碑树开放容器追加一条
    // （D8（核心索引结构） 已定项 16 第 1 条；`deleted_inodes` 的形态已定，C118 还清）。
    let inode_number_watermark = accounting
        .get(&(STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION))
        .copied();
    match (
        inode_number_watermark,
        walk.largest_inode_number_in_the_inode_tree,
    ) {
        (Some(watermark), Some(largest_inode)) => {
            judgements.judge("I-9.6", watermark > largest_inode, || {
                format!(
                    "记账里的 inode 号水位 {watermark} 不大于 inode 树内最大 key {largest_inode}"
                )
            });
        }
        (None, _) => judgements.not_applicable(
            "I-9.6",
            "最新的根下面没有记账树、或记账里没有 inode 号水位那一行",
        ),
        (Some(_), None) => {
            judgements.not_applicable("I-9.6", "最新的根下面走不到 inode 树里的任何一条记录")
        }
    }
    // I-9.6 墓碑那一半：水位 > 所选根的墓碑树里全部已发布的（按实例表判）墓碑记录的对象 ID；一条墓碑都没有时没有对象。
    if let Some(watermark) = inode_number_watermark {
        match largest_published_tombstone_object_identifier(
            reader,
            &roots[newest_index].2.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
            &roots[newest_index].2.record_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER],
            &published,
            &mut index_node_cache,
        ) {
            Ok(Some(largest_tombstone_object)) => {
                judgements.judge("I-9.6", watermark > largest_tombstone_object, || {
                    format!(
                        "记账里的 inode 号水位 {watermark} 不大于所选根（txg {newest_txg}）的墓碑树里已发布的墓碑记录的最大对象 ID {largest_tombstone_object}"
                    )
                });
            }
            Ok(None) => {}
            Err(gap) => gap.note_as_a_part_of_the_range_left_unjudged(&mut judgements, "I-9.6"),
        }
    }
    // I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）：挨着 I-9.6，同样只在最新的根上判。
    judge_inode_number_watermark_steps_of_the_newest_root(
        reader,
        &roots[newest_index].2.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
        &roots[newest_index].2.record_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER],
        newest_txg,
        &inode_number_watermark_rows_of_the_newest_root,
        &mut index_node_cache,
        &mut judgements,
    );
    if !walk.inode_tree_walked {
        judgements.not_applicable(
            "I-9.12",
            "最新的根下面走不到 inode 树的内部节点（树表 0 条或根读不出）",
        );
    }
    for (inode, unit_object_birth, what) in &walk.data_unit_objects {
        if let Some(record_birth) = walk.inode_object_birth.get(inode) {
            judgements.judge("I-9.10", record_birth == unit_object_birth, || {
                format!(
                    "{what} 的对象出生代 {unit_object_birth} 与 inode 记录的 {record_birth} 不符"
                )
            });
        }
    }
    // I-5.1：同一块盘上，不同的引用不许占重叠的槽（两条位置条目落在不同盘上是显式的多副本）。
    let mut per_device: BTreeMap<u32, Vec<(u64, u64, &String)>> = BTreeMap::new();
    for ((device, slot, span), what) in &walk.references {
        per_device
            .entry(*device)
            .or_default()
            .push((*slot, *span, what));
    }
    for (device, mut ranges) in per_device.clone() {
        ranges.sort_unstable_by_key(|(slot, span, _)| (*slot, *span));
        for pair in ranges.windows(2) {
            let (first_slot, first_span, first_what) = pair[0];
            let (second_slot, _, second_what) = pair[1];
            judgements.judge("I-5.1", first_slot + first_span <= second_slot, || {
                format!("盘 {device}：{first_what}（槽 {first_slot} 跨 {first_span}）与 {second_what}（槽 {second_slot}）重叠")
            });
        }
    }
    // I-3.1 判红时，说明文字里带一段机理标识：遍历覆盖了哪些根、剩下的按什么理由没覆盖。
    // 判读的一方（`singlefs-verification` 的 `history::allocation_statistic_mechanism`）按它分辨「记账多算」是哪一种机理造成的：
    // 环转过一圈把 F 之上的根挤出了环，还是回退下界把环里读得到的根挡在了外面——两者的签名（只有 I-3.1 红、记账多于遍历）
    // 一模一样，不带这一段就只能按签名认，机理不同的新问题会被「已知红」清单接走。
    let root_ring_slot_count = geometry.regions * geometry.slots_per_region;
    let oldest_readable_root_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .min()
        .unwrap_or(0);
    let readable_root_slot_count = u64::try_from(roots.len()).expect("根槽数");
    let walked_root_slot_count = u64::try_from(candidate_indexes.len()).expect("候选根槽数");
    let walked_versions_applied_only_by_records =
        u64::try_from(versions_applied_only_by_records.len()).expect("版本数");
    // 「并进遍历的由记录施加出来的版本」那一段：判读的一方按字段名取数（`leading_number_after`），
    // 不看各段的次序，所以插在「遍历的候选根槽」之后、与它挨着读。
    let mechanism = move || {
        format!(
            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {effective_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
        )
    };
    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
    // I-3.11：同一块盘上「已分配」减「defer 待释放」对只走最新根那一遍得到的和。
    let slots_referenced_by_every_walked_version = slots_referenced_per_device(&walk.references);
    let start_slots_referenced_by_every_walked_version: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .map(|(device, start_slot, _)| (*device, *start_slot))
        .collect();
    // 隔离的记录（已分配而没有根引用、每块盘上那一条都带隔离记号，只认记号、不看副本）在这两条上豁免：I-3.1 对全部走过的版本的引用、
    // I-3.11 对最新根这一遍的引用各算一份（`quarantined_slots_exempted_per_device`）。
    let newest_root_view = &roots[newest_index].2;
    let quarantined_exempted_against_every_walked_version = quarantined_slots_exempted_per_device(
        reader,
        newest_root_view,
        &start_slots_referenced_by_every_walked_version,
        &mut index_node_cache,
    );
    let walked_slots_of_every_walked_version: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .flat_map(|(device, start_slot, span_slots)| {
            (*start_slot..*start_slot + *span_slots).map(move |slot| (*device, slot))
        })
        .collect();
    let standing_in_the_newest_roots_table = |root: &crate::RootView| {
        if abandoned_by_the_newest_roots_table(root) {
            RootStandingInTheNewestRootsTable::Abandoned
        } else {
            RootStandingInTheNewestRootsTable::Valid
        }
    };
    let roots_below_the_floor: Vec<&crate::RootView> = roots
        .iter()
        .map(|(_, _, root)| root)
        .filter(|root| {
            root_references_are_merged_into_the_deferred_slots_below_the_floor(
                standing_in_the_newest_roots_table(root),
                root.checkpoint_txg,
                effective_rollback_floor,
            )
        })
        .collect();
    let deferred_threshold = deferred_slots_reclaim_threshold(
        roots.iter().map(|(_, _, root)| {
            (
                standing_in_the_newest_roots_table(root),
                root.checkpoint_txg,
            )
        }),
        effective_rollback_floor,
    );
    let deferred_slots_below_the_floor = deferred_slots_referenced_only_below_the_floor_per_device(
        reader,
        filesystem_identifier_low,
        newest_root_view,
        &roots_below_the_floor,
        &walked_slots_of_every_walked_version,
        deferred_threshold,
        &mut index_node_cache,
    );
    let quarantined_exempted_against_the_newest_root = quarantined_slots_exempted_per_device(
        reader,
        newest_root_view,
        &start_slots_referenced_by_the_newest_root,
        &mut index_node_cache,
    );
    // I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）：只判最新根、只判建账的发布（C773（读坏与全零根槽的回收门槛钉只在内存里））。
    judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(
        &roots,
        newest_index,
        &geometry,
        &standing_in_the_newest_roots_table,
        abandoned_roots_filtering,
        account_of_the_newest_root_version,
        &mut judgements,
    );
    // I-3.16 占着的槽里「钉带出来的那一截」（C773 合成形态）：最新根带的 t 低于不带钉现算的门槛（就是丁-defer 那个门槛）时，释放代落在这一截里的
    // 已释放槽不看引用并进，被抛弃根引用着的不并（[`released_slots_the_pin_kept_in_defer`]）。I-3.1 改成 RCW 之后不再并它（上沿的 U 按 t 取）。
    let slots_the_pin_kept_in_defer: BTreeSet<(u32, u64)> =
        if accounting_seen && newest_root_view.reclaim_threshold < deferred_threshold {
            let slots_already_counted: BTreeSet<(u32, u64)> = walked_slots_of_every_walked_version
                .iter()
                .chain(
                    quarantined_exempted_against_every_walked_version
                        .slots
                        .iter(),
                )
                .chain(deferred_slots_below_the_floor.iter())
                .copied()
                .collect();
            released_slots_the_pin_kept_in_defer(
                reader,
                newest_root_view,
                deferred_threshold,
                &slots_referenced_by_abandoned_roots(
                    reader,
                    &roots,
                    newest_index,
                    &standing_in_the_newest_roots_table,
                    filesystem_identifier_low,
                ),
                &slots_already_counted,
                &mut index_node_cache,
            )
        } else {
            BTreeSet::new()
        };
    // I-3.16 占着的槽只在最新根带的 F 等于 F 生效值时比：最新根的记账是按它自己带的 F 记的（抬 F 的回收在写第一条带新 F 的根之前做，
    // `mount::raise_rollback_floor`），它带的 F 低于 F 生效值时那一份并集与记账不可比。I-3.1 改成 RCW 之后不再有这一道例外
    // （这时 t ≤ θ_high，E(t) ⊇ E(θ_high)，三判照判不误红）。
    let accounting_was_written_under_the_effective_floor =
        roots[newest_index].2.rollback_floor == effective_rollback_floor;
    // I-3.21（走到的引用起点是最新根计入的分配记录的起点，C944）：与 I-3.1 同一份遍历、同一个 θ_high，不看有没有记账。
    judge_walked_reference_starts_against_the_allocation_records_of_the_newest_root(
        reader,
        newest_root_view,
        &start_slots_referenced_by_every_walked_version,
        deferred_threshold,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-3.1（RCW）读的最新根全部分配记录：任何一棵读不出，记录罩住的槽就数不全，这一条在每块盘上报部分没判。
    let records_of_the_newest_root_read_in_full = if accounting_seen {
        allocation_records_of_the_newest_root_read_in_full_without_judging(
            reader,
            newest_root_view,
            &mut index_node_cache,
        )
    } else {
        AllocationRecordsOfTheNewestRootReadInFull::NotReadInFull
    };
    if accounting_seen {
        for device in &devices {
            let allocated = accounting
                .get(&(STATISTIC_ALLOCATED_BYTES, *device))
                .copied();
            // I-3.1（已分配统计对得上，RCW）：记账对最新根账里的记录比，下沿取 checker 现算的门槛、上沿取最新根自己带的门槛与「遍历 + 还在 defer 的」两者的小者，
            // 另判遍历走到的每个槽都在下沿那一份里（`allocated_statistic_bounds_of_a_device`）。
            match &records_of_the_newest_root_read_in_full {
                AllocationRecordsOfTheNewestRootReadInFull::Read(records) => {
                    let bounds_of = |records_to_bound: &[AllocationRecordView]| {
                        allocated_statistic_bounds_of_a_device(
                            *device,
                            records_to_bound,
                            &walked_slots_of_every_walked_version,
                            slots_referenced_by_every_walked_version
                                .get(device)
                                .copied()
                                .unwrap_or(0),
                            &quarantined_exempted_against_every_walked_version.slots,
                            quarantined_exempted_against_every_walked_version
                                .slot_count_per_device
                                .get(device)
                                .copied()
                                .unwrap_or(0),
                            deferred_threshold,
                            newest_root_view.reclaim_threshold,
                        )
                    };
                    let bounds = bounds_of(records);
                    let lies_within_its_bounds =
                        allocated_statistic_lies_within_its_bounds(allocated, &bounds);
                    // 三判不成立时先问配不上的那块是不是全落在 I-3.18 判红的单元里：只看这块盘上的记录；
                    // 只认改记那一份读法整份过三判加逐槽两判。
                    // 这块盘上的记录（记录自述那一份）连同改记那一份，两份同次序；没有 I-3.18 判红的记录、或三判成立时是 None。
                    let records_on_this_device_with_the_spans_set_to_the_unit_classes = if lies_within_its_bounds {
                        None
                    } else {
                        let records_on_this_device: Vec<AllocationRecordView> = records
                            .iter()
                            .filter(|record| record.device == *device)
                            .copied()
                            .collect();
                        records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes(
                            &records_on_this_device,
                            |record| {
                                class_and_slots_of_the_unit_whose_usable_header_starts_at(
                                    reader,
                                    record.device,
                                    record.slot,
                                )
                                .map(|(_, slots_of_the_class)| slots_of_the_class)
                            },
                        )
                        .map(|records_set_to_the_unit_classes| {
                            (records_on_this_device, records_set_to_the_unit_classes)
                        })
                    };
                    let bounds_with_the_spans_set_to_the_unit_classes =
                        records_on_this_device_with_the_spans_set_to_the_unit_classes
                            .as_ref()
                            .map(|(_, records_set_to_the_unit_classes)| {
                                bounds_of(records_set_to_the_unit_classes)
                            });
                    // 取宽那一支第五判：
                    // 改记之后未释放的记录罩住的槽区间不与这块盘上别的记录相交。
                    let records_overlapping_in_the_unit_class_reading =
                        records_on_this_device_with_the_spans_set_to_the_unit_classes
                            .as_ref()
                            .map_or(
                                Vec::new(),
                                |(records_on_this_device, records_set_to_the_unit_classes)| {
                                    unreleased_records_overlapping_another_record_in_the_unit_class_reading(
                                        records_on_this_device,
                                        records_set_to_the_unit_classes,
                                    )
                                },
                            );
                    let the_misfit_lies_in_units_the_span_invariant_reddens =
                        bounds_with_the_spans_set_to_the_unit_classes.as_ref().is_some_and(
                            |bounds_with_the_spans_set_to_the_unit_classes| {
                                allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading(
                                    allocated,
                                    bounds_with_the_spans_set_to_the_unit_classes,
                                )
                            },
                        );
                    if the_misfit_lies_in_units_the_span_invariant_reddens {
                        if records_overlapping_in_the_unit_class_reading.is_empty() {
                            let (
                                as_the_unit_classes,
                                (records_on_this_device, records_set_to_the_unit_classes),
                            ) = match (
                                bounds_with_the_spans_set_to_the_unit_classes.as_ref(),
                                records_on_this_device_with_the_spans_set_to_the_unit_classes.as_ref(),
                            ) {
                                (
                                    Some(as_the_unit_classes),
                                    Some((records_on_this_device, records_set_to_the_unit_classes)),
                                ) => (
                                    as_the_unit_classes,
                                    (
                                        records_on_this_device.as_slice(),
                                        records_set_to_the_unit_classes.as_slice(),
                                    ),
                                ),
                                // 没有改记那一份（这块盘上没有 I-3.18 判红的记录）：两种读法都按记录自述那一份写，被改记的记录与差的槽都是空的。
                                (None, _) | (_, None) => (&bounds, (&[][..], &[][..])),
                            };
                            // 部分没判的说明带上记账的已分配、Ω ∪ U 的三样各自的槽数、两种读法各自的下沿与上沿、被改记的记录与两种读法差的槽
                            judgements.part_of_the_range_left_unjudged(
                                "I-3.1",
                                allocated_statistic_left_to_the_span_invariant(
                                    *device,
                                    allocated,
                                    deferred_threshold,
                                    newest_root_view.reclaim_threshold,
                                    slots_referenced_by_every_walked_version
                                        .get(device)
                                        .copied()
                                        .unwrap_or(0),
                                    quarantined_exempted_against_every_walked_version
                                        .slot_count_per_device
                                        .get(device)
                                        .copied()
                                        .unwrap_or(0),
                                    (&bounds, as_the_unit_classes),
                                    (records_on_this_device, records_set_to_the_unit_classes),
                                ),
                            );
                        } else {
                            judgements.judge_naming_the_slots(
                                "I-3.1",
                                false,
                                || {
                                    records_overlapping_in_the_unit_class_reading
                                        .iter()
                                        .flat_map(|pair| {
                                            [
                                                (*device, pair.unreleased_record.slot),
                                                (*device, pair.other_record.slot),
                                            ]
                                        })
                                        .collect()
                                },
                                || {
                                    format!(
                                        "{}{}",
                                        allocated_statistic_violated_by_records_overlapping_in_the_unit_class_reading(
                                            *device,
                                            &records_overlapping_in_the_unit_class_reading,
                                        ),
                                        mechanism()
                                    )
                                },
                            );
                        }
                    } else {
                        judgements.judge_naming_the_slots(
                            "I-3.1",
                            lies_within_its_bounds,
                            || {
                                quarantined_exempted_against_every_walked_version
                                    .start_slots_not_exempted_on(*device)
                            },
                            || {
                                let AllocatedStatisticBoundsOfADevice {
                                    slots_counted_at_the_checker_reclaim_threshold,
                                    slots_counted_at_the_root_reclaim_threshold,
                                    walked_and_still_deferred_slots,
                                    walked_slots_outside_the_records,
                                    // 逐槽的上沿只在取宽那一支判（`allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading`），记录自述那一份三判不读它。
                                    slots_counted_at_the_root_reclaim_threshold_outside_the_walk_and_the_deferred: _,
                                } = &bounds;
                                let walked = walked_and_still_deferred_slots.saturating_mul(SLOT_BYTES);
                                let lower_bound = slots_counted_at_the_checker_reclaim_threshold
                                    .saturating_mul(SLOT_BYTES);
                                let records_at_the_root_threshold = slots_counted_at_the_root_reclaim_threshold
                                    .saturating_mul(SLOT_BYTES);
                                format!(
                                    "盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}（遍历走到的槽加隔离豁免、加释放代高于最新根带的门槛 t {} 而遍历没走到的已释放槽）；\
                                     下沿：checker 现算的门槛 {deferred_threshold} 之上该算已分配的记录罩住 {lower_bound}；上沿：门槛 t 之上该算已分配的记录罩住 {records_at_the_root_threshold}；\
                                     遍历走到而不在下沿那一份里的槽 {walked_slots_outside_the_records:?}{}",
                                    newest_root_view.reclaim_threshold,
                                    mechanism()
                                )
                            },
                        );
                    }
                }
                AllocationRecordsOfTheNewestRootReadInFull::TreeTableWithEntriesButNoAllocationRecordTreeEntry {
                    tree_table_entries,
                } => judgements.part_of_the_range_left_unjudged(
                    "I-3.1",
                    format!(
                        "盘 {device}：最新根（txg {newest_txg}）的树表有 {tree_table_entries} 条、一条种类 3（分配记录树）都没有（I-9.16 ④ 判它）：\
                         这一版的账一棵分配记录树都没有，记录口径无从比，记账对记录那三判没判"
                    ),
                ),
                AllocationRecordsOfTheNewestRootReadInFull::NotReadInFull => judgements.part_of_the_range_left_unjudged(
                    "I-3.1",
                    format!(
                        "盘 {device}：最新根（txg {newest_txg}）的树表读不出、条目窄过字段表，或有一棵分配记录树读不出，记录罩住的槽数不全，记账对记录那三判没判"
                    ),
                ),
            }
            // 单元区起点是系统配置里读来的 8 字节、空闲与已分配是记账行里读来的 8 字节：减法在起点越过盘末时下溢、加法在两个大值上溢出，
            // checker 当场 panic（代码审阅 6c 第 6 条那一族）。起点越过盘末时单元区没有容量可比（`None`），这一格与盘容量读不出同样判红；
            // 两个值加起来溢出也判红、不回绕（与 I-3.11 那一判同一个写法）。
            let capacity = reader.device_bytes(*device).and_then(|bytes| {
                (bytes / SLOT_BYTES)
                    .checked_sub(geometry.unit_area_start_slot)
                    .map(|unit_area_slots| unit_area_slots * SLOT_BYTES)
            });
            let free = accounting.get(&(STATISTIC_FREE_BYTES, *device)).copied();
            judgements.judge("I-5.2", matches!((free, allocated, capacity), (Some(free), Some(allocated), Some(capacity)) if free.checked_add(allocated) == Some(capacity)), || {
                format!("盘 {device}：空闲 {free:?} + 已分配 {allocated:?} ≠ 单元区 {capacity:?}")
            });
            // I-3.20 (a)（C70）：同一份三项加上不可回收（统计量 3），两套独立维护的 += 各自没错而和对不上时这一判红。
            judge_free_allocated_and_unreclaimable_of_a_device_sum_to_the_capacity(
                *device,
                free,
                allocated,
                accounting
                    .get(&(STATISTIC_UNRECLAIMABLE_BYTES, *device))
                    .copied(),
                capacity,
                &mut judgements,
            );
            // I-3.11（已分配减 defer 等于最新根走读）：写者把活单元记成已释放、放进 defer 时，已分配与空闲两个和照样对得上
            // （I-3.1 / I-5.2 都绿），只有这一条的差对不上。判据写成加法（defer + 最新根走读 == 已分配），盘上读来的值大到溢出也判红、不回绕。
            let deferred = accounting
                .get(&(STATISTIC_DEFER_QUEUE_BYTES, *device))
                .copied();
            let quarantined_exempted_newest = quarantined_exempted_against_the_newest_root
                .slot_count_per_device
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES;
            let referenced_by_the_newest_root = slots_referenced_by_the_newest_root
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES
                + quarantined_exempted_newest;
            judgements.judge_naming_the_slots(
                "I-3.11",
                matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),
                || quarantined_exempted_against_the_newest_root.start_slots_not_exempted_on(*device),
                || {
                    format!("盘 {device}：记账的已分配 {allocated:?} 减 defer 待释放 {deferred:?}，不等于从最新根（txg {newest_txg}）走读到的 {referenced_by_the_newest_root}（其中隔离豁免 {quarantined_exempted_newest}）")
                },
            );
        }
        // I-3.20 (b)（C70）：最新根记账里的已承诺预留（标签 6）对 D3（空间分配） 已定项 2 的闭式现算。副本数取这一版记账里已分配那一行的盘数：
        // 写者按分配器里的每块盘各写一行（`crates/singlefs-core/src/transaction.rs` 的 `accounting_entries_of_this_publish`），降级镜像里少一块盘也照这个数。
        judge_the_committed_reservation_against_its_closed_form(
            accounting
                .get(&(
                    STATISTIC_COMMITTED_RESERVATION_BYTES,
                    STATISTIC_NO_DEVICE_DIMENSION,
                ))
                .copied(),
            committed_reservation_basis_of_a_version_without_judging(
                reader,
                &newest_root_view.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
                &newest_root_view.record_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER],
                &mut index_node_cache,
            ),
            u64::try_from(
                accounting
                    .keys()
                    .filter(|(statistic, _)| *statistic == STATISTIC_ALLOCATED_BYTES)
                    .count(),
            )
            .expect("盘数"),
            newest_txg,
            &mut judgements,
        );
        if accounting_was_written_under_the_effective_floor {
            // I-3.16（N5 = C413）：占着的槽取 I-3.1 改成 RCW 之前那一份并集（遍历、隔离豁免、F 之下仍在 defer 的、钉带出来的），按槽认
            // （[`judge_fragmentation_and_empty_segment_statistics`]）。
            let occupied_slots: BTreeSet<(u32, u64)> = walked_slots_of_every_walked_version
                .iter()
                .chain(
                    quarantined_exempted_against_every_walked_version
                        .slots
                        .iter(),
                )
                .chain(deferred_slots_below_the_floor.iter())
                .chain(slots_the_pin_kept_in_defer.iter())
                .copied()
                .collect();
            let newest_version = &versions_with_their_root_pointers[candidate_indexes
                .iter()
                .position(|index| *index == newest_index)
                .expect("候选集里恒有最新那条根（`candidate_indexes` 的过滤条件里它恒被收）")];
            let newest_root_records = tree_table_entries_of_a_version_without_judging(
                reader,
                newest_version.tree_table_pointer,
                &mut index_node_cache,
            )
            .and_then(|tree_table_entries| {
                allocation_records_of_a_version_without_judging(
                    reader,
                    newest_version,
                    &tree_table_entries,
                    &mut index_node_cache,
                )
            });
            judge_fragmentation_and_empty_segment_statistics(
                reader,
                geometry.unit_area_start_slot,
                &accounting,
                &occupied_slots,
                &newest_root_records,
                &mut judgements,
            );
        } else {
            judgements.not_applicable(
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
                "最新根带的 F 低于 F 生效值：占着的槽那一份并集与最新根的记账不可比，占着哪几槽认不出",
            );
        }
    } else {
        judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable(
            FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
            "最新的根下面还没有记账树（第 0 代树表）",
        );
        judgements.not_applicable("I-5.2", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable(
            FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM,
            "最新的根下面还没有记账树（第 0 代树表）",
        );
        judgements.not_applicable("I-3.11", "最新的根下面还没有记账树（第 0 代树表）");
    }
    // I-3.17（隔离记号与引用相符）：最新根那棵分配记录树上判三句。
    judge_quarantine_marks_against_the_references(
        reader,
        &roots[newest_index].2,
        &slots_covered_by_the_newest_root,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_unreleased_allocation_records_against_the_central_mapping(
        reader,
        &versions_with_their_root_pointers,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-3.18（C411）：同一份版本清单，每条未释放记录的跨度对它罩住的单元那一类。
    judge_allocation_record_spans_against_the_classes_of_the_units_they_cover(
        reader,
        &versions_with_their_root_pointers,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-7.8：根环全部有效根的水位取 max，要大于盘上出现过的最大树 ID（码 2 单元头 ∪ 走过的树表条目；
    // 单元头那一半不数没发布过的孤儿，读法见 `scanned_tree_identifiers`）。
    let watermark = roots
        .iter()
        .map(|(_, _, root)| root.tree_identifier_watermark)
        .max()
        .unwrap_or(0);
    let newest_published_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .max()
        .unwrap_or(0);
    let mut seen = scanned_tree_identifiers(
        reader,
        &geometry,
        newest_published_txg,
        &instance_table_rows,
        filesystem_identifier_low,
    );
    seen.extend(walk.tree_identifiers_in_tables.iter().copied());
    let highest_seen = seen.iter().max().copied().unwrap_or(0);
    judgements.judge("I-7.8", watermark > highest_seen, || {
        format!("根环水位最大 {watermark}，盘上出现过的最大树 ID {highest_seen}")
    });
    // 再判最新那条根自己的水位（`m3-inv-a-r1` 第 9 行，零轮）：只把最新那条根的水位压低时，上面取 max 那一格被更早的根盖住，
    // 而恢复按所选根的水位往下发号（core 的 `recover` 在这种镜像上报 I-7.8）。先判上面那一格，它的违例说明不变。
    let newest_root_watermark = roots[newest_index].2.tree_identifier_watermark;
    judgements.judge("I-7.8", newest_root_watermark > highest_seen, || {
        format!(
            "最新那条根（txg {newest_txg}）的水位 {newest_root_watermark}，盘上出现过的最大树 ID {highest_seen}"
        )
    });
    // I-1.13：扫描重建（只读、不写回）与遍历方向从最新根读出的索引逐项比，接在全部判定之后、同一份 judgements；
    // 扫描结果用 I-1.8 那一遍的（已发布谓词同取最新根指着的实例表，这是重建方读根的唯一一处）。
    judge_scan_rebuild_agrees_with_the_walked_index(
        reader,
        &roots[newest_index].2.record_bytes,
        &content_units,
        abandoned_roots_filtering,
        &mut index_node_cache,
        &mut judgements,
    );
    judgements
}
