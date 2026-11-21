//! 带读故障的一段历史走到某一刻的盘（世界）、读视图（盘减去读不出的落点）与被判的三步看到的盘。

use super::*;

// ── 一段带读故障的历史走到某一刻的盘（世界）

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MountOutcome {
    /// 可写挂载读到的更新状态读不出（重读一次仍读不出），或取号那一刻有槽读不出：在取号之前拒，盘上不变，不取号。
    Refused,
    WentDownOnTheFirstRead,
    WentDownOnTheSecondRead,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MountLogEntry {
    pub(super) fault: MountFaultMenuEntry,
    pub(super) outcome: MountOutcome,
    /// 往下走的挂载：写行那次发布的 txg；被拒的挂载 0。
    pub(super) row_write_txg: u64,
}

/// 系统配置轮换那一段（登记 5.3 第 8 条）里两块盘各自写成什么样：没写、写了、撕裂（新旧都读不出）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SystemWriteOutcome {
    Unwritten,
    Written,
    Torn,
}

/// 一次发布的崩溃点（登记 5.3 第 8 条，D13 已定项 4 按设备切段）：记录那一段 2k 份拷贝的任一子集落了、根没落（4^k 个）；
/// 根落了之后，两块盘的系统配置写各取三态之一（最后一次挂载并成「都没写」一个：被判的三步不读系统配置槽）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FaultCrashPoint {
    RecordCopiesLanded {
        copy_mask: u32,
    },
    RootLanded {
        system_writes: [SystemWriteOutcome; FAULT_DEVICE_COUNT],
    },
}

pub(super) fn fault_crash_points_of(
    record_count: usize,
    is_the_last_mount: bool,
) -> Vec<FaultCrashPoint> {
    let mut crash_points: Vec<FaultCrashPoint> = (0..(1u32 << (2 * record_count)))
        .map(|copy_mask| FaultCrashPoint::RecordCopiesLanded { copy_mask })
        .collect();
    if is_the_last_mount {
        crash_points.push(FaultCrashPoint::RootLanded {
            system_writes: [SystemWriteOutcome::Unwritten; FAULT_DEVICE_COUNT],
        });
        return crash_points;
    }
    let outcomes = [
        SystemWriteOutcome::Unwritten,
        SystemWriteOutcome::Written,
        SystemWriteOutcome::Torn,
    ];
    for first_device in outcomes {
        for second_device in outcomes {
            crash_points.push(FaultCrashPoint::RootLanded {
                system_writes: [first_device, second_device],
            });
        }
    }
    crash_points
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FaultCrashLabel {
    pub(super) version: RootVersion,
    pub(super) record_count: usize,
    pub(super) point: FaultCrashPoint,
}

#[derive(Clone, Debug)]
pub(super) struct FaultWorld {
    pub(super) geometry: RootChoiceDomain,
    /// 根环三个区域的槽，区域号主序、区域内槽号升序；没写过的槽是 `None`（读成 512 个 0）。
    pub(super) root_slots: Vec<Option<Arc<RootChoicePublishMaterial>>>,
    /// 盘上至少有一份的记录，按写的次序。
    pub(super) landed_records: Vec<RootChoiceLandedRecord>,
    pub(super) system_slots: SystemSlots,
    /// 崩溃点所在那次发布之前、fsync 已返回的用户发布，按写的次序。
    pub(super) confirmed_user_publishes: Vec<RootVersion>,
    /// 版本的父链（登记 5.3 第 7 条）：每次挂载里每次发布的父是上一次发布，一次挂载第一次发布的父是这次挂载往下走所用的那一版。
    pub(super) parents: Vec<(RootVersion, RootVersion)>,
    /// 开始过的每次发布（含崩在途中的）与 mkfs 的第 0 代根，按次序；第 0 个是第 0 代根。
    pub(super) started_publishes: Vec<Arc<RootChoicePublishMaterial>>,
    pub(super) mount_log: Vec<MountLogEntry>,
    pub(super) next_instance: u32,
    pub(super) refused_mounts: u64,
    pub(super) reread_mounts: u64,
    pub(super) final_crash: Option<FaultCrashLabel>,
    /// 闸与取号那一刻的判不出取法（登记 r5 修订 1）；mkfs 之后取甲，`--gate r2` 时换成 R2。
    pub(super) witness_gate_rule: WitnessGateRule,
    /// 被拒的挂载按拒因分（下标是 [`MountRefusalReason`] 的次序；各栏之和等于 `refused_mounts`）。
    pub(super) refusals_by_reason: [u64; MOUNT_REFUSAL_REASON_COUNT],
}

/// [`MountRefusalReason`] 的成员数。
pub(super) const MOUNT_REFUSAL_REASON_COUNT: usize = MountRefusalReason::ALL.len();

impl FaultWorld {
    /// mkfs 之后、一次挂载都还没有：第 0 代根在三个区域的槽 0，别处全 0，journal 环空，每块盘两个系统配置槽都是 (世代号 1, tail 0, 实例 0)。
    pub(super) fn right_after_mkfs(geometry: RootChoiceDomain) -> Self {
        let genesis_root = Arc::new(mkfs_root_material());
        let slots_per_region = geometry.slots_per_region();
        let mut root_slots = vec![
            None;
            usize::try_from(ROOT_CHOICE_REGION_COUNT * slots_per_region)
                .expect("根槽数不超过 24")
        ];
        for region in 0..ROOT_CHOICE_REGION_COUNT {
            root_slots[root_choice_slot_index_of(region, 0, slots_per_region)] =
                Some(Arc::clone(&genesis_root));
        }
        Self {
            geometry,
            root_slots,
            landed_records: Vec::new(),
            system_slots: system_slots_right_after_mkfs(),
            confirmed_user_publishes: Vec::new(),
            parents: Vec::new(),
            started_publishes: vec![genesis_root],
            mount_log: Vec::new(),
            next_instance: 1,
            refused_mounts: 0,
            reread_mounts: 0,
            final_crash: None,
            witness_gate_rule: WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable,
            refusals_by_reason: [0; MOUNT_REFUSAL_REASON_COUNT],
        }
    }

    /// mkfs 之后、闸取 `witness_gate_rule` 的那一份（登记 r5 修订 1：`--gate r2`）。
    pub(super) fn right_after_mkfs_under_the_gate(
        geometry: RootChoiceDomain,
        witness_gate_rule: WitnessGateRule,
    ) -> Self {
        let mut world = Self::right_after_mkfs(geometry);
        world.witness_gate_rule = witness_gate_rule;
        world
    }

    pub(super) fn slots_per_region(&self) -> u64 {
        self.geometry.slots_per_region()
    }

    /// 开始过几次发布（第 0 代根不算）：新根段按全历史的发布序号取。
    pub(super) fn publishes_started(&self) -> u64 {
        u64::try_from(self.started_publishes.len() - 1).expect("发布次数不超过 u64")
    }

    pub(super) fn parent_of(&self, version: RootVersion) -> Option<RootVersion> {
        self.parents
            .iter()
            .find(|(child, _parent)| *child == version)
            .map(|(_child, parent)| *parent)
    }

    /// `version` 是 `ancestor` 本身，或在它的后代上（沿父链往上走能走到它）。
    pub(super) fn is_the_version_or_a_descendant_of(
        &self,
        version: RootVersion,
        ancestor: RootVersion,
    ) -> bool {
        let mut current = version;
        for _step in 0..=self.parents.len() {
            if current == ancestor {
                return true;
            }
            match self.parent_of(current) {
                Some(parent) => current = parent,
                None => return false,
            }
        }
        false
    }

    pub(super) fn fields_of_a_version(&self, version: RootVersion) -> Option<VersionFields> {
        self.started_publishes
            .iter()
            .find(|material| material.version == version)
            .map(|material| material.fields)
    }

    /// 根环里键 (txg, 实例代号) 最大的那个根槽（几个槽同键取读根环次序里第一个）。
    pub(super) fn newest_root_locations(&self) -> FaultLocations {
        let mut newest: Option<(usize, RootVersion)> = None;
        for (root_slot_index, root) in self.root_slots.iter().enumerate() {
            if let Some(material) = root {
                if newest.is_none_or(|(_index, version)| material.version > version) {
                    newest = Some((root_slot_index, material.version));
                }
            }
        }
        newest.map_or(FaultLocations::NOWHERE, |(root_slot_index, _version)| {
            FaultLocations::NOWHERE.with_root_slot(root_slot_index)
        })
    }

    /// 「RN 那条根的实例」的全部根所在的槽；那个实例是 0（mkfs 的第 0 代）时没有落点。
    pub(super) fn newest_instance_root_locations(&self) -> FaultLocations {
        let newest_instance = self
            .root_slots
            .iter()
            .flatten()
            .map(|material| material.version)
            .max()
            .map_or(ROOT_CHOICE_MKFS_INSTANCE, |version| version.instance);
        let mut locations = FaultLocations::NOWHERE;
        if newest_instance == ROOT_CHOICE_MKFS_INSTANCE {
            return locations;
        }
        for (root_slot_index, root) in self.root_slots.iter().enumerate() {
            if root
                .as_ref()
                .is_some_and(|material| material.version.instance == newest_instance)
            {
                locations = locations.with_root_slot(root_slot_index);
            }
        }
        locations
    }

    /// 这块盘两槽里没撕裂、世代号较大的那一槽，世代号相同取槽号大的（两槽都撕了时没有落点）。
    pub(super) fn newest_system_slot_locations(&self, device: usize) -> FaultLocations {
        let mut newest: Option<(u64, usize)> = None;
        for (slot, content) in self.system_slots[device].iter().enumerate() {
            if let SystemSlotContent::Verified { generation, .. } = content {
                if newest.is_none_or(|(largest, _slot)| *generation >= largest) {
                    newest = Some((*generation, slot));
                }
            }
        }
        newest.map_or(FaultLocations::NOWHERE, |(_generation, slot)| {
            FaultLocations::NOWHERE.with_system_slot(device, slot)
        })
    }
}

/// 这次发布最后一条记录的计数器：系统配置轮换写的 tail（登记 5.3 第 4 条：tail = c + k − 1）。
fn last_counter_of_a_publish(material: &RootChoicePublishMaterial) -> u64 {
    material
        .records
        .last()
        .expect("每次发布至少一条记录")
        .parsed
        .as_ref()
        .expect("合法历史的每条记录都解得出（生成时断言过）")
        .counter
}

impl FaultWorld {
    /// 这次发布开始了：记下它的父（登记 5.3 第 7 条）与它的材料。崩溃点与整次落盘都在这之后。
    pub(super) fn register_a_started_publish(
        &mut self,
        material: &Arc<RootChoicePublishMaterial>,
        parent: RootVersion,
    ) {
        self.parents.push((material.version, parent));
        self.started_publishes.push(Arc::clone(material));
    }

    fn put_every_record_of_a_publish_on_both_devices(
        &mut self,
        material: &Arc<RootChoicePublishMaterial>,
    ) {
        for record_index in 0..material.records.len() {
            self.landed_records.push(RootChoiceWorld::landed_record_of(
                material,
                record_index,
                [true, true],
            ));
        }
    }

    fn put_the_root_of_a_publish_in_its_slot(&mut self, material: &Arc<RootChoicePublishMaterial>) {
        let slots_per_region = self.slots_per_region();
        let (region, slot) =
            root_choice_target_of_publish(material.version.checkpoint_txg, slots_per_region);
        self.root_slots[root_choice_slot_index_of(region, slot, slots_per_region)] =
            Some(Arc::clone(material));
    }

    /// 每块盘一次系统配置写（取号与每次发布末尾的轮换），写进它的落槽。`unreadable_all_mount_long` 是这次挂载里一直读不出的落点（TP 故障）。
    pub(super) fn write_the_system_configuration_of_every_device(
        &mut self,
        tail: u64,
        instance: u32,
        unreadable_all_mount_long: FaultLocations,
    ) {
        for device in 0..FAULT_DEVICE_COUNT {
            let (slot, content) = system_slot_write_of(
                &self.system_slots,
                device,
                tail,
                instance,
                unreadable_all_mount_long,
            );
            self.system_slots[device][slot] = content;
        }
    }

    /// 这次发布整次落了：每条记录两盘各一份，根写进它的落点，两块盘各一次系统配置写（登记 5.3 第 4 条）。
    /// `is_confirmed`：它之后 fsync 返回了（只有用户发布、而且后面还有用户发布时才是）。
    pub(super) fn apply_a_publish_in_full(
        &mut self,
        material: &Arc<RootChoicePublishMaterial>,
        is_confirmed: bool,
        unreadable_all_mount_long: FaultLocations,
    ) {
        self.put_every_record_of_a_publish_on_both_devices(material);
        self.put_the_root_of_a_publish_in_its_slot(material);
        self.write_the_system_configuration_of_every_device(
            last_counter_of_a_publish(material),
            material.version.instance,
            unreadable_all_mount_long,
        );
        if is_confirmed {
            self.confirmed_user_publishes.push(material.version);
        }
    }

    /// 这次发布崩在 `point` 上之后的盘。
    pub(super) fn with_crash_applied(
        &self,
        material: &Arc<RootChoicePublishMaterial>,
        point: FaultCrashPoint,
        unreadable_all_mount_long: FaultLocations,
    ) -> Self {
        let mut world = self.clone();
        match point {
            FaultCrashPoint::RecordCopiesLanded { copy_mask } => {
                for record_index in 0..material.records.len() {
                    let copies_on_devices = copies_of_a_record_in_a_mask(copy_mask, record_index);
                    if copies_on_devices[0] || copies_on_devices[1] {
                        world.landed_records.push(RootChoiceWorld::landed_record_of(
                            material,
                            record_index,
                            copies_on_devices,
                        ));
                    }
                }
            }
            FaultCrashPoint::RootLanded { system_writes } => {
                world.put_every_record_of_a_publish_on_both_devices(material);
                world.put_the_root_of_a_publish_in_its_slot(material);
                for (device, system_write) in system_writes.iter().enumerate() {
                    let (slot, content) = system_slot_write_of(
                        &self.system_slots,
                        device,
                        last_counter_of_a_publish(material),
                        material.version.instance,
                        unreadable_all_mount_long,
                    );
                    match system_write {
                        SystemWriteOutcome::Unwritten => {}
                        SystemWriteOutcome::Written => world.system_slots[device][slot] = content,
                        SystemWriteOutcome::Torn => {
                            world.system_slots[device][slot] = SystemSlotContent::Torn;
                        }
                    }
                }
            }
        }
        world.final_crash = Some(FaultCrashLabel {
            version: material.version,
            record_count: material.records.len(),
            point,
        });
        world
    }
}

// ── 读视图：盘减去读不出的落点

/// 世界在 `locations` 这个读视图里的样子：读不出的根槽当没有根，盘不在时那块盘上的记录拷贝读不出（只有一份在那块盘上的记录整条读不出）。
pub(super) fn fault_landing_of_a_view(
    world: &FaultWorld,
    locations: FaultLocations,
) -> RootChoiceLanding<'_> {
    let slots_per_region = world.slots_per_region();
    let roots_in_ring_order: Vec<Option<&RootChoicePublishMaterial>> = world
        .root_slots
        .iter()
        .enumerate()
        .map(|(root_slot_index, root)| {
            if locations.root_slot_is_unreadable(root_slot_index, slots_per_region) {
                None
            } else {
                root.as_deref()
            }
        })
        .collect();
    let mut records = Vec::with_capacity(world.landed_records.len());
    for landed in &world.landed_records {
        let copies_on_devices = [
            landed.copies_on_devices[0] && !locations.device_is_dead(0),
            landed.copies_on_devices[1] && !locations.device_is_dead(1),
        ];
        if copies_on_devices[0] || copies_on_devices[1] {
            records.push(RootChoiceLandedRecordRef {
                material: &landed.material,
                record_index: landed.record_index,
                copies_on_devices,
            });
        }
    }
    RootChoiceLanding {
        roots_in_ring_order,
        records,
    }
}

/// 条款的理想恢复在这个读视图上落到哪一版（登记 5.5 KJ3 的写法：读视图里版本最大的根起，同实例逐次往后）；读视图里一条根都没有时 `None`。
pub(super) fn ideal_landed_version_of_a_view(
    view: &RootChoiceLanding<'_>,
    in_flight_limit: usize,
) -> Option<RootVersion> {
    root_choice_newest_root(view).map(|newest| {
        root_choice_ideal_recovery_from(newest, &view.records, in_flight_limit).landed_version
    })
}

/// 可写挂载读到的更新状态读不出那道闸（D23 已定项 14）：`landed` 是读阶段落到的那一版，`locations` 是这一遍读视图里读不出的落点。
/// 为真 = 判不出或见证到比所选那一版更新的发布。判不出的取法跟 `world.witness_gate_rule`：取甲（r4 登记修订 1）或 R2（r5 登记修订 1）。
/// 写法照 `mount.rs` 的 `newer_publish_witness` 那一段条款文字，不调它。
pub(super) fn newer_publish_is_witnessed(
    world: &FaultWorld,
    locations: FaultLocations,
    landed: RootVersion,
) -> bool {
    let (readable_slot_count, largest_tail) =
        readable_system_slots_of(&world.system_slots, locations);
    match world.witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {}
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            return newer_publish_is_witnessed_under_the_read_error_rule(
                world,
                locations,
                landed,
                largest_tail,
            );
        }
    }
    if readable_slot_count < FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL {
        return true;
    }
    if largest_tail == 0 {
        return false;
    }
    newer_publish_is_witnessed_against_the_witnessed_tail(world, locations, landed, largest_tail)
}

/// 闸的 R2 支（登记 r5 修订 1 修1.五第 2 条；D22（单元原子性怎么合成） 已定项 8 射程）：判不出 ⟺ 这一遍读视图里有系统配置槽报读错，
/// 且没有一块盘两槽都读得出且没撕裂 ⇒ 真；c_见证（读得出且没撕裂的槽 tail 的最大值）= 0 ⇒ 假；其余与取甲同一段比法。
fn newer_publish_is_witnessed_under_the_read_error_rule(
    world: &FaultWorld,
    locations: FaultLocations,
    landed: RootVersion,
    largest_tail: u64,
) -> bool {
    if system_configuration_witness_is_undecidable_under_the_read_error_rule(
        &world.system_slots,
        locations,
    ) {
        return true;
    }
    let nothing_is_witnessed = largest_tail == 0;
    if nothing_is_witnessed {
        return false;
    }
    newer_publish_is_witnessed_against_the_witnessed_tail(world, locations, landed, largest_tail)
}

/// 闸判得出、c_见证 > 0 之后的比法（`mount.rs` 的 `newer_publish_witness` 那三支的条款文字）：所选那一版末条读得出 ⇒ c_见证 > 它的计数器；
/// 否则计数器等于 c_见证 的记录读得出 ⇒ 键最大那条的 (实例代号, txg) > 所选那一版；都读不出 ⇒ 真。
fn newer_publish_is_witnessed_against_the_witnessed_tail(
    world: &FaultWorld,
    locations: FaultLocations,
    landed: RootVersion,
    largest_tail: u64,
) -> bool {
    let view = fault_landing_of_a_view(world, locations);
    let last_record_counter_of_the_landed_publish = view
        .records
        .iter()
        .filter(|record| {
            record.material.version == landed
                && record.record_index + 1 == record.material.records.len()
        })
        .map(counter_of_the_landed_record)
        .max();
    if let Some(counter) = last_record_counter_of_the_landed_publish {
        return largest_tail > counter;
    }
    let newest_of_the_record_at_the_tail = view
        .records
        .iter()
        .filter(|record| counter_of_the_landed_record(record) == largest_tail)
        .map(|record| {
            (
                record.material.version.instance,
                record.material.version.checkpoint_txg,
            )
        })
        .max();
    match newest_of_the_record_at_the_tail {
        Some(instance_and_txg) => instance_and_txg > (landed.instance, landed.checkpoint_txg),
        None => true,
    }
}

// ── 被判的三步看到的盘：真相的内存盘加上读故障

/// 三步调用的读者：真相的内存盘（根槽、记录槽摆在各自的落点上），再减去读不出的落点：根槽交读错，盘不在时那块盘上的读全交读错。
pub(super) struct FaultedRootChoiceDisk<'disk, 'landing> {
    pub(super) inner: &'disk RootChoiceDisk<'landing>,
    pub(super) locations: FaultLocations,
    pub(super) slots_per_region: u64,
}

impl PoolReader for FaultedRootChoiceDisk<'_, '_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.inner.device_identities()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.inner.device_size_in_bytes(device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        self.inner.physical_block_size_probed_on_the_device(device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        if let Ok(device_index) = usize::try_from(device.0) {
            if device_index < FAULT_DEVICE_COUNT && self.locations.device_is_dead(device_index) {
                return None;
            }
        }
        if let Some(root_slot_index) =
            root_choice_root_slot_index_at(device, offset, self.slots_per_region)
        {
            if self
                .locations
                .root_slot_is_unreadable(root_slot_index, self.slots_per_region)
            {
                return None;
            }
        }
        self.inner.read(device, offset, length)
    }

    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}
