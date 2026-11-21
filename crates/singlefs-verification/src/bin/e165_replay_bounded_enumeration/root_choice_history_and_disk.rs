//! 择根与重放合判一族（登记 `research/prompts/e165-r3-prereg.md` 5.3 至 5.5、`research/prompts/e165-r7-prereg.md` 1.3）的历史与盘：本地常量、域与格、根环落点与暖机计划、一次发布的材料、一段历史走到某一刻的世界与崩溃点、摆着根槽与记录槽的内存盘。

use super::*;

// ── 择根与重放合判一族（登记 `research/prompts/e165-r3-prereg.md` 5.3 至 5.5：K、K9、KW，与第 7 次重跑登记 `research/prompts/e165-r7-prereg.md` 1.3 的 KL；只在 `--profile r3 --family root_choice` 时跑）
//
// 被测对象是 `recover` 的 `Consult` 支里串起来的三步：`choose_root`、`scan_journal`、`replay_journal(…, true)`。
// 一个状态 = 内存盘上同时摆好根环（三个区域、每区 S 个槽）与 journal 环（两盘镜像）的一份盘；根与记录由合法历史生成，按持久顺序取崩溃点。
// 判据一侧只用本地常量与生成时记下的真相（哪个根槽里是哪条根、哪条记录有哪几份在盘上、哪几次用户发布已确认），不调 core 的判定函数。

/// 根环区域数、区域起点三个常量、区域归属、根槽宽、固定结构槽距、mkfs 的第 0 代根（登记第三节的回比表；
/// [`cross_check_root_choice_local_constants_against_the_implementation`] 逐个回比）。
pub(super) const ROOT_CHOICE_REGION_COUNT: u64 = 3;
pub(super) const ROOT_CHOICE_BASE_SLOT: u64 = 64;
pub(super) const ROOT_CHOICE_PRIME_STEP: u64 = 3;
pub(super) const ROOT_CHOICE_CHUNK_BYTES: u64 = 1 << 20;
pub(super) const ROOT_CHOICE_REGION_DEVICES: [u32; 3] = [0, 1, 0];
pub(super) const ROOT_CHOICE_ROOT_SLOT_BYTES: usize = 512;
pub(super) const ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES: u32 = 4096;
pub(super) const ROOT_CHOICE_MKFS_INSTANCE: u32 = 0;
const ROOT_CHOICE_MKFS_CHECKPOINT_TXG: u64 = 0;
/// 暖机至多推几次空发布（D16 已定项 8「至多 R = 3 次」）；第一版实付 2 次是几何的结果，不是这里的常量。
const ROOT_CHOICE_WARM_UP_EMPTY_PUBLISHES_AT_MOST: usize = 3;
/// 每区槽数：K、K9、KL 取 8，KW 取 4（登记 5.3、第三节回比表）。
pub(super) const ROOT_CHOICE_SLOTS_PER_REGION_WIDE: u64 = 8;
pub(super) const ROOT_CHOICE_SLOTS_PER_REGION_NARROW: u64 = 4;
/// 内存盘根槽数组的长度：三个区域 × 每区至多 8 个槽。
pub(super) const ROOT_CHOICE_ROOT_SLOTS_AT_MOST: usize = 24;
/// 内存盘 journal 记录数组的长度：环至多 64 槽。
pub(super) const ROOT_CHOICE_RING_SLOTS_AT_MOST: usize = 64;

impl RootChoiceDomain {
    pub(super) const fn ring(self) -> RingGeometry {
        match self {
            Self::BigRing | Self::WrappingRootRing | Self::AdversarialRootRing => RingGeometry::BIG,
            Self::SmallRingNine => RingGeometry::SMALL_NINE,
        }
    }

    pub(super) const fn slots_per_region(self) -> u64 {
        match self {
            Self::BigRing | Self::SmallRingNine | Self::AdversarialRootRing => {
                ROOT_CHOICE_SLOTS_PER_REGION_WIDE
            }
            Self::WrappingRootRing => ROOT_CHOICE_SLOTS_PER_REGION_NARROW,
        }
    }

    /// 用户发布每次的记录条数（登记 5.3：k ∈ {1, 2}，KW 只取 1）。KL 没有历史。
    pub(super) const fn user_publish_record_counts(self) -> &'static [usize] {
        match self {
            Self::BigRing | Self::SmallRingNine => &[1, 2],
            Self::WrappingRootRing => &[1],
            Self::AdversarialRootRing => &[],
        }
    }

    /// 合法历史一族判 KJ1、KJ3；KL 不是合法历史，只判 KJ0、KJ2（登记 5.5 最后一列、第一节读法）。
    pub(super) const fn is_a_legal_history(self) -> bool {
        match self {
            Self::BigRing | Self::SmallRingNine | Self::WrappingRootRing => true,
            Self::AdversarialRootRing => false,
        }
    }

    /// 全界（登记 5.4：K M ≤ 3、U ≤ 4；K9 M ≤ 2、U ≤ 2；KW M = 1、U ≤ 14）。
    pub(super) const fn largest_mounts(self, tier: BoundsTier) -> usize {
        match (self, tier) {
            (Self::BigRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 3,
            (Self::SmallRingNine, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 2,
            (Self::WrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 1,
            (Self::AdversarialRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 0,
        }
    }

    pub(super) const fn largest_user_publishes(self, tier: BoundsTier) -> usize {
        match (self, tier) {
            (Self::BigRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 4,
            (Self::SmallRingNine, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 2,
            (Self::WrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 14,
            (Self::AdversarialRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 0,
        }
    }
}

/// 一个 (M, U) 格：M 次挂载、U 次用户发布（跨挂载的总数）。KL 没有挂载与发布，取 (0, 0)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RootChoiceCell {
    pub(super) mounts: usize,
    pub(super) user_publishes: usize,
}

/// 一个域在这一档、这一次命令行下要跑的格，按 (M 升序, U 升序)。`largest_user_publishes_cap` 是 `--max-u`（只给调试用）。
pub(super) fn root_choice_cells_of(
    domain: RootChoiceDomain,
    tier: BoundsTier,
    largest_user_publishes_cap: Option<usize>,
) -> Vec<RootChoiceCell> {
    if !domain.is_a_legal_history() {
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

/// 版本 = (`checkpoint_txg`, 实例代号)，txg 为主、实例代号破平局（D13 已定项 1 M3；与 D22 已定项 7 的择新序同一个序）。
/// 字段次序就是比较次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RootVersion {
    pub(super) checkpoint_txg: u64,
    pub(super) instance: u32,
}

impl RootVersion {
    pub(super) fn of_root(root: &RootRecord) -> Self {
        Self {
            checkpoint_txg: root.checkpoint_txg.0,
            instance: root.instance.0,
        }
    }

    pub(super) fn describe(self) -> String {
        format!("txg{}/instance{}", self.checkpoint_txg, self.instance)
    }
}

// ── 根环落点与暖机计划（本地算，不调 core；回比在 [`cross_check_root_choice_local_constants_against_the_implementation`]）

/// 本地算的区域起点：基槽 × 槽字节 + 区域号 × 步长 × chunk（D22 已定项 2；登记第三节回比表第 1 行）。
pub(super) const fn root_choice_region_start_in_bytes(region: u64) -> u64 {
    ROOT_CHOICE_BASE_SLOT * LOCAL_SLOT_BYTES
        + region * ROOT_CHOICE_PRIME_STEP * ROOT_CHOICE_CHUNK_BYTES
}

/// 本地算的根槽落点 (区域, 槽)：区域 = txg mod 3、槽 = (txg div 3) mod S（登记第三节回比表第 2 行）。
pub(super) const fn root_choice_target_of_publish(
    checkpoint_txg: u64,
    slots_per_region: u64,
) -> (u64, u64) {
    (
        checkpoint_txg % ROOT_CHOICE_REGION_COUNT,
        (checkpoint_txg / ROOT_CHOICE_REGION_COUNT) % slots_per_region,
    )
}

pub(super) fn root_choice_slot_index_of(region: u64, slot: u64, slots_per_region: u64) -> usize {
    usize::try_from(region * slots_per_region + slot).expect("根槽数不超过 24")
}

/// 这次发布的根写进根槽数组的哪一格（区域号主序）。
pub(super) fn root_choice_slot_index_of_publish(
    domain: RootChoiceDomain,
    checkpoint_txg: u64,
) -> usize {
    let slots_per_region = domain.slots_per_region();
    let (region, slot) = root_choice_target_of_publish(checkpoint_txg, slots_per_region);
    root_choice_slot_index_of(region, slot, slots_per_region)
}

fn root_choice_device_of_publish(checkpoint_txg: u64) -> usize {
    let region =
        usize::try_from(checkpoint_txg % ROOT_CHOICE_REGION_COUNT).expect("区域号不超过 2");
    usize::try_from(ROOT_CHOICE_REGION_DEVICES[region]).expect("盘号是 0 或 1")
}

/// 暖机计划（D16 已定项 8、登记 5.3 第 2 点）：写行那次发布取 `row_write_txg`，之后从它起逐个加一、看那个 txg 的根落哪块盘，
/// 推到本实例的根覆盖两块盘为止，空发布至多 [`ROOT_CHOICE_WARM_UP_EMPTY_PUBLISHES_AT_MOST`] 次。返回的第一个是写行那次发布。
pub(super) fn root_choice_warm_up_plan(row_write_txg: u64) -> Vec<u64> {
    let mut covered = [false; 2];
    covered[root_choice_device_of_publish(row_write_txg)] = true;
    let mut plan = vec![row_write_txg];
    let mut checkpoint_txg = row_write_txg;
    while !(covered[0] && covered[1]) && plan.len() <= ROOT_CHOICE_WARM_UP_EMPTY_PUBLISHES_AT_MOST {
        checkpoint_txg += 1;
        plan.push(checkpoint_txg);
        covered[root_choice_device_of_publish(checkpoint_txg)] = true;
    }
    plan
}

/// 每个生成出来的根槽，本地算的 (区域, 槽)、偏移与 `root_ring::target_for_publish`、`root_ring::slot_offset` 相等（登记第三节回比表第 2 行；不符走 TH2）。
pub(super) fn assert_the_local_root_slot_arithmetic_against_the_implementation(
    domain: RootChoiceDomain,
    checkpoint_txg: u64,
) {
    let slots_per_region = domain.slots_per_region();
    let (region, slot) = root_choice_target_of_publish(checkpoint_txg, slots_per_region);
    let implementation_target = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(checkpoint_txg),
        RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
            .expect("每区槽数 4 或 8 在格式承诺的区间里"),
    );
    assert_eq!(
        (region, slot),
        (implementation_target.region, implementation_target.slot),
        "TH2：本地算的根槽 (区域, 槽) 与 root_ring::target_for_publish 不等（txg {checkpoint_txg}，每区 {slots_per_region} 槽）"
    );
    assert_eq!(
        root_choice_region_start_in_bytes(region)
            + slot * u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES),
        singlefs_core::root_ring::slot_offset(
            implementation_target,
            ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES
        )
        .0,
        "TH2：本地算的根槽偏移与 root_ring::slot_offset 不等（txg {checkpoint_txg}）"
    );
}

// ── 一次发布的材料：它的根（字节与自证结果）与它的记录（每条一个槽）

/// 投影之外的四个字段：全历史取 r1 档那一组常量（登记 5.3 第 4 点）。
fn root_choice_fields_beyond_the_projection() -> FieldsBeyondTheProjection {
    RunProfile::CONSTANT_FIELDS.fields_beyond_the_projection_of_version(0)
}

/// 一次发布（含 mkfs 的第 0 代根与 KL 的根）的材料：生成一次、之后只借用。
#[derive(Debug)]
pub(super) struct RootChoicePublishMaterial {
    pub(super) version: RootVersion,
    pub(super) fields: VersionFields,
    pub(super) root_bytes: Vec<u8>,
    pub(super) records: Vec<GeneratedSlot>,
    pub(super) is_user_publish: bool,
}

/// 只有根、没有记录的材料：根过 `to_slot(512)` 再 `parse_slot` 必须等于原根，不等当场停（登记停机条款 TH9）。
pub(super) fn root_material_without_records(
    instance: u32,
    checkpoint_txg: u64,
    segment: NewRootSegment,
) -> RootChoicePublishMaterial {
    let fields = VersionFields {
        instance,
        checkpoint_txg,
        segment,
    };
    let root = fields.to_root_with(root_choice_fields_beyond_the_projection());
    let root_bytes = root.to_slot(ROOT_CHOICE_ROOT_SLOT_BYTES);
    assert_eq!(
        RootRecord::parse_slot(&root_bytes, &ROOT_FILESYSTEM_IDENTIFIER),
        Some(root),
        "停机条款 TH9：生成的根过不了 to_slot(512) 再 parse_slot（实例 {instance}，txg {checkpoint_txg}）"
    );
    RootChoicePublishMaterial {
        version: RootVersion {
            checkpoint_txg,
            instance,
        },
        fields,
        root_bytes,
        records: Vec::new(),
        is_user_publish: false,
    }
}

/// mkfs 的第 0 代根：实例代号 0、txg 0，种进三个区域各自的槽 0（D22 已定项 8 第 3 条；登记第三节回比表）。
pub(super) fn mkfs_root_material() -> RootChoicePublishMaterial {
    root_material_without_records(
        ROOT_CHOICE_MKFS_INSTANCE,
        ROOT_CHOICE_MKFS_CHECKPOINT_TXG,
        NewRootSegment::off_timeline(0, ROOT_CHOICE_MKFS_CHECKPOINT_TXG),
    )
}

pub(super) struct RootChoicePublishPlan {
    pub(super) domain: RootChoiceDomain,
    pub(super) instance: u32,
    pub(super) checkpoint_txg: u64,
    pub(super) publish_ordinal: u64,
    pub(super) first_counter: u64,
    pub(super) new_pool_file_creation: u64,
    pub(super) record_count: usize,
    pub(super) commit_marker_bits: u32,
    /// 本实例写的上一条记录的 496 字节头算出的链值；本实例第一条记录没有，取 0（I-8.6）。
    pub(super) previous_chain_value: Option<u32>,
    pub(super) is_user_publish: bool,
}

/// 一次发布的材料：k 条记录（写行与暖机的空发布一条：事务号 0、提交标记 1、序号 1、带末条标志），新根段按全历史的发布序号取。
pub(super) fn root_choice_publish_material(
    plan: &RootChoicePublishPlan,
) -> RootChoicePublishMaterial {
    assert_the_local_root_slot_arithmetic_against_the_implementation(
        plan.domain,
        plan.checkpoint_txg,
    );
    let segment = NewRootSegment::of_timeline_publish(plan.publish_ordinal, plan.checkpoint_txg);
    let mut material = root_material_without_records(plan.instance, plan.checkpoint_txg, segment);
    material.is_user_publish = plan.is_user_publish;
    let mut transaction = plan.new_pool_file_creation;
    let mut back_chain = plan.previous_chain_value.unwrap_or(0);
    for record_position in 0..plan.record_count {
        let is_last_record_of_the_publish = record_position + 1 == plan.record_count;
        let is_commit =
            is_last_record_of_the_publish || (plan.commit_marker_bits >> record_position) & 1 == 1;
        let counter = plan.first_counter
            + u64::try_from(record_position).expect("一次发布的记录条数不超过 2");
        assert!(
            counter <= plan.domain.ring().ring_slots,
            "停机条款 TH9：界内最大计数器 {counter} 超过环槽数 {}，journal 环回绕了",
            plan.domain.ring().ring_slots
        );
        let record = JournalRecord {
            instance: InstanceGeneration(plan.instance),
            counter,
            checkpoint_txg: CheckpointTxg(plan.checkpoint_txg),
            transaction: if plan.is_user_publish { transaction } else { 0 },
            is_commit,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(
                u32::try_from(record_position + 1).expect("一次发布的记录条数不超过 2"),
            ),
            place_in_publish: if is_last_record_of_the_publish {
                JournalRecordPlaceInPublish::LastRecordOfThePublish
            } else {
                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
            },
            back_chain,
            filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
            new_tree_table: segment.tree_table,
            new_mapping_root: segment.mapping_root,
            new_tree_identifier_watermark: segment.tree_identifier_watermark,
            new_rollback_floor: CheckpointTxg(segment.rollback_floor),
            new_instance_table: fields_beyond_the_projection_carried_by_every_generated_record()
                .instance_table,
            new_allocation_record_tree_root:
                fields_beyond_the_projection_carried_by_every_generated_record()
                    .allocation_record_tree_root,
            new_unmount_marker: fields_beyond_the_projection_carried_by_every_generated_record()
                .unmount_marker,
            named: Vec::new(),
            new_reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
            new_pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        };
        let slot = generate_slot(
            0,
            plan.domain.ring(),
            &record,
            ParseExpectation::MustRoundTrip,
        );
        assert_eq!(
            VersionFields::of_record(&record),
            material.fields,
            "停机条款 TH9：记录头里的新根段与这次发布的根不是同一组六个字段"
        );
        back_chain = slot.chain_value_of_header;
        if plan.is_user_publish && is_commit {
            transaction += 1;
        }
        material.records.push(slot);
    }
    material
}

// ── 一段历史走到某一刻的盘（世界）、崩溃点、判据看到的盘面

#[derive(Clone, Debug)]
pub(super) struct RootChoiceLandedRecord {
    pub(super) material: Arc<RootChoicePublishMaterial>,
    pub(super) record_index: usize,
    pub(super) counter: u64,
    /// 0 号盘、1 号盘上这条记录各有没有一份（任一份自证过即算在，D23 已定项 14）。
    pub(super) copies_on_devices: [bool; 2],
}

#[derive(Clone, Debug)]
pub(super) struct RootChoiceWorld {
    pub(super) domain: RootChoiceDomain,
    /// 根环三个区域的槽，区域号主序、区域内槽号升序（与 `every_root_ring_slot` 同一个次序）；没写过的槽是 `None`（读成 512 个 0）。
    pub(super) root_slots: Vec<Option<Arc<RootChoicePublishMaterial>>>,
    /// 盘上至少有一份的记录，按写的次序。
    pub(super) landed_records: Vec<RootChoiceLandedRecord>,
    /// 崩溃点所在那次发布之前、fsync 已返回的用户发布，按写的次序（D16（发布语义） 已定项 7；写行与暖机的空发布不是用户的 fsync）。
    pub(super) confirmed_user_publishes: Vec<RootVersion>,
    pub(super) publishes_started: u64,
    /// 前面各次挂载的写法（见证用）。
    pub(super) history: Vec<String>,
}

impl RootChoiceWorld {
    /// mkfs 之后、一次挂载都还没有：第 0 代根在三个区域的槽 0，别处全 0，journal 环空（登记 5.3「mkfs」）。
    pub(super) fn right_after_mkfs(domain: RootChoiceDomain) -> Self {
        let genesis_root = Arc::new(mkfs_root_material());
        let slots_per_region = domain.slots_per_region();
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
            domain,
            root_slots,
            landed_records: Vec::new(),
            confirmed_user_publishes: Vec::new(),
            publishes_started: 0,
            history: Vec::new(),
        }
    }

    /// 根环里现存的根与盘上读得出的记录里最大的 txg（新实例写行那次发布的 txg 取它加一，D23 已定项 14 注 3）。
    pub(super) fn highest_checkpoint_txg_on_disk(&self) -> u64 {
        let highest_root_txg = self
            .root_slots
            .iter()
            .flatten()
            .map(|material| material.version.checkpoint_txg)
            .max()
            .unwrap_or(0);
        let highest_record_txg = self
            .landed_records
            .iter()
            .map(|landed| landed.material.version.checkpoint_txg)
            .max()
            .unwrap_or(0);
        highest_root_txg.max(highest_record_txg)
    }

    /// 盘上读得出的最大计数器（新实例第一条记录的计数器取它加一；没有记录时取 0，于是第一条是 1）。
    pub(super) fn highest_landed_counter(&self) -> u64 {
        self.landed_records
            .iter()
            .map(|landed| landed.counter)
            .max()
            .unwrap_or(0)
    }

    pub(super) fn landed_record_of(
        material: &Arc<RootChoicePublishMaterial>,
        record_index: usize,
        copies_on_devices: [bool; 2],
    ) -> RootChoiceLandedRecord {
        RootChoiceLandedRecord {
            material: Arc::clone(material),
            record_index,
            counter: material.records[record_index]
                .parsed
                .as_ref()
                .expect("合法历史的每条记录都解得出（生成时断言过）")
                .counter,
            copies_on_devices,
        }
    }

    /// 这次发布整次落了：每条记录两盘各一份，根写进它的落点（盖掉那个槽原来的根）。`is_confirmed` 是它之后 fsync 返回了（用户发布才会）。
    pub(super) fn with_publish_applied(
        &self,
        material: &Arc<RootChoicePublishMaterial>,
        is_confirmed: bool,
    ) -> Self {
        let mut world = self.clone();
        for record_index in 0..material.records.len() {
            world
                .landed_records
                .push(Self::landed_record_of(material, record_index, [true, true]));
        }
        world.root_slots
            [root_choice_slot_index_of_publish(self.domain, material.version.checkpoint_txg)] =
            Some(Arc::clone(material));
        if is_confirmed {
            world.confirmed_user_publishes.push(material.version);
        }
        world.publishes_started += 1;
        world
    }

    /// 这次发布崩在 `crashed.crash_point` 上之后的盘；`trace` 是这次挂载到这里为止的写法，连同崩溃点记进历史（见证用）。
    pub(super) fn with_crash_applied(
        &self,
        crashed: &RootChoiceCrashedPublish<'_>,
        trace: &str,
    ) -> Self {
        let mut world = self.clone();
        match crashed.crash_point {
            RootChoiceCrashPoint::RecordCopiesLanded { copy_mask } => {
                for record_index in 0..crashed.material.records.len() {
                    let copies_on_devices = copies_of_a_record_in_a_mask(copy_mask, record_index);
                    if copies_on_devices[0] || copies_on_devices[1] {
                        world.landed_records.push(Self::landed_record_of(
                            crashed.material,
                            record_index,
                            copies_on_devices,
                        ));
                    }
                }
            }
            RootChoiceCrashPoint::RootLanded => {
                for record_index in 0..crashed.material.records.len() {
                    world.landed_records.push(Self::landed_record_of(
                        crashed.material,
                        record_index,
                        [true, true],
                    ));
                }
                world.root_slots[root_choice_slot_index_of_publish(
                    self.domain,
                    crashed.material.version.checkpoint_txg,
                )] = Some(Arc::clone(crashed.material));
            }
        }
        world.publishes_started += 1;
        world
            .history
            .push(format!("{trace};{}", crashed.describe()));
        world
    }
}

/// 一次发布的崩溃点（登记 5.3 第 6 点，D13 已定项 4 按设备切段）：记录那一段 2k 份拷贝的任一子集落了、根没落（4^k 个，位 2j 是第 j 条的
/// 0 号盘那一份、位 2j + 1 是 1 号盘那一份），或根落了（1 个：系统配置轮换那一段的各个子集在三步眼里是同一块盘，并成这一个）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RootChoiceCrashPoint {
    RecordCopiesLanded { copy_mask: u32 },
    RootLanded,
}

pub(super) fn copies_of_a_record_in_a_mask(copy_mask: u32, record_index: usize) -> [bool; 2] {
    [
        (copy_mask >> (2 * record_index)) & 1 == 1,
        (copy_mask >> (2 * record_index + 1)) & 1 == 1,
    ]
}

pub(super) fn root_choice_crash_points_of(record_count: usize) -> Vec<RootChoiceCrashPoint> {
    let mut crash_points: Vec<RootChoiceCrashPoint> = (0..(1u32 << (2 * record_count)))
        .map(|copy_mask| RootChoiceCrashPoint::RecordCopiesLanded { copy_mask })
        .collect();
    crash_points.push(RootChoiceCrashPoint::RootLanded);
    crash_points
}

pub(super) struct RootChoiceCrashedPublish<'material> {
    pub(super) material: &'material Arc<RootChoicePublishMaterial>,
    pub(super) crash_point: RootChoiceCrashPoint,
}

impl RootChoiceCrashedPublish<'_> {
    pub(super) fn describe(&self) -> String {
        let crash_point_text = match self.crash_point {
            RootChoiceCrashPoint::RecordCopiesLanded { copy_mask } => {
                format!("record_copies_mask_{copy_mask:b}_root_not_landed")
            }
            RootChoiceCrashPoint::RootLanded => "root_landed".to_string(),
        };
        format!(
            "crash_in_publish_txg{}_instance{}_{}records_{crash_point_text}",
            self.material.version.checkpoint_txg,
            self.material.version.instance,
            self.material.records.len()
        )
    }
}

/// 判据与三步调用看到的盘面：每个根槽里是哪条根（读根环次序），盘上至少有一份的每条记录与它在哪几块盘上。
pub(super) struct RootChoiceLandedRecordRef<'world> {
    pub(super) material: &'world RootChoicePublishMaterial,
    pub(super) record_index: usize,
    pub(super) copies_on_devices: [bool; 2],
}

pub(super) struct RootChoiceLanding<'world> {
    pub(super) roots_in_ring_order: Vec<Option<&'world RootChoicePublishMaterial>>,
    pub(super) records: Vec<RootChoiceLandedRecordRef<'world>>,
}

/// 世界加上崩溃在途的那次发布（没有就是世界本身）落在盘上的样子。
pub(super) fn root_choice_landing_of<'world>(
    world: &'world RootChoiceWorld,
    crashed: Option<&'world RootChoiceCrashedPublish<'world>>,
) -> RootChoiceLanding<'world> {
    let mut roots_in_ring_order: Vec<Option<&RootChoicePublishMaterial>> = world
        .root_slots
        .iter()
        .map(|slot| slot.as_deref())
        .collect();
    let mut records: Vec<RootChoiceLandedRecordRef<'_>> = world
        .landed_records
        .iter()
        .map(|landed| RootChoiceLandedRecordRef {
            material: &landed.material,
            record_index: landed.record_index,
            copies_on_devices: landed.copies_on_devices,
        })
        .collect();
    if let Some(crashed) = crashed {
        match crashed.crash_point {
            RootChoiceCrashPoint::RecordCopiesLanded { copy_mask } => {
                for record_index in 0..crashed.material.records.len() {
                    let copies_on_devices = copies_of_a_record_in_a_mask(copy_mask, record_index);
                    if copies_on_devices[0] || copies_on_devices[1] {
                        records.push(RootChoiceLandedRecordRef {
                            material: crashed.material,
                            record_index,
                            copies_on_devices,
                        });
                    }
                }
            }
            RootChoiceCrashPoint::RootLanded => {
                for record_index in 0..crashed.material.records.len() {
                    records.push(RootChoiceLandedRecordRef {
                        material: crashed.material,
                        record_index,
                        copies_on_devices: [true, true],
                    });
                }
                roots_in_ring_order[root_choice_slot_index_of_publish(
                    world.domain,
                    crashed.material.version.checkpoint_txg,
                )] = Some(crashed.material);
            }
        }
    }
    RootChoiceLanding {
        roots_in_ring_order,
        records,
    }
}

// ── 内存盘：根槽（512 字节）与 journal 记录槽（4096 字节）摆在各自的落点上

/// 三步调用的读者：两块盘；根环没写过的槽读成 512 个 0（mkfs 清零，登记 5.5 第一段）；journal 环没落盘的记录槽读成 4096 个 0；别处读不出。
pub(super) struct RootChoiceDisk<'landing> {
    pub(super) ring: RingGeometry,
    pub(super) slots_per_region: u64,
    root_slot_bytes: [Option<&'landing [u8]>; ROOT_CHOICE_ROOT_SLOTS_AT_MOST],
    record_bytes: [[Option<&'landing [u8]>; ROOT_CHOICE_RING_SLOTS_AT_MOST]; 2],
}

impl<'landing> RootChoiceDisk<'landing> {
    pub(super) fn of_the_landing(
        domain: RootChoiceDomain,
        landing: &RootChoiceLanding<'landing>,
    ) -> Self {
        let mut root_slot_bytes: [Option<&[u8]>; ROOT_CHOICE_ROOT_SLOTS_AT_MOST] =
            [None; ROOT_CHOICE_ROOT_SLOTS_AT_MOST];
        for (slot_index, root) in landing.roots_in_ring_order.iter().enumerate() {
            root_slot_bytes[slot_index] = root.map(|material| material.root_bytes.as_slice());
        }
        let mut record_bytes: [[Option<&[u8]>; ROOT_CHOICE_RING_SLOTS_AT_MOST]; 2] =
            [[None; ROOT_CHOICE_RING_SLOTS_AT_MOST]; 2];
        for record in &landing.records {
            let slot = &record.material.records[record.record_index];
            let ring_index = usize::try_from(
                (slot
                    .parsed
                    .as_ref()
                    .expect("合法历史的每条记录都解得出（生成时断言过）")
                    .counter
                    - LOCAL_FIRST_JOURNAL_COUNTER)
                    % domain.ring().ring_slots,
            )
            .expect("环槽数不超过 64");
            for (device_index, has_a_copy) in record.copies_on_devices.iter().enumerate() {
                if *has_a_copy {
                    assert!(
                        record_bytes[device_index][ring_index].is_none(),
                        "停机条款 TH9：两条记录落在同一个环槽里，journal 环回绕了"
                    );
                    record_bytes[device_index][ring_index] = Some(slot.bytes.as_slice());
                }
            }
        }
        Self {
            ring: domain.ring(),
            slots_per_region: domain.slots_per_region(),
            root_slot_bytes,
            record_bytes,
        }
    }
}

impl PoolReader for RootChoiceDisk<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        vec![DeviceIdentity(0), DeviceIdentity(1)]
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        (device.0 < 2).then_some(MEMORY_DISK_DEVICE_BYTES)
    }

    fn physical_block_size_probed_on_the_device(
        &self,
        _device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        unreachable!("择根那三步不挂载、不探测块宽")
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        for region in 0..ROOT_CHOICE_REGION_COUNT {
            let region_start = root_choice_region_start_in_bytes(region);
            let spacing = u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES);
            if offset.0 < region_start || offset.0 >= region_start + self.slots_per_region * spacing
            {
                continue;
            }
            let offset_in_region = offset.0 - region_start;
            let region_device =
                ROOT_CHOICE_REGION_DEVICES[usize::try_from(region).expect("区域号不超过 2")];
            if !offset_in_region.is_multiple_of(spacing)
                || device.0 != region_device
                || length != ROOT_CHOICE_ROOT_SLOT_BYTES
            {
                return None;
            }
            let slot_index = root_choice_slot_index_of(
                region,
                offset_in_region / spacing,
                self.slots_per_region,
            );
            return Some(match self.root_slot_bytes[slot_index] {
                Some(bytes) => bytes.get(..length)?.to_vec(),
                None => vec![0u8; length],
            });
        }
        let ring_start = RingGeometry::ring_start_offset_in_bytes();
        let is_a_whole_record_slot_of_the_ring = device.0 < 2
            && offset.0 >= ring_start
            && offset.0 < ring_start + self.ring.ring_bytes()
            && (offset.0 - ring_start).is_multiple_of(LOCAL_JOURNAL_RECORD_BYTES)
            && u64::try_from(length).ok() == Some(LOCAL_JOURNAL_RECORD_BYTES);
        if !is_a_whole_record_slot_of_the_ring {
            return None;
        }
        let ring_index = usize::try_from((offset.0 - ring_start) / LOCAL_JOURNAL_RECORD_BYTES)
            .expect("环槽数不超过 64");
        let device_index = usize::try_from(device.0).expect("盘号是 0 或 1");
        Some(match self.record_bytes[device_index][ring_index] {
            Some(bytes) => bytes.get(..length)?.to_vec(),
            None => vec![0u8; length],
        })
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

/// 喂给三步的系统配置：fsid 取时间线根的那个 16 字节常量，区域归属 0 / 1 / 0，根槽宽 512、槽距 4096、每区 S 槽、journal 环长与起点 1024
/// （`choose_root`、`scan_journal` 不读的字段同 r2 修订 4 那一份的取法）。
pub(super) fn system_configuration_for_the_root_choice(
    domain: RootChoiceDomain,
) -> SystemConfiguration {
    SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
            this_device: DeviceIdentity(0),
            device_count: SCAN_CONFIGURATION_DEVICE_COUNT,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            sizes: SystemImmutableSizes {
                physical_block_size: u32::try_from(ROOT_CHOICE_ROOT_SLOT_BYTES)
                    .expect("512 装得进 32 位"),
                minimum_input_output_bytes: u32::try_from(ROOT_CHOICE_ROOT_SLOT_BYTES)
                    .expect("512 装得进 32 位"),
                fixed_structure_slot_spacing: ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES,
                journal_ring_bytes: domain.ring().ring_bytes(),
                root_ring_slots_per_region:
                    RootRingSlotsPerRegion::from_system_configuration_field(
                        domain.slots_per_region(),
                    )
                    .expect("每区槽数 4 或 8 在格式承诺的区间里"),
            },
            journal_ring_start_slot: SlotNumber(LOCAL_JOURNAL_RING_START_SLOT),
            root_ring_base_slot: SlotNumber(
                SCAN_CONFIGURATION_ROOT_RING_BASE_SLOT_THE_SCAN_DOES_NOT_READ,
            ),
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: SCAN_CONFIGURATION_SLOT_GENERATION_THE_SCAN_DOES_NOT_READ,
            journal_tail: SCAN_CONFIGURATION_JOURNAL_TAIL_THE_SCAN_DOES_NOT_READ,
            journal_instance: InstanceGeneration(CHOSEN_INSTANCE),
            rollback_floor: CheckpointTxg(SCAN_CONFIGURATION_ROLLBACK_FLOOR_THE_SCAN_DOES_NOT_READ),
        },
    }
}

#[cfg(test)]
mod tests;
