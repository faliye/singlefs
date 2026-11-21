use super::*;

// ───────────────────────────── 造池（登记第五节「负载：池怎么造」） ─────────────────────────────

/// 建 inode 时在 P 个之外多建的空 inode：几何核之前把开着的那一段用户数据补满用（[`construct_the_pool`] 第 4 步）。
pub(super) const SPARE_INODES_FOR_THE_FRONTIER: u64 = 64;
/// 抬 F 到准入上限、再建一个 inode 那一圈最多转几圈（每圈之后核被覆盖写的旧落点都已释放、回到空闲）。
const RELEASE_ROUNDS_AT_MOST: u64 = 16;
/// 建池时用户改动写进记录的时间：第一个文件之后 600 秒起，每次改动加一。
pub(super) const SECONDS_FROM_THE_FIRST_FILE_TO_THE_WORKLOAD: u64 = 600;

/// 造池的配方：取样点、覆盖写的次序，加上几何没造成时累计改负载多覆盖写掉的 inode（登记第五节「先改负载重造」）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PoolRecipe {
    pub(super) point: SamplingPoint,
    pub(super) overwrite_order: OverwriteOrder,
    pub(super) extra_overwritten_inodes: BTreeSet<u64>,
}

/// 造池第 2 步覆盖写的次序（第 2 次重跑登记第一节「改法甲」、第五节改动一）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OverwriteOrder {
    /// 改法甲：占比 0.25 时三组依次写（[`OverwriteGroup`]），槽号最大的那一整段留到最后；占比 0.75 时与按旧槽号升序相同。
    HighestFullSegmentLast,
    /// 第 1 次重跑那一版：全部按旧槽号升序（`geometry-check --overwrite-order slot-order` 的对照）。
    SlotOrder,
}

impl OverwriteOrder {
    pub(super) const fn label(self) -> &'static str {
        match self {
            OverwriteOrder::HighestFullSegmentLast => "highest_full_segment_last",
            OverwriteOrder::SlotOrder => "slot_order",
        }
    }

    /// 命令行 `--overwrite-order` 的值：只认 `slot-order`（不给这个选项就是改法甲）。
    pub(super) fn of_argument(argument: &str) -> Option<OverwriteOrder> {
        (argument == "slot-order").then_some(OverwriteOrder::SlotOrder)
    }
}

/// 造池第 2 步要覆盖写的一个单元（排次序用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnitToOverwrite {
    old_slot: u64,
    inode: u64,
    segment_start_slot: u64,
    /// 它所在的段 32 个数据单元都是这次写的（「整段」）。
    is_in_a_segment_full_of_this_workload: bool,
    /// 配方里累计改负载点名要多覆盖写掉的 inode。
    is_named_by_the_recipe: bool,
}

/// 改法甲的三组（登记第一节「改法甲」那一格），按写的先后。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverwriteGroup {
    /// 不在「整段都是这次写的」段里的单元（混住段的全部），与配方累计点名的：最先写。
    MixedSegmentsAndNamedByTheRecipe,
    /// 整段里要覆盖写的单元，槽号最大的那一整段之外的。
    FullSegmentsBelowTheHighest,
    /// 槽号最大的那一整段里要覆盖写的（占比 0.25 时 24 个）：最后写。
    HighestFullSegment,
}

impl OverwriteGroup {
    /// 写的先后：小的先写。
    const fn writing_rank(self) -> u8 {
        match self {
            OverwriteGroup::MixedSegmentsAndNamedByTheRecipe => 0,
            OverwriteGroup::FullSegmentsBelowTheHighest => 1,
            OverwriteGroup::HighestFullSegment => 2,
        }
    }
}

/// 覆盖写的次序（改动一）：交回（旧槽、inode），按写的先后。改法甲只对占比 0.25 生效：三组依次、组内按旧槽号升序；
/// 占比 0.75 与 `slot-order` 一律按旧槽号升序（与第 1 次重跑 `sort_unstable` 逐项相同，旧槽号互不相同）。覆盖写哪些单元不变，只换次序。
fn overwrite_order_of(
    fraction: LiveSlotFraction,
    overwrite_order: OverwriteOrder,
    units: &[UnitToOverwrite],
) -> Vec<(u64, u64)> {
    let mut in_writing_order = units.to_vec();
    in_writing_order.sort_unstable_by_key(|unit| (unit.old_slot, unit.inode));
    let is_the_highest_full_segment_kept_last = match (overwrite_order, fraction) {
        (OverwriteOrder::HighestFullSegmentLast, LiveSlotFraction::OneQuarter) => true,
        (OverwriteOrder::HighestFullSegmentLast, LiveSlotFraction::ThreeQuarters) => false,
        (OverwriteOrder::SlotOrder, LiveSlotFraction::OneQuarter) => false,
        (OverwriteOrder::SlotOrder, LiveSlotFraction::ThreeQuarters) => false,
    };
    if is_the_highest_full_segment_kept_last {
        let full_segment_starts: BTreeSet<u64> = in_writing_order
            .iter()
            .filter(|unit| unit.is_in_a_segment_full_of_this_workload)
            .map(|unit| unit.segment_start_slot)
            .collect();
        let highest_full_segment_start_slot = full_segment_starts.last().copied();
        let group_of = |unit: &UnitToOverwrite| {
            if !unit.is_in_a_segment_full_of_this_workload || unit.is_named_by_the_recipe {
                OverwriteGroup::MixedSegmentsAndNamedByTheRecipe
            } else if Some(unit.segment_start_slot) == highest_full_segment_start_slot {
                OverwriteGroup::HighestFullSegment
            } else {
                OverwriteGroup::FullSegmentsBelowTheHighest
            }
        };
        // 稳定排序：同一组里保持上面排好的旧槽号升序
        in_writing_order.sort_by_key(|unit| group_of(unit).writing_rank());
    }
    in_writing_order
        .iter()
        .map(|unit| (unit.old_slot, unit.inode))
        .collect()
}

/// 数不数覆盖写副本的落点（第六节 N12）。封闭集合。数与不数造出的池逐块相同（单测
/// `counting_the_overwrite_landings_leaves_the_pool_digests_unchanged`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LandingCount {
    Counted,
    NotCounted,
}

/// 造池第 2 步覆盖写的副本落在哪（N12）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct OverwriteLanding {
    /// 新副本的槽是这次造池前面某次覆盖写空出来的旧槽。
    pub(super) copies_landing_in_holes_of_this_construction: u64,
    pub(super) copies_landing_elsewhere: u64,
}

/// 第 j 份副本的新槽在第 0 到 j − 1 次覆盖写的旧槽里，就算落进这次造池打出的旧洞；两串按写的先后、一一对应。
fn landings_of_the_copies(
    old_slots_in_writing_order: &[u64],
    new_slots_in_writing_order: &[u64],
) -> OverwriteLanding {
    let mut holes_opened_so_far: BTreeSet<u64> = BTreeSet::new();
    let mut landing = OverwriteLanding {
        copies_landing_in_holes_of_this_construction: 0,
        copies_landing_elsewhere: 0,
    };
    for (old_slot, new_slot) in old_slots_in_writing_order
        .iter()
        .zip(new_slots_in_writing_order)
    {
        if holes_opened_so_far.contains(new_slot) {
            landing.copies_landing_in_holes_of_this_construction += 1;
        } else {
            landing.copies_landing_elsewhere += 1;
        }
        holes_opened_so_far.insert(*old_slot);
    }
    landing
}

/// 推的落进旧洞的副本数（第七节「推的预测」，不是锚点）：覆盖写到第 25 次起每一份都落进 24 次之前打出的洞，共 `覆盖写份数 − 24`。
pub(super) const fn predicted_copies_landing_in_holes(overwritten_units: u64) -> u64 {
    overwritten_units.saturating_sub(LOCAL_ROOT_RING_ENTRIES)
}

/// 造好的池：两块内存盘（关掉、没卸载）与造的经过里要报的数。
pub(super) struct ConstructedPool {
    pub(super) devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    /// 现行那一版里这次写的数据单元：槽（盘 0）→ inode。
    pub(super) live_data_inode_by_slot: BTreeMap<u64, u64>,
    pub(super) unit_area_start_slot: u64,
    pub(super) overwritten_units: u64,
    pub(super) filler_units: u64,
    pub(super) floor_raises_pushed_by_the_session: u64,
    pub(super) release_rounds: u64,
    pub(super) is_every_overwritten_slot_released: bool,
    pub(super) construction_seconds: f64,
    /// 覆盖写副本的落点（N12）；没数（[`LandingCount::NotCounted`]）是 `None`。
    pub(super) overwrite_landing: Option<OverwriteLanding>,
}

/// 一个数据单元装满的内容：按 inode 与第几次写的不同而不同（净荷容量那么长，正好一个数据单元）。
fn content_of_a_data_unit(inode: u64, write_generation: u64) -> Vec<u8> {
    let capacity = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
    (0..capacity)
        .map(|position| {
            u8::try_from((inode * 131 + write_generation * 17 + position * 7) % 251)
                .expect("小于 251")
        })
        .collect()
}

/// 有树的那一版（建池与测量的会话里，第一个文件之后每一版都有树）。
pub(super) fn version_of_trees(session: &MountedSession) -> &TransactionOutput {
    session
        .current
        .version_of_trees()
        .expect("第一个文件发过之后，会话的现行那一版都有树")
}

/// 这一版里 `inode` 那个文件第一个数据单元的槽（盘 0）。
fn data_unit_slot_of(version: &TransactionOutput, inode: u64) -> u64 {
    let file = version
        .files_other_than_the_first
        .iter()
        .find(|file| file.inode == InodeNumber(inode))
        .expect("刚写过数据的 inode 在这一版的文件表里");
    version.unit(file.data_role_at(0)).slot.0
}

/// 一段的起点槽（单元区起点在聚簇段边界上）。
pub(super) const fn segment_start_of(slot: u64, unit_area_start_slot: u64) -> u64 {
    unit_area_start_slot
        + (slot - unit_area_start_slot) / LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_CLUSTER_SEGMENT_SLOTS
}

/// 建池会话发一次用户改动，计它推的抬 F 串数。
pub(super) fn publish_while_constructing(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
    change: UserChange<'_>,
    floor_raises: &mut u64,
) {
    let published = session
        .publish_user_change(devices, change)
        .unwrap_or_else(|refusal| panic!("建池的用户改动发得成：{refusal:?}"));
    *floor_raises += u64::try_from(published.floor_raises.len()).expect("抬 F 串数");
}

/// 抬 F 到准入上限（会话的分配器与现行那一版，同 harness 整理流前缀的做法）。
fn raise_the_floor_to_the_admission_ceiling<Device: BlockDevice>(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) {
    let mut current = version_of_trees(session).clone();
    let _raised = raise_rollback_floor_to_the_admission_ceiling(
        &experiment_pool::parameters(),
        devices,
        &mut session.allocator,
        &mut current,
        session.shadow_ledger,
    )
    .expect("抬 F 到准入上限");
    session.current = PoolVersion::of_a_version_of_trees(current);
}

/// 盘 0 上被未释放记录罩住的槽。
fn slots_under_an_unreleased_record_on_device_zero(version: &TransactionOutput) -> BTreeSet<u64> {
    version
        .allocation_records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && !record.is_released)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// mkfs → 取号 → 暖机 → 新池新建文件（同实验包库的 `experiment_pool::build_pool`，只是落在两块内存盘上、不录流），再可写挂载开会话。
pub(super) fn formatted_pool_with_the_first_file_and_a_session(
) -> (Vec<(DeviceIdentity, SparseBlockDevice)>, MountedSession) {
    let parameters = experiment_pool::parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0_u32, 1]
        .into_iter()
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(
                    LOCAL_DEVICE_BYTES,
                    PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file_content = experiment_pool::file_content();
        let _first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_file_content,
                write_time_seconds: experiment_pool::FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable_with_compaction_switches(
        &parameters,
        &mut devices,
        CompactionSwitches::PRODUCT,
    )
    .expect("建池那一次可写挂载");
    let (_mount_output, session) = MountedSession::of_the_mount(mounted);
    (devices, session)
}

/// 按配方造一个池（登记第五节「负载：池怎么造」）：
/// 1. 建 P + 64 个空 inode，前 P 个各顺序写一个数据单元（按段顺序写满 ⌈P ÷ 32⌉ 段；用户数据取最低的空槽对）；
/// 2. 整段 32 个数据单元都是这次写的段，按槽序每 4 个覆盖写其中 3 个（f = 0.25）或 1 个（f = 0.75）；没写满一整段的段（与别的单元混住的）
///    这次写的单元全覆盖写掉，配方里累计改负载点名的 inode 也覆盖写掉；次序照配方的 [`OverwriteOrder`]（[`overwrite_order_of`]），
///    `landing_count` 是 [`LandingCount::Counted`] 时逐份记下新副本的槽、数落点（N12；只读，不改池）；
/// 3. 下一个用户数据落点还在最高那一段里时，用多建的空 inode 一个一个写，把那一段补满（免得留一段活槽少的新数据段）；
/// 4. 抬 F 到准入上限、建一个 inode，转到被覆盖写的旧落点都按记录已释放、回到空闲（或已被别的这次写的单元占回）为止。
///
/// 关掉（不卸载）交回两块内存盘；测量那一侧再 `mount_writable_with_compaction_switches` 可写挂载，这样造出来的段不是那次挂载开过的段。
pub(super) fn construct_the_pool(
    recipe: &PoolRecipe,
    landing_count: LandingCount,
) -> ConstructedPool {
    let started = Instant::now();
    let (mut devices, mut session) = formatted_pool_with_the_first_file_and_a_session();
    let mut floor_raises = 0_u64;
    let mut write_time_seconds =
        experiment_pool::FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_FIRST_FILE_TO_THE_WORKLOAD;
    let mut next_write_time = || {
        write_time_seconds += 1;
        write_time_seconds
    };
    let first_inode = version_of_trees(&session).inode_number_watermark();
    let pool_data_units = recipe.point.pool_data_units;
    publish_while_constructing(
        &mut session,
        &mut devices,
        UserChange::NewInodes {
            count: pool_data_units + SPARE_INODES_FOR_THE_FRONTIER,
            write_time_seconds: next_write_time(),
        },
        &mut floor_raises,
    );
    let mut workload = WorkloadUnits {
        live_data_inode_by_slot: BTreeMap::new(),
        slot_of_inode: BTreeMap::new(),
        floor_raises,
    };
    for inode in first_inode..first_inode + pool_data_units {
        let write_time = next_write_time();
        workload.write_one_unit(&mut session, &mut devices, inode, 0, write_time);
    }
    let unit_area_start_slot = session.allocator.devices[0].unit_area_start().slot().0;
    let full_segment_units = LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_SLOT_BYTES / LOCAL_DATA_UNIT_BYTES;
    let mut units_by_segment: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for (slot, inode) in &workload.live_data_inode_by_slot {
        units_by_segment
            .entry(segment_start_of(*slot, unit_area_start_slot))
            .or_default()
            .push(*inode);
    }
    let mut units_to_overwrite: Vec<UnitToOverwrite> = Vec::new();
    for (segment_start_slot, inodes_in_slot_order) in &units_by_segment {
        let is_full_of_this_workload =
            u64::try_from(inodes_in_slot_order.len()).expect("一段的单元数") == full_segment_units;
        for (position, inode) in (0_u64..).zip(inodes_in_slot_order) {
            let is_named_by_the_recipe = recipe.extra_overwritten_inodes.contains(inode);
            if !is_full_of_this_workload
                || recipe.point.fraction.is_overwritten(position)
                || is_named_by_the_recipe
            {
                units_to_overwrite.push(UnitToOverwrite {
                    old_slot: workload.slot_of_inode[inode],
                    inode: *inode,
                    segment_start_slot: *segment_start_slot,
                    is_in_a_segment_full_of_this_workload: is_full_of_this_workload,
                    is_named_by_the_recipe,
                });
            }
        }
    }
    let inodes_to_overwrite = overwrite_order_of(
        recipe.point.fraction,
        recipe.overwrite_order,
        &units_to_overwrite,
    );
    let old_slots: Vec<u64> = inodes_to_overwrite
        .iter()
        .map(|(slot, _inode)| *slot)
        .collect();
    let mut new_slots_in_writing_order: Vec<u64> = Vec::new();
    for (_old_slot, inode) in &inodes_to_overwrite {
        let write_time = next_write_time();
        workload.write_one_unit(&mut session, &mut devices, *inode, 1, write_time);
        match landing_count {
            LandingCount::Counted => {
                new_slots_in_writing_order.push(workload.slot_of_inode[inode]);
            }
            LandingCount::NotCounted => {}
        }
    }
    let overwrite_landing = match landing_count {
        LandingCount::Counted => Some(landings_of_the_copies(
            &old_slots,
            &new_slots_in_writing_order,
        )),
        LandingCount::NotCounted => None,
    };
    let mut filler_units = 0_u64;
    let frontier_segment = workload
        .live_data_inode_by_slot
        .keys()
        .max()
        .map(|slot| segment_start_of(*slot, unit_area_start_slot));
    let first_spare_inode = first_inode + pool_data_units;
    let mut spare_inodes = first_spare_inode..first_spare_inode + SPARE_INODES_FOR_THE_FRONTIER;
    // 迭代上界：一段至多 32 个数据单元；多建的空 inode 用完也停。
    for _filler in 0..full_segment_units {
        let next_data_slot = session.allocator.devices[0]
            .lowest_user_data_slot(session.allocator.cluster_segments());
        let (Some(frontier), Some(next_slot)) = (frontier_segment, next_data_slot) else {
            break;
        };
        if segment_start_of(next_slot.0, unit_area_start_slot) != frontier {
            break;
        }
        let Some(spare_inode) = spare_inodes.next() else {
            break;
        };
        let write_time = next_write_time();
        workload.write_one_unit(&mut session, &mut devices, spare_inode, 0, write_time);
        filler_units += 1;
    }
    let mut release_rounds = 0_u64;
    let mut is_every_overwritten_slot_released = false;
    while release_rounds < RELEASE_ROUNDS_AT_MOST {
        release_rounds += 1;
        raise_the_floor_to_the_admission_ceiling(&mut session, &mut devices);
        let under_a_record =
            slots_under_an_unreleased_record_on_device_zero(version_of_trees(&session));
        is_every_overwritten_slot_released = old_slots.iter().all(|slot| {
            workload.live_data_inode_by_slot.contains_key(slot)
                || (!under_a_record.contains(slot)
                    && session.allocator.devices[0].is_free(SlotNumber(*slot)))
        });
        if is_every_overwritten_slot_released {
            break;
        }
        let write_time = next_write_time();
        publish_while_constructing(
            &mut session,
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: write_time,
            },
            &mut workload.floor_raises,
        );
    }
    drop(session);
    ConstructedPool {
        devices,
        live_data_inode_by_slot: workload.live_data_inode_by_slot,
        unit_area_start_slot,
        overwritten_units: u64::try_from(old_slots.len()).expect("覆盖写的单元数"),
        filler_units,
        floor_raises_pushed_by_the_session: workload.floor_raises,
        release_rounds,
        is_every_overwritten_slot_released,
        construction_seconds: started.elapsed().as_secs_f64(),
        overwrite_landing,
    }
}

/// 建池时这次写的数据单元在哪（槽 → inode、inode → 槽）与会话推的抬 F 串数。
pub(super) struct WorkloadUnits {
    pub(super) live_data_inode_by_slot: BTreeMap<u64, u64>,
    pub(super) slot_of_inode: BTreeMap<u64, u64>,
    pub(super) floor_raises: u64,
}

impl WorkloadUnits {
    /// 给 `inode` 写一个数据单元：第 0 次是顺序写（空 inode），之后是覆盖写；记下它的新槽，旧槽上还记着它的就抹掉。
    pub(super) fn write_one_unit(
        &mut self,
        session: &mut MountedSession,
        devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
        inode: u64,
        write_generation: u64,
        write_time: u64,
    ) {
        let content = content_of_a_data_unit(inode, write_generation);
        let change = if write_generation == 0 {
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(inode),
                content: &content,
                write_time_seconds: write_time,
            }
        } else {
            UserChange::OverwriteOfAnInode {
                inode: InodeNumber(inode),
                content: &content,
                write_time_seconds: write_time,
            }
        };
        publish_while_constructing(session, devices, change, &mut self.floor_raises);
        let new_slot = data_unit_slot_of(version_of_trees(session), inode);
        if let Some(old_slot) = self.slot_of_inode.insert(inode, new_slot) {
            if self.live_data_inode_by_slot.get(&old_slot) == Some(&inode) {
                self.live_data_inode_by_slot.remove(&old_slot);
            }
        }
        self.live_data_inode_by_slot.insert(new_slot, inode);
    }
}

#[cfg(test)]
mod tests;
