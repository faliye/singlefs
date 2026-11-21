//! 带读故障的择根合判一族（登记 `research/prompts/e165-r4-prereg.md` 5.3 至 5.6、`research/prompts/e165-r7-prereg.md` 1.3）的格、读故障的落点、系统配置槽与挂载故障菜单。

use super::*;

// ── 带读故障的择根合判一族（登记 `research/prompts/e165-r4-prereg.md` 5.3 至 5.6、第七节：KF、KF3、KF9、KFW、KG，与第 7 次重跑登记 `research/prompts/e165-r7-prereg.md` 1.3 的 KL；只在 `--profile r4 --family root_choice_faults` 时跑）
//
// 被测对象与 r3 的择根合判一族相同：`recover` 的 `Consult` 支里串起来的三步（`choose_root`、`scan_journal`、`replay_journal(…, true)`）。
// 这一族多出来的东西全在生成器一侧（可信件，按登记 5.3 与 D16 已定项 7、8，D22 已定项 7、8、16，D23 已定项 14、18 的字面写，不调 `mount.rs`、`transaction.rs`）：
// 每块盘两个系统配置槽、挂载开头的读故障（根槽、系统配置槽的一时读不出）、可写挂载读到的更新状态读不出那道闸（不给 `--gate` 是 r4 登记修订 1 的取甲；
// `--gate r2` 是 r5 登记修订 1 的 R2）、取号那一写与发布末尾的系统配置轮换。
// 被判的那一次恢复另有 5 种读故障（无、RN、RI、D0、D1）；故障只落在读者一侧，盘上的字节不变。

pub(super) const FAULT_FAMILY_LABEL: &str = "root_choice_faults";
/// 每块盘的系统配置槽数、mkfs 写下的世代号、可写挂载的设备数下限：本地常量，抄自 D22 已定项 16「每盘恒 2 个槽……槽世代号从 1 起」与 D2 已定项 13
/// 「可写设备数 ≥ w 的下限」（w = 2）；[`cross_check_root_choice_fault_local_constants_against_the_implementation`] 逐个回比。
pub(super) const FAULT_SYSTEM_SLOTS_PER_DEVICE: usize = 2;
pub(super) const FAULT_MKFS_SYSTEM_GENERATION: u64 = 1;
pub(super) const FAULT_WRITABLE_DEVICE_COUNT_LOWER_BOUND: usize = 2;
pub(super) const FAULT_DEVICE_COUNT: usize = 2;
/// 整池的系统配置槽数：可写挂载读到的自证过的槽不足这个数就判不出（实现的取甲，登记修订 1）。
pub(super) const FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL: usize =
    FAULT_DEVICE_COUNT * FAULT_SYSTEM_SLOTS_PER_DEVICE;
/// 去掉在飞上限的理想恢复用（登记 5.5「上限截掉」：去掉上限之后成立）。
pub(super) const FAULT_IN_FLIGHT_LIMIT_REMOVED: usize = 1_000_000_000;
/// 挂载开头故障菜单的项数（登记 5.3 第 5 条：无，RN、SN0、SN1、SB、RI、RNSB 六种落点各配 T1 / TP 两种时机）。
pub(super) const MOUNT_FAULT_MENU_LENGTH: usize = 13;
pub(super) const MOUNT_FAULT_MENU_LABELS: [&str; MOUNT_FAULT_MENU_LENGTH] = [
    "none", "RN-T1", "RN-TP", "SN0-T1", "SN0-TP", "SN1-T1", "SN1-TP", "SB-T1", "SB-TP", "RI-T1",
    "RI-TP", "RNSB-T1", "RNSB-TP",
];

impl FaultDomain {
    /// 这个域的几何（环槽数、每区槽数）与 r3 的哪个域相同。
    pub(super) const fn geometry(self) -> RootChoiceDomain {
        match self {
            Self::GateOn | Self::GateOnThreeMounts | Self::GateOff => RootChoiceDomain::BigRing,
            Self::GateOnSmallRing => RootChoiceDomain::SmallRingNine,
            Self::GateOnWrappingRootRing => RootChoiceDomain::WrappingRootRing,
            Self::AdversarialRootRing => RootChoiceDomain::AdversarialRootRing,
        }
    }

    /// 用户发布每次的记录条数（登记 5.4：KF、KF9、KG 取 {1, 2}，KF3、KFW 取 {1}）。
    pub(super) const fn user_publish_record_counts(self) -> &'static [usize] {
        match self {
            Self::GateOn | Self::GateOnSmallRing | Self::GateOff => &[1, 2],
            Self::GateOnThreeMounts | Self::GateOnWrappingRootRing => &[1],
            Self::AdversarialRootRing => &[],
        }
    }

    /// 条款的两道保护开着：可写挂载判 N-配置（实现的取甲）与取号那一刻、写行 txg 看记录（登记 5.3 第 6 条）。KG 关掉。
    pub(super) const fn gate_is_on(self) -> bool {
        match self {
            Self::GateOn
            | Self::GateOnThreeMounts
            | Self::GateOnSmallRing
            | Self::GateOnWrappingRootRing => true,
            Self::GateOff | Self::AdversarialRootRing => false,
        }
    }

    /// 合法历史：Q1、Q2 的域。KG 与 KL 不是。
    pub(super) const fn is_a_legal_history(self) -> bool {
        self.gate_is_on()
    }

    pub(super) const fn has_histories(self) -> bool {
        match self {
            Self::AdversarialRootRing => false,
            Self::GateOn
            | Self::GateOnThreeMounts
            | Self::GateOnSmallRing
            | Self::GateOnWrappingRootRing
            | Self::GateOff => true,
        }
    }

    /// 全界（登记 5.4：KF M ≤ 2、U ≤ 3；KF3 M ≤ 3、U ≤ 3；KF9 M ≤ 2、U ≤ 2；KFW M = 1、U ≤ 14；KG M ≤ 2、U ≤ 3）。
    pub(super) const fn largest_mounts(self, tier: BoundsTier) -> usize {
        match (self, tier) {
            (Self::AdversarialRootRing, _) => 0,
            (Self::GateOnThreeMounts, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 3,
            (
                Self::GateOn | Self::GateOnSmallRing | Self::GateOff,
                BoundsTier::SegmentOne | BoundsTier::SegmentTwo,
            ) => 2,
            (Self::GateOnWrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 1,
        }
    }

    pub(super) const fn largest_user_publishes(self, tier: BoundsTier) -> usize {
        match (self, tier) {
            (Self::AdversarialRootRing, _) => 0,
            (
                Self::GateOn | Self::GateOnThreeMounts | Self::GateOff,
                BoundsTier::SegmentOne | BoundsTier::SegmentTwo,
            ) => 3,
            (Self::GateOnSmallRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 2,
            (Self::GateOnWrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 14,
        }
    }
}

/// 一个域在这一档、这一次命令行下要跑的格，按 (M 升序, U 升序)。`largest_user_publishes_cap` 是 `--max-u`（只给调试用）。
pub(super) fn fault_cells_of(
    domain: FaultDomain,
    tier: BoundsTier,
    largest_user_publishes_cap: Option<usize>,
) -> Vec<RootChoiceCell> {
    if !domain.has_histories() {
        return vec![RootChoiceCell {
            mounts: 0,
            user_publishes: 0,
        }];
    }
    let largest_user_publishes = largest_user_publishes_cap
        .map_or(domain.largest_user_publishes(tier), |cap| {
            cap.min(domain.largest_user_publishes(tier))
        });
    let mut cells = Vec::new();
    for mounts in 1..=domain.largest_mounts(tier) {
        for user_publishes in 0..=largest_user_publishes {
            cells.push(RootChoiceCell {
                mounts,
                user_publishes,
            });
        }
    }
    cells
}

// ── 读故障的落点（登记 5.3 第 5 条）：根槽、系统配置槽、整块盘（盘不在）；journal 记录槽不单独加读故障，盘不在时那块盘上的记录拷贝随之读不出

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct FaultLocations {
    /// 位 = 根槽下标（区域号主序）。
    root_slot_bits: u32,
    /// 位 = 盘号 × 每盘槽数 + 槽号。
    system_slot_bits: u8,
    /// 位 = 盘号。
    dead_device_bits: u8,
}

impl FaultLocations {
    pub(super) const NOWHERE: Self = Self {
        root_slot_bits: 0,
        system_slot_bits: 0,
        dead_device_bits: 0,
    };

    pub(super) fn with_root_slot(self, root_slot_index: usize) -> Self {
        Self {
            root_slot_bits: self.root_slot_bits | (1 << root_slot_index),
            ..self
        }
    }

    pub(super) fn with_system_slot(self, device: usize, slot: usize) -> Self {
        Self {
            system_slot_bits: self.system_slot_bits
                | (1 << (device * FAULT_SYSTEM_SLOTS_PER_DEVICE + slot)),
            ..self
        }
    }

    fn with_dead_device(self, device: usize) -> Self {
        Self {
            dead_device_bits: self.dead_device_bits | (1 << device),
            ..self
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            root_slot_bits: self.root_slot_bits | other.root_slot_bits,
            system_slot_bits: self.system_slot_bits | other.system_slot_bits,
            dead_device_bits: self.dead_device_bits | other.dead_device_bits,
        }
    }

    pub(super) fn device_is_dead(self, device: usize) -> bool {
        (self.dead_device_bits >> device) & 1 == 1
    }

    /// 这个根槽读不出：它自己在故障里，或它所在的盘不在。
    pub(super) fn root_slot_is_unreadable(
        self,
        root_slot_index: usize,
        slots_per_region: u64,
    ) -> bool {
        (self.root_slot_bits >> root_slot_index) & 1 == 1
            || self.device_is_dead(root_choice_device_of_root_slot(
                root_slot_index,
                slots_per_region,
            ))
    }

    fn system_slot_is_unreadable(self, device: usize, slot: usize) -> bool {
        (self.system_slot_bits >> (device * FAULT_SYSTEM_SLOTS_PER_DEVICE + slot)) & 1 == 1
            || self.device_is_dead(device)
    }
}

/// 根槽下标（区域号主序）所在的盘：区域归属 0 / 1 / 0。
fn root_choice_device_of_root_slot(root_slot_index: usize, slots_per_region: u64) -> usize {
    let region = root_slot_index / usize::try_from(slots_per_region).expect("每区槽数 4 或 8");
    usize::try_from(ROOT_CHOICE_REGION_DEVICES[region]).expect("盘号是 0 或 1")
}

/// 读者收到的 (盘, 偏移) 落在哪个根槽上（区域归属与槽距照 r3 的内存盘）；不是根槽的落点交回空。
pub(super) fn root_choice_root_slot_index_at(
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    slots_per_region: u64,
) -> Option<usize> {
    let spacing = u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES);
    for region in 0..ROOT_CHOICE_REGION_COUNT {
        let region_start = root_choice_region_start_in_bytes(region);
        if offset.0 < region_start || offset.0 >= region_start + slots_per_region * spacing {
            continue;
        }
        let offset_in_region = offset.0 - region_start;
        let region_device =
            ROOT_CHOICE_REGION_DEVICES[usize::try_from(region).expect("区域号不超过 2")];
        if !offset_in_region.is_multiple_of(spacing) || device.0 != region_device {
            return None;
        }
        return Some(root_choice_slot_index_of(
            region,
            offset_in_region / spacing,
            slots_per_region,
        ));
    }
    None
}

// ── 系统配置槽：每块盘两个，每槽 (世代号, tail, 实例代号) 或「撕裂」（新旧都读不出，D13 已定项 4）

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SystemSlotContent {
    Torn,
    Verified {
        generation: u64,
        tail: u64,
        instance: u32,
    },
}

pub(super) type SystemSlots =
    [[SystemSlotContent; FAULT_SYSTEM_SLOTS_PER_DEVICE]; FAULT_DEVICE_COUNT];

/// mkfs 之后：每块盘两槽都是 (世代号 1, tail 0, 实例 0)（登记 5.3 第 2 条）。
pub(super) fn system_slots_right_after_mkfs() -> SystemSlots {
    [[SystemSlotContent::Verified {
        generation: FAULT_MKFS_SYSTEM_GENERATION,
        tail: 0,
        instance: ROOT_CHOICE_MKFS_INSTANCE,
    }; FAULT_SYSTEM_SLOTS_PER_DEVICE]; FAULT_DEVICE_COUNT]
}

/// 一次系统配置写（取号与发布末尾的轮换都是它，登记 5.3 第 3 条）：世代号 = 这块盘两槽里这一刻读得出（不是撕裂、也不在这次挂载一直读不出的落点里）的
/// 最大世代号 + 1（一槽都读不出时 1），写在槽 世代号 mod 槽数。
pub(super) fn system_slot_write_of(
    slots: &SystemSlots,
    device: usize,
    tail: u64,
    instance: u32,
    unreadable_all_mount_long: FaultLocations,
) -> (usize, SystemSlotContent) {
    let mut largest_generation = 0;
    for (slot, content) in slots[device].iter().enumerate() {
        if let SystemSlotContent::Verified { generation, .. } = content {
            if !unreadable_all_mount_long.system_slot_is_unreadable(device, slot) {
                largest_generation = largest_generation.max(*generation);
            }
        }
    }
    let generation = largest_generation + 1;
    let slot =
        usize::try_from(generation % u64::try_from(FAULT_SYSTEM_SLOTS_PER_DEVICE).expect("2"))
            .expect("槽号是 0 或 1");
    (
        slot,
        SystemSlotContent::Verified {
            generation,
            tail,
            instance,
        },
    )
}

/// 这个读视图里读得出的系统配置槽的 tail：个数与最大值（没有时 0）。
pub(super) fn readable_system_slots_of(
    slots: &SystemSlots,
    locations: FaultLocations,
) -> (usize, u64) {
    let mut count = 0;
    let mut largest_tail = 0;
    for (device, device_slots) in slots.iter().enumerate() {
        for (slot, content) in device_slots.iter().enumerate() {
            if let SystemSlotContent::Verified { tail, .. } = content {
                if !locations.system_slot_is_unreadable(device, slot) {
                    count += 1;
                    largest_tail = largest_tail.max(*tail);
                }
            }
        }
    }
    (count, largest_tail)
}

/// 可写挂载读到的更新状态读不出那道闸、与取号那一刻的「判不出」取法（登记 r5 修订 1：`--gate r2` 只在 `--profile r4 --family root_choice_faults`
/// 时认；不给是取甲，r4 档输出逐字节不变）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WitnessGateRule {
    /// 取甲（r4 登记修订 1）：读得出且没撕裂的系统配置槽不足整池槽数就判不出；取号那一刻同判。
    OneUnreadableOrTornSlotMeansUndecidable,
    /// R2（D22（单元原子性怎么合成） 已定项 8 射程，今天的 `recovery.rs` 的 `system_configuration_witness_is_undecidable`）：判不出 ⟺ 有槽设备报读错，
    /// 且没有一块盘两槽都自证过；取号那一刻判不出、或某块盘两槽里一份自证过的都没有，就拒。撕裂（读得出、自证不过）不报读错（S-辨）。
    SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
}

impl WitnessGateRule {
    /// config 行与收尾判决行里闸那一段。取甲那一份与 r4 档的 config 行逐字相同。
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::OneUnreadableOrTornSlotMeansUndecidable => {
                "the_implementation_choice_one_unreadable_or_torn_slot_means_undecidable"
            }
            Self::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                "r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots"
            }
        }
    }
}

/// R2 读法：这个系统配置槽是不是「设备报读错」——读故障落点与不在的盘报读错；撕裂的槽读得出、自证不过，设备不报错
/// （D22 已定项 8 射程 S-辨；`recovery.rs` 的 `system_configuration_slots_of_one_device_read_once` 只在读不出时记读错）。
fn system_slot_reports_a_read_error(
    content: SystemSlotContent,
    is_unreadable_in_the_view: bool,
) -> bool {
    match content {
        SystemSlotContent::Torn => is_unreadable_in_the_view,
        SystemSlotContent::Verified { .. } => is_unreadable_in_the_view,
    }
}

/// R2 的判不出（D22 已定项 8 射程「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过」）：在 `locations` 那一遍读视图里判。
pub(super) fn system_configuration_witness_is_undecidable_under_the_read_error_rule(
    slots: &SystemSlots,
    locations: FaultLocations,
) -> bool {
    let some_slot_reports_a_read_error = slots.iter().enumerate().any(|(device, device_slots)| {
        device_slots.iter().enumerate().any(|(slot, content)| {
            system_slot_reports_a_read_error(
                *content,
                locations.system_slot_is_unreadable(device, slot),
            )
        })
    });
    let some_device_self_verified_both_slots =
        slots.iter().enumerate().any(|(device, device_slots)| {
            device_slots.iter().enumerate().all(|(slot, content)| {
                *content != SystemSlotContent::Torn
                    && !locations.system_slot_is_unreadable(device, slot)
            })
        });
    some_slot_reports_a_read_error && !some_device_self_verified_both_slots
}

/// 某块盘两槽里一份读得出且没撕裂的都没有（D22 已定项 8 射程「某块盘一份自证过的系统配置都没有时，取号那一步也拒」；
/// 与 `mount.rs` 逐盘核第一支同一判、同一视图）。
fn some_device_has_no_self_verified_system_slot(
    slots: &SystemSlots,
    locations: FaultLocations,
) -> bool {
    slots.iter().enumerate().any(|(device, device_slots)| {
        !device_slots.iter().enumerate().any(|(slot, content)| {
            matches!(content, SystemSlotContent::Verified { .. })
                && !locations.system_slot_is_unreadable(device, slot)
        })
    })
}

/// 取号那一刻（读视图 = 盘减去 TP 的落点，登记 r4 5.3 第 6 条、r5 修订 1 修1.五第 3 条）拒不拒、拒在哪一道：取甲下四槽有一槽读不出或撕裂就拒；
/// R2 下 (a) 判不出拒，(b) 某块盘一份自证过的都没有拒。
pub(super) fn acquisition_refusal_of(
    world: &FaultWorld,
    locations: FaultLocations,
) -> Option<MountRefusalReason> {
    match world.witness_gate_rule {
        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable => {
            let (readable_slot_count, _largest_tail) =
                readable_system_slots_of(&world.system_slots, locations);
            (readable_slot_count < FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL)
                .then_some(MountRefusalReason::AcquisitionUnderTheImplementationChoice)
        }
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
            if system_configuration_witness_is_undecidable_under_the_read_error_rule(
                &world.system_slots,
                locations,
            ) {
                return Some(MountRefusalReason::AcquisitionWitnessUndecidable);
            }
            if some_device_has_no_self_verified_system_slot(&world.system_slots, locations) {
                return Some(MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot);
            }
            None
        }
    }
}

/// 挂载开头故障菜单的一项（登记 5.3 第 5 条）：时机 T1 = 这次挂载里对那个落点的第一次读读不出、之后每一次读都读得出（D23 已定项 14「重读一次」的 R = 1）；
/// TP = 这次挂载里对那个落点的每一次读都读不出；两种都在下一次挂载开头恢复。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MountFaultMenuEntry {
    pub(super) menu_index: usize,
    /// 读阶段第一遍的读视图里读不出的落点（T1 与 TP 相同）。
    pub(super) first_read_unreadable: FaultLocations,
    /// 重读那一遍、取号那一刻、写侧的读里仍读不出的落点：T1 是空，TP 同第一遍。
    pub(super) every_read_unreadable: FaultLocations,
}

impl MountFaultMenuEntry {
    pub(super) fn label(self) -> &'static str {
        MOUNT_FAULT_MENU_LABELS[self.menu_index]
    }

    /// 历史标签（登记第一节「承诺内」第 1 条）：RNSB 是 D22 已定项 8 写明不罩的双故障形，别的落点罩着。
    pub(super) const fn is_inside_the_single_fault_promise(self) -> bool {
        self.menu_index < RNSB_FIRST_MENU_INDEX
    }
}

const RNSB_FIRST_MENU_INDEX: usize = 11;

/// 菜单里六种落点的名字次序：RN、SN0、SN1、SB、RI、RNSB（每种配 T1、TP 两项，前面是「无」）。
const MOUNT_FAULT_SITE_COUNT: usize = 6;

fn locations_of_a_mount_fault_site(world: &FaultWorld, site_number: usize) -> FaultLocations {
    let newest_root = world.newest_root_locations();
    let newest_system_slot_of_device_0 = world.newest_system_slot_locations(0);
    let newest_system_slot_of_device_1 = world.newest_system_slot_locations(1);
    match site_number {
        0 => newest_root,
        1 => newest_system_slot_of_device_0,
        2 => newest_system_slot_of_device_1,
        3 => newest_system_slot_of_device_0.union(newest_system_slot_of_device_1),
        4 => world.newest_instance_root_locations(),
        5 => newest_root
            .union(newest_system_slot_of_device_0)
            .union(newest_system_slot_of_device_1),
        unknown_site => panic!("故障落点只有 6 种，收到 {unknown_site}"),
    }
}

/// 挂载 `mount_number` 开头的故障菜单：挂载 1 没有故障（只有「无」一项），挂载 2 起 13 项（登记 5.3 第 5 条）。
pub(super) fn mount_fault_menu(
    world: &FaultWorld,
    mount_number: usize,
) -> Vec<MountFaultMenuEntry> {
    let mut menu = vec![MountFaultMenuEntry {
        menu_index: 0,
        first_read_unreadable: FaultLocations::NOWHERE,
        every_read_unreadable: FaultLocations::NOWHERE,
    }];
    if mount_number < 2 {
        return menu;
    }
    for site_number in 0..MOUNT_FAULT_SITE_COUNT {
        let locations = locations_of_a_mount_fault_site(world, site_number);
        menu.push(MountFaultMenuEntry {
            menu_index: menu.len(),
            first_read_unreadable: locations,
            every_read_unreadable: FaultLocations::NOWHERE,
        });
        menu.push(MountFaultMenuEntry {
            menu_index: menu.len(),
            first_read_unreadable: locations,
            every_read_unreadable: locations,
        });
    }
    assert_eq!(menu.len(), MOUNT_FAULT_MENU_LENGTH, "故障菜单 13 项");
    menu
}

impl JudgedFault {
    /// 被判时标签（登记第一节「承诺内」第 2 条）：RI 一次让一个实例的全部根读不出，超出单故障，不罩；别的罩着。
    pub(super) const fn is_inside_the_single_fault_promise(self) -> bool {
        match self {
            Self::Nothing | Self::NewestRoot | Self::DeviceZeroGone | Self::DeviceOneGone => true,
            Self::NewestInstanceRoots => false,
        }
    }

    pub(super) fn locations(self, world: &FaultWorld) -> FaultLocations {
        match self {
            Self::Nothing => FaultLocations::NOWHERE,
            Self::NewestRoot => world.newest_root_locations(),
            Self::NewestInstanceRoots => world.newest_instance_root_locations(),
            Self::DeviceZeroGone => FaultLocations::NOWHERE.with_dead_device(0),
            Self::DeviceOneGone => FaultLocations::NOWHERE.with_dead_device(1),
        }
    }
}

#[cfg(test)]
mod root_choice_fault_tests;
